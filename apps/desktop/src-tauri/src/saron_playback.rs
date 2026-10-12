use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use sandikala_core::{Event, Pitch, Rational, ScoreDocument, TimeDomain};

const SAMPLE_BLOCK: usize = 2_048;
const MAX_RENDER_SECONDS: f64 = 120.0;
const DECAY_TAIL_SECONDS: f64 = 2.0;
const PROGRAM_SARON_PELOG: i32 = 0;

pub const SARON_SOUNDFONT_NAME: &str = "Gamelan_Saron_Pelog_F3.sf2";

/// Resolve external sound assets without copying them into the score or executable.
pub fn find_soundfont(candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|path| {
        path.extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("sf2"))
            && path.is_file()
            && File::open(path).is_ok()
    })
}

enum PlayerCommand {
    Play {
        document: Box<ScoreDocument>,
        soundfont_path: PathBuf,
        pulse_bpm: u32,
        response: SyncSender<Result<(), String>>,
    },
    Stop {
        response: SyncSender<Result<(), String>>,
    },
    Pause {
        paused: bool,
        response: SyncSender<Result<(), String>>,
    },
    Seek {
        pulse: f64,
        response: SyncSender<Result<(), String>>,
    },
    Status {
        response: SyncSender<Result<PlaybackStatus, String>>,
    },
}

/// A command handle; the native stream stays owned by its worker thread.
pub struct SaronPlayer {
    sender: SyncSender<PlayerCommand>,
}

struct OutputState {
    finished: AtomicBool,
    failed: AtomicBool,
    frames_played: AtomicUsize,
    sample_rate: u32,
    pulse_bpm: u32,
    paused: AtomicBool,
    seek_frame: AtomicUsize,
    total_frames: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatus {
    pub playing: bool,
    pub paused: bool,
    pub pulse_position: Option<f64>,
}

impl SaronPlayer {
    pub fn pause(&self, paused: bool) -> Result<(), String> {
        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(PlayerCommand::Pause { paused, response })
            .map_err(|_| "audio worker stopped".to_owned())?;
        result
            .recv()
            .map_err(|_| "audio worker stopped".to_owned())?
    }
    pub fn seek(&self, pulse: f64) -> Result<(), String> {
        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(PlayerCommand::Seek { pulse, response })
            .map_err(|_| "audio worker stopped".to_owned())?;
        result
            .recv()
            .map_err(|_| "audio worker stopped".to_owned())?
    }
    pub fn new() -> Result<Self, String> {
        let (sender, receiver) = mpsc::sync_channel(2);
        thread::Builder::new()
            .name("sandikala-saron-playback".to_owned())
            .spawn(move || run_player(receiver))
            .map_err(|error| format!("could not start audio worker: {error}"))?;
        Ok(Self { sender })
    }

    pub fn play(
        &self,
        document: &ScoreDocument,
        soundfont_path: PathBuf,
        pulse_bpm: u32,
    ) -> Result<(), String> {
        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(PlayerCommand::Play {
                document: Box::new(document.clone()),
                soundfont_path,
                pulse_bpm,
                response,
            })
            .map_err(|_| "audio worker stopped".to_owned())?;
        result
            .recv()
            .map_err(|_| "audio worker stopped".to_owned())?
    }

    pub fn stop(&self) -> Result<(), String> {
        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(PlayerCommand::Stop { response })
            .map_err(|_| "audio worker stopped".to_owned())?;
        result
            .recv()
            .map_err(|_| "audio worker stopped".to_owned())?
    }

    pub fn is_playing(&self) -> Result<bool, String> {
        self.status().map(|status| status.playing)
    }

    pub fn status(&self) -> Result<PlaybackStatus, String> {
        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(PlayerCommand::Status { response })
            .map_err(|_| "audio worker stopped".to_owned())?;
        result
            .recv()
            .map_err(|_| "audio worker stopped".to_owned())?
    }
}

fn run_player(receiver: Receiver<PlayerCommand>) {
    let mut _stream: Option<Stream> = None;
    let mut finished: Option<Arc<OutputState>> = None;
    while let Ok(command) = receiver.recv() {
        match command {
            PlayerCommand::Pause { paused, response } => {
                let result = finished
                    .as_ref()
                    .ok_or_else(|| "Start playback before pausing".to_owned())
                    .map(|state| state.paused.store(paused, Ordering::Release));
                let _ = response.send(result);
            }
            PlayerCommand::Seek { pulse, response } => {
                let result = finished
                    .as_ref()
                    .ok_or_else(|| "Start playback before seeking".to_owned())
                    .and_then(|state| {
                        let frame = pulse * 60.0 / f64::from(state.pulse_bpm)
                            * f64::from(state.sample_rate);
                        if !frame.is_finite() || frame < 0.0 || frame >= state.total_frames as f64 {
                            return Err("Seek position is outside this playback".into());
                        }
                        state
                            .seek_frame
                            .store(frame.round() as usize, Ordering::Release);
                        Ok(())
                    });
                let _ = response.send(result);
            }
            PlayerCommand::Play {
                document,
                soundfont_path,
                pulse_bpm,
                response,
            } => {
                _stream = None;
                finished = None;
                match start_saron_stream(&document, &soundfont_path, pulse_bpm) {
                    Ok((next_stream, next_finished)) => {
                        _stream = Some(next_stream);
                        finished = Some(next_finished);
                        let _ = response.send(Ok(()));
                    }
                    Err(message) => {
                        let _ = response.send(Err(message));
                    }
                }
            }
            PlayerCommand::Stop { response } => {
                _stream = None;
                finished = None;
                let _ = response.send(Ok(()));
            }
            PlayerCommand::Status { response } => {
                if finished
                    .as_ref()
                    .is_some_and(|state| state.failed.load(Ordering::Acquire))
                {
                    _stream = None;
                    finished = None;
                    let _ = response.send(Err(
                        "audio output stopped unexpectedly; check your output device and replay"
                            .to_owned(),
                    ));
                    continue;
                }
                if finished
                    .as_ref()
                    .is_some_and(|state| state.finished.load(Ordering::Acquire))
                {
                    _stream = None;
                    finished = None;
                }
                let pulse_position = finished.as_ref().map(|state| {
                    state.frames_played.load(Ordering::Acquire) as f64
                        / f64::from(state.sample_rate)
                        * f64::from(state.pulse_bpm)
                        / 60.0
                });
                let _ = response.send(Ok(PlaybackStatus {
                    playing: finished.is_some(),
                    paused: finished
                        .as_ref()
                        .is_some_and(|state| state.paused.load(Ordering::Acquire)),
                    pulse_position,
                }));
            }
        }
    }
}

fn start_saron_stream(
    document: &ScoreDocument,
    soundfont_path: &PathBuf,
    pulse_bpm: u32,
) -> Result<(Stream, Arc<OutputState>), String> {
    if !(30..=300).contains(&pulse_bpm) {
        return Err("pulse tempo must be between 30 and 300 pulses per minute".to_owned());
    }
    if !soundfont_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("sf2"))
    {
        return Err("choose a Pelog Saron SoundFont in .sf2 format".to_owned());
    }

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default audio output device is available".to_owned())?;
    let supported = device
        .default_output_config()
        .map_err(|error| format!("could not read audio output configuration: {error}"))?;
    let config = supported.config();
    #[cfg(debug_assertions)]
    eprintln!(
        "Saron output: {} ({} Hz, {} channels)",
        device.name().unwrap_or_default(),
        config.sample_rate.0,
        config.channels
    );
    let channels = usize::from(config.channels);
    if channels == 0 {
        return Err("the default audio device reports zero output channels".to_owned());
    }
    let samples = render_saron(document, soundfont_path, pulse_bpm, config.sample_rate.0)?;
    let samples = Arc::<[f32]>::from(samples);

    let (stream, finished) = match supported.sample_format() {
        SampleFormat::F32 => {
            build_output_stream::<f32>(&device, &config, channels, samples, pulse_bpm, |sample| {
                sample
            })?
        }
        SampleFormat::I16 => {
            build_output_stream::<i16>(&device, &config, channels, samples, pulse_bpm, |sample| {
                (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
            })?
        }
        SampleFormat::U16 => {
            build_output_stream::<u16>(&device, &config, channels, samples, pulse_bpm, |sample| {
                ((sample.clamp(-1.0, 1.0) + 1.0) * 0.5 * u16::MAX as f32) as u16
            })?
        }
        sample_format => {
            return Err(format!(
                "the default audio device uses unsupported sample format {sample_format:?}"
            ));
        }
    };
    stream
        .play()
        .map_err(|error| format!("could not start the audio output stream: {error}"))?;
    Ok((stream, finished))
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    samples: Arc<[f32]>,
    pulse_bpm: u32,
    convert: impl Fn(f32) -> T + Copy + Send + 'static,
) -> Result<(Stream, Arc<OutputState>), String>
where
    T: cpal::SizedSample,
{
    let mut cursor = 0_usize;
    let finished = Arc::new(OutputState {
        finished: AtomicBool::new(false),
        failed: AtomicBool::new(false),
        frames_played: AtomicUsize::new(0),
        sample_rate: config.sample_rate.0,
        pulse_bpm,
        paused: AtomicBool::new(false),
        seek_frame: AtomicUsize::new(usize::MAX),
        total_frames: samples.len() / 2,
    });
    let callback_finished = Arc::clone(&finished);
    let error_finished = Arc::clone(&finished);
    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [T], _| {
                let seek = callback_finished
                    .seek_frame
                    .swap(usize::MAX, Ordering::AcqRel);
                if seek != usize::MAX {
                    cursor = seek * 2;
                    callback_finished
                        .frames_played
                        .store(seek, Ordering::Release);
                }
                if callback_finished.paused.load(Ordering::Acquire) {
                    for sample in output {
                        *sample = convert(0.0);
                    }
                    return;
                }
                fill_output(output, &samples, channels, &mut cursor, convert);
                callback_finished
                    .frames_played
                    .store(cursor.min(samples.len()) / 2, Ordering::Release);
                if cursor >= samples.len() {
                    callback_finished.finished.store(true, Ordering::Release);
                }
            },
            move |_error| error_finished.failed.store(true, Ordering::Release),
            None,
        )
        .map_err(|error| format!("could not create the audio output stream: {error}"))?;
    Ok((stream, finished))
}

fn fill_output<T>(
    output: &mut [T],
    samples: &[f32],
    channels: usize,
    cursor: &mut usize,
    convert: impl Fn(f32) -> T,
) {
    for frame in output.chunks_mut(channels) {
        let left = samples
            .get(*cursor)
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0);
        let right = samples
            .get(cursor.saturating_add(1))
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0);
        if frame.len() == 1 {
            frame[0] = convert((left + right) * 0.5);
        } else {
            frame[0] = convert(left);
            frame[1] = convert(right);
            for extra_channel in &mut frame[2..] {
                *extra_channel = convert(0.0);
            }
        }
        *cursor = cursor.saturating_add(2);
    }
}

fn render_saron(
    document: &ScoreDocument,
    soundfont_path: &PathBuf,
    pulse_bpm: u32,
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    if document
        .notation_views
        .first()
        .is_some_and(|v| v.provider_id == sandikala_engine::studio::PROVIDER)
    {
        return super::score_audio::render(
            document,
            soundfont_path
                .parent()
                .ok_or("Sound library directory is missing")?,
            pulse_bpm,
            sample_rate,
        );
    }
    let voice_ids: HashSet<&str> = document
        .parts
        .iter()
        .filter(|part| {
            part.name.to_ascii_lowercase().contains("saron")
                || part
                    .instrument_ref
                    .id
                    .to_ascii_lowercase()
                    .contains("saron")
        })
        .flat_map(|part| part.voice_ids.iter().map(String::as_str))
        .collect();
    if voice_ids.is_empty() {
        return Err("this playback slice requires a Saron part".to_owned());
    }

    let mut actions = Vec::new();
    let mut last_frame = 0_usize;
    for event in &document.events {
        let Event::Sound {
            voice_id,
            anchor,
            extent,
            payload,
            ..
        } = event
        else {
            continue;
        };
        if !voice_ids.contains(voice_id.as_str()) {
            continue;
        }

        let domain = document
            .time_domains
            .iter()
            .find(|domain| domain.id() == anchor.domain_id)
            .ok_or_else(|| format!("event {} refers to a missing time domain", event.id()))?;
        let unit = match domain {
            TimeDomain::Cycle { unit, .. } | TimeDomain::Metric { unit, .. } => unit,
        };
        if unit != "pulse" {
            return Err(format!(
                "event {} uses time unit '{unit}'; this prototype currently plays pulse-based scores only",
                event.id()
            ));
        }
        let Pitch::Degree {
            degree, register, ..
        } = &payload.pitch
        else {
            return Err(format!(
                "event {} is not a numbered Gamelan degree",
                event.id()
            ));
        };
        let key = saron_midi_key(degree, *register)?;
        let start_frame = pulse_frame(anchor.position, pulse_bpm, sample_rate)?;
        let end_position = Rational {
            numerator: anchor.position.numerator * extent.denominator
                + extent.numerator * anchor.position.denominator,
            denominator: anchor.position.denominator * extent.denominator,
        };
        let end_frame = pulse_frame(end_position, pulse_bpm, sample_rate)?;
        if end_frame <= start_frame {
            return Err(format!("event {} has no playable duration", event.id()));
        }
        last_frame = last_frame.max(end_frame);
        actions.push(NoteAction {
            frame: start_frame,
            key,
            on: true,
        });
        actions.push(NoteAction {
            frame: end_frame,
            key,
            on: false,
        });
    }
    if actions.is_empty() {
        return Err("the score has no Saron degree notes to play".to_owned());
    }
    let tail_frames = (sample_rate as f64 * DECAY_TAIL_SECONDS) as usize;
    let total_frames = last_frame.saturating_add(tail_frames);
    if total_frames as f64 / f64::from(sample_rate) > MAX_RENDER_SECONDS {
        return Err("this prototype limits one playback render to two minutes".to_owned());
    }
    actions.sort_by_key(|action| (action.frame, action.on));

    let mut file = BufReader::new(
        File::open(soundfont_path)
            .map_err(|error| format!("could not open the selected SoundFont: {error}"))?,
    );
    let sound_font =
        Arc::new(SoundFont::new(&mut file).map_err(|error| {
            format!("the selected file is not a readable SF2 SoundFont: {error}")
        })?);
    let has_saron_program = sound_font.get_presets().iter().any(|preset| {
        let name = preset.get_name().to_ascii_lowercase();
        preset.get_bank_number() == 0
            && preset.get_patch_number() == PROGRAM_SARON_PELOG
            && name.contains("saron")
            && name.contains("pelog")
    });
    if !has_saron_program {
        return Err(
            "the selected SoundFont has no Pelog Saron preset at bank 0, program 0".to_owned(),
        );
    }
    let settings = SynthesizerSettings::new(
        i32::try_from(sample_rate).map_err(|_| "unsupported audio sample rate".to_owned())?,
    );
    let mut synthesizer = Synthesizer::new(&sound_font, &settings)
        .map_err(|error| format!("could not initialize the SoundFont synthesizer: {error}"))?;
    synthesizer.process_midi_message(0, 0xC0, PROGRAM_SARON_PELOG, 0);

    let mut interleaved = Vec::with_capacity(total_frames.saturating_mul(2));
    let mut left = vec![0.0_f32; SAMPLE_BLOCK];
    let mut right = vec![0.0_f32; SAMPLE_BLOCK];
    let mut cursor = 0_usize;
    let mut action_index = 0_usize;
    while cursor < total_frames {
        while actions
            .get(action_index)
            .is_some_and(|action| action.frame == cursor)
        {
            let action = actions[action_index];
            if action.on {
                synthesizer.note_on(0, i32::from(action.key), 100);
            } else {
                synthesizer.note_off(0, i32::from(action.key));
            }
            action_index += 1;
        }
        let next_action = actions
            .get(action_index)
            .map_or(total_frames, |action| action.frame.min(total_frames));
        let frames = SAMPLE_BLOCK.min(next_action.saturating_sub(cursor));
        if frames == 0 {
            continue;
        }
        synthesizer.render(&mut left[..frames], &mut right[..frames]);
        for index in 0..frames {
            interleaved.push(left[index]);
            interleaved.push(right[index]);
        }
        cursor += frames;
    }
    Ok(interleaved)
}

#[derive(Clone, Copy)]
struct NoteAction {
    frame: usize,
    key: u8,
    on: bool,
}

fn pulse_frame(position: Rational, pulse_bpm: u32, sample_rate: u32) -> Result<usize, String> {
    if position.numerator < 0 || position.denominator <= 0 {
        return Err("score playback requires non-negative rational pulse positions".to_owned());
    }
    let pulses = position.numerator as f64 / position.denominator as f64;
    let frames = pulses * 60.0 / f64::from(pulse_bpm) * f64::from(sample_rate);
    if !frames.is_finite() || frames > usize::MAX as f64 {
        return Err("score timing is too large to render".to_owned());
    }
    Ok(frames.round() as usize)
}

pub(crate) fn saron_midi_key(degree: &str, register: i32) -> Result<u8, String> {
    const LOWER: [u8; 7] = [48, 50, 52, 53, 55, 57, 59];
    const NORMAL: [u8; 7] = [60, 62, 64, 65, 67, 69, 71];
    const UPPER: [u8; 7] = [72, 74, 76, 77, 79, 81, 83];
    let degree_index = degree
        .parse::<usize>()
        .ok()
        .and_then(|value| value.checked_sub(1))
        .filter(|index| *index < 7)
        .ok_or_else(|| format!("unsupported Saron degree '{degree}'; expected 1–7"))?;
    let keys = match register {
        -1 => &LOWER,
        0 => &NORMAL,
        1 => &UPPER,
        _ => {
            return Err(format!(
                "unsupported Saron register {register}; expected -1, 0, or 1"
            ));
        }
    };
    Ok(keys[degree_index])
}

#[cfg(test)]
mod tests {
    use super::saron_midi_key;

    #[test]
    fn maps_saron_degrees_to_pelog_register_zones() {
        assert_eq!(saron_midi_key("1", -1).unwrap(), 48);
        assert_eq!(saron_midi_key("7", -1).unwrap(), 59);
        assert_eq!(saron_midi_key("1", 0).unwrap(), 60);
        assert_eq!(saron_midi_key("7", 0).unwrap(), 71);
        assert_eq!(saron_midi_key("1", 1).unwrap(), 72);
        assert_eq!(saron_midi_key("7", 1).unwrap(), 83);
    }

    #[test]
    fn rejects_unknown_degree_and_register_without_fallback() {
        assert!(saron_midi_key("8", 0).is_err());
        assert!(saron_midi_key("2", 2).is_err());
    }

    #[test]
    fn discovers_existing_soundfont_after_stale_saved_path() {
        let directory =
            std::env::temp_dir().join(format!("sandikala-sound-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let existing = directory.join(super::SARON_SOUNDFONT_NAME);
        std::fs::write(&existing, b"fixture").unwrap();
        assert_eq!(
            super::find_soundfont([
                directory.join("missing.sf2"),
                directory.clone(),
                existing.clone()
            ]),
            Some(existing.clone())
        );
        std::fs::remove_file(&existing).unwrap();
        assert_eq!(super::find_soundfont([existing]), None);
        std::fs::remove_dir(&directory).unwrap();
    }

    #[test]
    #[ignore = "plays audio on the host default output; requires SANDIKALA_SARON_SF2"]
    fn native_output_plays_reference_fixture() {
        let path = std::env::var("SANDIKALA_SARON_SF2").expect("provide the real Saron SoundFont");
        let document =
            serde_json::from_str(include_str!("../../../../fixtures/q1/gamelan-basic.sdlk"))
                .unwrap();
        let player = super::SaronPlayer::new().unwrap();
        player
            .play(&document, std::path::PathBuf::from(path), 300)
            .expect("native audio should start");
        assert!(player.is_playing().unwrap());
        let timeout = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut advanced = false;
        loop {
            let status = player.status().unwrap();
            if !status.playing {
                break;
            }
            advanced |= status.pulse_position.is_some_and(|position| position > 0.0);
            assert!(
                std::time::Instant::now() < timeout,
                "audio callback did not finish"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(
            advanced,
            "native output must consume samples and advance the playhead"
        );
        player.stop().unwrap();
        assert!(!player.is_playing().unwrap());
    }

    #[test]
    #[ignore = "requires the real SF2 at SANDIKALA_SARON_SF2"]
    fn renders_reference_saron_sf2_when_path_is_provided() {
        let path = std::env::var("SANDIKALA_SARON_SF2").expect("provide the real Saron SoundFont");
        let document =
            serde_json::from_str(include_str!("../../../../fixtures/q1/gamelan-basic.sdlk"))
                .expect("the checked-in Gamelan playback fixture should parse");
        let samples = super::render_saron(&document, &std::path::PathBuf::from(path), 120, 44_100)
            .expect("reference Saron SF2 should render the fixture");
        assert!(samples.iter().any(|sample| sample.abs() > 1.0e-6));
    }
}

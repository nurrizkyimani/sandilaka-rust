//! Render studio parts together; recorded gamelan samples and explicit piano preview.
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use sandikala_core::{Event, Pitch, PitchLetter, PitchSystem, ScoreDocument};
use sandikala_engine::studio::value;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};

pub fn sample_file(instrument: &str) -> Option<&'static str> {
    match instrument {
        "saron" => Some("Gamelan_Saron_Pelog_F3.sf2"),
        "gender" => Some("Gamelan_Gender_F3.sf2"),
        "gambang" => Some("Gamelan_Gambang_F3.sf2"),
        "siter" => Some("Gamelan_Siter_F3.sf2"),
        "rebab" => Some("Gamelan_Rebab_F3.sf2"),
        "suling" => Some("Gamelan_Suling_F3.sf2"),
        "bonang" => Some("Gamelan_Bonang_Barung_F3.sf2"),
        _ => None,
    }
}

pub fn render(
    doc: &ScoreDocument,
    library: &Path,
    bpm: u32,
    rate: u32,
) -> Result<Vec<f32>, String> {
    doc.validate().map_err(|e| e.to_string())?;
    if let Some(reason) = sandikala_engine::studio::unsupported_reason(doc) {
        return Err(reason);
    }
    if !(30..=300).contains(&bpm) || rate == 0 {
        return Err("Invalid playback tempo or sample rate".into());
    }
    let last = doc
        .events
        .iter()
        .filter(|e| matches!(e, Event::Sound { .. }))
        .map(|e| value(e.anchor().position) + value(e.extent()))
        .fold(0.0, f64::max);
    if last == 0.0 {
        return Err("Enter some notes before playing the score".into());
    }
    let seconds = last * 60.0 / f64::from(bpm) + 2.0;
    if seconds > 120.0 {
        return Err("This playback slice supports scores up to two minutes".into());
    }
    let frames = (seconds * f64::from(rate)).ceil() as usize;
    let mut output = vec![0.0; frames * 2];
    let mixer = sandikala_engine::studio::mixer(doc)?;
    let any_solo = mixer.values().any(|m| m.solo);
    let frame =
        |position: f64| (position * 60.0 / f64::from(bpm) * f64::from(rate)).round() as usize;
    for part in &doc.parts {
        let mix = mixer.get(&part.id).cloned().unwrap_or_default();
        if mix.mute || (any_solo && !mix.solo) {
            continue;
        }
        let angle = (f64::from(mix.pan) / 100.0 + 1.0) * std::f64::consts::FRAC_PI_4;
        let gain_left =
            (f64::from(mix.gain) / 100.0 * angle.cos() * std::f64::consts::SQRT_2) as f32;
        let gain_right =
            (f64::from(mix.gain) / 100.0 * angle.sin() * std::f64::consts::SQRT_2) as f32;
        let sounds: Vec<_> = doc
            .events
            .iter()
            .filter(|e| {
                part.voice_ids.iter().any(|id| id == e.voice_id())
                    && matches!(e, Event::Sound { .. })
            })
            .collect();
        if sounds.is_empty() {
            continue;
        }
        if part.instrument_ref.id == "piano" {
            for event in sounds {
                let velocity =
                    f32::from(sandikala_engine::studio::event_marks(event)?.velocity) / 100.0;
                let Event::Sound { payload, .. } = event else {
                    unreachable!()
                };
                let Pitch::Spelled {
                    context_id,
                    letter,
                    alteration,
                    octave,
                } = &payload.pitch
                else {
                    return Err("Piano requires spelled pitch".into());
                };
                let (reference_hz, reference_key) = doc
                    .pitch_systems
                    .iter()
                    .find_map(|s| match s {
                        PitchSystem::EqualTemperament { id, reference, .. } if id == context_id => {
                            Some((
                                reference.frequency_hz,
                                midi(reference.letter, 0, reference.octave),
                            ))
                        }
                        _ => None,
                    })
                    .ok_or("Piano tuning context is missing")?;
                let frequency = reference_hz
                    * 2.0_f64.powf(
                        f64::from(midi(*letter, *alteration, *octave) - reference_key) / 12.0,
                    );
                let start = frame(value(event.anchor().position));
                let end = frame(value(event.anchor().position) + value(event.extent()));
                for index in start..(end + rate as usize / 5).min(frames) {
                    let t = (index - start) as f64 / f64::from(rate);
                    let release = if index >= end {
                        1.0 - (index - end) as f64 / (f64::from(rate) / 5.0)
                    } else {
                        1.0
                    };
                    let envelope = (t * 300.0).min(1.0) * (-t * 2.3).exp() * release.max(0.0);
                    let phase = std::f64::consts::TAU * frequency * t;
                    let sample =
                        ((phase.sin() + 0.35 * (2.0 * phase).sin() + 0.15 * (3.0 * phase).sin())
                            * envelope
                            * 0.16) as f32;
                    output[index * 2] += sample * gain_left * velocity;
                    output[index * 2 + 1] += sample * gain_right * velocity;
                }
            }
        } else {
            let filename = sample_file(&part.instrument_ref.id)
                .ok_or_else(|| format!("No sound profile for {}", part.name))?;
            let mut reader = BufReader::new(
                File::open(library.join(filename))
                    .map_err(|e| format!("{} sound is missing ({filename}): {e}", part.name))?,
            );
            let font = Arc::new(SoundFont::new(&mut reader).map_err(|e| e.to_string())?);
            let program = if part.instrument_ref.id == "gender" {
                11
            } else {
                0
            };
            if !font
                .get_presets()
                .iter()
                .any(|p| p.get_bank_number() == 0 && p.get_patch_number() == program)
            {
                return Err(format!(
                    "{} has no expected bank 0/program {program}",
                    part.name
                ));
            }
            let mut synth = Synthesizer::new(&font, &SynthesizerSettings::new(rate as i32))
                .map_err(|e| e.to_string())?;
            for channel in 0..part.voice_ids.len() {
                synth.process_midi_message(channel as i32, 0xc0, program, 0);
            }
            let mut actions = vec![];
            for event in sounds {
                let Event::Sound { payload, .. } = event else {
                    unreachable!()
                };
                let Pitch::Degree {
                    degree, register, ..
                } = &payload.pitch
                else {
                    return Err("Gamelan requires degree pitches".into());
                };
                let key = super::saron_playback::saron_midi_key(degree, *register)?;
                let channel = part
                    .voice_ids
                    .iter()
                    .position(|id| id == event.voice_id())
                    .ok_or("Voice is missing")? as i32;
                let velocity = i32::from(sandikala_engine::studio::event_marks(event)?.velocity);
                actions.push((
                    frame(value(event.anchor().position)),
                    true,
                    key,
                    channel,
                    velocity,
                ));
                actions.push((
                    frame(value(event.anchor().position) + value(event.extent())),
                    false,
                    key,
                    channel,
                    velocity,
                ));
            }
            actions.sort_by_key(|a| (a.0, a.1));
            let mut left = vec![0.0; 1024];
            let mut right = vec![0.0; 1024];
            let mut cursor = 0;
            let mut action = 0;
            while cursor < frames {
                while action < actions.len() && actions[action].0 == cursor {
                    let (_, on, key, channel, velocity) = actions[action];
                    if on {
                        synth.note_on(channel, i32::from(key), velocity)
                    } else {
                        synth.note_off(channel, i32::from(key))
                    }
                    action += 1;
                }
                let next = actions.get(action).map_or(frames, |a| a.0).min(frames);
                let count = (next - cursor).min(1024);
                if count == 0 {
                    break;
                }
                synth.render(&mut left[..count], &mut right[..count]);
                for i in 0..count {
                    output[(cursor + i) * 2] += left[i] * 0.55 * gain_left;
                    output[(cursor + i) * 2 + 1] += right[i] * 0.55 * gain_right;
                }
                cursor += count;
            }
        }
    }
    let peak = output.iter().map(|v| v.abs()).fold(0.0, f32::max);
    if peak > 0.98 {
        for sample in &mut output {
            *sample *= 0.98 / peak;
        }
    }
    Ok(output)
}
fn midi(letter: PitchLetter, alteration: i32, octave: i32) -> i32 {
    (octave + 1) * 12
        + match letter {
            PitchLetter::C => 0,
            PitchLetter::D => 2,
            PitchLetter::E => 4,
            PitchLetter::F => 5,
            PitchLetter::G => 7,
            PitchLetter::A => 9,
            PitchLetter::B => 11,
        }
        + alteration
}

/// IEEE-float WAV retains the rendered sample values without quantization.
pub fn wav(samples: &[f32], rate: u32) -> Result<Vec<u8>, String> {
    let length = u32::try_from(samples.len() * 4).map_err(|_| "Audio export is too large")?;
    let mut bytes = Vec::with_capacity(samples.len() * 4 + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(48 + length).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&3u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 8).to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(&32u16.to_le_bytes());
    bytes.extend_from_slice(b"fact");
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&((samples.len() / 2) as u32).to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&length.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn piano_document() -> ScoreDocument {
        let mut doc = sandikala_engine::studio::new_score();
        let voice = doc.parts[1].voice_ids[0].clone();
        sandikala_engine::studio::set_event(
            &mut doc,
            &voice,
            sandikala_core::Rational::from_i64(1),
            sandikala_core::Rational::from_i64(1),
            Some(Pitch::Spelled {
                context_id: "piano-tuning".into(),
                letter: PitchLetter::A,
                octave: 4,
                alteration: 0,
            }),
        )
        .unwrap();
        doc
    }
    #[test]
    fn piano_preview_keeps_a_quarter_of_silence_and_exports_float_wav() {
        let samples = render(&piano_document(), Path::new("/missing-library"), 120, 22050).unwrap();
        assert_eq!(samples.len(), 132300);
        assert!(samples[..22050].iter().all(|s| *s == 0.0));
        assert!(samples[22050..44100].iter().any(|s| s.abs() > 0.01));
        assert!(samples.iter().all(|s| s.is_finite()));
        let encoded = wav(&[-0.125, 0.25, 0.5, -0.75], 44100).unwrap();
        assert_eq!(&encoded[20..22], &3u16.to_le_bytes());
        assert_eq!(&encoded[36..40], b"fact");
        assert_eq!(encoded.len(), 72);
        assert_eq!(
            f32::from_le_bytes(encoded[56..60].try_into().unwrap()),
            -0.125
        );
    }
    #[test]
    #[ignore = "requires real Gamelan SoundFonts and plays audio on the default output"]
    fn mixed_native_score_pauses_seeks_and_exports_without_silent_instruments() {
        let path = std::path::PathBuf::from(
            std::env::var("SANDIKALA_SARON_SF2").expect("provide the real Saron SF2"),
        );
        let mut doc = piano_document();
        sandikala_engine::studio::add_instrument(&mut doc, "gender").unwrap();
        let context = doc.pitch_systems[0].id().to_owned();
        for voice in [
            doc.parts[0].voice_ids[0].clone(),
            doc.parts[2].voice_ids[0].clone(),
            doc.parts[2].voice_ids[1].clone(),
        ] {
            sandikala_engine::studio::set_event(
                &mut doc,
                &voice,
                sandikala_core::Rational::from_i64(0),
                sandikala_core::Rational::from_i64(1),
                Some(Pitch::Degree {
                    context_id: context.clone(),
                    degree: "3".into(),
                    register: 0,
                }),
            )
            .unwrap();
        }
        let samples = render(&doc, path.parent().unwrap(), 120, 44100).unwrap();
        assert!(samples.iter().any(|s| s.abs() > 0.02));
        let player = crate::saron_playback::SaronPlayer::new().unwrap();
        player.play(&doc, path, 120).unwrap();
        player.pause(true).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(70));
        let first = player.status().unwrap();
        assert!(first.paused);
        std::thread::sleep(std::time::Duration::from_millis(70));
        assert_eq!(
            first.pulse_position,
            player.status().unwrap().pulse_position
        );
        player.seek(0.5).unwrap();
        player.pause(false).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(player.status().unwrap().pulse_position.unwrap() >= 0.5);
        player.stop().unwrap();
        assert!(!player.is_playing().unwrap());
    }
}

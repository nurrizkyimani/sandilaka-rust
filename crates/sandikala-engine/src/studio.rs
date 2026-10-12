//! Original page layout and editing contracts for the score studio.
use base64::{Engine, engine::general_purpose::STANDARD};
use sandikala_core::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

pub const PROVIDER: &str = "sandikala.score.v1";
pub const SETTINGS: &str = "sandikala.studio.v1";
pub const MIXER: &str = "sandikala.mixer.v1";
pub const MARKS: &str = "sandikala.event-marks.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartMix {
    pub gain: u32,
    pub pan: i32,
    pub mute: bool,
    pub solo: bool,
}
impl Default for PartMix {
    fn default() -> Self {
        Self {
            gain: 100,
            pan: 0,
            mute: false,
            solo: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventMarks {
    pub text: String,
    pub velocity: u8,
}
impl Default for EventMarks {
    fn default() -> Self {
        Self {
            text: String::new(),
            velocity: 100,
        }
    }
}
pub fn mixer(doc: &ScoreDocument) -> Result<std::collections::HashMap<String, PartMix>, String> {
    let Some(value) = doc.extensions.get(MIXER) else {
        return Ok(Default::default());
    };
    let map: std::collections::HashMap<String, PartMix> =
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if map.iter().any(|(id, m)| {
        !doc.parts.iter().any(|p| &p.id == id) || m.gain > 100 || !(-100..=100).contains(&m.pan)
    }) {
        return Err("Mixer values or part references are invalid".into());
    }
    Ok(map)
}
pub fn event_marks(event: &Event) -> Result<EventMarks, String> {
    let extensions = match event {
        Event::Rest { extensions, .. } | Event::Sound { extensions, .. } => extensions,
    };
    let Some(value) = extensions.get(MARKS) else {
        return Ok(Default::default());
    };
    let marks: EventMarks = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if marks.text.chars().count() > 24 || marks.velocity == 0 || marks.velocity > 127 {
        return Err("Use annotations up to 24 characters and velocity 1–127".into());
    }
    Ok(marks)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScoreSettings {
    pub numerator: u32,
    pub denominator: u32,
    pub measures: u32,
    pub subdivision: u32,
    pub tempo: u32,
    pub letter_paper: bool,
    pub landscape: bool,
    pub system_breaks: Vec<u32>,
}
impl Default for ScoreSettings {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
            measures: 8,
            subdivision: 1,
            tempo: 120,
            letter_paper: false,
            landscape: false,
            system_breaks: vec![],
        }
    }
}
impl ScoreSettings {
    pub fn measure_length(&self) -> Result<Rational, String> {
        if !(1..=12).contains(&self.numerator)
            || ![4, 8, 16].contains(&self.denominator)
            || !(1..=128).contains(&self.measures)
            || ![1, 2, 4].contains(&self.subdivision)
            || !(30..=300).contains(&self.tempo)
            || self
                .system_breaks
                .iter()
                .any(|n| *n == 0 || *n >= self.measures)
        {
            return Err("Use 1–12 beats, denominator 4/8/16, 1–128 bars, subdivision 1/2/4, and tempo 30–300".into());
        }
        Rational::new(i64::from(self.numerator * 4), i64::from(self.denominator))
            .map_err(|e| e.to_string())
    }
}

pub fn settings(document: &ScoreDocument) -> Result<ScoreSettings, String> {
    let value = document
        .extensions
        .get(SETTINGS)
        .ok_or("Score configuration is missing")?;
    let value: ScoreSettings = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    value.measure_length()?;
    Ok(value)
}

pub fn instrument(id: &str) -> Option<(&'static str, u32, bool)> {
    match id {
        "saron" => Some(("Saron", 1, false)),
        "gender" => Some(("Gendèr", 2, false)),
        "gambang" => Some(("Gambang", 2, false)),
        "siter" => Some(("Siter", 2, false)),
        "rebab" => Some(("Rebab", 1, false)),
        "suling" => Some(("Suling", 1, false)),
        "bonang" => Some(("Bonang Barung", 2, false)),
        "piano" => Some(("Piano", 2, true)),
        _ => None,
    }
}

pub fn new_score() -> ScoreDocument {
    let mut doc: ScoreDocument =
        serde_json::from_str(include_str!("../../../fixtures/q1/gamelan-basic.sdlk"))
            .expect("valid fixture");
    doc.id = Uuid::new_v4().to_string();
    doc.metadata = Metadata {
        title: "Untitled score".into(),
        composer: String::new(),
    };
    doc.parts.clear();
    doc.voices.clear();
    doc.events.clear();
    doc.time_domains = vec![TimeDomain::Metric {
        id: "main-time".into(),
        unit: "quarter".into(),
        length: Rational::from_i64(32),
    }];
    doc.pitch_systems.push(PitchSystem::EqualTemperament {
        id: "piano-tuning".into(),
        steps_per_octave: 12,
        reference: EqualTemperamentReference {
            letter: PitchLetter::A,
            octave: 4,
            frequency_hz: 440.0,
        },
    });
    doc.notation_views = vec![NotationView {
        id: "score".into(),
        provider_id: PROVIDER.into(),
        part_ids: vec![],
        settings: serde_json::json!({}),
    }];
    doc.extensions.insert(
        SETTINGS.into(),
        serde_json::to_value(ScoreSettings::default()).unwrap(),
    );
    add_instrument(&mut doc, "saron").unwrap();
    add_instrument(&mut doc, "piano").unwrap();
    doc
}

pub fn add_instrument(doc: &mut ScoreDocument, id: &str) -> Result<(), String> {
    let (name, staff_count, _) = instrument(id).ok_or("Unknown instrument")?;
    if doc.parts.len() >= 8 {
        return Err("This score studio currently supports up to eight parts".into());
    }
    let part_id = Uuid::new_v4().to_string();
    let mut voice_ids = vec![];
    for _ in 0..staff_count {
        let voice_id = Uuid::new_v4().to_string();
        doc.voices.push(Voice {
            id: voice_id.clone(),
            part_id: part_id.clone(),
            time_domain_id: "main-time".into(),
        });
        voice_ids.push(voice_id);
    }
    doc.parts.push(Part {
        id: part_id.clone(),
        name: name.into(),
        instrument_ref: InstrumentRef {
            id: id.into(),
            version: "1".into(),
        },
        voice_ids,
    });
    doc.notation_views[0].part_ids.push(part_id);
    Ok(())
}

pub fn unsupported_reason(doc: &ScoreDocument) -> Option<String> {
    if let Err(reason) = mixer(doc) {
        return Some(reason);
    }
    let cfg = match settings(doc) {
        Ok(v) => v,
        Err(e) => return Some(e),
    };
    let length = cfg
        .measure_length()
        .ok()?
        .checked_mul(Rational::from_i64(i64::from(cfg.measures)))
        .ok()?;
    if doc.time_domains.len() != 1
        || !matches!(&doc.time_domains[0], TimeDomain::Metric { id, unit, length: l } if id == "main-time" && unit == "quarter" && *l == length)
    {
        return Some(
            "Studio requires one quarter-unit metric timeline matching the bar configuration"
                .into(),
        );
    }
    if doc.parts.len() > 8
        || doc.parts.iter().any(|p| {
            instrument(&p.instrument_ref.id).is_none_or(|i| p.voice_ids.len() != i.1 as usize)
        })
    {
        return Some("An instrument is outside the current studio catalog".into());
    }
    for event in &doc.events {
        if let Err(reason) = event_marks(event) {
            return Some(reason);
        }
        let bar = value(cfg.measure_length().ok()?);
        if (value(event.anchor().position) / bar).floor()
            != ((value(event.anchor().position) + value(event.extent()) - 1e-8) / bar).floor()
        {
            return Some(
                "Split events at barlines; ties are outside this provider's current subset".into(),
            );
        }
        if event.anchor().position.numerator < 0
            || ![1, 2, 4].contains(&event.anchor().position.denominator)
            || ![0.25, 0.5, 1.0, 2.0, 4.0].contains(&value(event.extent()))
        {
            return Some(
                "Studio currently supports nonnegative quarter/eighth/sixteenth timing".into(),
            );
        }
        if let Event::Sound { payload, .. } = event {
            let part = doc
                .parts
                .iter()
                .find(|p| p.voice_ids.iter().any(|v| v == event.voice_id()));
            let western = part
                .and_then(|p| instrument(&p.instrument_ref.id))
                .is_some_and(|i| i.2);
            if western != matches!(payload.pitch, Pitch::Spelled { .. }) {
                return Some(
                    "Use spelled pitches for piano and numeric degrees for gamelan".into(),
                );
            }
            match &payload.pitch {
                Pitch::Degree { register, .. } if !(-1..=1).contains(register) => {
                    return Some("Studio register range is lower/base/upper".into());
                }
                _ => {}
            }
        }
    }
    None
}

pub fn configure(doc: &mut ScoreDocument, cfg: ScoreSettings) -> Result<(), String> {
    let length = cfg
        .measure_length()?
        .checked_mul(Rational::from_i64(i64::from(cfg.measures)))
        .map_err(|e| e.to_string())?;
    doc.time_domains[0] = TimeDomain::Metric {
        id: "main-time".into(),
        unit: "quarter".into(),
        length,
    };
    doc.extensions.insert(
        SETTINGS.into(),
        serde_json::to_value(cfg).map_err(|e| e.to_string())?,
    );
    Ok(())
}

pub fn set_event(
    doc: &mut ScoreDocument,
    voice_id: &str,
    position: Rational,
    extent: Rational,
    pitch: Option<Pitch>,
) -> Result<(), String> {
    let cfg = settings(doc)?;
    let bar = value(cfg.measure_length()?);
    if value(position) < 0.0
        || ![0.25, 0.5, 1.0, 2.0, 4.0].contains(&value(extent))
        || ![1, 2, 4].contains(&position.denominator)
        || (value(position) / bar).floor()
            != ((value(position.checked_add(extent).map_err(|e| e.to_string())?) - 1e-8) / bar)
                .floor()
    {
        return Err("Choose a positive supported duration that fits inside this bar; split notes at the barline".into());
    }
    let domain = doc
        .voices
        .iter()
        .find(|v| v.id == voice_id)
        .ok_or("Unknown staff")?
        .time_domain_id
        .clone();
    let existing = doc
        .events
        .iter()
        .find(|e| e.voice_id() == voice_id && e.anchor().position == position);
    let id = existing
        .map(|e| e.id().to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let extensions = existing
        .map(|e| match e {
            Event::Rest { extensions, .. } | Event::Sound { extensions, .. } => extensions.clone(),
        })
        .unwrap_or_default();
    doc.events
        .retain(|e| !(e.voice_id() == voice_id && e.anchor().position == position));
    let anchor = EventAnchor {
        domain_id: domain,
        position,
    };
    doc.events.push(match pitch {
        Some(pitch) => Event::Sound {
            id,
            voice_id: voice_id.into(),
            anchor,
            extent,
            payload: SoundPayload { pitch },
            extensions,
        },
        None => Event::Rest {
            id,
            voice_id: voice_id.into(),
            anchor,
            extent,
            payload: EmptyPayload {},
            extensions,
        },
    });
    Ok(())
}

pub fn value(r: Rational) -> f64 {
    r.numerator as f64 / r.denominator as f64
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HitCell {
    pub voice_id: String,
    pub position: Rational,
    pub end_position: Rational,
    pub event_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScorePage {
    pub number: u32,
    pub svg: String,
    pub cells: Vec<HitCell>,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageLayout {
    pub width: u32,
    pub height: u32,
    pub font_css: String,
    pub pages: Vec<ScorePage>,
}

pub fn font_css() -> String {
    let leland = STANDARD.encode(include_bytes!("../../../assets/fonts/Leland.otf"));
    let edwin = STANDARD.encode(include_bytes!("../../../assets/fonts/Edwin-Roman.otf"));
    format!(
        "@font-face{{font-family:Leland;src:url(data:font/otf;base64,{leland})}}@font-face{{font-family:Edwin;src:url(data:font/otf;base64,{edwin})}}"
    )
}

/// Minimum rhythmic widths are shared across staves. Breaks minimize squared unused space.
fn line_breaks(widths: &[f64], available: f64, forced: &[u32]) -> Vec<(usize, usize)> {
    let n = widths.len();
    let mut costs = vec![f64::INFINITY; n + 1];
    let mut next = vec![n; n];
    costs[n] = 0.0;
    for start in (0..n).rev() {
        let mut width = 0.0;
        for end in start + 1..=n {
            if end > start + 1 && forced.contains(&((end - 1) as u32)) {
                break;
            }
            width += widths[end - 1];
            if width > available && end > start + 1 {
                break;
            }
            let cost = if end == n {
                0.0
            } else {
                (available - width).max(0.0).powi(2)
            } + costs[end];
            if cost < costs[start] {
                costs[start] = cost;
                next[start] = end;
            }
        }
    }
    let mut result = vec![];
    let mut start = 0;
    while start < n {
        let end = next[start].max(start + 1);
        result.push((start, end));
        start = end;
    }
    result
}

pub fn layout(doc: &ScoreDocument, part_id: Option<&str>) -> Result<PageLayout, String> {
    doc.validate().map_err(|e| e.to_string())?;
    if let Some(reason) = unsupported_reason(doc) {
        return Err(reason);
    }
    let cfg = settings(doc)?;
    let (mut width, mut height) = if cfg.letter_paper {
        (816, 1056)
    } else {
        (794, 1123)
    };
    if cfg.landscape {
        std::mem::swap(&mut width, &mut height);
    }
    let voices: Vec<_> = doc
        .parts
        .iter()
        .filter(|p| part_id.is_none_or(|id| id == p.id))
        .flat_map(|p| {
            p.voice_ids
                .iter()
                .enumerate()
                .map(move |(i, id)| (p, i, id))
        })
        .collect();
    if voices.is_empty() {
        return Err("Choose an existing part".into());
    }
    let bar = value(cfg.measure_length()?);
    let grid = 1.0 / f64::from(cfg.subdivision);
    let mut columns = vec![];
    for measure in 0..cfg.measures {
        let mut ticks: BTreeSet<i64> = (0..(bar * 4.0).round() as i64)
            .filter(|tick| *tick % (4 / cfg.subdivision as i64) == 0)
            .collect();
        for event in &doc.events {
            let position = value(event.anchor().position);
            if position >= f64::from(measure) * bar && position < f64::from(measure + 1) * bar {
                ticks.insert(((position - f64::from(measure) * bar) * 4.0).round() as i64);
            }
            let end = position + value(event.extent());
            if end >= f64::from(measure) * bar && end < f64::from(measure + 1) * bar {
                ticks.insert(((end - f64::from(measure) * bar) * 4.0).round() as i64);
            }
        }
        columns.push(ticks.into_iter().collect::<Vec<_>>());
    }
    let available = f64::from(width) - 190.0;
    let column_weights: Vec<Vec<f64>> = columns
        .iter()
        .enumerate()
        .map(|(bar_index, ticks)| {
            ticks
                .iter()
                .map(|tick| {
                    let position = bar_index as f64 * bar + *tick as f64 / 4.0;
                    doc.events
                        .iter()
                        .filter(|e| (value(e.anchor().position) - position).abs() < 1e-8)
                        .map(|e| {
                            event_marks(e).map_or(32.0, |m| {
                                (m.text.chars().count() as f64 * 11.0 + 20.0).max(32.0)
                            })
                        })
                        .fold(32.0, f64::max)
                })
                .collect()
        })
        .collect();
    let widths: Vec<_> = column_weights
        .iter()
        .map(|weights| 46.0 + weights.iter().sum::<f64>())
        .collect();
    if widths.iter().any(|w| *w > available) {
        return Err("This bar is too dense for the selected paper. Use landscape or a coarser display subdivision".into());
    }
    let systems = line_breaks(&widths, available, &cfg.system_breaks);
    let mut row_offsets = vec![];
    let mut row_tops = vec![];
    let mut row_heights = vec![];
    let mut cursor = 0.0;
    for (part, staff, voice) in &voices {
        let mut above = 0.0_f64;
        let mut below = 0.0_f64;
        {
            for event in doc.events.iter().filter(|e| e.voice_id() == **voice) {
                let mut top =
                    (-event_top(event, part.instrument_ref.id == "piano", *staff)).max(0.0);
                if event_marks(event).is_ok_and(|m| !m.text.is_empty()) {
                    top += 24.0;
                }
                above = above.max(top);
                if let Event::Sound { payload, .. } = event
                    && let Pitch::Spelled { letter, octave, .. } = payload.pitch
                {
                    let offset = staff_offset(letter, octave, *staff);
                    let up = offset >= 20.0;
                    above = above.max(-(offset - if up { 32.0 } else { 12.0 }));
                    below = below.max(offset + if up { 12.0 } else { 32.0 } - 55.0);
                }
            }
        }
        row_tops.push(cursor);
        row_heights.push(72.0 + above + below);
        row_offsets.push(cursor + above);
        cursor += 72.0 + above + below;
    }
    let system_height = cursor + 28.0;
    if system_height > f64::from(height) - 210.0 {
        return Err(
            "Too many staves for this paper. Publish individual parts or use portrait".into(),
        );
    }
    let mut pages = vec![];
    let mut page = ScorePage {
        number: 1,
        svg: String::new(),
        cells: vec![],
    };
    let mut y = 158.0;
    let begin_page = |number: u32| {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><rect width=\"{width}\" height=\"{height}\" fill=\"white\"/><text x=\"{}\" y=\"76\" text-anchor=\"middle\" font-family=\"Edwin\" font-size=\"30\">{}</text><text x=\"{}\" y=\"112\" text-anchor=\"end\" font-family=\"Edwin\" font-size=\"15\">{}</text><text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Edwin\" font-size=\"12\">{number}</text>",
            width / 2,
            escape(&doc.metadata.title),
            width - 48,
            escape(&doc.metadata.composer),
            width / 2,
            height - 28
        )
    };
    page.svg = begin_page(1);
    for (first, last) in systems {
        if y + system_height > f64::from(height) - 48.0 {
            page.svg.push_str("</svg>");
            pages.push(page);
            let number = pages.len() as u32 + 1;
            page = ScorePage {
                number,
                svg: begin_page(number),
                cells: vec![],
            };
            y = 158.0;
        }
        text(
            &mut page.svg,
            144.0,
            y - 18.0,
            &(first + 1).to_string(),
            12,
            "Edwin",
            "start",
        );
        let sum: f64 = widths[first..last].iter().sum();
        for (row, (part, staff_index, voice_id)) in voices.iter().enumerate() {
            let row_y = y + row_offsets[row];
            let baseline = row_y + 42.0;
            let western = instrument(&part.instrument_ref.id).is_some_and(|i| i.2);
            if *staff_index == 0 {
                text(
                    &mut page.svg,
                    128.0,
                    baseline - 8.0
                        + if part.voice_ids.len() == 2 {
                            (row_offsets[row + 1] - row_offsets[row]) / 2.0
                        } else {
                            0.0
                        },
                    &part.name,
                    14,
                    "Edwin",
                    "end",
                );
            }
            if western {
                for line in 0..5 {
                    line_svg(
                        &mut page.svg,
                        144.0,
                        row_y + line as f64 * 10.0,
                        f64::from(width) - 48.0,
                        row_y + line as f64 * 10.0,
                        0.8,
                    );
                }
            } else {
                line_svg(
                    &mut page.svg,
                    144.0,
                    baseline,
                    f64::from(width) - 48.0,
                    baseline,
                    0.8,
                );
            }
            if *staff_index == 0 && part.voice_ids.len() == 2 {
                let end = y + row_offsets[row + 1] + 42.0;
                let middle = (row_y + end) / 2.0;
                page.svg.push_str(&format!("<path d=\"M 138 {row_y} C 122 {row_y} 139 {} 127 {middle} C 139 {} 122 {end} 138 {end}\" fill=\"none\" stroke=\"black\" stroke-width=\"1.6\"/>",middle-10.0,middle+10.0));
            }
            if western {
                text(
                    &mut page.svg,
                    145.0,
                    row_y + 30.0,
                    if *staff_index == 0 {
                        "\u{e050}"
                    } else {
                        "\u{e062}"
                    },
                    36,
                    "Leland",
                    "start",
                );
            }
            let mut x = 174.0;
            for measure in first..last {
                let measure_width = widths[measure] / sum * (available - 30.0);
                line_svg(
                    &mut page.svg,
                    x - 6.0,
                    if western { row_y } else { row_y + 12.0 },
                    x - 6.0,
                    row_y + 60.0,
                    1.0,
                );
                if measure == first {
                    text(
                        &mut page.svg,
                        x + 2.0,
                        row_y + 22.0,
                        &cfg.numerator.to_string(),
                        17,
                        "Edwin",
                        "start",
                    );
                    text(
                        &mut page.svg,
                        x + 2.0,
                        row_y + 42.0,
                        &cfg.denominator.to_string(),
                        17,
                        "Edwin",
                        "start",
                    );
                }
                let scale = (measure_width - 28.0) / column_weights[measure].iter().sum::<f64>();
                for (column, tick) in columns[measure].iter().enumerate() {
                    let column_width = column_weights[measure][column] * scale;
                    let pos = measure as f64 * bar + *tick as f64 / 4.0;
                    let note_x =
                        x + 28.0 + column_weights[measure][..column].iter().sum::<f64>() * scale;
                    let event = doc.events.iter().find(|e| {
                        e.voice_id() == **voice_id
                            && (value(e.anchor().position) - pos).abs() < 1e-8
                    });
                    let covering = doc.events.iter().find(|e| {
                        e.voice_id() == **voice_id
                            && value(e.anchor().position) < pos
                            && value(e.anchor().position) + value(e.extent()) > pos + 1e-8
                    });
                    page.cells.push(HitCell {
                        voice_id: (*voice_id).clone(),
                        position: Rational::new((pos * 4.0).round() as i64, 4)
                            .map_err(|e| e.to_string())?,
                        end_position: Rational::new(
                            ((measure as f64 * bar
                                + columns[measure]
                                    .get(column + 1)
                                    .map_or(bar, |n| *n as f64 / 4.0))
                                * 4.0)
                                .round() as i64,
                            4,
                        )
                        .map_err(|e| e.to_string())?,
                        event_id: event.map(|e| e.id().into()),
                        x: note_x - 8.0,
                        y: y + row_tops[row] - 8.0,
                        width: column_width.max(14.0),
                        height: row_heights[row],
                    });
                    if let Some(event) = event {
                        render_event(&mut page.svg, event, note_x, row_y, western, *staff_index);
                    } else if !western && (*tick as f64 / 4.0 / grid).fract().abs() < 1e-8 {
                        if covering.is_some_and(|e| matches!(e, Event::Sound { .. })) {
                            text(
                                &mut page.svg,
                                note_x,
                                baseline - 2.0,
                                "—",
                                17,
                                "Edwin",
                                "middle",
                            );
                        } else {
                            page.svg.push_str(&format!(
                                "<circle cx=\"{note_x}\" cy=\"{}\" r=\"2.3\" fill=\"black\"/>",
                                baseline - 4.0
                            ));
                        }
                    } else if western && event.is_none() && covering.is_none() {
                        let empty_bar = !doc.events.iter().any(|e| {
                            e.voice_id() == **voice_id
                                && value(e.anchor().position) < (measure + 1) as f64 * bar
                                && value(e.anchor().position) + value(e.extent())
                                    > measure as f64 * bar
                        });
                        if empty_bar && *tick != 0 {
                            continue;
                        }
                        text(
                            &mut page.svg,
                            note_x,
                            row_y + 20.0,
                            if empty_bar {
                                "\u{e4e3}"
                            } else if grid >= 1.0 {
                                "\u{e4e5}"
                            } else if grid >= 0.5 {
                                "\u{e4e6}"
                            } else {
                                "\u{e4e7}"
                            },
                            34,
                            "Leland",
                            "start",
                        );
                    }
                }
                x += measure_width;
            }
            line_svg(
                &mut page.svg,
                f64::from(width) - 48.0,
                row_y + if western { 0.0 } else { 12.0 },
                f64::from(width) - 48.0,
                row_y + 60.0,
                1.2,
            );
        }
        y += system_height;
    }
    page.svg.push_str("</svg>");
    pages.push(page);
    Ok(PageLayout {
        width,
        height,
        font_css: font_css(),
        pages,
    })
}

fn render_event(svg: &mut String, event: &Event, x: f64, y: f64, western: bool, staff: usize) {
    if let Ok(marks) = event_marks(event)
        && !marks.text.is_empty()
    {
        text(
            svg,
            x - 8.0,
            y + event_top(event, western, staff) - 12.0,
            &marks.text,
            11,
            "Edwin",
            "start",
        );
    }
    let extent = value(event.extent());
    match event {
        Event::Rest { .. } => {
            if western {
                text(
                    svg,
                    x,
                    y + 20.0,
                    if extent >= 4.0 {
                        "\u{e4e3}"
                    } else if extent >= 2.0 {
                        "\u{e4e4}"
                    } else if extent >= 1.0 {
                        "\u{e4e5}"
                    } else if extent >= 0.5 {
                        "\u{e4e6}"
                    } else {
                        "\u{e4e7}"
                    },
                    36,
                    "Leland",
                    "start",
                );
            } else {
                svg.push_str(&format!(
                    "<circle cx=\"{x}\" cy=\"{}\" r=\"2.3\"/>",
                    y + 38.0
                ));
            }
        }
        Event::Sound { payload, .. } => match &payload.pitch {
            Pitch::Degree {
                degree, register, ..
            } if !western => {
                text(svg, x, y + 40.0, degree, 26, "Edwin", "middle");
                if *register != 0 {
                    svg.push_str(&format!(
                        "<circle cx=\"{x}\" cy=\"{}\" r=\"2.7\"/>",
                        if *register > 0 { y + 9.0 } else { y + 53.0 }
                    ));
                }
                let beams = if extent <= 0.25 {
                    2
                } else if extent <= 0.5 {
                    1
                } else {
                    0
                };
                for beam in 0..beams {
                    line_svg(
                        svg,
                        x - 8.0,
                        y + 5.0 - beam as f64 * 4.0,
                        x + 9.0,
                        y + 5.0 - beam as f64 * 4.0,
                        1.4,
                    );
                }
            }
            Pitch::Spelled {
                letter,
                octave,
                alteration,
                ..
            } if western => {
                let step = match letter {
                    PitchLetter::C => 0,
                    PitchLetter::D => 1,
                    PitchLetter::E => 2,
                    PitchLetter::F => 3,
                    PitchLetter::G => 4,
                    PitchLetter::A => 5,
                    PitchLetter::B => 6,
                } + octave * 7;
                let bottom = if staff == 0 { 4 * 7 + 2 } else { 2 * 7 + 4 };
                let note_y = y + 40.0 - f64::from(step - bottom) * 5.0;
                for ledger in -8..=14 {
                    let ly = y + 40.0 - ledger as f64 * 5.0;
                    if ledger % 2 == 0
                        && !(0..=8).contains(&ledger)
                        && ((ly >= note_y && ly < y) || (ly <= note_y && ly > y + 40.0))
                    {
                        line_svg(svg, x - 4.0, ly, x + 16.0, ly, 0.9);
                    }
                }
                if *alteration != 0 {
                    text(
                        svg,
                        x - 15.0,
                        note_y,
                        if *alteration == 2 {
                            "\u{e263}"
                        } else if *alteration == -2 {
                            "\u{e264}"
                        } else if *alteration > 0 {
                            "\u{e262}"
                        } else {
                            "\u{e260}"
                        },
                        36,
                        "Leland",
                        "start",
                    );
                }
                text(
                    svg,
                    x,
                    note_y,
                    if extent >= 4.0 {
                        "\u{e0a2}"
                    } else if extent >= 2.0 {
                        "\u{e0a3}"
                    } else {
                        "\u{e0a4}"
                    },
                    40,
                    "Leland",
                    "start",
                );
                if extent < 4.0 {
                    let up = note_y >= y + 20.0;
                    let sx = if up { x + 12.0 } else { x };
                    let end = note_y + if up { -32.0 } else { 32.0 };
                    line_svg(svg, sx, note_y, sx, end, 1.2);
                    if extent <= 0.5 {
                        text(
                            svg,
                            sx,
                            end,
                            if extent <= 0.25 {
                                if up { "\u{e242}" } else { "\u{e243}" }
                            } else if up {
                                "\u{e240}"
                            } else {
                                "\u{e241}"
                            },
                            36,
                            "Leland",
                            "start",
                        );
                    }
                }
            }
            _ => text(svg, x, y + 30.0, "?", 22, "Edwin", "middle"),
        },
    }
}

pub fn printable_html(layout: &PageLayout) -> String {
    let mut html = format!(
        "<!doctype html><html><head><meta charset=\"UTF-8\"><style>{}@page{{size:{}px {}px;margin:0}}body{{margin:0}}section{{break-after:page;width:{}px;height:{}px}}section:last-child{{break-after:auto}}</style></head><body>",
        layout.font_css, layout.width, layout.height, layout.width, layout.height
    );
    for page in &layout.pages {
        html.push_str(&format!("<section>{}</section>", page.svg));
    }
    html.push_str("</body></html>");
    html
}

fn staff_offset(letter: PitchLetter, octave: i32, staff: usize) -> f64 {
    let step = match letter {
        PitchLetter::C => 0,
        PitchLetter::D => 1,
        PitchLetter::E => 2,
        PitchLetter::F => 3,
        PitchLetter::G => 4,
        PitchLetter::A => 5,
        PitchLetter::B => 6,
    } + octave * 7;
    40.0 - f64::from(step - if staff == 0 { 30 } else { 18 }) * 5.0
}

fn event_top(event: &Event, western: bool, staff: usize) -> f64 {
    if western
        && let Event::Sound { payload, .. } = event
        && let Pitch::Spelled { letter, octave, .. } = payload.pitch
    {
        let offset = staff_offset(letter, octave, staff);
        return (offset - if offset >= 20.0 { 32.0 } else { 20.0 }).min(0.0);
    }
    0.0
}
fn text(svg: &mut String, x: f64, y: f64, value: &str, size: u32, font: &str, align: &str) {
    svg.push_str(&format!("<text x=\"{x}\" y=\"{y}\" font-family=\"{font}\" font-size=\"{size}\" text-anchor=\"{align}\">{}</text>",escape(value)));
}
fn line_svg(svg: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, width: f64) {
    svg.push_str(&format!("<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"black\" stroke-width=\"{width}\"/>"));
}
fn escape(value: &str) -> String {
    value
        .chars()
        .filter(|c| matches!(*c as u32,9|10|13|0x20..=0xd7ff|0xe000..=0xfffd|0x10000..=0x10ffff))
        .take(256)
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_score_has_shared_grid_and_extractable_piano_part() {
        let doc = new_score();
        doc.validate().unwrap();
        let score = layout(&doc, None).unwrap();
        assert_eq!((score.width, score.height), (794, 1123));
        assert_eq!(
            score.pages.iter().map(|p| p.cells.len()).sum::<usize>(),
            8 * 4 * 3
        );
        let piano = layout(&doc, Some(&doc.parts[1].id)).unwrap();
        assert_eq!(
            piano.pages.iter().map(|p| p.cells.len()).sum::<usize>(),
            8 * 4 * 2
        );
        assert!(score.font_css.contains("data:font/otf;base64,"));
    }
    #[test]
    fn seven_eighth_grid_has_seven_positions_per_staff() {
        let mut doc = new_score();
        let cfg = ScoreSettings {
            numerator: 7,
            denominator: 8,
            measures: 1,
            subdivision: 2,
            ..Default::default()
        };
        configure(&mut doc, cfg).unwrap();
        let score = layout(&doc, None).unwrap();
        assert_eq!(score.pages[0].cells.len(), 21);
        assert_eq!(score.pages[0].cells[6].position, Rational::from_i64(3));
    }
    #[test]
    fn page_breaks_are_deterministic_and_fonts_print_with_pages() {
        let mut doc = new_score();
        configure(
            &mut doc,
            ScoreSettings {
                measures: 32,
                ..Default::default()
            },
        )
        .unwrap();
        let score = layout(&doc, None).unwrap();
        assert!(score.pages.len() > 1);
        assert_eq!(score, layout(&doc, None).unwrap());
        let html = printable_html(&score);
        assert_eq!(html.matches("<section>").count(), score.pages.len());
    }
}

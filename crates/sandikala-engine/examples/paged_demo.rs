//! Repeatable mixed-notation publishing fixture, not a traditional composition.
use sandikala_core::{Pitch, Rational};
use sandikala_engine::{publishing, studio};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("provide an output directory")?,
    );
    std::fs::create_dir_all(&directory)?;
    let mut doc = studio::new_score();
    doc.metadata.title = "Sandikala - mixed ensemble".into();
    doc.metadata.composer = "Score studio validation".into();
    studio::add_instrument(&mut doc, "gender")?;
    studio::configure(
        &mut doc,
        studio::ScoreSettings {
            measures: 12,
            subdivision: 2,
            ..Default::default()
        },
    )?;
    let context = doc
        .pitch_systems
        .iter()
        .find(|p| matches!(p, sandikala_core::PitchSystem::Degree { .. }))
        .unwrap()
        .id()
        .to_owned();
    for (index, voice) in doc.voices.clone().iter().enumerate() {
        let piano = doc
            .parts
            .iter()
            .find(|p| p.id == voice.part_id)
            .unwrap()
            .instrument_ref
            .id
            == "piano";
        for n in 0..24 {
            let pitch = if piano {
                Pitch::Spelled {
                    context_id: "piano-tuning".into(),
                    letter: match n % 4 {
                        0 => sandikala_core::PitchLetter::C,
                        1 => sandikala_core::PitchLetter::E,
                        2 => sandikala_core::PitchLetter::G,
                        _ => sandikala_core::PitchLetter::D,
                    },
                    octave: if index == 2 { 3 } else { 4 },
                    alteration: 0,
                }
            } else {
                Pitch::Degree {
                    context_id: context.clone(),
                    degree: ((n + index as i64) % 7 + 1).to_string(),
                    register: if n % 7 == 0 {
                        -1
                    } else if n % 7 == 1 {
                        1
                    } else {
                        0
                    },
                }
            };
            studio::set_event(
                &mut doc,
                &voice.id,
                Rational::from_i64(n),
                Rational::new(1, if index >= 3 { 2 } else { 1 })?,
                Some(pitch),
            )?;
        }
    }
    doc.validate()?;
    std::fs::write(
        directory.join("mixed-ensemble.sdlk"),
        serde_json::to_vec_pretty(&doc)?,
    )?;
    let layout = studio::layout(&doc, None)?;
    std::fs::write(directory.join("layout.json"), serde_json::to_vec(&layout)?)?;
    let piano_id = doc
        .parts
        .iter()
        .find(|p| p.instrument_ref.id == "piano")
        .unwrap()
        .id
        .clone();
    std::fs::write(
        directory.join("piano-layout.json"),
        serde_json::to_vec(&studio::layout(&doc, Some(&piano_id))?)?,
    )?;
    std::fs::write(
        directory.join("score-studio-preview.pdf"),
        publishing::pdf(&layout)?,
    )?;
    std::fs::write(
        directory.join("score-studio-preview.html"),
        studio::printable_html(&layout),
    )?;
    eprintln!(
        "{} pages, {} events, {} staves",
        layout.pages.len(),
        doc.events.len(),
        doc.voices.len()
    );
    Ok(())
}

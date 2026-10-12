//! Safe SVG layout for the deliberately small Q1 numerical demo view.

use sandikala_core::{CoreError, Event, Pitch, PitchSystem, Rational, ScoreDocument, TimeDomain};
use serde::Serialize;

const PROVIDER_ID: &str = "sandikala.numeric-demo.v1";
const CELL_WIDTH: u32 = 44;
const CELL_HEIGHT: u32 = 64;
const PADDING: u32 = 24;
const HEADER_HEIGHT: u32 = 64;
const ROW_GAP: u32 = 16;
const SVG_WIDTH: u32 = 752;
const CONTENT_WIDTH: u32 = 704;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellGeometry {
    pub voice_id: String,
    pub pulse_index: u32,
    pub event_id: Option<String>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LayoutResult {
    Pages {
        layout: crate::studio::PageLayout,
    },
    Svg {
        provider_id: String,
        width: u32,
        height: u32,
        svg: String,
        cells: Vec<CellGeometry>,
    },
    UnsupportedView {
        view_id: String,
        reason: String,
    },
}

impl LayoutResult {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Pages { .. } => "pages",
            Self::Svg { .. } => "svg",
            Self::UnsupportedView { .. } => "unsupportedView",
        }
    }
}

/// Render the selected view when it matches the fixed one-part numerical demo subset.
/// Unsupported documents are returned as a tagged result and are never converted.
pub fn layout_document(document: &ScoreDocument, view_id: &str) -> Result<LayoutResult, CoreError> {
    document.validate()?;
    let Some(view) = document
        .notation_views
        .iter()
        .find(|view| view.id == view_id)
    else {
        return Ok(unsupported(view_id, "notation view does not exist"));
    };
    if view.provider_id == crate::studio::PROVIDER {
        return Ok(match crate::studio::layout(document, None) {
            Ok(layout) => LayoutResult::Pages { layout },
            Err(reason) => unsupported(view_id, &reason),
        });
    }
    if view.provider_id != PROVIDER_ID {
        return Ok(unsupported(
            view_id,
            "notation provider is not supported by Q1",
        ));
    }

    if let Some(reason) = numeric_view_unsupported_reason(document) {
        return Ok(unsupported(view_id, &reason));
    }

    let domain = document.time_domains.as_slice();
    let (cycle_count, cycle_length) = match &domain[0] {
        TimeDomain::Cycle {
            unit,
            cycle_length,
            cycle_count,
            ..
        } if unit == "pulse" && *cycle_length == 16 => (*cycle_count, *cycle_length),
        _ => {
            return Ok(unsupported(
                view_id,
                "numeric demo requires a 16-pulse cycle domain",
            ));
        }
    };
    let voice = &document.voices[0];

    let mut events_by_pulse: Vec<Option<&Event>> =
        vec![None; (cycle_count * cycle_length) as usize];
    for event in &document.events {
        let position = event.anchor().position;
        let Ok(index) = usize::try_from(position.numerator) else {
            return Ok(unsupported(
                view_id,
                "numeric demo event position is outside the view",
            ));
        };
        if index >= events_by_pulse.len() || events_by_pulse[index].is_some() {
            return Ok(unsupported(
                view_id,
                "numeric demo event position is outside the view or duplicated",
            ));
        }
        events_by_pulse[index] = Some(event);
    }

    let height = HEADER_HEIGHT + cycle_count * CELL_HEIGHT + (cycle_count - 1) * ROW_GAP + PADDING;
    let mut cells = Vec::with_capacity(events_by_pulse.len());
    for (pulse, event_at_pulse) in events_by_pulse.iter().enumerate() {
        let row = pulse as u32 / cycle_length;
        let column = pulse as u32 % cycle_length;
        cells.push(CellGeometry {
            voice_id: voice.id.clone(),
            pulse_index: pulse as u32,
            event_id: event_at_pulse.as_ref().map(|event| event.id().to_owned()),
            x: PADDING + column * CELL_WIDTH,
            y: HEADER_HEIGHT + row * (CELL_HEIGHT + ROW_GAP),
            width: CELL_WIDTH,
            height: CELL_HEIGHT,
        });
    }
    let svg = render_svg(document, cycle_count, &cells, &events_by_pulse, height);
    Ok(LayoutResult::Svg {
        provider_id: PROVIDER_ID.to_owned(),
        width: SVG_WIDTH,
        height,
        svg,
        cells,
    })
}

/// Return the reason a document cannot be edited or rendered by the fixed Q1
/// numeric view. Session capabilities and SVG layout share this check so the UI
/// cannot offer edits for a score that the renderer must reject.
pub(crate) fn numeric_view_unsupported_reason(document: &ScoreDocument) -> Option<String> {
    if document.time_domains.len() != 1 {
        return Some("numeric demo requires exactly one time domain".to_owned());
    }
    let (domain_id, cycle_count) = match &document.time_domains[0] {
        TimeDomain::Cycle {
            id,
            unit,
            cycle_length: 16,
            cycle_count,
        } if unit == "pulse" && (1..=64).contains(cycle_count) => (id, *cycle_count),
        _ => return Some("numeric demo requires 16-pulse cycles, with 1 to 64 cycles".to_owned()),
    };
    if document.parts.len() != 1 || document.voices.len() != 1 {
        return Some("numeric demo requires exactly one part and one voice".to_owned());
    }
    let part = &document.parts[0];
    let voice = &document.voices[0];
    if part.voice_ids != [voice.id.as_str()]
        || voice.part_id != part.id
        || voice.time_domain_id != *domain_id
    {
        return Some("numeric part and voice references are inconsistent".to_owned());
    }

    let degree_contexts: Vec<_> = document
        .pitch_systems
        .iter()
        .filter_map(|system| match system {
            PitchSystem::Degree { id, degrees, .. } => Some((id, degrees)),
            PitchSystem::EqualTemperament { .. } => None,
        })
        .collect();
    let expected_degrees: Vec<String> = (1..=7).map(|degree| degree.to_string()).collect();
    if degree_contexts.len() != 1 || degree_contexts[0].1 != &expected_degrees {
        return Some(
            "numeric demo requires one degree context declaring exactly 1 through 7".to_owned(),
        );
    }
    let context_id = degree_contexts[0].0;

    if document.notation_views.len() != 1
        || document.notation_views[0].provider_id != PROVIDER_ID
        || document.notation_views[0].part_ids != [part.id.as_str()]
    {
        return Some("numeric demo view must target the sole part".to_owned());
    }

    let max_pulse = u64::from(cycle_count) * 16;
    for event in &document.events {
        if event.voice_id() != voice.id || event.anchor().domain_id != *domain_id {
            return Some("numeric events must belong to the sole voice and cycle".to_owned());
        }
        let position = event.anchor().position;
        if position.numerator < 0
            || position.denominator != 1
            || event.extent() != Rational::from_i64(1)
            || position.numerator as u64 >= max_pulse
        {
            return Some(
                "numeric events must occupy one integer pulse within the cycle extent".to_owned(),
            );
        }
        if let Event::Sound { payload, .. } = event {
            match &payload.pitch {
                Pitch::Degree {
                    context_id: event_context,
                    degree,
                    register,
                } if event_context == context_id
                    && expected_degrees.iter().any(|value| value == degree)
                    && (-1..=1).contains(register) => {}
                _ => {
                    return Some(
                        "numeric demo supports degrees 1 through 7 in registers -1, 0, and +1"
                            .to_owned(),
                    );
                }
            }
        }
    }
    None
}

fn unsupported(view_id: &str, reason: &str) -> LayoutResult {
    LayoutResult::UnsupportedView {
        view_id: view_id.to_owned(),
        reason: reason.to_owned(),
    }
}

fn render_svg(
    document: &ScoreDocument,
    cycle_count: u32,
    cells: &[CellGeometry],
    events: &[Option<&Event>],
    height: u32,
) -> String {
    let title = visible_text(&document.metadata.title, 20, CONTENT_WIDTH);
    let composer = visible_text(&document.metadata.composer, 14, CONTENT_WIDTH);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{SVG_WIDTH}\" height=\"{height}\" viewBox=\"0 0 {SVG_WIDTH} {height}\"><title>{}</title><desc>{}</desc><defs><clipPath id=\"header-clip\"><rect x=\"{PADDING}\" y=\"0\" width=\"{CONTENT_WIDTH}\" height=\"{HEADER_HEIGHT}\"/></clipPath></defs><g clip-path=\"url(#header-clip)\"><text x=\"{PADDING}\" y=\"28\" font-family=\"sans-serif\" font-size=\"20\">{}</text><text x=\"{PADDING}\" y=\"52\" font-family=\"sans-serif\" font-size=\"14\">{}</text></g>",
        escape_xml(&document.metadata.title),
        escape_xml(&document.metadata.composer),
        escape_xml(&title),
        escape_xml(&composer),
    );
    for row in 0..cycle_count {
        let y = HEADER_HEIGHT + row * (CELL_HEIGHT + ROW_GAP);
        for group in 1..4 {
            let x = PADDING + group * 4 * CELL_WIDTH;
            svg.push_str(&format!("<line x1=\"{x}\" y1=\"{y}\" x2=\"{x}\" y2=\"{}\" stroke=\"#98a2ad\" stroke-width=\"1\"/>", y + CELL_HEIGHT));
        }
    }
    for (index, cell) in cells.iter().enumerate() {
        if let Some(event) = events[index] {
            match event {
                Event::Rest { .. } => svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"20\">.</text>",
                    cell.x + CELL_WIDTH / 2, cell.y + 32
                )),
                Event::Sound { payload, .. } => if let Pitch::Degree { degree, register, .. } = &payload.pitch {
                    svg.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"20\">{}</text>",
                        cell.x + CELL_WIDTH / 2, cell.y + 32, degree
                    ));
                    if *register != 0 {
                        let dot_y = if *register > 0 { cell.y + 14 } else { cell.y + 44 };
                        svg.push_str(&format!("<circle cx=\"{}\" cy=\"{dot_y}\" r=\"2\" fill=\"#202830\"/>", cell.x + CELL_WIDTH / 2));
                    }
                },
            }
        }
    }
    svg.push_str("</svg>");
    svg
}

fn visible_text(value: &str, font_size: u32, max_width: u32) -> String {
    let max_chars = (max_width / (font_size * 3 / 5).max(1)) as usize;
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let take = max_chars.saturating_sub(1);
    let mut output: String = value.chars().take(take).collect();
    output.push('…');
    output
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if !is_xml_10_character(character) {
            escaped.push('\u{fffd}');
            continue;
        }
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn is_xml_10_character(character: char) -> bool {
    let codepoint = character as u32;
    matches!(codepoint, 0x9 | 0xA | 0xD)
        || (0x20..=0xD7FF).contains(&codepoint)
        || (0xE000..=0xFFFD).contains(&codepoint)
        || (0x10000..=0x10FFFF).contains(&codepoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sandikala_core::Event;

    fn fixture(name: &str) -> ScoreDocument {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/q1")
            .join(name);
        let source = std::fs::read_to_string(path).expect("fixture is readable");
        ScoreDocument::parse_json(&source).expect("fixture validates")
    }

    #[test]
    fn gamelan_fixture_uses_independent_geometry_and_glyph_oracle() {
        let document = fixture("gamelan-basic.sdlk");
        let LayoutResult::Svg {
            width,
            height,
            cells,
            svg,
            ..
        } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert_eq!((width, height), (752, 152));
        assert_eq!(cells.len(), 16);
        for (pulse, cell) in cells.iter().enumerate() {
            assert_eq!(cell.pulse_index, pulse as u32);
            assert_eq!(cell.x, PADDING + (pulse as u32 % 16) * CELL_WIDTH);
            assert_eq!(cell.y, HEADER_HEIGHT);
            assert_eq!((cell.width, cell.height), (CELL_WIDTH, CELL_HEIGHT));
        }
        assert_eq!(
            (cells[0].x, cells[0].y, cells[0].width, cells[0].height),
            (24, 64, 44, 64)
        );
        assert_eq!(
            (cells[5].x, cells[5].y, cells[5].width, cells[5].height),
            (244, 64, 44, 64)
        );
        let tokens = [
            "1", "2", "3", "5", "6", ".", "5", "3", "2", "1", "2", "3", "5", "6", "5", "1",
        ];
        for (cell, token) in cells.iter().zip(tokens) {
            let expected = format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"20\">{token}</text>",
                cell.x + CELL_WIDTH / 2,
                cell.y + 32
            );
            assert!(svg.contains(&expected), "missing token {token}");
        }
        assert_eq!(svg.matches("<circle ").count(), 2);
        assert!(svg.contains("<circle cx=\"46\" cy=\"78\" r=\"2\" fill=\"#202830\"/>"));
        assert!(svg.contains("<circle cx=\"222\" cy=\"108\" r=\"2\" fill=\"#202830\"/>"));
        assert!(matches!(&document.events[5], Event::Rest { .. }));
        assert_eq!(cells[5].event_id.as_deref(), Some("event-06"));
        assert_eq!(cells[6].event_id.as_deref(), Some("event-07"));
    }

    #[test]
    fn two_cycles_have_the_fixed_height_and_row_positions() {
        let mut document = fixture("gamelan-basic.sdlk");
        if let TimeDomain::Cycle { cycle_count, .. } = &mut document.time_domains[0] {
            *cycle_count = 2;
        }
        let LayoutResult::Svg {
            width,
            height,
            cells,
            ..
        } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert_eq!((width, height), (752, 232));
        assert_eq!(cells.len(), 32);
        for (pulse, cell) in cells.iter().enumerate() {
            assert_eq!(cell.pulse_index, pulse as u32);
            assert_eq!(cell.x, PADDING + (pulse as u32 % 16) * CELL_WIDTH);
            assert_eq!(
                cell.y,
                HEADER_HEIGHT + (pulse as u32 / 16) * (CELL_HEIGHT + ROW_GAP)
            );
            assert_eq!((cell.width, cell.height), (CELL_WIDTH, CELL_HEIGHT));
        }
        assert_eq!((cells[16].x, cells[16].y), (24, 144));
        assert_eq!(cells[16].pulse_index, 16);
    }

    #[test]
    fn piano_and_unknown_providers_are_tagged_unsupported() {
        let piano = fixture("piano-model.sdlk");
        assert!(matches!(
            layout_document(&piano, "view-staff").unwrap(),
            LayoutResult::UnsupportedView { .. }
        ));
        let mut unknown = fixture("gamelan-basic.sdlk");
        unknown.notation_views[0].provider_id = "unknown.provider".to_owned();
        assert!(matches!(
            layout_document(&unknown, "view-numeric").unwrap(),
            LayoutResult::UnsupportedView { .. }
        ));
    }

    #[test]
    fn blank_cell_has_no_glyph_and_rest_is_a_period() {
        let mut document = fixture("gamelan-basic.sdlk");
        document.events.retain(|event| event.id() != "event-07");
        let LayoutResult::Svg { svg, cells, .. } =
            layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert_eq!(cells[5].event_id, Some("event-06".to_owned()));
        assert!(svg.contains(">.</text>"));
        assert_eq!(cells[6].event_id, None);
        let blank_glyph = format!(
            "<text x=\"{}\" y=\"{}\"",
            cells[6].x + cells[6].width / 2,
            cells[6].y + 32
        );
        assert!(!svg.contains(&blank_glyph));
    }

    #[test]
    fn metadata_is_escaped_and_long_visible_text_is_bounded() {
        let mut document = fixture("gamelan-basic.sdlk");
        document.metadata.title = "<script>alert(1)</script>".to_owned();
        document.metadata.composer = "C & D".to_owned();
        let LayoutResult::Svg { svg, .. } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert!(svg.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!svg.contains("<script>"));
        assert!(svg.contains("C &amp; D"));
        assert!(!svg.contains("foreignObject"));
        assert!(!svg.contains("onload="));

        document.metadata.title = "before\u{0001}middle\u{FFFE}after".to_owned();
        let LayoutResult::Svg { svg, .. } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert!(svg.contains("before\u{FFFD}middle\u{FFFD}after"));
        assert!(!svg.contains('\u{0001}'));
        assert!(!svg.contains('\u{FFFE}'));

        document.metadata.title = "x".repeat(256);
        let LayoutResult::Svg { svg, .. } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert!(svg.contains(format!("<title>{}</title>", "x".repeat(256)).as_str()));
        assert!(svg.contains("…"));
    }

    #[test]
    fn renders_degree_tokens_without_document_derived_markup() {
        let document = fixture("gamelan-basic.sdlk");
        let LayoutResult::Svg { svg, .. } = layout_document(&document, "view-numeric").unwrap()
        else {
            panic!("numeric fixture is supported")
        };
        assert!(!svg.contains("<image"));
        assert!(!svg.contains("href="));
        assert!(!svg.contains("<foreignObject"));
    }
}

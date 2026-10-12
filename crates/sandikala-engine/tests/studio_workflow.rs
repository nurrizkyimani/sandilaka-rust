use sandikala_core::{Pitch, Rational};
use sandikala_engine::{
    session::{DocumentSession, SessionCommand},
    studio,
};

#[test]
fn mixer_and_annotations_are_validated_persisted_and_undoable() {
    let mut session = DocumentSession::new_studio();
    let initial = session.projection();
    let id = initial.session_id.clone();
    let part = initial.document.parts[0].id.clone();
    let invalid = session.apply_command(
        &id,
        0,
        SessionCommand::SetPartMix {
            part_id: part.clone(),
            gain: 101,
            pan: 0,
            mute: false,
            solo: false,
        },
    );
    assert!(invalid.is_err());
    assert_eq!(session.projection().revision, 0);
    session
        .apply_command(
            &id,
            0,
            SessionCommand::SetPartMix {
                part_id: part.clone(),
                gain: 65,
                pan: -30,
                mute: false,
                solo: true,
            },
        )
        .unwrap();
    let next = session
        .apply_command(
            &id,
            1,
            SessionCommand::StudioEvent {
                voice_id: initial.document.voices[0].id.clone(),
                position: Rational::from_i64(0),
                extent: Rational::from_i64(1),
                pitch: Some(Pitch::Degree {
                    context_id: initial.document.pitch_systems[0].id().into(),
                    degree: "1".into(),
                    register: 0,
                }),
            },
        )
        .unwrap();
    let event_id = next.document.events[0].id().to_owned();
    let marked = session
        .apply_command(
            &id,
            2,
            SessionCommand::AnnotateEvent {
                event_id,
                text: "mf & expressive".into(),
                velocity: 85,
            },
        )
        .unwrap();
    assert_eq!(studio::mixer(&marked.document).unwrap()[&part].gain, 65);
    assert_eq!(
        studio::event_marks(&marked.document.events[0])
            .unwrap()
            .velocity,
        85
    );
    assert!(
        studio::layout(&marked.document, None).unwrap().pages[0]
            .svg
            .contains("mf &amp; expressive")
    );
    let mut reopened = DocumentSession::new();
    let loaded = reopened
        .open_bytes(&serde_json::to_vec(&marked.document).unwrap(), true)
        .unwrap();
    assert_eq!(loaded.document, marked.document);
    let undone = session.apply_command(&id, 3, SessionCommand::Undo).unwrap();
    assert!(
        studio::event_marks(&undone.document.events[0])
            .unwrap()
            .text
            .is_empty()
    );
}

#[test]
fn fractional_input_preserves_six_remaining_dots_in_seven_eighth_bar() {
    let mut doc = studio::new_score();
    studio::configure(
        &mut doc,
        studio::ScoreSettings {
            numerator: 7,
            denominator: 8,
            measures: 1,
            subdivision: 2,
            ..Default::default()
        },
    )
    .unwrap();
    let voice = doc.voices[0].id.clone();
    let context = doc.pitch_systems[0].id().to_owned();
    studio::set_event(
        &mut doc,
        &voice,
        Rational::from_i64(0),
        Rational::new(1, 2).unwrap(),
        Some(Pitch::Degree {
            context_id: context,
            degree: "2".into(),
            register: 0,
        }),
    )
    .unwrap();
    doc.validate().unwrap();
    let layout = studio::layout(&doc, None).unwrap();
    assert_eq!(layout.pages[0].svg.matches("r=\"2.3\"").count(), 6);
    assert_eq!(
        layout.pages[0]
            .cells
            .iter()
            .filter(|c| c.voice_id == voice && c.event_id.is_some())
            .count(),
        1
    );
}

#[test]
fn studio_commands_are_atomic_undoable_and_round_trip() {
    let mut session = DocumentSession::new_studio();
    let initial = session.projection();
    let voice = initial.document.voices[0].id.clone();
    let pitch = Pitch::Degree {
        context_id: initial.document.pitch_systems[0].id().into(),
        degree: "3".into(),
        register: 1,
    };
    let first = session
        .apply_command(
            &initial.session_id,
            0,
            SessionCommand::StudioEvent {
                voice_id: voice.clone(),
                position: Rational::from_i64(0),
                extent: Rational::from_i64(1),
                pitch: Some(pitch),
            },
        )
        .unwrap();
    let failed = session.apply_command(
        &initial.session_id,
        1,
        SessionCommand::StudioEvent {
            voice_id: voice,
            position: Rational::new(1, 2).unwrap(),
            extent: Rational::new(1, 2).unwrap(),
            pitch: None,
        },
    );
    assert!(failed.is_err());
    assert_eq!(session.projection().document, first.document);
    assert_eq!(session.projection().revision, 1);
    session
        .apply_command(&initial.session_id, 1, SessionCommand::Undo)
        .unwrap();
    assert_eq!(session.projection().document, initial.document);
    let restored = session
        .apply_command(&initial.session_id, 2, SessionCommand::Redo)
        .unwrap();
    let bytes = serde_json::to_vec(&restored.document).unwrap();
    let mut reopened = DocumentSession::new();
    let loaded = reopened.open_bytes(&bytes, true).unwrap();
    assert!(loaded.capabilities.can_edit);
    assert_eq!(loaded.document, restored.document);
}

#[test]
fn manual_system_break_preserves_time_and_part_contents() {
    let mut doc = studio::new_score();
    studio::configure(
        &mut doc,
        studio::ScoreSettings {
            system_breaks: vec![1],
            ..Default::default()
        },
    )
    .unwrap();
    let layout = studio::layout(&doc, None).unwrap();
    let voice = &doc.voices[0].id;
    let first = layout.pages[0]
        .cells
        .iter()
        .find(|c| &c.voice_id == voice && c.position == Rational::from_i64(0))
        .unwrap();
    let next = layout.pages[0]
        .cells
        .iter()
        .find(|c| &c.voice_id == voice && c.position == Rational::from_i64(4))
        .unwrap();
    assert!(next.y > first.y + 200.0);
    assert!(doc.events.is_empty());
}

//! Headless document sessions, command transactions, and persistence.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crate::layout::numeric_view_unsupported_reason;
use sandikala_core::{
    EmptyPayload, Event, EventAnchor, Extensions, Metadata, Pitch, PitchSystem, Rational,
    ScoreDocument, SoundPayload, TimeDomain,
};
use sandikala_io::{CodecError, read_json, write_json_atomic};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const HISTORY_LIMIT: usize = 100;
const NUMERIC_VIEW_PROVIDER: &str = "sandikala.numeric-demo.v1";

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionProjection {
    pub session_id: String,
    pub document: ScoreDocument,
    pub revision: u32,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub capabilities: SessionCapabilities,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    pub can_edit: bool,
    pub can_render: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CellValue {
    Degree { degree: u8, register: i32 },
    Rest,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SessionCommand {
    SetCell {
        voice_id: String,
        pulse_index: u32,
        value: CellValue,
    },
    ClearCell {
        voice_id: String,
        pulse_index: u32,
    },
    AddCycle {
        domain_id: String,
    },
    SetMetadata {
        title: String,
        composer: String,
    },
    Undo,
    Redo,
}

#[derive(Debug)]
pub enum SessionError {
    Core(sandikala_core::CoreError),
    Codec(CodecError),
    UnsavedChanges,
    StaleSession { expected: String, actual: String },
    StaleRevision { expected: u32, actual: u32 },
    RevisionLimit,
    HistoryEmpty,
    UnsupportedDocument(String),
    InvalidCommand(String),
    NoSaveDestination,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Codec(error) => write!(formatter, "{error}"),
            Self::UnsavedChanges => formatter.write_str("document has unsaved changes"),
            Self::StaleSession { expected, actual } => write!(
                formatter,
                "stale session {expected}; current session is {actual}"
            ),
            Self::StaleRevision { expected, actual } => write!(
                formatter,
                "stale revision {expected}; current revision is {actual}"
            ),
            Self::RevisionLimit => formatter.write_str("session revision limit reached"),
            Self::HistoryEmpty => formatter.write_str("requested history is empty"),
            Self::UnsupportedDocument(reason) => {
                write!(formatter, "document is read-only: {reason}")
            }
            Self::InvalidCommand(reason) => formatter.write_str(reason),
            Self::NoSaveDestination => formatter.write_str("no save destination was provided"),
        }
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Codec(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sandikala_core::CoreError> for SessionError {
    fn from(error: sandikala_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<CodecError> for SessionError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

#[derive(Clone)]
struct Transaction {
    before: ScoreDocument,
    after: ScoreDocument,
}

/// Mutable document state for UI adapters. Paths and history never enter the score.
pub struct DocumentSession {
    session_id: String,
    document: ScoreDocument,
    baseline: ScoreDocument,
    revision: u32,
    path: Option<PathBuf>,
    undo: VecDeque<Transaction>,
    redo: Vec<Transaction>,
}

impl Default for DocumentSession {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentSession {
    /// Create a clean session containing the empty 16-pulse demo document.
    pub fn new() -> Self {
        let document = empty_demo_document();
        Self::from_valid_document(document, None)
    }

    /// Open an already decoded document without changing its content.
    pub fn from_document(
        document: ScoreDocument,
        path: Option<PathBuf>,
    ) -> Result<Self, SessionError> {
        document.validate()?;
        Ok(Self::from_valid_document(document, path))
    }

    fn from_valid_document(document: ScoreDocument, path: Option<PathBuf>) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            baseline: document.clone(),
            document,
            revision: 0,
            path,
            undo: VecDeque::new(),
            redo: Vec::new(),
        }
    }

    pub fn projection(&self) -> SessionProjection {
        SessionProjection {
            session_id: self.session_id.clone(),
            document: self.document.clone(),
            revision: self.revision,
            dirty: self.document != self.baseline,
            can_undo: !self.undo.is_empty(),
            can_redo: !self.redo.is_empty(),
            capabilities: capabilities_for(&self.document),
        }
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Replace the current score with a fresh empty demo session.
    pub fn new_document(
        &mut self,
        discard_unsaved: bool,
    ) -> Result<SessionProjection, SessionError> {
        self.ensure_discard_allowed(discard_unsaved)?;
        *self = Self::new();
        Ok(self.projection())
    }

    /// Decode, validate, and then replace the current score as one operation.
    pub fn open_document(
        &mut self,
        path: impl AsRef<Path>,
        discard_unsaved: bool,
    ) -> Result<SessionProjection, SessionError> {
        let path = path.as_ref();
        let value: serde_json::Value = read_json(path)?;
        let document = ScoreDocument::from_value(value)?;
        document.validate()?;
        self.replace_opened(document, Some(path.to_path_buf()), discard_unsaved)
    }

    /// Open bytes supplied by a dialog or adapter; useful for non-filesystem callers.
    pub fn open_bytes(
        &mut self,
        bytes: &[u8],
        discard_unsaved: bool,
    ) -> Result<SessionProjection, SessionError> {
        let value: serde_json::Value = sandikala_io::decode_json(bytes)?;
        let document = ScoreDocument::from_value(value)?;
        document.validate()?;
        self.replace_opened(document, None, discard_unsaved)
    }

    fn replace_opened(
        &mut self,
        document: ScoreDocument,
        path: Option<PathBuf>,
        discard_unsaved: bool,
    ) -> Result<SessionProjection, SessionError> {
        self.ensure_discard_allowed(discard_unsaved)?;
        *self = Self::from_valid_document(document, path);
        Ok(self.projection())
    }

    fn ensure_discard_allowed(&self, discard_unsaved: bool) -> Result<(), SessionError> {
        if self.document != self.baseline && !discard_unsaved {
            return Err(SessionError::UnsavedChanges);
        }
        Ok(())
    }

    /// Save atomically. The clean baseline changes only after the write succeeds.
    pub fn save_document(
        &mut self,
        session_id: &str,
        expected_revision: u32,
        save_as: Option<PathBuf>,
    ) -> Result<SessionProjection, SessionError> {
        self.check_version(session_id, expected_revision)?;
        let destination = save_as
            .or_else(|| self.path.clone())
            .ok_or(SessionError::NoSaveDestination)?;
        write_json_atomic(&destination, &self.document)?;
        self.path = Some(destination);
        self.baseline = self.document.clone();
        Ok(self.projection())
    }

    /// Validate and commit one command transaction.
    pub fn apply_command(
        &mut self,
        session_id: &str,
        expected_revision: u32,
        command: SessionCommand,
    ) -> Result<SessionProjection, SessionError> {
        self.check_version(session_id, expected_revision)?;
        match command {
            SessionCommand::Undo => self.undo_once()?,
            SessionCommand::Redo => self.redo_once()?,
            command => self.commit_edit(command)?,
        }
        Ok(self.projection())
    }

    fn commit_edit(&mut self, command: SessionCommand) -> Result<(), SessionError> {
        let capability = capabilities_for(&self.document);
        if !capability.can_edit {
            return Err(SessionError::UnsupportedDocument(
                capability
                    .reason
                    .unwrap_or_else(|| "unsupported view".to_owned()),
            ));
        }

        let mut after = self.document.clone();
        match command {
            SessionCommand::SetCell {
                voice_id,
                pulse_index,
                value,
            } => {
                set_cell(&mut after, &voice_id, pulse_index, value)?;
            }
            SessionCommand::ClearCell {
                voice_id,
                pulse_index,
            } => {
                clear_cell(&mut after, &voice_id, pulse_index)?;
            }
            SessionCommand::AddCycle { domain_id } => add_cycle(&mut after, &domain_id)?,
            SessionCommand::SetMetadata { title, composer } => {
                if title.chars().count() > 256 || composer.chars().count() > 256 {
                    return Err(SessionError::InvalidCommand(
                        "title and composer must each be at most 256 characters".to_owned(),
                    ));
                }
                after.metadata = Metadata { title, composer };
            }
            SessionCommand::Undo | SessionCommand::Redo => unreachable!(),
        }

        after.validate()?;
        if after == self.document {
            return Ok(());
        }
        let revision = self.next_revision()?;
        self.push_undo(Transaction {
            before: self.document.clone(),
            after: after.clone(),
        });
        self.redo.clear();
        self.document = after;
        self.revision = revision;
        Ok(())
    }

    fn undo_once(&mut self) -> Result<(), SessionError> {
        let transaction = self.undo.back().ok_or(SessionError::HistoryEmpty)?;
        if transaction.after != self.document {
            return Err(SessionError::InvalidCommand(
                "undo history is inconsistent".to_owned(),
            ));
        }
        let revision = self.next_revision()?;
        let transaction = self.undo.pop_back().expect("checked above");
        self.document = transaction.before.clone();
        self.redo.push(transaction);
        self.revision = revision;
        Ok(())
    }

    fn redo_once(&mut self) -> Result<(), SessionError> {
        let transaction = self.redo.last().ok_or(SessionError::HistoryEmpty)?;
        if transaction.before != self.document {
            return Err(SessionError::InvalidCommand(
                "redo history is inconsistent".to_owned(),
            ));
        }
        let revision = self.next_revision()?;
        let transaction = self.redo.pop().expect("checked above");
        self.document = transaction.after.clone();
        self.undo.push_back(transaction);
        self.revision = revision;
        Ok(())
    }

    fn push_undo(&mut self, transaction: Transaction) {
        if self.undo.len() == HISTORY_LIMIT {
            self.undo.pop_front();
        }
        self.undo.push_back(transaction);
    }

    fn next_revision(&self) -> Result<u32, SessionError> {
        self.revision
            .checked_add(1)
            .ok_or(SessionError::RevisionLimit)
    }

    fn check_version(&self, session_id: &str, revision: u32) -> Result<(), SessionError> {
        if session_id != self.session_id {
            return Err(SessionError::StaleSession {
                expected: session_id.to_owned(),
                actual: self.session_id.clone(),
            });
        }
        if revision != self.revision {
            return Err(SessionError::StaleRevision {
                expected: revision,
                actual: self.revision,
            });
        }
        Ok(())
    }
}

fn empty_demo_document() -> ScoreDocument {
    let id = || Uuid::new_v4().to_string();
    let part_id = id();
    let voice_id = id();
    let domain_id = id();
    let context_id = id();
    let view_id = id();
    let degrees: Vec<_> = (1..=7).map(|degree| degree.to_string()).collect();
    let tuning_values = [220.0, 247.0, 270.0, 300.0, 330.0, 360.0, 400.0];
    let tuning_hz = degrees
        .iter()
        .enumerate()
        .map(|(index, degree)| (degree.clone(), tuning_values[index]))
        .collect();
    ScoreDocument {
        schema_version: sandikala_core::SCHEMA_VERSION.to_owned(),
        id: id(),
        metadata: Metadata {
            title: "Untitled".to_owned(),
            composer: String::new(),
        },
        parts: vec![sandikala_core::Part {
            id: part_id.clone(),
            name: "Saron (demo)".to_owned(),
            instrument_ref: sandikala_core::InstrumentRef {
                id: "sandikala.demo.saron".to_owned(),
                version: "1".to_owned(),
            },
            voice_ids: vec![voice_id.clone()],
        }],
        voices: vec![sandikala_core::Voice {
            id: voice_id,
            part_id: part_id.clone(),
            time_domain_id: domain_id.clone(),
        }],
        time_domains: vec![TimeDomain::Cycle {
            id: domain_id,
            unit: "pulse".to_owned(),
            cycle_length: 16,
            cycle_count: 1,
        }],
        pitch_systems: vec![PitchSystem::Degree {
            id: context_id,
            degrees,
            tuning_hz,
        }],
        events: Vec::new(),
        relations: Vec::new(),
        notation_views: vec![sandikala_core::NotationView {
            id: view_id,
            provider_id: NUMERIC_VIEW_PROVIDER.to_owned(),
            part_ids: vec![part_id],
            settings: serde_json::Value::Object(Default::default()),
        }],
        extensions: Extensions::default(),
    }
}

fn capabilities_for(document: &ScoreDocument) -> SessionCapabilities {
    let reason = numeric_view_unsupported_reason(document);
    SessionCapabilities {
        can_edit: reason.is_none(),
        can_render: reason.is_none(),
        reason,
    }
}

fn cell_position(pulse_index: u32) -> Rational {
    Rational::from_i64(i64::from(pulse_index))
}

fn set_cell(
    document: &mut ScoreDocument,
    voice_id: &str,
    pulse_index: u32,
    value: CellValue,
) -> Result<(), SessionError> {
    let voice = document
        .voices
        .iter()
        .find(|voice| voice.id == voice_id)
        .ok_or_else(|| SessionError::InvalidCommand(format!("unknown voice '{voice_id}'")))?;
    let domain_id = voice.time_domain_id.clone();
    let domain_limit = match document
        .time_domains
        .iter()
        .find(|domain| domain.id() == domain_id)
    {
        Some(TimeDomain::Cycle {
            cycle_length,
            cycle_count,
            ..
        }) => cycle_length * cycle_count,
        _ => {
            return Err(SessionError::InvalidCommand(
                "voice does not use a cycle domain".to_owned(),
            ));
        }
    };
    if pulse_index >= domain_limit {
        return Err(SessionError::InvalidCommand(
            "pulse index is outside the cycle extent".to_owned(),
        ));
    }
    let context_id = document
        .pitch_systems
        .iter()
        .find_map(|system| match system {
            PitchSystem::Degree { id, .. } => Some(id.clone()),
            PitchSystem::EqualTemperament { .. } => None,
        })
        .ok_or_else(|| {
            SessionError::InvalidCommand("numeric degree context is unavailable".to_owned())
        })?;
    let replacement_kind = match value {
        CellValue::Degree { degree, register } => {
            if !(1..=7).contains(&degree) || !(-1..=1).contains(&register) {
                return Err(SessionError::InvalidCommand(
                    "degree must be 1 through 7 and register -1, 0, or 1".to_owned(),
                ));
            }
            Some(Pitch::Degree {
                context_id,
                degree: degree.to_string(),
                register,
            })
        }
        CellValue::Rest => None,
    };
    let position = cell_position(pulse_index);
    let existing = document.events.iter_mut().find(|event| {
        event.voice_id() == voice_id
            && event.anchor().domain_id == domain_id
            && event.anchor().position == position
    });
    if let Some(event) = existing {
        let id = event.id().to_owned();
        let extensions = match event {
            Event::Rest { extensions, .. } | Event::Sound { extensions, .. } => extensions.clone(),
        };
        let anchor = EventAnchor {
            domain_id,
            position,
        };
        *event = match replacement_kind {
            Some(pitch) => Event::Sound {
                id,
                voice_id: voice_id.to_owned(),
                anchor,
                extent: Rational::from_i64(1),
                payload: SoundPayload { pitch },
                extensions,
            },
            None => Event::Rest {
                id,
                voice_id: voice_id.to_owned(),
                anchor,
                extent: Rational::from_i64(1),
                payload: EmptyPayload {},
                extensions,
            },
        };
    } else {
        let id = Uuid::new_v4().to_string();
        let anchor = EventAnchor {
            domain_id,
            position,
        };
        let extensions = Extensions::default();
        document.events.push(match replacement_kind {
            Some(pitch) => Event::Sound {
                id,
                voice_id: voice_id.to_owned(),
                anchor,
                extent: Rational::from_i64(1),
                payload: SoundPayload { pitch },
                extensions,
            },
            None => Event::Rest {
                id,
                voice_id: voice_id.to_owned(),
                anchor,
                extent: Rational::from_i64(1),
                payload: EmptyPayload {},
                extensions,
            },
        });
    }
    Ok(())
}

fn clear_cell(
    document: &mut ScoreDocument,
    voice_id: &str,
    pulse_index: u32,
) -> Result<(), SessionError> {
    let voice = document
        .voices
        .iter()
        .find(|voice| voice.id == voice_id)
        .ok_or_else(|| SessionError::InvalidCommand(format!("unknown voice '{voice_id}'")))?;
    let domain_id = voice.time_domain_id.clone();
    let domain_limit = match document
        .time_domains
        .iter()
        .find(|domain| domain.id() == domain_id)
    {
        Some(TimeDomain::Cycle {
            cycle_length,
            cycle_count,
            ..
        }) => cycle_length * cycle_count,
        _ => {
            return Err(SessionError::InvalidCommand(
                "voice does not use a cycle domain".to_owned(),
            ));
        }
    };
    if pulse_index >= domain_limit {
        return Err(SessionError::InvalidCommand(
            "pulse index is outside the cycle extent".to_owned(),
        ));
    }
    let position = cell_position(pulse_index);
    document.events.retain(|event| {
        !(event.voice_id() == voice_id
            && event.anchor().domain_id == domain_id
            && event.anchor().position == position)
    });
    Ok(())
}

fn add_cycle(document: &mut ScoreDocument, domain_id: &str) -> Result<(), SessionError> {
    let domain = document
        .time_domains
        .iter_mut()
        .find(|domain| domain.id() == domain_id)
        .ok_or_else(|| {
            SessionError::InvalidCommand(format!("unknown time domain '{domain_id}'"))
        })?;
    match domain {
        TimeDomain::Cycle { cycle_count, .. } if *cycle_count < 64 => *cycle_count += 1,
        TimeDomain::Cycle { .. } => {
            return Err(SessionError::InvalidCommand(
                "cycle count cannot exceed 64".to_owned(),
            ));
        }
        TimeDomain::Metric { .. } => {
            return Err(SessionError::InvalidCommand(
                "cannot add a cycle to a metric domain".to_owned(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutResult, layout_document};
    use sandikala_core::{EqualTemperamentReference, PitchLetter};

    fn fixture(name: &str) -> ScoreDocument {
        let path = format!("{}/../../fixtures/q1/{name}", env!("CARGO_MANIFEST_DIR"));
        let document: ScoreDocument = read_json(path).expect("fixture decodes");
        document.validate().expect("fixture validates");
        document
    }

    fn apply(session: &mut DocumentSession, command: SessionCommand) -> SessionProjection {
        let state = session.projection();
        session
            .apply_command(&state.session_id, state.revision, command)
            .expect("command succeeds")
    }

    #[test]
    fn new_session_has_empty_demo_and_unique_ids() {
        let first = DocumentSession::new();
        let second = DocumentSession::new();
        let first_state = first.projection();
        assert_eq!(first_state.document.metadata.title, "Untitled");
        assert!(first_state.document.events.is_empty());
        assert_eq!(first_state.revision, 0);
        assert!(!first_state.dirty);
        assert!(first_state.capabilities.can_edit);
        assert_ne!(first_state.document.id, second.projection().document.id);
        assert_ne!(first_state.session_id, second.projection().session_id);
    }

    #[test]
    fn edits_are_transactional_preserve_extensions_and_undo_to_clean_baseline() {
        let mut document =
            DocumentSession::from_document(fixture("gamelan-basic.sdlk"), None).unwrap();
        document
            .document
            .extensions
            .insert("vendor.test".into(), serde_json::json!({"a": 1}));
        document.baseline = document.document.clone();
        let old_event_id = document.document.events[0].id().to_owned();
        if let Event::Sound { extensions, .. } = &mut document.document.events[0] {
            extensions.insert("vendor.event".into(), serde_json::json!([1, 2]));
        }
        document.baseline = document.document.clone();
        let state = document.projection();
        let changed = document
            .apply_command(
                &state.session_id,
                state.revision,
                SessionCommand::SetCell {
                    voice_id: "voice-saron".into(),
                    pulse_index: 0,
                    value: CellValue::Degree {
                        degree: 7,
                        register: -1,
                    },
                },
            )
            .unwrap();
        assert!(changed.dirty);
        assert_eq!(changed.document.events[0].id(), old_event_id);
        assert_eq!(
            changed.document.extensions["vendor.test"],
            serde_json::json!({"a": 1})
        );
        let extensions = match &changed.document.events[0] {
            Event::Rest { extensions, .. } | Event::Sound { extensions, .. } => extensions,
        };
        assert_eq!(extensions["vendor.event"], serde_json::json!([1, 2]));
        let undone = apply(&mut document, SessionCommand::Undo);
        assert!(!undone.dirty);
        assert!(undone.can_redo);
        let redone = apply(&mut document, SessionCommand::Redo);
        assert!(redone.dirty);
    }

    #[test]
    fn no_op_and_rejected_command_do_not_advance_revision() {
        let mut session = DocumentSession::new();
        let state = session.projection();
        let same = session
            .apply_command(
                &state.session_id,
                state.revision,
                SessionCommand::SetMetadata {
                    title: "Untitled".into(),
                    composer: String::new(),
                },
            )
            .unwrap();
        assert_eq!(same.revision, 0);
        assert!(!same.can_undo);
        let error = session
            .apply_command(
                &same.session_id,
                same.revision,
                SessionCommand::SetCell {
                    voice_id: session.document.voices[0].id.clone(),
                    pulse_index: 99,
                    value: CellValue::Rest,
                },
            )
            .unwrap_err();
        assert!(matches!(error, SessionError::InvalidCommand(_)));
        assert_eq!(session.projection().revision, 0);
        assert!(!session.projection().dirty);
    }

    #[test]
    fn clear_empty_is_no_op_and_stale_session_or_revision_is_rejected() {
        let mut session = DocumentSession::new();
        let state = session.projection();
        let cleared = session
            .apply_command(
                &state.session_id,
                state.revision,
                SessionCommand::ClearCell {
                    voice_id: state.document.voices[0].id.clone(),
                    pulse_index: 0,
                },
            )
            .unwrap();
        assert_eq!(cleared.revision, 0);
        assert!(matches!(
            session.apply_command("old-session", 0, SessionCommand::Undo),
            Err(SessionError::StaleSession { .. })
        ));
        assert!(matches!(
            session.apply_command(&state.session_id, 1, SessionCommand::Undo),
            Err(SessionError::StaleRevision { .. })
        ));
    }

    #[test]
    fn cycle_edit_clear_undo_redo_and_discard_gate_use_global_pulses() {
        let mut session = DocumentSession::new();
        let initial = session.projection();
        let domain_id = initial.document.time_domains[0].id().to_owned();
        let expanded = session
            .apply_command(
                &initial.session_id,
                initial.revision,
                SessionCommand::AddCycle { domain_id },
            )
            .unwrap();
        assert!(matches!(
            &expanded.document.time_domains[0],
            TimeDomain::Cycle { cycle_count: 2, .. }
        ));
        let voice_id = expanded.document.voices[0].id.clone();
        let filled = session
            .apply_command(
                &expanded.session_id,
                expanded.revision,
                SessionCommand::SetCell {
                    voice_id: voice_id.clone(),
                    pulse_index: 16,
                    value: CellValue::Degree {
                        degree: 3,
                        register: 0,
                    },
                },
            )
            .unwrap();
        assert_eq!(
            filled.document.events[0].anchor().position,
            Rational::from_i64(16)
        );
        let cleared = session
            .apply_command(
                &filled.session_id,
                filled.revision,
                SessionCommand::ClearCell {
                    voice_id,
                    pulse_index: 16,
                },
            )
            .unwrap();
        assert!(cleared.document.events.is_empty());
        let restored = apply(&mut session, SessionCommand::Undo);
        assert_eq!(
            restored.document.events[0].anchor().position,
            Rational::from_i64(16)
        );
        assert!(
            apply(&mut session, SessionCommand::Redo)
                .document
                .events
                .is_empty()
        );

        let dirty = apply(
            &mut session,
            SessionCommand::SetMetadata {
                title: "Changed".into(),
                composer: String::new(),
            },
        );
        assert!(matches!(
            session.new_document(false),
            Err(SessionError::UnsavedChanges)
        ));
        assert_eq!(session.projection().document, dirty.document);
        let replaced = session.new_document(true).unwrap();
        assert_ne!(replaced.session_id, dirty.session_id);
        assert_eq!(replaced.revision, 0);
        assert!(!replaced.dirty);
    }

    #[test]
    fn unsupported_valid_document_opens_read_only_without_rewriting() {
        let piano = fixture("piano-model.sdlk");
        let mut session = DocumentSession::from_document(piano.clone(), None).unwrap();
        let before = session.projection();
        assert!(!before.capabilities.can_edit);
        assert!(!before.capabilities.can_render);
        assert!(matches!(
            session.apply_command(
                &before.session_id,
                before.revision,
                SessionCommand::SetMetadata {
                    title: "Changed".into(),
                    composer: "".into()
                }
            ),
            Err(SessionError::UnsupportedDocument(_))
        ));
        assert_eq!(session.projection().document, piano);
    }

    #[test]
    fn numeric_capabilities_and_layout_reject_spelled_pitch_events_together() {
        let mut document = fixture("gamelan-basic.sdlk");
        document.pitch_systems.insert(
            0,
            PitchSystem::EqualTemperament {
                id: "equal-temperament".to_owned(),
                steps_per_octave: 12,
                reference: EqualTemperamentReference {
                    letter: PitchLetter::A,
                    octave: 4,
                    frequency_hz: 440.0,
                },
            },
        );
        if let Event::Sound { payload, .. } = &mut document.events[0] {
            payload.pitch = Pitch::Spelled {
                context_id: "equal-temperament".to_owned(),
                letter: PitchLetter::C,
                alteration: 0,
                octave: 4,
            };
        } else {
            panic!("fixture pulse zero is a sound");
        }
        document
            .validate()
            .expect("spelled event is valid core data");

        let mut session = DocumentSession::from_document(document.clone(), None).unwrap();
        let state = session.projection();
        assert!(!state.capabilities.can_edit);
        assert!(!state.capabilities.can_render);
        assert!(matches!(
            layout_document(&document, "view-numeric").unwrap(),
            LayoutResult::UnsupportedView { .. }
        ));
        assert!(matches!(
            session.apply_command(
                &state.session_id,
                state.revision,
                SessionCommand::SetCell {
                    voice_id: "voice-saron".to_owned(),
                    pulse_index: 0,
                    value: CellValue::Degree {
                        degree: 2,
                        register: 0,
                    },
                }
            ),
            Err(SessionError::UnsupportedDocument(_))
        ));
        assert_eq!(session.projection().document, document);
        assert_eq!(session.projection().revision, 0);
    }

    #[test]
    fn numeric_view_accepts_an_unused_equal_temperament_context() {
        let mut document = fixture("gamelan-basic.sdlk");
        document.pitch_systems.push(PitchSystem::EqualTemperament {
            id: "equal-temperament".to_owned(),
            steps_per_octave: 12,
            reference: EqualTemperamentReference {
                letter: PitchLetter::A,
                octave: 4,
                frequency_hz: 440.0,
            },
        });
        document.validate().expect("both pitch contexts are valid");
        let mut session = DocumentSession::from_document(document.clone(), None).unwrap();
        assert!(session.projection().capabilities.can_edit);
        assert!(matches!(
            layout_document(&document, "view-numeric").unwrap(),
            LayoutResult::Svg { .. }
        ));
        let updated = apply(
            &mut session,
            SessionCommand::SetCell {
                voice_id: "voice-saron".to_owned(),
                pulse_index: 0,
                value: CellValue::Degree {
                    degree: 2,
                    register: 0,
                },
            },
        );
        assert!(matches!(
            &updated.document.events[0],
            Event::Sound {
                payload: SoundPayload {
                    pitch: Pitch::Degree { context_id, .. },
                },
                ..
            } if context_id == "ensemble-demo"
        ));
    }

    #[test]
    fn both_fixtures_round_trip_and_failed_open_or_save_preserves_state() {
        for name in ["gamelan-basic.sdlk", "piano-model.sdlk"] {
            let document = fixture(name);
            let bytes = serde_json::to_vec(&document).unwrap();
            let mut session = DocumentSession::new();
            let opened = session.open_bytes(&bytes, true).unwrap();
            assert_eq!(opened.document, document);
            let path =
                std::env::temp_dir().join(format!("sandikala-session-{}.sdlk", Uuid::new_v4()));
            session
                .save_document(&opened.session_id, opened.revision, Some(path.clone()))
                .expect("save fixture");
            let mut reopened = DocumentSession::new();
            let state = reopened.open_document(&path, true).expect("reopen fixture");
            assert_eq!(state.document, document);
            std::fs::remove_file(path).expect("remove temporary fixture");
        }

        let mut session = DocumentSession::new();
        let initial = session.projection();
        let dirty = session
            .apply_command(
                &initial.session_id,
                initial.revision,
                SessionCommand::SetMetadata {
                    title: "Changed".into(),
                    composer: String::new(),
                },
            )
            .unwrap();
        assert!(matches!(
            session.open_bytes(b"not json", true),
            Err(SessionError::Codec(_))
        ));
        assert_eq!(session.projection().document, dirty.document);
        let directory = std::env::temp_dir();
        assert!(matches!(
            session.save_document(&dirty.session_id, dirty.revision, Some(directory)),
            Err(SessionError::Codec(_))
        ));
        assert!(session.projection().dirty);
    }
}

use std::path::PathBuf;
use std::sync::Mutex;

use sandikala_engine::layout::{LayoutResult, layout_document};
use sandikala_engine::session::{DocumentSession, SessionCommand, SessionError, SessionProjection};
use sandikala_io::{CodecError, write_bytes_atomic};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

mod saron_playback;
mod score_audio;
use saron_playback::SaronPlayer;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandError {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ids: Vec<String>,
}

impl CommandError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            field: None,
            ids: Vec::new(),
        }
    }

    fn io(message: impl Into<String>) -> Self {
        Self::new("IoError", message)
    }
}

impl From<SessionError> for CommandError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::Core(error) => Self {
                code: format!("{:?}", error.code),
                message: error.message,
                field: error.field,
                ids: error.ids,
            },
            SessionError::Codec(error) => codec_error(error),
            SessionError::UnsavedChanges => {
                Self::new("UnsavedChanges", "document has unsaved changes")
            }
            SessionError::StaleSession { expected, actual } => Self::new(
                "StaleSession",
                format!("stale session {expected}; current session is {actual}"),
            ),
            SessionError::StaleRevision { expected, actual } => Self::new(
                "StaleRevision",
                format!("stale revision {expected}; current revision is {actual}"),
            ),
            SessionError::RevisionLimit => {
                Self::new("ArithmeticLimit", "session revision limit reached")
            }
            SessionError::HistoryEmpty => Self::new("HistoryEmpty", "requested history is empty"),
            SessionError::UnsupportedDocument(reason) => Self::new("UnsupportedView", reason),
            SessionError::InvalidCommand(message) => Self::new("InvalidDocument", message),
            SessionError::NoSaveDestination => Self::io("no save destination was provided"),
        }
    }
}

fn codec_error(error: CodecError) -> CommandError {
    match error {
        error @ (CodecError::InputTooLarge { .. } | CodecError::JsonDepthExceeded { .. }) => {
            CommandError::new("InputLimit", error.to_string())
        }
        error @ CodecError::Parse(_) => CommandError::new("InvalidDocument", error.to_string()),
        error @ (CodecError::Io(_) | CodecError::Serialize(_) | CodecError::AtomicWrite(_)) => {
            CommandError::io(error.to_string())
        }
    }
}

type SessionState = Mutex<DocumentSession>;
type SaronPlaybackState = Mutex<SaronPlayer>;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CloseRequest {
    request_id: String,
    session_id: String,
    revision: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CloseAuthorization {
    session_id: String,
    revision: u32,
}

#[derive(Default)]
struct CloseState {
    pending: Option<CloseRequest>,
    authorized: Option<CloseAuthorization>,
}

struct ManagedCloseState(Mutex<CloseState>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CloseResolution {
    status: &'static str,
}

fn lock_session<'a>(
    state: &'a State<'a, SessionState>,
) -> Result<std::sync::MutexGuard<'a, DocumentSession>, CommandError> {
    state
        .lock()
        .map_err(|_| CommandError::io("document session is unavailable"))
}

fn ensure_version(
    projection: &SessionProjection,
    session_id: &str,
    revision: u32,
) -> Result<(), CommandError> {
    if projection.session_id != session_id {
        return Err(CommandError::new(
            "StaleSession",
            format!(
                "stale session {session_id}; current session is {}",
                projection.session_id
            ),
        ));
    }
    if projection.revision != revision {
        return Err(CommandError::new(
            "StaleRevision",
            format!(
                "stale revision {revision}; current revision is {}",
                projection.revision
            ),
        ));
    }
    Ok(())
}

#[tauri::command]
fn get_state(state: State<'_, SessionState>) -> Result<SessionProjection, CommandError> {
    Ok(lock_session(&state)?.projection())
}

#[tauri::command(rename_all = "camelCase")]
async fn play_saron(
    app: AppHandle,
    state: State<'_, SessionState>,
    session_id: String,
    expected_revision: u32,
    soundfont_path: String,
    pulse_bpm: u32,
) -> Result<(), CommandError> {
    let document = {
        let session = lock_session(&state)?;
        let projection = session.projection();
        ensure_version(&projection, &session_id, expected_revision)?;
        projection.document
    };
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<SaronPlaybackState>()
            .lock()
            .map_err(|_| CommandError::io("audio playback is unavailable"))?
            .play(&document, PathBuf::from(soundfont_path), pulse_bpm)
            .map_err(|message| CommandError::new("PlaybackError", message))
    })
    .await
    .map_err(|error| CommandError::new("PlaybackError", error.to_string()))?
}

#[tauri::command(rename_all = "camelCase")]
fn find_saron_soundfont(app: AppHandle, preferred_path: Option<String>) -> Option<String> {
    use saron_playback::SARON_SOUNDFONT_NAME;
    let mut candidates = Vec::new();
    if let Some(path) = preferred_path {
        candidates.push(PathBuf::from(path));
    }
    if let Some(path) = std::env::var_os("SANDIKALA_SARON_SF2") {
        candidates.push(PathBuf::from(path));
    }
    for directory in [app.path().resource_dir(), app.path().app_data_dir()]
        .into_iter()
        .flatten()
    {
        candidates.push(directory.join("soundfont").join(SARON_SOUNDFONT_NAME));
    }
    // Development checkouts share the existing MuseScore sound library.
    if let Some(dev_directory) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
    {
        candidates.push(
            dev_directory
                .join("MuseScore/soundfont")
                .join(SARON_SOUNDFONT_NAME),
        );
    }
    let resolved =
        saron_playback::find_soundfont(candidates).map(|path| path.to_string_lossy().into_owned());
    #[cfg(debug_assertions)]
    eprintln!(
        "Saron sound asset: {}",
        resolved.as_deref().unwrap_or("missing")
    );
    resolved
}

#[tauri::command]
fn saron_playback_status(
    playback: State<'_, SaronPlaybackState>,
) -> Result<saron_playback::PlaybackStatus, CommandError> {
    playback
        .lock()
        .map_err(|_| CommandError::io("audio playback is unavailable"))?
        .status()
        .map_err(|message| CommandError::new("PlaybackError", message))
}

#[tauri::command]
fn stop_saron(playback: State<'_, SaronPlaybackState>) -> Result<(), CommandError> {
    playback
        .lock()
        .map_err(|_| CommandError::io("audio playback is unavailable"))?
        .stop()
        .map_err(|message| CommandError::new("PlaybackError", message))
}

#[tauri::command]
fn pause_playback(
    playback: State<'_, SaronPlaybackState>,
    paused: bool,
) -> Result<(), CommandError> {
    playback
        .lock()
        .map_err(|_| CommandError::io("Audio unavailable"))?
        .pause(paused)
        .map_err(|e| CommandError::new("PlaybackError", e))
}
#[tauri::command]
fn seek_playback(playback: State<'_, SaronPlaybackState>, pulse: f64) -> Result<(), CommandError> {
    playback
        .lock()
        .map_err(|_| CommandError::io("Audio unavailable"))?
        .seek(pulse)
        .map_err(|e| CommandError::new("PlaybackError", e))
}

#[tauri::command]
fn saron_playback_active(playback: State<'_, SaronPlaybackState>) -> Result<bool, CommandError> {
    playback
        .lock()
        .map_err(|_| CommandError::io("audio playback is unavailable"))?
        .is_playing()
        .map_err(|message| CommandError::new("PlaybackError", message))
}

#[tauri::command(rename_all = "camelCase")]
fn new_document(
    state: State<'_, SessionState>,
    discard_unsaved: bool,
) -> Result<SessionProjection, CommandError> {
    lock_session(&state)?
        .new_document(discard_unsaved)
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
async fn open_document(
    app: AppHandle,
    state: State<'_, SessionState>,
    discard_unsaved: bool,
) -> Result<Option<SessionProjection>, CommandError> {
    let before_dialog = lock_session(&state)?.projection();
    if before_dialog.dirty && !discard_unsaved {
        return Err(CommandError::new(
            "UnsavedChanges",
            "document has unsaved changes",
        ));
    }

    let Some(path) = app
        .dialog()
        .file()
        .add_filter("Sandikala score", &["sdlk"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|error| CommandError::io(error.to_string()))?;

    let mut session = lock_session(&state)?;
    ensure_version(
        &session.projection(),
        &before_dialog.session_id,
        before_dialog.revision,
    )?;
    session
        .open_document(path, discard_unsaved)
        .map(Some)
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
fn apply_command(
    state: State<'_, SessionState>,
    session_id: String,
    expected_revision: u32,
    command: SessionCommand,
) -> Result<SessionProjection, CommandError> {
    lock_session(&state)?
        .apply_command(&session_id, expected_revision, command)
        .map_err(Into::into)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaggedLayout {
    session_id: String,
    revision: u32,
    layout: LayoutResult,
}

fn current_layout(
    state: &State<'_, SessionState>,
    session_id: &str,
    revision: u32,
    view_id: &str,
) -> Result<TaggedLayout, CommandError> {
    let session = lock_session(state)?;
    let projection = session.projection();
    ensure_version(&projection, session_id, revision)?;
    let layout = layout_document(&projection.document, view_id).map_err(|error| CommandError {
        code: format!("{:?}", error.code),
        message: error.message,
        field: error.field,
        ids: error.ids,
    })?;
    Ok(TaggedLayout {
        session_id: projection.session_id,
        revision: projection.revision,
        layout,
    })
}

#[tauri::command(rename_all = "camelCase")]
fn get_layout(
    state: State<'_, SessionState>,
    session_id: String,
    revision: u32,
    view_id: String,
) -> Result<TaggedLayout, CommandError> {
    current_layout(&state, &session_id, revision, &view_id)
}

#[tauri::command(rename_all = "camelCase")]
async fn save_document(
    app: AppHandle,
    state: State<'_, SessionState>,
    session_id: String,
    expected_revision: u32,
    save_as: bool,
) -> Result<Option<SessionProjection>, CommandError> {
    let current_path = {
        let session = lock_session(&state)?;
        ensure_version(&session.projection(), &session_id, expected_revision)?;
        if save_as {
            None
        } else {
            session.current_path().map(PathBuf::from)
        }
    };
    let destination = match current_path {
        Some(path) => path,
        None => {
            let Some(path) = app
                .dialog()
                .file()
                .add_filter("Sandikala score", &["sdlk"])
                .blocking_save_file()
            else {
                return Ok(None);
            };
            path.into_path()
                .map_err(|error| CommandError::io(error.to_string()))?
        }
    };

    lock_session(&state)?
        .save_document(&session_id, expected_revision, Some(destination))
        .map(Some)
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
fn resolve_close_request(
    state: State<'_, SessionState>,
    close_state: State<'_, ManagedCloseState>,
    request_id: String,
    action: String,
    session_id: String,
    expected_revision: u32,
) -> Result<CloseResolution, CommandError> {
    let session = lock_session(&state)?;
    let projection = session.projection();
    ensure_version(&projection, &session_id, expected_revision)?;

    let mut close_state = close_state
        .0
        .lock()
        .map_err(|_| CommandError::io("close confirmation is unavailable"))?;
    let request = close_state
        .pending
        .as_ref()
        .ok_or_else(|| CommandError::new("StaleSession", "close request is no longer active"))?;
    if request.request_id != request_id || request.session_id != session_id {
        return Err(CommandError::new(
            "StaleSession",
            "close request no longer matches the current document session",
        ));
    }

    match action.as_str() {
        "cancel" => {
            close_state.pending = None;
            close_state.authorized = None;
            Ok(CloseResolution {
                status: "cancelled",
            })
        }
        "save" if projection.dirty => Err(CommandError::new(
            "UnsavedChanges",
            "save the document before confirming close",
        )),
        "save" | "discard" => {
            close_state.pending = None;
            close_state.authorized = Some(CloseAuthorization {
                session_id: projection.session_id,
                revision: projection.revision,
            });
            Ok(CloseResolution {
                status: "closeAuthorized",
            })
        }
        _ => Err(CommandError::new(
            "InvalidDocument",
            "close action must be save, discard, or cancel",
        )),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportResult {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[tauri::command(rename_all = "camelCase")]
async fn export_svg(
    app: AppHandle,
    state: State<'_, SessionState>,
    session_id: String,
    revision: u32,
    view_id: String,
) -> Result<ExportResult, CommandError> {
    let tagged = current_layout(&state, &session_id, revision, &view_id)?;
    let svg = match tagged.layout {
        LayoutResult::Pages { layout } => studio_svg(&layout),
        LayoutResult::Svg { svg, .. } => svg,
        LayoutResult::UnsupportedView { reason, .. } => {
            return Err(CommandError::new("UnsupportedView", reason));
        }
    };
    let Some(path) = app
        .dialog()
        .file()
        .add_filter("SVG image", &["svg"])
        .blocking_save_file()
    else {
        return Ok(ExportResult {
            status: "cancelled",
            path: None,
        });
    };
    let path = path
        .into_path()
        .map_err(|error| CommandError::io(error.to_string()))?;
    ensure_version(&lock_session(&state)?.projection(), &session_id, revision)?;
    write_bytes_atomic(&path, svg.as_bytes()).map_err(codec_error)?;
    Ok(ExportResult {
        status: "exported",
        path: Some(path.to_string_lossy().into_owned()),
    })
}

fn studio_svg(layout: &sandikala_engine::studio::PageLayout) -> String {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><style>{}</style>",
        layout.width,
        layout.height as usize * layout.pages.len(),
        layout.font_css
    );
    for (index, page) in layout.pages.iter().enumerate() {
        svg.push_str(&format!(
            "<g transform=\"translate(0 {})\">{}</g>",
            index * layout.height as usize,
            page.svg
        ));
    }
    svg.push_str("</svg>");
    svg
}

#[tauri::command(rename_all = "camelCase")]
fn get_studio_layout(
    state: State<'_, SessionState>,
    session_id: String,
    revision: u32,
    part_id: Option<String>,
) -> Result<TaggedLayout, CommandError> {
    let projection = lock_session(&state)?.projection();
    ensure_version(&projection, &session_id, revision)?;
    let layout = sandikala_engine::studio::layout(&projection.document, part_id.as_deref())
        .map_err(|e| CommandError::new("UnsupportedView", e))?;
    Ok(TaggedLayout {
        session_id,
        revision,
        layout: LayoutResult::Pages { layout },
    })
}

#[tauri::command(rename_all = "camelCase")]
async fn export_score(
    app: AppHandle,
    state: State<'_, SessionState>,
    session_id: String,
    revision: u32,
    part_id: Option<String>,
    format: String,
) -> Result<ExportResult, CommandError> {
    let tagged = get_studio_layout(state.clone(), session_id.clone(), revision, part_id)?;
    let LayoutResult::Pages { layout } = tagged.layout else {
        return Err(CommandError::new(
            "UnsupportedView",
            "Score pages unavailable",
        ));
    };
    let (contents, extension) = match format.as_str() {
        "svg" => (studio_svg(&layout).into_bytes(), "svg"),
        "html" => (
            sandikala_engine::studio::printable_html(&layout).into_bytes(),
            "html",
        ),
        "pdf" => (
            sandikala_engine::publishing::pdf(&layout).map_err(CommandError::io)?,
            "pdf",
        ),
        _ => {
            return Err(CommandError::new(
                "InvalidDocument",
                "Choose SVG, PDF, or printable HTML",
            ));
        }
    };
    let Some(path) = app
        .dialog()
        .file()
        .add_filter("Score pages", &[extension])
        .blocking_save_file()
    else {
        return Ok(ExportResult {
            status: "cancelled",
            path: None,
        });
    };
    let path = path
        .into_path()
        .map_err(|e| CommandError::io(e.to_string()))?;
    ensure_version(&lock_session(&state)?.projection(), &session_id, revision)?;
    write_bytes_atomic(&path, &contents).map_err(codec_error)?;
    Ok(ExportResult {
        status: "exported",
        path: Some(path.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
fn print_score(window: tauri::WebviewWindow) -> Result<(), CommandError> {
    window.print().map_err(|e| CommandError::io(e.to_string()))
}

#[tauri::command(rename_all = "camelCase")]
async fn export_audio(
    app: AppHandle,
    state: State<'_, SessionState>,
    session_id: String,
    revision: u32,
    soundfont_path: String,
    pulse_bpm: u32,
) -> Result<ExportResult, CommandError> {
    let projection = lock_session(&state)?.projection();
    ensure_version(&projection, &session_id, revision)?;
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(soundfont_path);
        let samples = score_audio::render(
            &projection.document,
            path.parent().ok_or("Missing sound library")?,
            pulse_bpm,
            44_100,
        )?;
        score_audio::wav(&samples, 44_100)
    })
    .await
    .map_err(|e| CommandError::io(e.to_string()))?
    .map_err(|e| CommandError::new("PlaybackError", e))?;
    let Some(path) = app
        .dialog()
        .file()
        .add_filter("Audio WAV", &["wav"])
        .blocking_save_file()
    else {
        return Ok(ExportResult {
            status: "cancelled",
            path: None,
        });
    };
    let path = path
        .into_path()
        .map_err(|e| CommandError::io(e.to_string()))?;
    ensure_version(&lock_session(&state)?.projection(), &session_id, revision)?;
    write_bytes_atomic(&path, &bytes).map_err(codec_error)?;
    Ok(ExportResult {
        status: "exported",
        path: Some(path.to_string_lossy().into_owned()),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(DocumentSession::new_studio()))
        .manage(Mutex::new(
            SaronPlayer::new().expect("audio control thread should start"),
        ))
        .manage(ManagedCloseState(Mutex::new(CloseState::default())))
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_studio_layout,
            export_score,
            print_score,
            export_audio,
            find_saron_soundfont,
            play_saron,
            stop_saron,
            pause_playback,
            seek_playback,
            saron_playback_active,
            saron_playback_status,
            new_document,
            apply_command,
            get_layout,
            open_document,
            save_document,
            resolve_close_request,
            export_svg,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let session_state = app.state::<SessionState>();
                let Ok(session) = session_state.lock() else {
                    api.prevent_close();
                    return;
                };
                let projection = session.projection();
                let close_state = app.state::<ManagedCloseState>();
                if let Ok(mut close_state) = close_state.0.lock() {
                    let expected = CloseAuthorization {
                        session_id: projection.session_id.clone(),
                        revision: projection.revision,
                    };
                    if close_state.authorized.as_ref() == Some(&expected) {
                        close_state.authorized = None;
                        return;
                    }

                    if !projection.dirty {
                        close_state.authorized = None;
                        return;
                    }

                    api.prevent_close();
                    let request = CloseRequest {
                        request_id: Uuid::new_v4().to_string(),
                        session_id: projection.session_id,
                        revision: projection.revision,
                    };
                    close_state.pending = Some(request.clone());
                    let _ = window.emit("document-close-requested", request);
                } else {
                    if projection.dirty {
                        api.prevent_close();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Sandikala");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_errors_keep_stable_codes_for_frontend() {
        let unsaved = CommandError::from(SessionError::UnsavedChanges);
        let history = CommandError::from(SessionError::HistoryEmpty);
        assert_eq!(unsaved.code, "UnsavedChanges");
        assert_eq!(history.code, "HistoryEmpty");
    }

    #[test]
    fn core_errors_preserve_field_and_ids() {
        let mapped = CommandError::from(SessionError::Core(sandikala_core::CoreError {
            code: sandikala_core::ErrorCode::InvalidReference,
            message: "voice does not exist".to_owned(),
            field: Some("voiceId".to_owned()),
            ids: vec!["missing-voice".to_owned()],
        }));
        assert_eq!(mapped.code, "InvalidReference");
        assert_eq!(mapped.field.as_deref(), Some("voiceId"));
        assert_eq!(mapped.ids, vec!["missing-voice".to_owned()]);
    }

    #[test]
    fn codec_errors_keep_contract_codes() {
        let too_large = codec_error(CodecError::InputTooLarge {
            limit: 1024,
            at_least: 1025,
        });
        let parse = codec_error(CodecError::Parse(
            serde_json::from_str::<serde_json::Value>("{").expect_err("malformed JSON should fail"),
        ));
        assert_eq!(too_large.code, "InputLimit");
        assert_eq!(parse.code, "InvalidDocument");
    }
}

# Saron SF2 playback prototype

Status: accepted for the first Sandikala playback slice.

## Decision

- Keep score meaning in `ScoreDocument`; playback derives a temporary performance
  plan and does not rewrite degree or register values.
- Discover the external Pelog Saron Barung SF2 automatically. Try a remembered
  selection, `SANDIKALA_SARON_SF2`, the app resource/data `soundfont` directory,
  then the sibling MuseScore checkout's `soundfont` directory. Remember manual
  selections; fall through if a remembered file was moved. Keep audio outside Git.
- Map degrees 1–7 plus registers -1/0/+1 to the SF2's documented Pelog key zones.
  Unsupported instruments, pitch forms, tempo units, or registers return visible
  errors; there is no piano or equal-tempered fallback.
- The prototype uses a user-selected constant pulse tempo (30–300 pulses/minute)
  and plays Saron parts only. It is not the full transport, mixer, export, or
  live-edit playback milestone.
- Render SoundFont audio before opening the native output stream. The CPAL data
  callback only copies a bounded immutable PCM buffer and performs no document
  access, file I/O, allocation, or blocking synchronization.
- Run the preparation command on Tauri's blocking pool so the desktop event loop
  stays responsive. Space toggles Play/Stop in the sheet, without stealing spaces
  from metadata inputs or the native keyboard activation of toolbar buttons.
- Provide Play/Stop in the top toolbar and show a separate blue playback cursor.
  Its pulse position comes from samples consumed by the native output callback,
  rather than a frontend timer. Device stream failures return a visible error.

## Validation target

Verify all three degree/register zones, successful loading of the supplied Saron
SF2, non-silent rendered PCM, the native output start/stop path, and visible
errors for missing or unsupported resources. Human listening is still required
to judge timbre and levels.

## Evidence: 2026-10-12

- The running native app invoked discovery and resolved the existing sibling
  MuseScore `Gamelan_Saron_Pelog_F3.sf2` without a file-picker selection.
- The explicit resource/device tests passed using the real SF2: non-silent PCM,
  native output to **MacBook Air Speakers**, 44,100 Hz stereo, playhead advancement,
  completion, and Stop. macOS output was unmuted. Human timbre/listening judgment
  is separate from these checks.
- `cargo test --workspace --locked`: 34 passed; the two host/resource tests are
  ignored in the default suite and were executed separately with
  `SANDIKALA_SARON_SF2` and `--include-ignored` (desktop suite: 8 passed).
- Rust formatting and desktop Clippy with `-D warnings` passed. Frontend build
  and six unit tests passed. One Playwright journey passed, including automatic
  sound readiness, Space Play/Stop, audio-position cursor, and metadata-space
  behavior. That journey mocks native IPC; the device test above uses real CPAL.
- This verifies the Saron playback slice, not completion of the Q3 milestone.

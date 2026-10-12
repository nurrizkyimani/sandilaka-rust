# Q1 Status

Updated: 2026-10-12

Overall: the Q1 implementation and automated checks are in place and passing;
the Q1 acceptance gate is **not complete** because the native file-dialog and
real-app workflow has not been interactively verified.

| Task | Status | Evidence and remaining work |
| --- | --- | --- |
| Q1-01 Scaffold | Implemented; automated smoke passed | Rust workspace, Tauri 2 shell, React/TypeScript/Vite app, exact direct dependency pins, and both lockfiles are present. `cargo metadata`, frontend typecheck/build, and a Tauri dev build/start passed. The dev process could not be attached to for a native UI screenshot in this session; full Xcode is also absent. |
| Q1-02 Core model | Implemented; workspace tests passed | Typed score, event, time/pitch systems, rational arithmetic, validation, limits, and both independent fixtures pass the workspace tests. The suite includes noncanonical-rational rejection. |
| Q1-03 Session and I/O | Implemented; workspace tests passed | Rust sessions provide revision checks, typed edits, bounded history, dirty tracking, validated open, and atomic `.sdlk` save. Tests cover both fixture round trips, failed-open/save state preservation, and an injected atomic replacement failure that preserves existing destination bytes. |
| Q1-04 Layout | Implemented; workspace tests passed | Rust generates numerical sheet SVG and absolute-pulse cell geometry; unsupported views return a read-only result. SVG export uses the same layout and atomic replacement. XML-invalid metadata characters are handled safely; tests cover glyph positions, register dots, blank cells, and cycle geometry. |
| Q1-05 Desktop editor | Implemented; automated checks passed | React editor uses Rust session/layout commands, a serialized 32-action edit queue, geometry-based selection, keyboard entry, dirty-document prompts, native dialogs, SVG export, and close confirmation. Five frontend unit tests and one Playwright UI journey pass. The Playwright journey mocks Tauri IPC; it does not prove native dialogs or filesystem behavior. |
| Q1-06 Integrated native gate | Pending | Not yet verified in a real Tauri window: create/edit/rest/register/cycle/undo/redo/save/reopen/export with native dialogs, cancellation/error paths, and the piano fixture staying unchanged/read-only. The Tauri dev binary built and started, but the current UI automation inventory did not expose its window for interaction. |

## Checks

| Check | Result |
| --- | --- |
| `rtk proxy cargo fmt --all -- --check` | Passed. |
| `rtk proxy cargo test --workspace --locked` | Passed: 31 tests, 0 failures (core 7, desktop 3, engine 15, I/O 6). Build artifacts were placed in `/private/tmp/sandilaka-rust-q1-target`. |
| `rtk proxy cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | Passed. |
| `rtk proxy pnpm typecheck` | Passed. |
| `rtk proxy pnpm test` | Passed: 5 frontend unit tests. |
| `rtk proxy pnpm build` | Passed. |
| `rtk proxy pnpm test:e2e` | Passed: 1 Playwright editor journey with mocked Tauri IPC (insert, save, reopen, export). It is a frontend integration check, not native file-I/O evidence. |
| `rtk proxy pnpm tauri dev` | Tauri dev build completed and its desktop process started. No native UI interaction or screenshot was verified. |
| Xcode availability | Full Xcode is absent; only Command Line Tools are selected. The `xcodebuild` command therefore cannot be used for a normal macOS app bundle/signing workflow. |

## Next Gate

Run the app on a host where the Tauri window is accessible to UI automation (or
complete the workflow manually) and record Q1-06 evidence for native dialogs,
save/reopen/export, cancellation/error handling, and read-only preservation of
the piano fixture. Do not treat mocked browser IPC or this implementation note
as proof of native file I/O or cross-platform release readiness.

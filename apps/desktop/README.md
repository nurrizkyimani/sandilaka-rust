# Desktop development

Run `pnpm install --frozen-lockfile`, then the React development server with
`pnpm dev` or the native Tauri app with `pnpm tauri dev`. Use `pnpm typecheck`,
`pnpm test`, `pnpm test:e2e`, and `pnpm build` for frontend checks. The Playwright
journey uses mocked Tauri IPC; it verifies frontend wiring, not native dialogs or
filesystem persistence. From the repository root, run Rust checks with
`cargo fmt --all -- --check`, `cargo test --workspace --locked`, and
`cargo clippy --workspace --all-targets --locked -- -D warnings`.

## Saron playback

The desktop app automatically finds `Gamelan_Saron_Pelog_F3.sf2` in a sibling
MuseScore checkout's `soundfont` directory. A remembered selection, the
`SANDIKALA_SARON_SF2` environment variable, or the app resource/data `soundfont`
directory can also provide it. Use **Locate Saron sound** if the library lives
elsewhere; manual selections persist across restarts.

Open a numerical Saron composition with notes, then click **Play** in the top
toolbar or press **Space** while working in the score. Space also stops playback.
A blue cursor follows the audio output. Spaces in title/composer fields retain
their normal text-entry behavior. Legacy Q1 documents retain Saron-only playback.

## Paged score studio preview (target v0.4)

New documents use the independent Rust score provider. It supports numerical
gamelan and two-staff piano in the same score, rational durations, subdivision
dots, register marks, instrument changes, annotations, undo/redo, and part views.
Leland and Edwin are bundled unchanged with their OFL licenses. The page canvas,
SVG, HTML, vector PDF, and native print use the same layout geometry.

Space starts or pauses/resumes studio playback. The transport also provides stop
and seek. Volume, pan, mute, solo and event velocity are saved with the score.
Gamelan playback loads each instrument's SoundFont from the Saron library folder;
Gendèr uses bank 0/program 11, other mapped instruments use bank 0/program 0.
Piano currently uses an explicitly labelled synthesized preview, not a piano SF2.
Missing libraries return an error rather than silently substituting piano.

Generate an engineering rehearsal-sheet fixture with
`cargo run -p sandikala-engine --example paged_demo -- output/pdf` from the root.
See `docs/14-v04-preview-status.md` for evidence and remaining acceptance gates.
Advanced tuplets, beams across events, ties, grace notes, changing/additive meters,
percussion strokes, and complete professional collision handling are not claimed.

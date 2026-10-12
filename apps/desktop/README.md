# Desktop development

Run `pnpm install --frozen-lockfile`, then the React development server with
`pnpm dev` or the native Tauri app with `pnpm tauri dev`. Use `pnpm typecheck`,
`pnpm test`, `pnpm test:e2e`, and `pnpm build` for frontend checks. The Playwright
journey uses mocked Tauri IPC; it verifies frontend wiring, not native dialogs or
filesystem persistence. From the repository root, run Rust checks with
`cargo fmt --all -- --check`, `cargo test --workspace --locked`, and
`cargo clippy --workspace --all-targets --locked -- -D warnings`.

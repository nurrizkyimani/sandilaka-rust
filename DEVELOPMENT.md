# Developer setup

Q1-01 scaffold baseline, checked 2026-10-12 on macOS 15.6.1, Apple silicon.

## Installed tools

| Tool | Installed version or status |
| --- | --- |
| Rust compiler | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Node.js | `v26.5.0` |
| pnpm | `12.3.4` |
| Rust components | `rustfmt` and `clippy` requested by `rust-toolchain.toml`; `rustup` is not installed |
| Tauri CLI | Not installed as a global Cargo command; use the pinned local package via `pnpm tauri` after dependency installation |
| Xcode | Not installed; only Command Line Tools are selected, so macOS native app build/smoke is unavailable in this environment |

The workspace pins Rust 1.97.1. `Cargo.lock` and the desktop `pnpm-lock.yaml`
are committed so transitive dependency resolution is reproducible. Tauri 2.12.3,
its 2.7.2 build helper, and the 2.8.2 dialog plugin are the selected Rust
versions. The desktop package pins React 19.3.0, Vite 8.3.4, TypeScript 7.0.2,
Vitest 5.0.3, and Playwright 1.64.0.

## Commands

From `apps/desktop`, run `pnpm install --frozen-lockfile`, then `pnpm dev`,
`pnpm build`, `pnpm typecheck`, `pnpm test`, `pnpm test:e2e`, or `pnpm tauri dev`.
`pnpm test:e2e` uses Playwright's Chromium runtime and mocks Tauri IPC; it checks
frontend wiring but not native file dialogs or disk persistence. The Tauri CLI
command launches Vite plus the native shell. From the repository root, Rust
checks are `cargo fmt --all -- --check`, `cargo test --workspace --locked`, and
`cargo clippy --workspace --all-targets --locked -- -D warnings`. To keep Cargo
artifacts out of the source tree, optionally set
`CARGO_TARGET_DIR=/private/tmp/sandilaka-rust-q1-target`.

## Current Q1 evidence

As of 2026-10-12, workspace formatting, all 31 Rust tests, Clippy with
`-D warnings`, frontend typecheck, five Vitest tests, the production frontend
build, and one mocked-IPC Playwright editor journey pass. The Tauri dev build
completed and started its process. Native file dialogs and the real window's
create/edit/save/reopen/export journey remain unverified; the current UI
automation inventory did not expose the window, and full Xcode is not installed
for the usual macOS app-bundle workflow. See [Q1 status](docs/q1-status.md) for
the exact gate and remaining evidence.

# Sandikala

Sandikala is a local-first music composition application in development for macOS
and Windows. Its Rust core represents musical meaning independently of any single
notation, tuning, or sound engine. Piano and numerical gamelan notation are the
first contrasting reference cases, not the limits of the product.

The repository directory is currently named `sandilaka-rust`. The product name
used in these documents is Sandikala. Renaming the directory is optional and is
not part of this planning pack.

## Current status

The Q1 Rust model, session, numerical layout, and Tauri/React editor implementation
are in place. Formatting, 31 Rust tests, Clippy, frontend typecheck/build, five
frontend unit tests, and a mocked-IPC Playwright editor journey pass. Q1 native
acceptance is still pending: native dialogs and real-window save/reopen/export
have not been interactively verified. See [Q1 status](docs/q1-status.md) for
evidence and the remaining gate. No milestone is complete without its acceptance
evidence. There are no delivery-date or full-feature-parity promises.

## Start here

For the first visible result, start with [Q1: gamelan sheet editor](docs/11-gamelan-sheet-quick-win.md).
This is the active implementation path as of 2026-10-12. It freezes a small
contract and orders six bounded tasks. The [Q1-Q7 milestone table](docs/05-milestones.md)
defines the current product delivery order: sheet and persistence first, useful
gamelan composition second, sound third. M0-M8 remains a technical reference,
not a competing sequence or a piano/audio prerequisite for Q1.

| Document | Purpose |
| --- | --- |
| [Product requirements](docs/00-product-requirements.md) | Users, workflows, scope, functional and nonfunctional requirements |
| [Architecture](docs/01-architecture.md) | Frontend, native backend, ownership, dependencies, and data flow |
| [Score and persistence](docs/02-score-and-file-format.md) | ScoreDocument, events, time, pitch, extensions, and `.sdlk` |
| [Notation and engraving](docs/03-notation-and-engraving.md) | Input, notation intent, layout, editing geometry, and publishing |
| [Performance and audio](docs/04-performance-and-audio.md) | Realization, timing, sample playback, transport, and integrations |
| [Milestones](docs/05-milestones.md) | Active Q1-Q7 table, Q1 execution order, and original M0-M8 workstream detail |
| [Feature matrix](docs/06-feature-matrix.md) | Requirement coverage and staged feature commitments |
| [Quality and performance](docs/07-quality-and-performance.md) | Tests, human review, measurement, and optimization policy |
| [AI execution workflow](docs/08-ai-execution.md) | Sol/Luna task routing, context management, and review gates |
| [First tasks](docs/09-first-tasks.md) | Concrete M0/M1 task cards and expected evidence |
| [Decisions and risks](docs/10-decisions-and-risks.md) | Accepted direction, unresolved choices, risks, and primary sources |
| [Gamelan sheet quick win](docs/11-gamelan-sheet-quick-win.md) | Fixed Q1 contract, fixtures, six implementation tasks, and Luna starting prompt |
| [Task template](templates/task.md) | Reusable implementation handoff |

Read the product and architecture documents first. An implementation session then
reads only its task card and the relevant specialist document. Read
[AGENTS.md](AGENTS.md) before changing files.

## First product target

First visible milestone: create and edit a numerical gamelan-style sheet, with
16-pulse cycles, degree/register entry, explicit rests, undo/redo, save/reopen,
and SVG export. This is a deliberately limited numerical demo, not a claim of
complete Kepatihan practice. Audio, professional engraving, and piano UI follow.
Retain a spelled-pitch/metric fixture as a headless neutrality check from the start.

The following remains a later capability target, not the order of the first milestones:

A user can create, edit, save, reopen, view, and play a short piano composition
and a short numerical gamelan composition using the same document engine.
Q1 proves the sheet/persistence path; Q3 adds audio; Q5 adds the piano notation
provider. Later milestones expand publishing, interoperability, and reliability.

## Long-term rule

Keep musical meaning, notation intent, performance interpretation, and physical
output distinct. Preserve information and report limitations. Add complexity
when a requirement or a measurement justifies it; avoid speculative plugin
frameworks, GPU renderers, databases, and distributed systems.

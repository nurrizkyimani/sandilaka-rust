# Implementation instructions

## Active delivery path

- `docs/05-milestones.md` defines the active Q1-Q7 product sequence. Sound starts
  in Q3. Original M0-M8 sections are technical references, not execution order.

- Start with `docs/11-gamelan-sheet-quick-win.md` and its next unfinished Q1 task.
- That document controls Q1 schema, commands, scope, geometry, and acceptance.
  Older conceptual examples and M0/M1 blueprints do not override its fixed subset.
- The remaining M0-M8 work is not automatically execution-ready. Resolve its
  contracts when reached; do not force Q1 to implement audio or piano engraving.
- Complete one Q1 task and record evidence before advancing. Do not implement
  a static frontend-only mockup instead of the Rust-backed editor.
- Q1 completion is a separate gate, not evidence that all of M0-M3 is complete.

## Product contract

- Read `README.md`, the assigned task, and the documents it names.
- Product requirements are in `docs/00-product-requirements.md`; acceptance gates
  are in `docs/05-milestones.md` and `docs/07-quality-and-performance.md`.
- A ScoreDocument is authoritative. Events belong to it. Neither MIDI, Western
  note names, nor numerical notation is the universal internal representation.
- Keep musical meaning and notation intent. Do not silently flatten tuning,
  change pitch spelling, quantize timing, or discard unknown extension data.
- The Rust core must run without React, Tauri, or an audio device.

## Scope and decisions

- Implement one task contract at a time. Respect its public interface and scope.
- Inspect the current files first. Preserve unrelated changes.
- If the contract is contradictory or insufficient for correct behavior, record
  the specific ambiguity. Do unaffected work; route the decision to the planning
  owner instead of inventing permanent musical semantics.
- Architecture changes need a short decision record, affected requirements, and
  regression evidence. This is a review gate, not a reason to ask permission for
  every routine implementation choice.
- Use established libraries for formats, device access, text shaping, and other
  standard machinery after checking maintenance, compatibility, and licenses.
- Do not copy MuseScore source, layouts of internal classes, or proprietary
  assets. Public standards and established music rules are valid references.

## Research and efficiency

- Follow applicable parent workspace instructions. Use RTK when installed.
- If `.codegraph/` exists and its index is current, use CodeGraph for relevant
  symbol/call-path questions before broad searching. Index creation is a separate
  decision. When unavailable, use focused `rg` and bounded file reads.
- Use Context Mode for large logs or searches when its tools are available. Do
  not claim a tool was used when it was unavailable.
- Keep evidence, decisions, and task status in concise repository documents.
  Avoid feeding every session the complete conversation or all planning files.
- Do not create agents or parallel implementations unless the current task
  authorizes delegation. Ownership must be explicit when delegation is used.

## Quality

- Add behavioral tests proportional to the task. The important acceptance cases
  come from the contract, not from reproducing the implementation in a test.
- Do not weaken checks, delete a fixture, or update a golden image only to make
  a failing implementation pass. Explain intentional baseline changes.
- Reject invalid values and references with actionable errors. Handle missing
  providers, sound assets, fonts, and audio devices without crashing or data loss.
- Audio callbacks must not perform document traversal, blocking I/O, allocation,
  or acquire blocking locks. Precompute and transfer bounded work outside them.
- Optimize from measured evidence; do not introduce custom allocators, ECS,
  lock-free infrastructure, SIMD, or GPU rendering for speculative future load.
- Use safe Rust by default. Any unsafe code requires a documented invariant and
  focused tests, and must be justified over a maintained alternative.

## Completion evidence

Report changed files, satisfied requirement IDs, checks actually run, remaining
limitations, and the next dependency. A build is not evidence of audible
playback, good engraving, or working native installers. Update only the relevant
task status. Do not mark an entire milestone complete without its exit evidence.

Once code exists, expected Rust checks are formatting, relevant tests, and Clippy.
Expected frontend checks are type checking, focused tests, and applicable UI
checks. The exact executable commands will be established in M1; do not report
placeholder commands as passing.

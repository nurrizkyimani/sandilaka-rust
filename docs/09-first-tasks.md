# First execution tasks

These are task blueprints, not completed tickets or invented API signatures.
Before code execution, fill contract-dependent details and ownership with the
current repository paths. M0 drafts already exist in this planning pack, but their
review, fixtures, and technical-spike evidence remain pending.

## Start with Q1 instead

The [gamelan sheet quick win](11-gamelan-sheet-quick-win.md) now supplies the
immediate fixed subset and six executable task cards. Follow Q1-01 through Q1-06
for the first visual milestone. The M0/M1 blueprints below remain a larger backlog,
not a requirement to finish audio/piano research before showing a numerical sheet.

## M0 task cards

### M0-01: Review and accept the baseline

- Goal: align product scope and architecture before constructing the workspace.
- Owner/model: planning owner with Sol/high.
- Read: product requirements, architecture, milestones, feature matrix, decisions.
- Deliver: reviewed requirement scope, selected initial notation conventions,
  decision status updates, and a concise M0 task-status note.
- Acceptance: each PR/NFR ID maps to a milestone; name/platform/release language
  is consistent; deferred DAW/plugin features cannot be mistaken for M3/M7 scope.
- Failure path: revise contradictions in the planning files, not the musical code.

### M0-02: Piano fixture specification

- Goal: define independent expectations for a short Western composition.
- Owner/model: Sol planning plus knowledgeable musician review; Luna may format
  agreed data. Do not ask Luna to infer the notation convention.
- Read: score/persistence, engraving, audio, and quality documents.
- Deliver: complete fixture specification with two staves/voices, ownership,
  pitch spellings, rests, simultaneity, a tie, durations, A4 reference, and tempo.
- Expected behavior: at 120 quarter beats/minute, onset at beat 2 is 1 second;
  a one-beat logical extent is 0.5 seconds. A tied pair has one attack with the
  agreed sustained extent. Independent events at the same onset remain distinct.
- Acceptance: every event resolves its context; notation and audio expectations
  are enumerated; names/assets have recorded provenance.
- Failure path: missing voice/tie/spelling semantics become M0-04 decisions.

### M0-03: Numerical/cyclic fixture specification

- Goal: expose assumptions that a piano-only model would miss.
- Owner/model: Sol and a reviewer familiar with the chosen numerical convention.
- Read: score/persistence, engraving, audio, and quality documents.
- Deliver: complete 16-pulse cycle, structural anchors, degree/register identities,
  nonuniform synthetic tuning table, assigned instrument, and damping behavior.
- Expected behavior: at 60 pulses/minute, pulse 3 starts at 3 seconds. Degree 1
  resolves to the fixture's 220 Hz or exact mapped sample, not a default MIDI note.
  A strike's acoustic tail is separate from its logical extent; damping is explicit.
- Acceptance: all marks and mappings have independent expected results and the
  document uses the same aggregate/event primitives as the piano fixture.
- Failure path: record which convention requires provider-specific behavior;
  never label the synthetic tuning as a universal gamelan standard.

### M0-04: Freeze the M1 contract subset

- Goal: choose a small coherent schema and editing interface for the fixtures.
- Owner/model: Sol/high; high-risk contract review before Luna implementation.
- Deliver: versioned schema draft, type/field semantics, integer/arithmetic bounds,
  context mappings, relation rules, command transaction behavior, and error taxonomy.
- Acceptance: validate ownership, unique IDs, missing context, cross-domain timing,
  invalid rational, required/optional extension, and notation-intent policies.
  Decide how chord grouping and per-event extents interact.
- Failure path: reduce the subset while preserving both fixtures; do not replace
  explicit semantics with a permissive arbitrary JSON blob.

### M0-05: Focused library/platform feasibility

- Goal: discover blocking integration issues with small disposable experiments.
- Owner/model: Sol defines hypotheses; Luna can execute bounded spikes.
- Deliver: evidence for Tauri/React SVG/font rendering, available native device
  configuration and clean stream shutdown, plus candidate PDF/SoundFont libraries.
- Acceptance: record versions, platforms actually tested, dependency/licenses,
  font output, device result, shutdown behavior, and reasons for selected candidates.
- Failure path: an untested OS remains unverified; a failed candidate revises its
  decision before dependent implementation. Do not build the final editor in a spike.

### M0-06: Prepare executable M1 handoff

- Goal: make routine implementation possible without reopening architecture.
- Owner/model: Sol/high for decomposition; concise task cards for Luna.
- Deliver: completed M1 task contracts using the template, named modules and
  independent expected cases, review boundaries, and prerequisite order.
- Acceptance: each task has a known interface and error behavior. Any unknown
  semantic decision has an owner and is resolved or excluded before implementation.
- Failure path: strengthen the handoff rather than starting a speculative framework.

## M1 task blueprints

### M1-01: Minimal Rust workspace and checks

- Prerequisites: M0-04 and immediate library/toolchain choices.
- Model: Luna/medium against a reviewed structure.
- Scope: workspace/toolchain configuration and only necessary core/I/O modules.
- Deliver: reproducible build, formatting/tests/lint commands, platform CI, and
  a short developer setup document. Pin compatible tool versions.
- Acceptance: actual macOS/Windows CI results; headless core tests need no device.
  If CI cannot run, report that gap instead of calling the milestone complete.
- Non-goals: desktop scaffolding, provider framework, native audio engine.

### M1-02: IDs and rational temporal values

- Prerequisites: M1-01, finalized bounds and domain-unit contract.
- Model: Sol for arithmetic/semantic implementation or review; Luna only for a
  tightly specified subtask with independent cases.
- Scope: core identity/time/context modules and tests.
- Acceptance examples: 2/4 normalizes to 1/2; 1/3 + 1/6 equals 1/2; invalid zero
  denominator fails; overflow fails explicitly; cross-domain comparison requires
  a mapping; a declared pickup remains representable.
- Deliver: documented values/operations and error behavior without floating-point
  logical timing. Finite checked bounds are part of the public contract.

### M1-03: Aggregate and semantic validation

- Prerequisites: M1-02 and finalized event/relation subset.
- Model: Luna/medium for validation against Sol-reviewed types/rules.
- Scope: core model/validation modules and fixture-based tests.
- Acceptance: both complete fixtures pass; duplicate IDs, dangling relations,
  missing voice/context, invalid ownership, and incompatible tie references fail
  with affected IDs. Unpitched strokes do not need fabricated pitches.
- Deliver: validator diagnostics and documented strict/restricted modes where
  unknown provider requirements apply.
- Non-goals: engraving decisions or conversion into MIDI storage.

### M1-04: Editing commands and undo

- Prerequisites: M1-03 and chosen undo strategy/revision policy.
- Model: Sol sets transactions; Luna implements specific commands under review.
- Scope: core/application command modules and focused tests.
- Acceptance: insert/delete/update/grouped paste either all succeed or all fail;
  stale revision rejects cleanly; undo restores a known prior document; redo
  reapplies it; a new edit clears the appropriate redo branch; history is bounded.
- Deliver: interface and state-transition evidence, no React or audio dependency.

### M1-05: `.sdlk` persistence and safe saves

- Prerequisites: M1-03; accepted serialization/version/resource rules.
- Model: Luna/medium for codec tasks; Sol reviews durability/version contracts.
- Scope: I/O module, fixture round trips, file-operation failure tests.
- Acceptance: meaning/spelling/extensions survive a round trip; invalid documents
  cannot replace valid state; future core schema is refused; a missing sound profile
  does not prevent opening; failed save preserves the original destination.
- Deliver: actual file encoding and platform-tested atomic-save strategy.
- Non-goals: bundled SoundFonts, database, ZIP/container support, compression.

### M1-06: Migration and headless integration gate

- Prerequisites: M1-04/M1-05.
- Model: Sol review; Luna can implement a precisely specified migration test path.
- Scope: controlled prior-schema fixture, migration/refusal logic, integration tests.
- Acceptance: source remains intact on failure; supported migration preserves
  meaning; both full fixtures load/edit/undo/save/reopen; no desktop/device required.
- Deliver: M1 review packet with requirement IDs, actual check results, remaining
  limitations, and concrete M2 tasks.

## Next-task rule

Do not hand Luna all of M1 as one prompt. Finish one accepted task, record its
evidence, then advance to a dependent task. When a current implementation differs
from a proposed path/name here, update ownership in the task before editing;
preserve contracts rather than forcing a cosmetic folder layout.

# Q1: First visible gamelan sheet editor

Status: implementation contract, 2026-10-12. Automated implementation checks pass; native Q1 acceptance remains pending. See [Q1 status](q1-status.md).
This contract controls the Q1 subset when broader planning proposals differ.
Do not treat Q1 completion as complete M0/M1/M2/M3 coverage.

## User outcome and scope

Open the desktop app, create a short numerical gamelan-style composition, see its
sheet update, revise it, save/reopen it, and export SVG. This is enough for the
first visible milestone: it proves composition-to-sheet through the real Rust core.
A screenshot-only/static grid is not enough.

Q1 supports one editable Saron demo part/voice, 16-pulse cycles, degree symbols
1-7, an explicit rest displayed as `.`, and register -1/0/+1 as a dot below/no dot/
dot above. These are deliberately restricted demo conventions, not a claim to
implement every Kepatihan tradition. The fixture is an original engineering
pattern, not a named traditional composition or an authoritative tuning table.

No playback, SF2, samples, tempo ramps, PDF, professional pagination, slurs,
tuplets, multi-voice UI, solfa, staff UI, plugin framework, or external integration
belongs to this gate. The core must also store/validate a metric spelled-pitch
fixture, so the first implementation does not make numerical notation universal.

## Fixed implementation choices

| Concern | Q1 choice |
| --- | --- |
| Desktop/UI | Tauri 2, React + TypeScript, Vite; pnpm frontend scripts |
| Musical state | Rust owns document, commands, history, validation, and layout |
| Workspace | `crates/sandikala-core`, `crates/sandikala-io`, `crates/sandikala-engine`, `apps/desktop` |
| Crate boundary | Core has no Tauri/React/audio dependency; engine and I/O depend on core |
| Rendering | Rust-generated safe SVG and cell geometry; React selection overlay uses that geometry |
| Glyphs | Ordinary digits, period, register circles, and text; system sans-serif; no notation-font dependency |
| Storage | Plain JSON `.sdlk`, `schemaVersion` exactly `0.1.0-q1` |
| History | At most 100 before/after document transactions; normal snapshots, not event sourcing |
| Dependencies | Serde/serde_json, maintained UUID v4 implementation, Tauri dialog support; choose compatible versions at bootstrap and commit lockfiles |
| Performance | Full small-score validation/layout after an edit; no GPU/ECS/incremental cache/audio code |

Q1-01 records the chosen installed compatible toolchain/library versions in a
developer setup note. Version selection is routine bootstrap work, not permission
to redesign the architecture. A proven incompatibility gets a decision note.

## Fixed document contract

Complete examples: [numerical fixture](../fixtures/q1/gamelan-basic.sdlk) and
[neutrality fixture](../fixtures/q1/piano-model.sdlk). Field order is immaterial;
array order is retained. All listed root fields are required. No extra root fields
are allowed in this version. Unknown optional data belongs inside `extensions`.

| Field/type | Definition |
| --- | --- |
| schemaVersion | String `0.1.0-q1`; refuse other versions with UnsupportedSchemaVersion |
| id | Nonempty stable document ID string, at most 128 characters |
| metadata | Object with required title/composer strings, each at most 256 characters |
| parts | Array of `{id, name, instrumentRef, voiceIds}`; instrumentRef is `{id, version}` strings |
| voices | Array of `{id, partId, timeDomainId}`; IDs refer to existing parts/domains |
| timeDomains | Cycle `{id, kind:"cycle", unit:"pulse", cycleLength:16, cycleCount}` or metric `{id, kind:"metric", unit:"quarter", length:Rational}` |
| pitchSystems | Degree context `{id, kind:"degree", degrees:string[], tuningHz:object}` or equal-temperament context `{id, kind:"equalTemperament", stepsPerOctave:12, reference:{letter:"A", octave:4, frequencyHz:440}}` |
| events | Array of Event objects described below; order is retained, not used as time identity |
| relations | Empty array only; a nonempty array is UnsupportedCoreFeature in Q1 |
| notationViews | Array of `{id, providerId, partIds, settings:object}`; unknown provider cannot render/edit but must not destroy the document |
| extensions | Object of namespace keys to arbitrary preserved JSON; all are optional opaque data in Q1, not executable behavior |

All IDs are unique within their category, and references have the required
category. Part/voice ownership must agree both ways. Every event has one voice.
Cycle count is an integer 1-64. Domain length is positive. Synthetic tuningHz keys
match declared degree strings, and values are finite positive Hz at base register;
Q1 does not infer an acoustic octave formula from register dots.

Rational is `{numerator: integer, denominator: integer}`. Absolute numerator and
denominator are at most 1,000,000; denominator is positive. Normalize with GCD;
zero normalizes to 0/1. Use checked i128 intermediate arithmetic, reduce, then
reject a result outside these bounds. Domain-agnostic rational arithmetic may be
used; comparing anchored positions across domains without a mapping is an error.
Cycle event starts are nonnegative. Metric model positions can be negative for
pickups. Event endpoints must not exceed the domain's declared upper extent.

Event is `{id, voiceId, anchor, extent, kind, payload, extensions}`:

- anchor: `{domainId, position:Rational}`, matching the voice's domain.
- extent: Rational strictly greater than zero, measured in that domain's units.
- kind `rest`: payload is `{}`; it is distinct from an absent event/blank cell.
- kind `sound`: payload is `{pitch}`. Pitch is degree
  `{kind:"degree", contextId, degree:string, register:integer}` or spelled
  `{kind:"spelled", contextId, letter:A-G, alteration:integer, octave:integer}`.
- Degree must exist in its degree context; register range for this schema is -32
  through 32. The numerical Q1 view renders/edits only -1/0/+1.
- Spelled alteration is -2 through 2 and octave -1 through 9; context must be the
  declared equal-temperament form. Q1 validates/preserves it without staff output.
- Reject unknown core kinds/extra core fields rather than silently ignoring them.
  Preserve optional JSON inside event/root extensions without interpreting it.
- Overlapping event extents in the same voice are rejected in this version.
  Different voices can have simultaneous events; chord/tie semantics are later work.

Reject input files over 1 MiB before decoding and reject nested JSON deeper than
64 levels. Cap events at 4,096, parts at 64, voices at 128, and domains/contexts/
views at 128 each. These are documented Q1 safety limits, not long-term product
limits. Report limit violations, do not truncate. Do not use unbounded custom parsing.

The editable view additionally requires exactly one cycle domain, one part/voice,
one degree context declaring precisely tokens 1-7, a numeric-demo view naming that
part, degree events referencing that sole context or rests at integer pulse
positions, extent 1, and supported registers. SetCell uses that sole context when
creating a pitch reference; it must not guess from a display token or instrument
label. Valid documents outside that UI subset open in a
read-only unsupported-view state; they may be saved-as unchanged. They must not
be silently rewritten, quantized, or replaced by the demo.

## Fixed command and session contract

Rust SessionProjection contains `{sessionId, document, revision, dirty, canUndo,
canRedo, capabilities}`. Capabilities is `{canEdit:boolean, canRender:boolean,
reason:string|null}` computed by Rust using the subset rules above. Unsupported
documents have false editing/rendering capabilities and a useful reason; backend
commands refuse edits as well as disabling UI controls. They can still be opened
and saved-as unchanged. sessionId is a new UUID on each new/open operation; revision starts at
0, uses checked u32 increments, and is not persisted. Reject stale session or
revision on commands. Render results carry both and the UI discards stale results.

Define these commands as typed Tauri adapters over headless Rust operations:

| Operation | Behavior |
| --- | --- |
| get_state | Return the current SessionProjection |
| new_document({discardUnsaved}) | Create an empty 16-pulse demo document; new stable IDs; title Untitled; empty composer/events/history |
| apply_command({sessionId, expectedRevision, command}) | Validate and commit one transaction; return SessionProjection |
| get_layout({sessionId, revision, viewId}) | Return tagged SVG, dimensions, and CellGeometry array or UnsupportedView |
| open_document({discardUnsaved}) | Native file dialog; parse/validate before replacing current document; cancel/failure preserves it |
| save_document({sessionId, expectedRevision, saveAs}) | Save current snapshot; native dialog when needed; cancellation leaves dirty state unchanged |
| export_svg({sessionId, revision, viewId}) | Export the same Rust layout through a native save dialog; never mark the composition saved |

Discarding unsaved work requires an explicit user-confirmed UI choice; the backend
returns UnsavedChanges otherwise. File paths/current save destination belong to
session state, not the score. Native I/O dialogs are scoped to user-selected paths.

`command` is one of:

- SetCell `{voiceId, pulseIndex, value}`; value is `{kind:"degree", degree, register}`
  or `{kind:"rest"}`. `pulseIndex` is the zero-based absolute pulse position in
  the full cycle domain, from 0 through `cycleLength * cycleCount - 1`; it is not
  reset at each displayed row. Replace one integer-pulse cell with extent 1. Preserve an
  existing event's ID/extensions when changing it; create a UUID only for a new
  occupied cell. Switching sound/rest replaces payload, not unrelated metadata.
- ClearCell `{voiceId, pulseIndex}` uses the same absolute pulse index and removes
  that event, leaving a blank. Do not
  shift other events. Clearing an empty cell is a no-op, with no revision/history.
- AddCycle `{domainId}` increases cycleCount by 1, up to 64; adds no musical events.
- SetMetadata `{title, composer}` sets both strings after validation.
- Undo or Redo, with no other fields. Empty history returns HistoryEmpty; disable
  the corresponding button. New edits clear redo. History is reset on new/open.

Identical metadata/SetCell value is a no-op. Invalid operations never alter
document, history, revision, or dirty state. Successful content edits increment
revision. Dirty is semantic document inequality with the last saved/new baseline,
so undo back to the saved state becomes clean. Runtime selection/zoom is not dirty.
Undoing AddCycle clamps selection to a remaining valid cell.

Required errors: InvalidDocument, InvalidReference, InvalidRational, ArithmeticLimit,
UnsupportedSchemaVersion, UnsupportedCoreFeature, UnsupportedView, StaleSession,
StaleRevision, UnsavedChanges, HistoryEmpty, InputLimit, and IoError. Return a code,
message, and affected field/IDs where applicable. Dialog cancellation is a normal
cancelled outcome, not an error. No panic/partial state on user input.

Atomic saves use a sibling temporary file and platform-supported replacement;
never delete the old destination first. Failed write/replace leaves old file and
dirty state intact. Choose a maintained safe file-replacement implementation or
verify the platform-specific standard-library strategy; record actual OS evidence.

## Fixed sheet and input behavior

Start directly in an empty editor. Provide New/Open/Save/Save As/Export SVG, undo/
redo, title/composer, register selector, degree/rest entry, and Add Cycle. No fake
playback button or marketing screen. Familiar icons have accessible labels/tooltips.

Rust lays out one row per cycle, with 16 columns and separators after each group
of four. Each cell is 44 by 64 SVG units; padding is 24, header 64, row gap 16.
Width is 752; height is 64 + cycleCount*64 + (cycleCount-1)*16 + 24. Center digits
at cell x+22, baseline y+32; upper/lower register circles at y+14/y+44, radius 2.
Period uses the digit baseline. Empty cells contain no musical glyph. Title and
composer are escaped plain text in the header. Use a distinct border for selection.
Title uses 20-unit text and composer uses 14-unit text on separate header baselines;
clip/ellipsize overflow within the 704-unit content width, preserving complete
strings in metadata and escaped SVG title elements. Long text must not cover cells.
Separators are visual grouping, not metric barlines or sounding gong events.

CellGeometry is `{voiceId, pulseIndex, eventId:string|null, x, y, width, height}`;
`pulseIndex` is the same absolute domain position used by SetCell/ClearCell, while
the visible column is `pulseIndex % cycleLength` and the row is
`floor(pulseIndex / cycleLength)`. Overlay and hit-testing use these exact coordinates. Zoom/pan only transforms the
viewport; map pointer coordinates through the inverse SVG screen transform.
Narrow windows may scroll horizontally; never shrink text to illegibility or
reinterpret a cycle. Zoom changes no document fields.

Click selects a cell. Keys 1-7 replace/enter a degree using the current register;
period inserts an explicit rest; Delete/Backspace clears the selected cell.
Left/right move one absolute pulse and wrap across cycle rows; up/down move by one
cycle while keeping the same column. Selection clamps at the first and last pulse.
After a note/rest,
advance one cell, stopping at the final cell until Add Cycle is chosen. Input
shortcuts do not capture typing in title/composer
or other form fields. Only one cell is selected in Q1; no copy/paste/selection ranges.

Serialize input actions in a bounded queue of at most 32 pending actions. Apply
each against the last acknowledged session/revision; process entry/cursor movement
in order so rapid typing does not overwrite one cell or lose notes. Queue full
reports Busy without silently dropping an action. New/open clears pending actions;
late replies from the prior session cannot change the current document or cursor.
Backend errors leave document state unchanged and keep the affected selection
visible. Do not send concurrent edits using the same expectedRevision.

SVG must contain no scripts, foreignObject, event-handler attributes, or external
asset URLs. Escape title/composer and any document-derived strings. Native file
content is untrusted. Use typed layout geometry and safe generated primitives.

## Ordered implementation tasks

### Q1-01: Scaffold and actual desktop smoke

Ownership: workspace configs, empty core/I/O/engine crates, apps/desktop scaffold,
developer setup note. Use only fixed stack choices above. Add scripts `dev`,
`build`, `typecheck`, `test`, `test:e2e`, and `tauri` at desktop package; document
root commands. `dev` runs Vite; `tauri dev` launches the native app. Vite alone is
not the desktop backend. Pin compatible stable Rust and library/tool versions.
Gate: Rust checks and frontend checks run; a native window opens on the local OS.
Record unavailable platform tests. No implementation of audio or extra providers.

### Q1-02: Core model, validation, and independent fixtures

Ownership: core model/time/validation tests; fixture acceptance tests. Read this
contract and both complete fixtures; broader schema proposals are not extra scope.
Implement the field/types/bounds above with typed sound/rest and degree/spelled
forms. Gate: both fixtures validate; rational cases 2/4=1/2 and 1/3+1/6=1/2;
invalid IDs/references/overlaps/bounds/denominators fail without coercion. Add an
unknown optional extension preservation case. No renderer or playback dependency.

### Q1-03: Transactions, persistence, and undo

Ownership: headless session/command module and I/O codec/safe-save tests. Implement
the fixed operations/history/dirty rules. Gate: enter/replace/clear/add-cycle,
undo/redo, no-ops, stale-session/revision, round trip of both fixtures, unsupported
core refusal, unknown extensions, and injected save failure. Document does not
change on rejected input or failed open. Save state and UI state remain separate.

### Q1-04: Rust numerical layout and SVG

Ownership: engine numeric-demo provider/layout and focused geometry/security tests.
Use the fixed geometry and fixture; no provider framework or external music font.
Gate: 16 CellGeometry entries for one cycle, 32 for two; exact occupied glyphs,
register dots, blank/rest distinction, row bounds, safe title escaping. Piano fixture
returns UnsupportedView rather than altered pitches or a fake numerical conversion.
Render/export output shares one layout path. No audio prerequisites.

### Q1-05: Native editor interaction and dialogs

Ownership: Tauri adapters and frontend components/controls/tests. Rust remains
authoritative; JavaScript does not maintain a second mutable composition. Wire
the session/revision protocol, SVG geometry, editing, history, dirty warnings,
new/open/save/export, error/cancelled states, zoom, and read-only unsupported view.
Gate: create from empty state, edit notes/rests/registers/cycles, save/reopen,
undo to saved state, and export through the actual native app. Test text-field
typing, zoomed hit-testing, and late response rejection. No static screenshot-only demo.

### Q1-06: Integrated visible-result gate

Ownership: focused integration/UI tests and `docs/q1-status.md` evidence. Verify:
empty -> enter `1 2 3 5` -> rest -> replace -> Add Cycle -> undo/redo -> save ->
close/reopen -> same sheet -> export SVG. Inspect both supported register markers.
Open the piano fixture into explicit read-only state and save-as without changing
its musical meaning. Test unknown extensions, invalid input, dirty-close cancel,
and missing/unavailable native support. Browser tests do not prove native file I/O.

Run `cargo fmt --all -- --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, desktop typecheck/tests/
build, and relevant UI tests. Record actual commands/output summaries/screenshots
and native manual checks. If a platform or check cannot run, mark it unverified.
Do not call Q1 a released cross-platform app from one successful local run.

## Independent acceptance oracle

The supplied gamelan fixture has 16 events: 15 sounds, one explicit rest at
zero-based pulse 5. Tokens are `1 2 3 5 | 6 . 5 3 | 2 1 2 3 | 5 6 5 1`.
Pulse 0 has upper-register dot; pulse 4 has lower-register dot. No other dots.
The first row starts at (24,64); pulse 5's cell is (244,64,44,64).
One-cycle SVG dimensions are 752 by 152; two-cycle dimensions are 752 by 232.
These numbers are independent expected values, not computed by the renderer test.

The piano fixture has one C4 sound at metric position 0 with extent 1. It must
survive model/persistence unchanged; Q1 staff rendering/playback is unsupported.

Additional cases: corrupt or >1 MiB file; denominator 0; duplicate ID; degree not
in context; overlap in voice; unsupported version/core event; stale session after
new/open; extension object round trip; saved-state undo; cancelled dialog;
metadata `<script>alert(1)</script>` displayed/escaped as text, never executable SVG.

## Luna starting prompt and handoff

```text
Work in /Users/nurrizky/dev/sandilaka-rust.
Read AGENTS.md and docs/11-gamelan-sheet-quick-win.md.
Implement the next unfinished Q1 task, starting with Q1-01.
Use the fixed contract and its independent fixtures. Preserve unrelated edits.
Do not implement audio, piano engraving, PDF, plugins, or the entire M0-M8 roadmap.
Run the task's relevant real checks. Record evidence in docs/q1-status.md:
task status, files changed, commands/results, native verification, and next task.
Do not silently change the schema or acceptance oracle to make checks pass.
Report a concrete contract conflict if found; continue unaffected work.
Finish one task before taking its dependent task.
```

Use high/maximum reasoning if desired, but bounded tasks and independent review
provide the acceptance standard. They cannot guarantee an error-free agent.
Sol reviews new contract changes; a musician validates authentic convention claims.
After Q1, expand useful gamelan composition in Q2, add one tuned instrument's
audible playback in Q3, and add a real piano provider in Q5. Follow the active
Q1-Q7 table in `docs/05-milestones.md`; each later stage needs its own contract.
Neither audio nor piano engraving is secretly part of Q1.

# Milestone delivery plan

All milestones are planned, not completed. This pack supplies the initial M0
drafts; fixture review, technical spikes, and acceptance evidence are still needed.
Milestones represent capability gates, not deadlines. Estimate after M0/M1 reveal
actual integration costs. Implement tasks sequentially where contracts depend
on one another.

## Active product milestones: Q1-Q7

This table is the current product delivery order. The original M0-M8 sections
below remain technical workstream references, not a competing execution sequence.
In particular, sound starts in Q3, not Q1 or Q2. Only Q1 has a fixed implementation
contract today; expand Q2 and Q3 into reviewed task contracts when reached.

| Milestone | Focus | Done when |
| --- | --- | --- |
| Q1: Model, File, and First Sheet | Instrument definitions, pitch/number identities, parts/voices, timing, basic numerical sheet, desktop open/save | Create a small composition, save it, close the app, reopen it, and recover the same musical content and presentation choices |
| Q2: Useful Gamelan Composition | Expand the chosen numerical notation: register/rhythm marks, grouping, cycle structure, annotations, editing, undo/redo, and instrument changes | Compose and revise a useful short gamelan score without editing its JSON |
| Q3: Playback | Tuning resolution, instrument/sample mappings, transport, tempo, dynamics, damping, and audio export | Hear the score with the declared instrument/tuning and verify playback timing |
| Q4: Publishing | Pagination, full score/parts, PDF, printing, and layout overrides | Produce readable rehearsal and performance sheets |
| Q5: More Notation Systems | Western staff notation, then separately scoped solfa/tab/other providers | Another notation works through the same core without flattening its meaning |
| Q6: Interchange and Connections | Supported MusicXML/MIDI subsets, OSC/Max, and separately scoped Link synchronization | Exchange/connect with explicit compatibility and loss reporting |
| Q7: Release Reliability | Recovery, migrations, installers, accessibility, performance, and platform testing | The claimed workflows reliably work on macOS and Windows |

### Q1 execution order

1. Fix the small document and instrument contracts.
2. Implement validation and save/load.
3. Prove that saving preserves every supported field.
4. Generate the basic numerical sheet.
5. Connect the desktop editor and file dialogs.
6. Verify the complete reopen workflow.

The six Q1 task cards also include a minimal bootstrap task so the contracts and
headless checks can run. Bootstrap does not replace the model-first design order.

The `.sdlk` preserves events, instrument configuration, pitch/tuning context,
timing, notation intent, and presentation settings. External resources need stable
identities; document-critical definitions must remain recoverable. Large sound
libraries can remain external. "Same copy" means equivalent musical meaning and
saved presentation, not identical JSON bytes. Computed layout is regenerated using
compatible provider/resources. Verify this distinction in round-trip tests.

Q1 establishes a minimal coherent model, not a permanently finished universal
schema. Q2 onward may extend it through explicit versions and migrations.
MuseScore-equivalent behavior remains individually tracked in the feature matrix;
it is not an implicit prerequisite for creating and reopening a gamelan sheet.

## Active first milestone: Q1

Deliver the [gamelan sheet editor](11-gamelan-sheet-quick-win.md) first. It combines
small pieces of document/editing/persistence, numerical SVG, and desktop UI into
one visible milestone, without waiting for audio, piano engraving, PDF, or packs.
Keep the metric/spelled-pitch fixture as a model test. Its fixed contract is the
Q1 implementation scope; code is present, but Q1 acceptance remains incomplete.
Broader M0 choices still require later review.

Q1 exit: a user starts an empty document, enters/revises numerical notes/rests,
adds a cycle, undoes/redoes changes, saves/reopens the score, and exports the same
sheet as SVG. Rust owns document state and layout. No audio is claimed. Record
native evidence for each OS separately; a local macOS win does not verify Windows.

After Q1, expand useful gamelan composition in Q2, then add sound in Q3.
Finish relevant foundation checks as their features are introduced. Piano UI is
Q5; its model fixture remains an early neutrality check. Q1 does not mark the
original M0-M3 workstreams complete.

## Original workstream reference: M0-M8

| Milestone | Product state | Dependency | Exit evidence |
| --- | --- | --- | --- |
| M0 | Reviewed foundations | None | Contracts, two full fixture specifications, resolved initial spikes |
| M1 | Headless editable document | M0 | Validated model, commands, persistence, CI |
| M2 | Headless end-to-end proof | M1 | Both fixtures generate basic SVG and meaningful WAV |
| M3 | Internal desktop alpha | M2 and native-audio spike | Both compositions created/edited/reopened/played through the UI |
| M4 | Usable publishing subset | M3 | Defined notation coverage, page/part preview, SVG/PDF/print evidence |
| M5 | Reliable rehearsal audio | M3 and relevant M4 semantics | Transport, profiles, mixer, export, platform measurements |
| M6 | Proven instrument extension | M4 and M5 | New data pack works without domain-model changes |
| M7 | First public release | M4-M6 | Installer/runtime, recovery, musician/usability, and compatibility gates |
| M8 | Professional expansion | M7; staged independent proposals | Each approved feature has its own fixtures and release gate |

M4 and M5 can be interleaved once shared semantics are fixed; this does not require
parallel agents. M6 validates contracts drafted earlier, rather than introducing
extensibility for the first time after a monolith has been built.

## M0: Product and architectural contracts

### Features and artifacts

Product workflows, initial feature scope, ScoreDocument subset, core invariants,
provider contracts, output diagnostics, and two contrasting fixture specifications.
Choose an explicit initial numerical convention and a Western piano subset.

### Task sequence

1. M0-01: Review this planning baseline; resolve product-name/platform/scope
   ambiguities and assign requirement ownership. Do not expand into DAW features.
2. M0-02: Specify a short piano fixture with two staves, rests, simultaneous notes,
   a simple tie, spelled pitches, and a declared tempo/tuning context.
3. M0-03: Specify a short cycle-based numerical fixture with degree/register
   identity, structural marks, synthetic tuning, and strike/damping expectations.
4. M0-04: Define exact time units, repeat/voice relations, pitch references,
   persistence versions, unknown-extension policy, and public editing commands.
5. M0-05: Run a small Tauri/SVG/font/audio-device feasibility spike on the target
   platforms. Evaluate PDF and SoundFont candidates without implementing a framework.
6. M0-06: Finalize M1 contracts and independent acceptance cases. Record unresolved
   advanced questions as deferred, not as silent implementation freedom.

### Done means

- A developer can explain how both fixtures produce notation and performance
  actions from the same model; all time/pitch references resolve unambiguously.
- The fixture expectations are written before renderer/scheduler implementations.
- The first JSON schema/API subset and compatibility rules are reviewed.
- Libraries selected for immediate use have maintenance, platform, and license
  evidence. A failed spike revises the decision before dependent work continues.
- No requirement says "support anything automatically" or promises complete parity.

### Exclusions and routing

No full editor, DAW engine, general plugin runtime, or production engraving.
Sol owns contracts and reviews; Luna can collect focused evidence and format
reviewed fixtures. Escalate domain ambiguities to Sol and a musician when needed.

## M1: Rust document, editing, and persistence

### Features

Core types and IDs; normalized rational timing; parts/voices; pitched/unpitched
sound, rests, annotations; context and relation validation; command transactions;
undo/redo; metadata; `.sdlk` encoding; schema versioning and safe saves.

### Task sequence

1. M1-01: Create the minimal Rust workspace/toolchain and format/test/lint CI on
   macOS and Windows. Only create the crates needed for this milestone.
2. M1-02: Implement IDs, checked rational values, temporal anchors, and contexts.
3. M1-03: Implement event/part/voice/relation structures and semantic validation.
4. M1-04: Implement the agreed editing commands with all-or-nothing transactions,
   revision checks, and bounded undo/redo.
5. M1-05: Implement JSON persistence, typed diagnostics, resource references,
   unknown optional payload preservation, and atomic save/save-as behavior.
6. M1-06: Test a controlled schema migration/refusal path and full fixtures.

### Done means

- Both complete fixtures validate and survive save/reopen with equivalent meaning.
- Failed edits leave the document unchanged; undo/redo restores known states.
- Dangling IDs, duplicate IDs, invalid rationals, overflow, missing required
  contexts, and unsupported core versions have explicit diagnostic cases.
- Simulated failed saves preserve the previous valid file; migrations do not
  overwrite their source on failure.
- No domain dependency on Tauri, React, a device, or a mandatory MIDI representation.
- Formatting, relevant tests, and lint gates run in CI. Commands are documented.

### Exclusions and routing

No polished UI, audio-device integration, or arbitrary notation playback.
Sol implements/reviews semantic arithmetic and transaction contracts. Luna takes
scoped validation, serialization, and fixture tasks after interfaces are fixed.

## M2: Headless notation and audio proof

### Features

Built-in Western/numerical provider subsets; basic layout primitives and SVG;
constant-tempo/domain mapping; frequency/sample/stroke resolution; simple
performance plan; deterministic test synth and small licensed sample fixture;
offline WAV; capability diagnostics.

### Task sequence

1. M2-01: Implement notation interpretation for only the accepted fixture subset.
2. M2-02: Implement basic geometry/SVG with stable IDs and pinned font resources.
3. M2-03: Implement logical-time and pitch resolution plus performance occurrences
   required by the fixtures. Add tie/damping rules explicitly.
4. M2-04: Render the performance plan offline with the test synth and sample case.
5. M2-05: Produce and inspect SVG/WAV reference outputs through a small test driver.
6. M2-06: Test missing-provider/resource states and alternate-view diagnostics.

### Done means

- Both fixtures produce nonblank, musically interpretable SVG and non-silent WAV.
- Known onset/duration/frequency/attack-count/damping cases match expectations.
- Musical reference identity survives rendering, including degree versus spelled
  pitch. A SoundFont or MIDI backend is not required to prove the model.
- Layout diagnostics and audio diagnostics distinguish supported behavior from
  placeholders. A musician inspects the selected notation conventions.
- Source provenance and fixture asset licenses are recorded.

### Exclusions and routing

No claim of full piano engraving, complete ensemble sound realism, or latency.
Sol owns realization rules and provider contract review. Luna implements bounded
SVG/output/fixture components under those contracts.

## M3: Desktop internal alpha

### Features

Tauri + React/Vite app; revisioned IPC; score viewport and selection; properties;
instrument/context choice; event entry/edit/delete; copy/paste subset; undo/redo;
open/save/save-as; resource status; basic play/pause/stop/seek; zoom and keyboard
focus; unsaved-change warning; native audio output.

### Task sequence

1. M3-01: Scaffold desktop/UI and connect one read-only projection/layout request.
2. M3-02: Wire typed transactions, result/error states, and stale-result rejection.
3. M3-03: Add input/selection/properties/undo and the supported copy/paste policy.
4. M3-04: Add file dialogs, resource selection, dirty-state/close behavior.
5. M3-05: Add prepared-plan native playback and explicit initial seek/edit policy.
6. M3-06: Run reference user journeys and record baseline UI/audio performance.

### Done means

- A user creates and edits both fixture compositions without modifying JSON.
- Save/reopen and undo work through the actual UI; selection survives relayout.
- Native playback is audible and its document/revision state is clear.
- Missing device/resources and rejected commands are recoverable UI states.
- The frontend never schedules audio notes using web timers.
- Browser component tests and native macOS/Windows smoke checks are distinct
  recorded evidence. Platform gaps keep the gate open.

### Exclusions and routing

Not public-release quality. No claim of professional PDF, plugin hosting, or live
recording. Luna handles UI wiring and scoped IPC work; Sol handles callback/lifecycle
ownership and reviews shared-state changes.

## M4: Defined engraving and publishing subset

### Features

Expand Western/numerical notation coverage; chords, multiple voices, rests, ties,
slurs, tuplets, grace notes, beams, dynamics, articulations, text, context changes;
page/system layout; persistent overrides; basic part views; SVG/PDF and print preview.

### Task sequence

1. M4-01: Freeze the first publishing subset with independent musical fixtures.
2. M4-02: Implement grouping/spacing/collision rules for each supported feature.
3. M4-03: Implement page/system breaks, margins, title/credits, and basic part views.
4. M4-04: Add export backend/font handling and verify screen/PDF semantic agreement.
5. M4-05: Implement override editing/reset and conversion/capability reporting.
6. M4-06: Review dense and edge-case examples with musicians and print on both OSes.

### Done means

- Every claimed feature has a semantic test and an inspected visual example.
- Expected accidental/register/rhythm/grouping rules work; unsupported cases
  remain explicit. No undocumented symbol substitution or musical data loss.
- Exports have correct page dimensions, readable text, consistent symbols, and
  usable print output; part filters do not alter the full score.
- Golden changes are explained and reviewed, not automatically accepted.

### Exclusions and routing

Advanced cross-staff notation, arbitrary graphic notation, and professional linked
parts remain M8 scope. Sol owns new engraving semantics; Luna handles bounded
rules, controls, and regression cases once their expected geometry is explicit.

## M5: Reliable rehearsal playback and sound profiles

### Features

Piecewise tempo changes, scoped ramp policies, repeat/section traversal, dynamics,
articulation and damping interpretation; robust transport/seek/range loop;
metronome/count-in; profile selection; sampler and SoundFont adapter; part mixer;
WAV settings and tails; native device errors and performance diagnostics.

### Task sequence

1. M5-01: Expand performance policies and reference occurrence/timing tests.
2. M5-02: Add sound-profile/sample mapping and chosen SoundFont engine adapter.
3. M5-03: Implement robust transport reconstruction and revision replacement.
4. M5-04: Add gain/pan/mute/solo/meter controls and documented clipping policy.
5. M5-05: Align native/offline behavior, export settings, and cancellation.
6. M5-06: Measure callback deadlines, underruns, memory, and transport on both OSes.

### Done means

- Repeats, tempo boundaries, ties, release tails, and damping match accepted rules.
- Native and offline outputs agree within a declared tolerance/configuration.
- Seeking/looping does not leave stuck voices or obsolete controller state.
- SoundFont/sample resources resolve, relink, and report failures correctly.
- Target buffer workloads meet measured budgets on recorded hardware; tests and
  listening identify artifacts rather than relying on a successful build.

### Exclusions and routing

No live recording, effects workstation, sample editor, VST/AU hosting, or MIDI 2
promise. Sol owns concurrency, lifecycle, and difficult realization; Luna handles
profile forms, bounded adapters, and mixer/UI tasks with explicit contracts.

## M6: Demonstrated instrument extensibility

### Features

Versioned instrument/tuning/resource manifests; data validation; capability
registry; compatible profile/provider selection; pack install/remove and relink;
missing/unknown payload persistence; one contrasting new instrument pack.

### Task sequence

1. M6-01: Freeze manifest/registry compatibility contracts from actual M4/M5 needs.
2. M6-02: Add pack parsing, validation, resource identity, and scoped installation.
3. M6-03: Add one instrument that uses existing providers without modifying core
   event/time/pitch types. Examples: a second tuned percussion mapping or instrument.
4. M6-04: Remove/reinstall resources and verify document preservation and recovery.
5. M6-05: Publish a pack-author guide with limits and a working licensed example.

### Done means

- The new data pack can be added without changing the domain model.
- Unknown optional extensions survive open/save; required unsupported behavior
  restricts the affected operation with a clear reason.
- Existing fixture behavior and public interfaces remain compatible.
- A manifest cannot execute code or escape its allowed resource paths.

### Exclusions and routing

No marketplace or arbitrary executable plugins. Sandbox/ABI behavior remains
M8 research. Luna handles agreed schemas/pack UI; Sol reviews capability contracts.

## M7: Public release readiness

### Features

Autosave/recovery, migrations/backups, first-run resource flow, diagnostics,
keyboard/accessibility improvements, native installer/file associations, versioned
release notes, compatibility report, dependency/assets license inventory.

### Task sequence

1. M7-01: Add and failure-test autosave/recovery and previous-schema compatibility.
2. M7-02: Run complete compose/reopen/play/publish journeys with reference users.
3. M7-03: Build/install/uninstall on macOS and Windows; verify paths/fonts/devices.
4. M7-04: Finish packaging/signing/distribution decisions and relevant notices.
5. M7-05: Resolve release blockers and publish an honest feature/limitation matrix.

### Done means

- No known score-loss, failed-save corruption, callback-lifecycle crash, or silent
  unsupported conversion in the claimed release workflows.
- Recovery and migrations pass injected failure tests; dependencies/resources
  have approved distribution licenses and attribution.
- Native installation and runtime are verified on both platforms, not inferred.
- Musicians can complete the core journeys; known limitations are documented.
- Performance baselines/regressions and accessibility results are reviewed.

### Exclusions and routing

Release does not claim all MuseScore/Sibelius/Ableton features. Any platform or
feature with no runtime evidence stays explicitly unsupported. Sol reviews the
release gate; Luna addresses well-defined bugs/docs/packaging tasks.

## M8: Professional expansion, one proposal at a time

M8 is a roadmap family, not one enormous implementation task:

- M8-A interchange/integration: supported MusicXML and MIDI import/export,
  external MIDI/OSC, Max examples, Link clock policy and synchronization tests.
- M8-B notation/publishing: solfa, tablature/percussion, advanced voices/beams/
  tuplets/cross-staff notation, lyrics, transposition, linked parts and page turns.
- M8-C extensibility: independently versioned executable providers, sandbox,
  capability negotiation, migration and lifecycle/error containment.
- M8-D product proposals: richer timelines, automation, effects, live recording,
  plugin hosting, collaboration, or additional platforms only after user research
  and technical/licensing feasibility.

For each submilestone, specify a user/job, format/provider contract, dependencies,
loss/compatibility policy, independent fixtures, platform tests, and release gate.
Sol plans risky boundaries; Luna implements bounded tasks after those are settled.
No future feature gets universal support merely because `extensions` exists.

## Milestone review packet

Use the [task template](../templates/task.md) for individual work. At a milestone
gate collect satisfied requirement IDs, changed public contracts, fixture outputs,
test/platform commands and results, musician/usability review, performance data,
remaining limitations, and the next approved tasks. Any unmet exit criterion
keeps that milestone open; it does not require stopping unrelated useful work.

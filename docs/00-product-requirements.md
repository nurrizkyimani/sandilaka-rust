# Product requirements

Status: proposed product baseline. Owner: product/planning owner. Scope changes
are recorded in the feature matrix and a decision record.

## Purpose and positioning

Sandikala helps people compose, arrange, rehearse, and publish music whose timing,
tuning, and notation may differ from Western staff conventions. It must also
become useful for ordinary Western composition. The product is neutral; gamelan
is an early validation case, not an assumption embedded in every subsystem.

The initial product is a desktop composition and notation application, not a
complete DAW. Long-term ambitions include professional notation and integration
with external music tools. Ableton/Sibelius/MuseScore feature ambitions become
tracked requirements; they are not blanket release promises.

## Users and jobs

| User | Job | Product success |
| --- | --- | --- |
| Composer or arranger | Write and revise a piece in a suitable notation | Input is predictable and changes retain musical meaning |
| Ensemble musician or teacher | Prepare parts and listen to a rehearsal rendering | Clear notation, useful playback, tunings tied to the actual ensemble |
| Researcher or transcriber | Preserve nonstandard notation and tuning | Source context, symbols, and unsupported information survive saving |
| Piano or Western-score user | Work with familiar staff conventions | Basic editing and publishing do not feel like a conversion workaround |
| Pack author | Add instrument data or supported notation behavior | Documented boundaries and validation identify what data alone can express |

## Essential journeys

1. Create a document, choose instruments and notation views, set a time/tuning
   context, enter events, revise, save, and reopen offline.
2. Hear the composition, choose an available sound profile, pause/seek, adjust
   tempo or dynamics, and export a rehearsal audio file.
3. Prepare a full score or part, inspect page layout, adjust notation presentation,
   and export a printable file without changing the composition's pitches.
4. Open a score that references an unavailable instrument pack or sound library.
   Preserve the document, identify what is missing, and let the user relink it.
5. Display an alternate notation when supported. Explain omissions or ambiguity;
   do not pretend every notation conversion is lossless.
6. Add an instrument using existing tuning, notation, and playback capabilities.
   If new behavior is required, make the provider requirement explicit.

## Functional requirements

| ID | Required behavior |
| --- | --- |
| PR-01 | Desktop operation on macOS and Windows; compose/save/play offline with installed resources |
| PR-02 | Create/open/save/save-as documents, including title, composer, credits, and other metadata |
| PR-03 | Parts, instruments, voices, ordered sections, and editable document structure |
| PR-04 | Add/select/edit/delete/copy/paste events; undo/redo and atomic multi-event operations |
| PR-05 | Exact logical timing; metric, cyclic, and explicitly untimed structures with defined interpretation |
| PR-06 | Explicit pitch/tuning contexts, ensemble-specific tuning, and unpitched percussion identities |
| PR-07 | Western and numerical notation providers with preserved spelling/register/rhythm intent |
| PR-08 | Additional notation providers, including solfa and future graphic systems, through defined contracts |
| PR-09 | Chords, rests, voices, ties, tuplets, grace notes, slurs, articulations, and text with staged coverage |
| PR-10 | Score/part layout, reliable symbols/fonts, pagination, SVG/PDF export, and print workflows |
| PR-11 | Play/pause/stop/seek; later looping, count-in, metronome, and configurable rehearsal controls |
| PR-12 | Tempo changes, repeats, section navigation, dynamics, and documented performance interpretation |
| PR-13 | Sample playback and selectable sound profiles; SoundFont adapter and synthesized fallback |
| PR-14 | Mixer controls: part gain, mute/solo, pan, routing; audio export with documented settings |
| PR-15 | Data-driven instrument/resource packs with validation and missing-resource handling |
| PR-16 | MIDI import/export and external MIDI output with explicit quantization/tuning-loss reports |
| PR-17 | MusicXML interchange for a documented supported Western subset, with loss reports |
| PR-18 | OSC connection to compatible environments such as Max; independently scoped Link synchronization |
| PR-19 | Autosave, recovery, version migrations, diagnostics, and preservation of unknown extension payloads |
| PR-20 | Keyboard/pointer editing, readable controls, zoom, focus navigation, and accessible noncanvas UI |
| PR-21 | License/source metadata for code, fonts, packs, samples, and imported content |
| PR-22 | A capability/compatibility report distinguishes storage, editing, engraving, and playback support |

PR-05 does not promise immediate support for every independent tempo domain or
free-time interpretation. PR-08 does not promise an automatic renderer for an
unknown notation. See the staged scope in the feature matrix.

## Nonfunctional requirements

| ID | Constraint and evidence |
| --- | --- |
| NFR-01 | Deterministic core semantics: identical input/configuration gives equivalent resolved events and layout |
| NFR-02 | Data integrity: atomic saves, validated references, migration fixtures, recovery, and explicit conversion losses |
| NFR-03 | Modular ownership: the headless core has no dependency on a webview or desktop runtime |
| NFR-04 | Responsive editing and reliable audio demonstrated on recorded representative hardware/workloads |
| NFR-05 | Native platform testing; a successful browser test does not replace macOS/Windows runtime verification |
| NFR-06 | Restrained permissions, bounded parsing, safe asset access, and no mandatory telemetry or cloud account |
| NFR-07 | Maintainable code, supported dependencies, documented interfaces, and independent behavioral tests |
| NFR-08 | Usability/accessibility checks and musician evaluation for the claimed notation systems |

## Release scope

- Q1, the active first visual milestone: numerical sheet entry/editing, undo,
  save/reopen, and SVG export. A piano-shaped model fixture protects neutrality;
  audible playback and piano engraving are outside this gate. See the
  [fixed Q1 contract](11-gamelan-sheet-quick-win.md).

- M0-M2: engineering proof, not a consumer product.
- M3: internal alpha with editing and basic audible playback for the two fixtures.
- M4-M6: usable notation/audio workflow and a demonstrated instrument extension.
- M7: first public release after platform, reliability, and usability gates.
- M8: staged professional expansion; release labels describe actual coverage.

No mandatory account, cloud collaboration, arbitrary native plugins, plugin
marketplace, live audio recording, VST/AU hosting, nonlinear clip launcher, mobile
app, or Linux support is required for the first release. They can be reconsidered
as separately costed product decisions. Audio recording and plugin hosting are
not small side effects of adding sample playback.

## Product measures

Record whether reference users can finish the essential journeys, time to finish,
input mistakes, lost work, confusing conversions, and missing notation/sound
capabilities. Track recovery failures, failed saves, audible glitches, and export
discrepancies. Set release thresholds after M3 baseline sessions. Neither token
count nor number of supported symbols is an adequate measure of product quality.

## Requirement changes

Every new feature receives a user/job, requirement ID, milestone, acceptance
example, compatibility implications, and owner. Change a requirement by updating
this document and its feature-matrix row, not by quietly redefining a test.

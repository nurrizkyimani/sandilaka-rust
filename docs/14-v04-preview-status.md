# Target v0.4: score studio preview

This delivery advances Milestones 2-4 (useful composition, playback, publishing)
and brings forward a bounded piano notation subset. It is not certification of
stable v0.4 or complete MuseScore parity. Package version remains 0.1.0 pending
the acceptance gates below; the document schema remains 0.1.0-q1 with explicitly
versioned provider extensions.

| Domain | Implemented | Evidence / remaining limits |
| --- | --- | --- |
| Editor | Dark paged canvas, instrument/staff selection, numeric and piano input, register controls, durations, rests, fast input queue, undo/redo, single-event copy/paste | Playwright exercises real Rust layout with delayed mocked IPC; native dialogs and keyboard journeys still require interactive review |
| Rhythm | Rational quarter-unit timeline, fixed global meter, configurable quarter/eighth/sixteenth grid, event starts/ends in selectable columns | 7/8 with one eighth note retains six dots; ties, tuplets and mixed/additive meter changes remain outside the supported subset |
| Engraving | Original synchronized interval columns, weighted spacing, DP system breaks, pagination, ledger lines, clefs, noteheads, individual stems/flags, numeric overbars/register dots | Bundled licensed Leland/Edwin; no copied MuseScore layout code. Multi-note beams, chords and comprehensive collision tests remain |
| Score/parts | Up to eight catalog parts, two-staff instruments, full-score/part views, A4/Letter, orientation, manual system breaks | Generated 120-event, five-staff, three-page fixture; all PDF pages visually checked |
| Playback | Per-part SoundFont routing, piano synthesis preview, tempo, exact-time scheduling, pause/resume/seek, output cursor | Real Saron/Gendèr samples and mixed native output test passed on macOS; other profiles need complete register/pitch coverage and musical listening review |
| Mixer/marks | Saved gain/pan/mute/solo, event velocity and short text annotations | Transaction validation, undo and round-trip tests; annotation XML escaping verified |
| Publishing | SVG, self-contained HTML, embedded-font vector PDF, native print entry, float WAV audio export | PDF generated and rasterized for review; native save/print dialogs and physical print workflow not interactively verified |

## Recorded checks

- Rust workspace tests pass (43 active tests; three host-dependent audio tests
  excluded from the ordinary workspace run).
- All ten desktop tests pass with host audio enabled and serialized, using the
  existing local Saron SoundFont. This proves rendered PCM and device execution,
  not a human assessment of sample authenticity.
- Six frontend unit tests and two Playwright journeys pass. Browser journeys use
  mocked IPC; they do not prove native file dialogs or persistence across restarts.
- Frontend production build and Rust Clippy with warnings denied pass.
- `output/pdf/score-studio-preview.pdf` has three vector A4 pages. Its fonts and
  notation were checked by rasterizing every page; UI screenshot saved alongside.

## Gates still open before stable v0.4

1. Native create/edit/save/close/reopen/print journey on macOS, and separate Windows
   evidence. Do not infer this from browser mocks or codec tests.
2. Musician review of numeric rhythm conventions, continuation marks, register
   placement, piano spacing and readability at rehearsal print size.
3. Faithful tuning resolution, damping and pitch/register coverage for each
   claimed gamelan profile. Current degree-to-key routing is not proof that the
   recording's tuning matches every document context.
4. Unsupported notation must remain explicitly bounded. The supplied advanced
   examples need separate contracts for tuplets, grace, ties, additive/mixed meter
   changes, multi-note beams and percussion symbols.
5. Recovery, migration, accessibility, performance and packaged runtime checks.

No new Git commit or release package is created by this delivery.

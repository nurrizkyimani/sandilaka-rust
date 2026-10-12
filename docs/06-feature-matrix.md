# Feature and requirement matrix

All rows below are planned. A target milestone is not an implemented claim. M8
items need separate proposals before execution. Update this matrix when scope
changes. Record implementation evidence in task/milestone review packets.

## Q1 scope overlay

The [active Q1 route](11-gamelan-sheet-quick-win.md) delivers limited early subsets
of PR-02/03/04/05/06/07/10/19/20/22 and NFR-01/02/03/06/07. It includes numerical
editing, exact stored positions, degree/spelled model forms, persistence, history,
and SVG output. It does not complete those broad requirements: no copy/paste,
audio, piano rendering, PDF, migration from other released versions, or pack system
is claimed. The first-target column below describes the larger original roadmap.

## Product requirement traceability

| ID | Feature | First target | Completion evidence |
| --- | --- | --- | --- |
| PR-01 | macOS/Windows offline desktop | M3, release M7 | Native compose/save/play journeys on both platforms |
| PR-02 | Create/open/save-as and metadata | M1, UI M3 | Round trips, failed-save tests, dirty-state flow |
| PR-03 | Parts/instruments/voices/sections | M1 | Ownership/reference and structure fixtures |
| PR-04 | Editing transactions/copy-paste/undo | M1, UI M3 | All-or-nothing changes and restored state |
| PR-05 | Exact metric/cyclic/untimed time | M1, realization M2/M5 | Exact coordinates and explicit unsupported mappings |
| PR-06 | Tuning/pitch/stroke contexts | M1-M2 | Resolved frequencies/strokes and missing-context tests |
| PR-07 | Western/numerical views and intent | M2, useful subset M4 | Semantic and visual fixture review |
| PR-08 | Additional notation providers | M8-B/M8-C | One specified system with editing/output support |
| PR-09 | Detailed musical notation | M4, advanced M8-B | Per-feature semantic/layout/performance cases |
| PR-10 | Score/part publishing | SVG M2, PDF/print M4 | Output inspection and platform printing |
| PR-11 | Transport and rehearsal controls | Basic M3, robust M5 | Lifecycle, seek/loop and native checks |
| PR-12 | Tempo/repeats/dynamics interpretation | Basic M2, expanded M5 | Timing/occurrence/voice tests |
| PR-13 | Samples/SoundFonts/test synth | M2/M5 | Licensed fixtures, backend audio, missing-resource UI |
| PR-14 | Mixer/routing/audio export | WAV M2, mixer M5 | Gain/pan/mute/solo and export configuration tests |
| PR-15 | Instrument/resource packs | M6 | New pack without domain changes and relink tests |
| PR-16 | MIDI interchange/external output | M8-A | Interchange fixtures and explicit loss reports |
| PR-17 | MusicXML supported subset | M8-A | Independent reference round trips and loss reports |
| PR-18 | OSC/Max and Link | M8-A | Receiver patch, clock policy and interoperability tests |
| PR-19 | Recovery/migrations/extensions | Foundation M1, release M7 | Failure injection and unknown-payload preservation |
| PR-20 | Keyboard/pointer/accessibility | M3, audited M7 | Focus/selection/user and assistive UI checks |
| PR-21 | Licensing/provenance | M0 onward, audited M7 | Inventory and allowed distribution evidence |
| PR-22 | Capability/compatibility reporting | M2 onward | Unsupported/missing provider output states |
| NFR-01 | Deterministic core | M1 onward | Equivalent plans/layout for identical inputs |
| NFR-02 | Data integrity | M1/M7 | Atomic save/recovery/migration tests |
| NFR-03 | Headless modular core | M1 onward | Dependency check and device-free tests |
| NFR-04 | Responsiveness/reliable audio | Baseline M3, gate M5/M7 | Recorded workloads and regression measures |
| NFR-05 | Native platform evidence | M1 CI, M3/M7 runtime | Separate macOS/Windows results |
| NFR-06 | Permissions/bounded parsing/offline | M1 onward | Parser/resource failure and capability checks |
| NFR-07 | Maintainability/tests/dependencies | M0 onward | Contract reviews, CI, dependency decisions |
| NFR-08 | Usability/accessibility/musician review | M0/M3/M4/M7 | Reviewed notation and completed user journeys |

## Detailed feature staging

| Feature group | Initial implementation | Later expansion |
| --- | --- | --- |
| Document lifecycle | Create/open/save/save-as, metadata, versioned JSON M1/M3 | Portable asset package proposal M8 |
| Safety | Validation/atomic saves M1; unsaved-close guard M3 | Autosave/recovery/compatibility audit M7 |
| Input | Select/insert/update/delete, duration/context properties M3 | Rich keyboard workflows and advanced tools M8-B |
| Clipboard | Scoped event/relation copy-paste M3 | Cross-system conversion with conflict choices M8-B |
| History | Atomic transactions and bounded undo/redo M1/M3 | Persistent history only if a user need justifies it |
| Organization | Parts/voices/sections M1; basic part views M4 | Professional linked parts/transposition M8-B |
| Time | Rational metric/cycle coordinates M1 | Independent clocks/seconds/frames and free-time policies M8 |
| Meter | Defined beat units and fixture meter M2 | Meter changes M4; complex polymeter M8-B |
| Cycles | Pulse length and structural anchors M1/M2 | Multiple interacting cycle conventions by proposal |
| Pitch | Spelled/degree/frequency/stroke references M1/M2 | Ratio/symbolic systems with real providers M8 |
| Tuning | Piano reference and synthetic ensemble table M2 | User-facing calibration/editor proposal M8 |
| Views | Western/numerical fixture view M2/M3 | Solfa/tab/percussion/graphic systems M8-B |
| Chords/voices | Explicit simultaneous grouping M1/M2 | General chord and multi-voice layout M4 |
| Duration/rests | Supported basic rhythms/rests M2 | Dotted values/complex subdivisions by M4 subset |
| Ties/slurs | Fixture tie M2; distinct relation semantics | General engraving and performance policies M4/M5 |
| Beams/tuplets | Simple fixture grouping M2 | Declared useful subset M4; advanced forms M8-B |
| Grace/ornaments | Explicit unsupported states initially | Defined M4 display/M5 performance; advanced M8-B |
| Register/accidentals | Fixture marks and spelling preservation M2 | Context changes and collision rules M4 |
| Dynamics/articulations | Stored intent and selected fixture policy M2 | Broader display/performance mapping M4/M5 |
| Lyrics/text | Metadata and simple annotations M1/M2 | Layout text M4; lyrics underlay M8-B |
| Repeats | Stored structure and bounded realization contract | User-facing scoped repeats/sections M5; advanced jumps M8 |
| Tempo | Constant tempo M2 | Piecewise/ramp policy M5; advanced clock relationships M8 |
| Layout | Basic SVG/IDs/hit geometry M2 | Systems/pages/overrides M4; incremental layout if measured |
| Publishing | SVG M2 | PDF/print/basic part preview M4; advanced parts M8-B |
| Navigation | Zoom/pan/selection M3 | Efficient large-score navigation driven by user evidence |
| Transport | Basic play/pause/stop/seek M3 | Range loop/count-in/metronome/reconstruction M5 |
| Native audio | Prepared-plan device playback M3 | Device errors/performance gate M5 |
| Sound | Test synth and licensed sample fixture M2 | Profiles, sample mapping, SoundFont adapter M5 |
| Mixer | Basic backend gain sufficient for fixture | Gain/pan/mute/solo/meters/basic routing M5 |
| Export audio | Defined small WAV path M2 | Settings/ranges/tails/cancellation M5; other codecs by proposal |
| Resources | Referenced sound/font identities M1/M2 | Pack management/relink and licensing UI M6/M7 |
| Extensibility | Built-in contracts and preserved namespaces M1 | New instrument pack M6; executable sandbox M8-C |
| MIDI | Not required as internal representation | Import/export/external output subset M8-A |
| MusicXML | Not required as native format | Documented supported-subset interchange M8-A |
| OSC/Max | No mandatory external connection | Address/schema contract and patch example M8-A |
| Ableton Link | No mandatory network sync | Clock-policy and sync integration M8-A |
| Accessibility | Focusable controls/keyboard and companion UI M3 | Audit and refinement M7, expanded notation support M8 |
| Distribution | Development builds M3 | Native installers/signing/file association M7 |

## Ambitions requiring new product decisions

Complete MuseScore/Sibelius parity, DAW clip launching, live multitrack recording,
VST/AU hosting, effects/mastering, automation editor, notation marketplace,
collaboration, mobile, and additional platforms are not accepted M7 scope.

For professional notation parity, maintain a granular public-behavior checklist
covering each intended feature and reference output. Mark supported, partial,
unsupported, or deferred with evidence. Do not copy another application's code
to obtain parity, or use a single generic "parity complete" checkbox.

New feature proposals must state whether they need a model extension, a provider,
a backend, UI work, or all four. The ability to preserve a payload is not a claim
that any of those behaviors already exists.

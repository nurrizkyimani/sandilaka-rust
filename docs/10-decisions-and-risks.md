# Decisions, open questions, and risks

Baseline date: 2026-10-02. These entries distinguish product direction from
unvalidated implementation choices. Change a decision with its rationale,
affected requirement IDs, migration implications, and acceptance evidence.

## 2026-10-12 execution review

The baseline is architecturally suitable but was not a complete implementation
contract: schemas, integer bounds, command shapes, undo policy, and provider
interfaces remained open, and audio preceded the first visible desktop editor.
The [Q1 contract](11-gamelan-sheet-quick-win.md) freezes those choices for a small
numerical sheet editor and supplies independent fixtures. This is the active
delivery sequence. Wider contracts remain proposals until their own gates pass.

The chosen display is a numerical engineering demo with explicit scope, not an
assertion of authoritative gamelan notation. Musician validation is still required
before advertising cultural fidelity. Synthetic tuning values remain test data.

## Decision register

| ID | Decision | Status | Revisit trigger |
| --- | --- | --- | --- |
| D-01 | Rust owns the authoritative musical document and editing transactions | Accepted direction | A demonstrated requirement that cannot fit the ownership boundary |
| D-02 | Typed ScoreDocument/events/relations, independent notation/performance paths | Accepted direction | A contrasting fixture exposes a missing primitive |
| D-03 | No compulsory Western/MIDI/numerical pitch or measure hierarchy | Accepted direction | Never replace neutrality merely for adapter convenience |
| D-04 | Tauri + React/TypeScript + Vite desktop shell | Proposed, M0 spike | Target-platform rendering, integration, accessibility, or distribution blocker |
| D-05 | Small workspace with internal modules before many independent crates | Initial policy | Dependency/ownership/reuse/build evidence supports a split |
| D-06 | Versioned plain JSON `.sdlk` with resource references first | Proposed, M0/M1 | Measured load/save or portability requirement justifies new encoding/container |
| D-07 | Built-in providers and validated data packs before executable plugins | Initial scope | A real external provider needs safely executable new behavior |
| D-08 | SVG first, shared layout output for later PDF | Proposed, M0/M4 spike | Font/text/backend or measured viewport limit |
| D-09 | Native prepared-plan audio; CPAL is the initial device candidate | Proposed, M0/M3 spike | Device/latency/lifecycle/platform or maintenance evidence |
| D-10 | SoundFont is an adapter, not the canonical sound or score representation | Accepted direction | Adapter selection changes, not document meaning |
| D-11 | Exact bounded logical arithmetic; explicit domain mappings | Accepted principle, bounds open M0 | A real workload exceeds bounds; versioned expansion required |
| D-12 | Sol plans/reviews risky contracts; Luna executes bounded tasks | Trial workflow | Measured acceptance/repair/review cost suggests different routing |
| D-13 | Professional parity is granular staged scope, not a first-release promise | Product scope | User research funds a concrete expansion proposal |

## Questions to settle before dependent work

| Question | Owner/gate | Required answer |
| --- | --- | --- |
| Which numerical convention and musical reviewers? | Product/domain, M0 | Explicit symbol/rhythm/register/cycle rules and reviewer evidence |
| Rational bounds and pickup/origin rules? | Core owner, M0 | Checked-arithmetic and serialization contract |
| Chord/group ownership and ties across voices? | Core/notation, M0 | Valid examples, invalid cases, and extent/performance interpretation |
| Undo representation and memory cap? | Application, M1 | Simple tested strategy and exhaustion policy |
| PDF backend and notation/text fonts? | Engraving, M0/M4 | Correct output, platform compatibility, and distribution licenses |
| Audio device and SoundFont libraries? | Audio, M0/M3/M5 | Callback/lifecycle/backend evidence and maintained dependency choice |
| Resource identity/relink/required provider policy? | Persistence/registry, M0/M1/M6 | Compatible version/fingerprint and safe missing-resource behavior |
| Edits/seek during playback? | Application/audio, M3/M5 | Explicit initial policy, then robust reconstruction |
| Packaging, signing, app license, asset licenses? | Release owner, M0/M7 | Reviewed compatible distribution plan; no license assumed from language choice |
| Performance baseline machines and thresholds? | Product/quality, M3/M5 | Recorded representative workloads and accepted measurable gates |
| External clock authority and integration coverage? | Performance/product, M8-A | Scoped Link/MIDI/OSC contracts and loss/fallback policies |

Open advanced questions do not block unrelated tasks. They block only work whose
correctness depends on their answers. Do not conceal an unresolved choice inside
an arbitrary default or a vague `parameters` payload.

## Risks and mitigation

| Risk | Mitigation/evidence |
| --- | --- |
| A piano-first model quietly assumes 12-tone tuning or compulsory bars | Contrasting degree/cycle fixture before UI architecture is entrenched |
| Extension JSON becomes an unvalidated replacement for domain types | Typed core, namespaced schemas/versions, explicit capability contracts |
| Neutrality is mistaken for automatic support of every notation | Separate preserve/edit/render/play capabilities and missing-provider tests |
| Number/solfa view conversion erases original meaning | Preserve notation intent/context; explicit ambiguity/loss reports |
| Numerical gamelan convention or tuning is misrepresented | Synthetic engineering fixture labels and musician-reviewed real conventions |
| Engraving complexity is severely underestimated | Small defined subsets, per-feature fixtures, visual and musician review |
| Real-time audio works once but fails on seek/shutdown/device loss | Callback ownership constraints, lifecycle tests and recorded native runs |
| Save/migration loses years of compositions | Atomic saves, source-preserving migrations, backups/recovery and failure injection |
| Plugin/DAW ambitions swamp the first product | Explicit M7 non-goals and separately approved M8 proposals |
| Cheap model implements plausible but wrong semantics | Independent contracts/tests, focused review, measured routing and escalation |
| Premature optimization creates an opaque architecture | Baseline measurements, simple modules, justified local changes |
| Samples/fonts/dependencies cannot be redistributed | Early provenance inventory and release licensing gate |

## Change record template

```text
Decision ID / date / owner:
Problem and real user case:
Previous behavior and proposed behavior:
Options considered and evidence:
Affected requirement IDs and public interfaces:
Persistence/provider compatibility and migration:
Acceptance fixtures and checks:
Status: proposed / accepted / superseded
```

## Primary technical references

References were opened during preparation on 2026-10-02. They support framework/
format facts, not Sandikala's unimplemented product capabilities. Pin actual
library versions during implementation; recheck compatibility and licenses then.

| Topic | Primary source | Use |
| --- | --- | --- |
| Desktop boundary | [Tauri architecture](https://v2.tauri.app/concept/architecture/) | Webview/Rust/native communication |
| Native permissions | [Tauri capabilities](https://v2.tauri.app/security/capabilities/) | Scoped desktop capabilities |
| Desktop tests | [Tauri tests](https://v2.tauri.app/develop/tests/) | Evaluate actual native testing options |
| Rust serialization | [Serde](https://serde.rs/) | Serialization machinery, separate from semantic validation |
| Native audio device | [CPAL documentation](https://docs.rs/cpal/latest/cpal/) | Candidate device/stream implementation |
| Higher-level audio candidate | [Rodio documentation](https://docs.rs/rodio/latest/rodio/) | Evaluate whether its abstractions fit transport/callback needs |
| SoundFont candidate | [FluidSynth](https://www.fluidsynth.org/) | Candidate SoundFont engine; bindings/distribution need a spike |
| Notation glyph mapping | [SMuFL 1.4](https://www.w3.org/2021/03/smufl14/) | Public musical glyph mapping, not an engraving algorithm |
| Sheet-music interchange | [MusicXML 4.0](https://www.w3.org/2021/06/musicxml40/) | Scope a future supported-subset adapter |
| OSC | [OSC specification](https://opensoundcontrol.stanford.edu/spec-1_0.html) | Future external message contract |
| External synchronization | [Ableton Link](https://ableton.github.io/link/) | Tempo/beat/phase synchronization, not project/sample transfer |
| AI model roles | [OpenAI models](https://developers.openai.com/api/docs/models) | Verify named model roles/availability |
| AI selection | [OpenAI selection guidance](https://developers.openai.com/api/docs/guides/model-selection) | Empirical lightest-acceptable-model selection |

MIDI version/interchange details need a focused primary-specification review in
M8-A. Library suggestions here are candidates, not imported dependencies or final
license decisions. Do not infer library suitability solely from its existence.

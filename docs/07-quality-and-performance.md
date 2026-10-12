# Quality, verification, and performance

Status: proposed checks. M1 establishes executable commands; M3 establishes
measured baseline hardware/workloads. No benchmark in this document has run yet.

## Correctness gates

| Area | Independent cases |
| --- | --- |
| Time | Normalize equal fractions; checked overflow; boundary comparison; domain mismatch; pickups; invalid denominator |
| Pitch | Known spelled/degree resolution; unpitched identity; missing tuning; finite positive frequencies; no silent nearest-note mapping |
| Document | Duplicate/dangling IDs; ownership; invalid relations; transaction rejection; undo/redo |
| Persistence | Complete semantic round trip; unknown extensions; future core version; bounded parsing; failed save; migration refusal |
| Performance | Known onsets; repeats/occurrences; tempo boundaries; simultaneous-action order; ties versus slurs; damping/release tails |
| Engraving | Spelling/register/rhythm marks; collisions; long text; multiple voices; page breaks; stable hit IDs |
| Desktop | IPC validation; stale revision; selection after relayout; dirty close; cancellation; missing resources/devices |
| Native audio | Callback ownership; deadlines; start/stop/seek/loop; swap/drop lifecycle; device loss; stuck voice prevention |
| Interchange | Supported subset; explicit loss/quantization reports; independently generated references |

Property tests are useful for rational arithmetic and document/command invariants.
Fuzz bounded file/import parsing when those entry points exist. Keep simple unit
tests for obvious cases and integrated fixtures for cross-boundary behavior.

Tests must not derive their expected result using the same implementation under
test. Define timing/pitch/layout acceptance examples independently. Test native
and offline audio with a shared plan but independent onset/voice-count assertions.

## Reference fixture families

- `piano-basic`: two staves, rests, simultaneity, spelling, one simple tie.
- `gamelan-cycle-basic`: 16 pulses, structural anchors, numerical identity,
  synthetic nonuniform degree mapping, register indication, strike/damping.
- `missing-resource`: valid composition with absent sound/font/provider references.
- `extension-roundtrip`: unknown optional payloads and unavailable required behavior.
- `persistence-failure`: truncated/invalid inputs and simulated interrupted save.
- Later families: tempo/repeats, dense engraving, multiple voices, complex rhythm,
  transport/device lifecycle, migrations, and interchange supported subsets.

Each fixture records schema/provider version, conventions, licenses, expected
semantic actions, expected geometry where useful, and audio expectations. Use
licensed original/synthetic data unless permission is documented. Synthetic
tuning is not evidence of authentic performance practice.

## Human verification

Have a musician familiar with each claimed notation inspect the reference score,
grouping, tuning assumptions, and performance interpretation. Listen to native
and exported sound, including release tails and seek/loop boundaries. Inspect
printed/PDF pages and the missing-resource workflow.

Automated pitch/time/image tests complement human review; neither substitutes
for the other. Record reviewer/date/fixture/findings, not just "looks good".

## Performance policy

First choose clear ownership and suitable existing data structures. Measure with
release builds. Correctness constraints such as avoiding callback blocking and
exact logical arithmetic are required immediately. Speculative speed features are
not: no custom allocator, ECS, custom database, GPU renderer, SIMD DSP, or broad
lock-free infrastructure without a measured problem.

Use a repeatable optimization loop:

1. Record a real slow/glitching user action and its workload/hardware.
2. Profile to identify the dominant cost and establish repeated measurements.
3. Select the smallest change that addresses that cost.
4. Rerun correctness checks and compare latency/memory against the baseline.
5. Keep the change only if the improvement is meaningful and maintainable.

Caching keys must include document revision, relevant view/provider configuration,
and resource/tuning identity. Incorrect cached output is worse than a slower correct
result. Full recomputation and straightforward snapshots are acceptable initially.

## Initial measurement workloads

| Workload | Proposed composition | Measurements |
| --- | --- | --- |
| Tiny | Both reference fixtures | Correctness, startup, visible edit feedback |
| Ordinary | 1,000 events, 4 parts, representative relationships | Command latency, relayout, save/load, memory |
| Larger | 10,000 events, 8 parts, long pages and repeats | Scaling, cancellation, stale-result behavior |
| Audio | 48 kHz supported device configuration, up to 64 active test voices | Callback duration/underruns/transport behavior |

Record CPU, memory, OS, build, device/backend, sample rate, buffer frames, fonts,
profiles, and test duration. Workloads are starting points, not maximum product
limits. Unavailable buffer sizes must be reported, not treated as passed tests.

Provisional goals for M3/M5 review: ordinary edit feedback p95 under 100 ms,
ordinary full relayout under 500 ms, and larger document load/save under 2 seconds
on agreed baseline machines. Long jobs remain cancellable regardless of their
duration. These are design targets, not measurements or guarantees; M3 must review
them against user needs before adopting a release gate.

For audio, the callback deadline is buffer_frames / sample_rate: 256 frames at
48 kHz allow about 5.33 ms. Aim for substantial headroom and no observed persistent
underruns during a recorded five-minute ordinary-load test. Percentile profiling
does not prove a worst-case real-time guarantee. Device/OS limits and measured
results determine the supported configuration policy.

Do not silently lower targets to pass a gate. Explain workload/user impact and
record an approved product decision if the baseline needs to change.

## CI and runtime checks

Once M1 code exists, establish these actual commands in the repository:

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Use dependency/version choices compatible with both target platforms and pin the
toolchain appropriately. Add frontend type checking, focused component tests,
and production builds in M3. Browser UI checks can use a mature browser test tool.
Native Tauri testing uses available platform mechanisms; investigate automation
support rather than assuming browser automation covers the embedded webview.
[Tauri testing documentation](https://v2.tauri.app/develop/tests/) is the starting
reference. Keep native manual smoke evidence when automation is unavailable.

Run relevant fast checks per task and integrated checks at milestone gates. A new
shared-contract change warrants wider regression testing. Do not endlessly rerun
passing unrelated checks after a documentation-only change.

## Release blocker examples

Score-loss/corrupt saves; silent pitch/timing alteration; unbounded repeat traversal;
stale document playback presented as current; callback-lifecycle crashes; unusable
printed output for a claimed feature; unsupported required payloads destroyed;
missing distribution license; and unverified claimed native platform support.

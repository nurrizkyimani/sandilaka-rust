# Performance realization and audio

Status: proposed pipeline. Sample playback, native device output, SoundFont
support, and external synchronization are separate capabilities.

## Pipeline

```text
ScoreDocument + realization policies + tuning/sound registry
  -> validated repeat/section traversal
  -> logical time and pitch/gesture interpretation
  -> performance plan (ordered actions, source IDs, resolved time/pitch)
  -> offline renderer OR native scheduler OR external adapter
```

Keep score time, performance occurrences, and device sample frames distinct. A
repeated score event has one document identity but multiple performance occurrence
identities. Do not duplicate the user's notes to implement a repeat.

## Realization rules

1. Resolve instrument-specific degree/spelled pitch/stroke into a playable target.
   Keep the original pitch reference; never overwrite it with the derived frequency.
2. Map declared domain units through the tempo map to performance time. Use exact
   logical arithmetic where appropriate; device frames require documented rounding.
3. Traverse repeats/sections with validated structure and bounded expansion. Detect
   cycles without valid termination rather than hanging on a malformed score.
4. Apply defined dynamics, ties, articulations, ornaments, and damping policies.
   Visual marks without a performance rule remain visual and produce a diagnostic.
5. Produce ordered actions with deterministic simultaneous-action priority. Include
   source event ID and occurrence identity for playhead, errors, and seeking.
6. Convert to the chosen backend's controls only at its boundary. MIDI note numbers
   and velocities are output encodings, not mandatory score fields.

Constant tempo is first. Piecewise changes come before complex ramps. For ramps,
define the curve and integration policy explicitly and test boundary continuity.
Untimed gestures/fermatas require an agreed duration policy. Linked external tempo
needs a declared clock authority; it cannot quietly override every score tempo map.

## Sound backends

| Backend | Purpose | Scope |
| --- | --- | --- |
| Deterministic test synth | Prove pitch/timing and generate inspectable WAV fixtures | M2; clearly labeled test/rehearsal sound |
| Direct sample renderer | Play licensed instrument samples with explicit degree/pitch/stroke mapping | M2 small fixture; M5 expanded profiles |
| SoundFont adapter | Use compatible SF2/SF3 libraries through a suitable licensed engine | M5; engine choice is a spike |
| External MIDI | Drive hardware/software that interprets MIDI | M8 documented mappings and limitations |
| External OSC | Send agreed addresses/values to tools such as Max | M8 integration examples |
| Plugin instruments | Potential external synthesis/processing | Future product decision, not initial implementation |

Select maintained parsers/decoders/resamplers and device libraries instead of
implementing standard formats from scratch. CPAL is the initial native audio
device candidate; its [documentation](https://docs.rs/cpal/latest/cpal/) covers
device/stream configuration. It is not a SoundFont synthesizer or sample library.

Sound-profile selection maps event meaning/articulation to an appropriate voice,
sample zone, synthesis control, or external action. A sample profile declares
root tuning, mapping, channel format, envelopes, gain, looping, and release/damping
behavior as needed. Prefer an actual degree/stroke sample over stretching one
sample across an entire ensemble when sound quality matters.

Missing/unsupported SoundFonts must not be represented as working. A visible test
synth fallback can enable rehearsal if the user chooses it. The product must not
silently replace the intended sound and claim fidelity.

## Concrete flows

Piano: spelled C4 + tuning context -> resolved frequency -> piano sample/zone or
test synth -> envelope/release -> output buffer. A tie across two notation events
can mean one sustained voice instead of two attacks.

Gamelan: degree 1 + ensemble/instrument context -> exact sample mapping or resolved
frequency -> strike -> natural decay -> an explicit damping action if specified.
Logical notation duration is not automatically the acoustic decay length.

The `.sdlk` stores those events, policies, and profile references. Large sound files
normally live outside it. Asset relinking and optional later portable packaging
are described in [persistence](02-score-and-file-format.md).

## Native threading and callback contract

- Document edits run through the application owner.
- Loading/decoding samples, resolving plans, and exports run outside the callback.
- A native callback consumes prepared bounded actions and renders prepared voices.
- No blocking locks, allocation, filesystem access, webview messages, document
  traversal, or unbounded diagnostics occur in the callback.
- Publish plan revisions and transport changes through a bounded, documented
  mechanism. Select an established queue/snapshot pattern after the M3 spike.
- Avoid destroying large plans/samples on the callback thread when swapping them.
- Underrun/device-error reporting uses bounded counters/events collected elsewhere.

These are correctness requirements for reliable audio, not speculative performance
optimization. Custom lock-free frameworks, SIMD mixers, and advanced DSP are deferred
until measured needs justify them.

## Transport, edit, and seek policies

M3 provides basic play/pause/stop and a documented initial seek behavior. M5
adds range loops, count-in, metronome, robust seeking, and live plan replacement.
Seeking must reconstruct required controller/sustain/damping state and clear
obsolete voices. Test restart, end-of-score, device loss, and changing revisions.

Define whether edits during playback take effect at a safe boundary or trigger a
controlled restart. Initially, a visible restart policy is acceptable. A stale
plan must never be presented as the current composition without a clear status.
UI playhead updates are informational and cannot drive scheduling.

## Offline export and mixer

Offline rendering and native playback share the performance plan and voice rules.
Record sample rate, channels, bit depth/encoding, resource identities, peak policy,
and render tail behavior. M2 uses a small deterministic export path; M5 adds
documented WAV options, cancellation, and selected-range export.

M5 includes part gain, mute/solo, pan, basic routing, meters, and clipping warnings.
Effects chains, mastering, live recording, time stretching, and plugin hosting
need separate requirements. Floating-point rendering can differ slightly across
backends; use a defined numerical tolerance rather than promising bit-identical
audio on every machine.

## External integrations

MIDI import/export requires declared supported messages and timing resolution.
Quantization, nonrepresentable tuning, enharmonic spelling, and notation intent
losses are reported. MPE or MIDI 2 support is not implied by ordinary MIDI support.

OSC requires an address/schema contract with the receiving patch. An integration
test with a supplied Max example demonstrates the claimed behavior.

[Ableton Link](https://ableton.github.io/link/) synchronizes tempo and beat/phase
relationships. It does not transfer scores, samples, or complete Ableton projects.
Link clock policy, native bindings, licensing, network permission, and offline
fallback are an M8 spike. Do not label generic MIDI/OSC output as full Ableton
integration or promise native Ableton project-file support.

## Acceptance

Check known onset/frame positions, tempo boundaries, repeat occurrence counts,
tie attack counts, damping behavior, missing resources, callback safety, and
transport lifecycle. Use nonblank waveform, frequency/onset analysis, loopback
where available, and human listening. Passing compilation does not prove sound
quality or native playback reliability.

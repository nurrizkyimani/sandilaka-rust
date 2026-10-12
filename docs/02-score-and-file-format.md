# ScoreDocument and file format

Status: conceptual contract, not a released JSON schema. M0 freezes the small M1
subset after fixture review. Additive future fields and migrations must remain
possible; do not promise a permanently fixed schema.

For immediate implementation, the [Q1 contract](11-gamelan-sheet-quick-win.md)
freezes version `0.1.0-q1` and supplies complete fixtures. Its explicit bounds,
field definitions, and error rules take precedence for that subset. The proposals
and event fragments below describe broader future capabilities, not additional
Q1 requirements. Unsupported future core constructs are refused, not silently lost.

## Score versus event

ScoreDocument is the composition aggregate. Events are typed elements inside it.
An event list alone cannot capture tuning contexts, voices, repeat structure,
notation intent, layout settings, or relationships.

```text
ScoreDocument
  format/schema version and document identity
  metadata and resource requirements
  parts -> instrument references -> voices
  time domains, tempo maps, sections, cycles, repeat structure
  pitch/tuning contexts and mappings
  events and relationships
  notation views and persistent presentation intent
  versioned namespaced extension payloads
```

## Core concepts

| Concept | Contract |
| --- | --- |
| ID | Stable serializable identity; never an array index, address, or render coordinate |
| Part | Musical role with instrument assignment and one or more voices |
| Voice | An event stream; whether overlap is allowed depends on explicit semantics |
| Event | Typed payload plus a valid temporal anchor and ownership; optional extent where meaningful |
| Relation | Typed reference between existing elements, with its own identity and validation |
| Context | Explicit interpretation for time, pitch, notation, or performance |
| View | A provider/configuration plus persistent notation intent and layout settings |
| Resource reference | Pack/profile identity and compatible version, optional fingerprint and relink hint |
| Extension | Namespace, version, and preserved payload with declared capabilities |

Start with sound, rest, and annotation event kinds. A sound can be a pitched
attack or an unpitched stroke. Add gestures, patterns, automation, or other kinds
only with semantics and acceptance fixtures. A chord can initially be simultaneous
events with an explicit grouping relation; decide duration/voice ownership in M0.

Do not force every event to contain pitch, velocity, and duration. A text label
does not emit sound, and a percussion stroke may have no meaningful pitched note.

## Time contract

- Store exact rational coordinates with an explicit domain and unit.
- Metric domains use a defined beat unit; time signatures and bar structure are
  interpretation/notation data, not a compulsory global event container.
- Cycle domains declare cycle length, pulse/subdivision units, structural anchors,
  and mapping to the performance timeline.
- An untimed/ordinal domain can preserve sequence and constraints. Playback needs
  an explicit realization policy or supplied durations; order alone is not seconds.
- Later seconds/frame domains require declared rates and clock relationships.
- Never compare positions in different domains without a defined mapping. A
  domain graph with conflicting or missing mappings produces a diagnostic.
- Normalized rationals have a positive denominator. Check arithmetic overflow,
  unreasonable size, duplicate anchors, and invalid extents.
- Proposed JSON coordinates use bounded safe integers for JS interoperability;
  choose the exact bound and Rust checked-arithmetic representation in M0.
  Do not let large integers silently round in a webview.
- Define pickups/negative coordinates explicitly. Do not reject them universally
  or assume every score starts at a complete Western measure.

Tempo maps relate a stated unit to time. Constant tempo is the first supported
policy; piecewise changes and specified ramp curves follow. A fermata/free gesture
does not have a unique playback duration until a performance policy supplies one.
Simultaneous independent tempo domains are advanced scope, not M1 functionality.

## Pitch and tuning contract

Pitch is a reference interpreted by a pitch context, not necessarily an integer.
Support structured forms: spelled pitch, scale degree and register, absolute
frequency, ratio to a reference, and provider-specific symbolic identity. Unpitched
strokes use stroke/instrument identity instead of a fake MIDI pitch.

Spelled pitch retains letter, alteration, and register. Scale degree references
retain the intended degree and the selected scale/ensemble context. Movable solfa
requires a tonic/context; syllables alone cannot resolve frequency. Frequency
values must be finite and positive. Missing context is an error or an explicitly
unresolved state, never an implicit 12-tone equal-temperament conversion.

The resolved frequency is derived for playback. Retain the original reference
and spelling in the document. For gamelan, a generic label such as pelog or
slendro does not specify every ensemble's physical tuning or instrument mapping.

## Relations and notation intent

Ties, slurs, tuplets, groupings, patterns, damping, and repeat structures have
distinct meanings. A tie can extend one sounding note; a slur must not be treated
as a tie. Provider rules validate which references can participate.

Notation intent may include spelling, register dots, rhythmic grouping, clefs,
stem/beam preferences, or manual layout overrides. Store persistent choices by
stable IDs in a view/presentation namespace. Computed glyph positions and current
viewport geometry are caches, not composition semantics.

## Illustrative piano event

The following is an illustrative event fragment, not a complete `.sdlk` file:

```json
{
  "id": "piano-note-1",
  "voiceId": "piano-right",
  "anchor": {"domainId": "metric-main", "position": {"numerator": 0, "denominator": 1}},
  "extent": {"numerator": 1, "denominator": 1},
  "kind": "sound",
  "payload": {
    "pitch": {"kind": "spelled", "contextId": "piano-tuning", "letter": "C", "alteration": 0, "octave": 4}
  },
  "extensions": {}
}
```

Its surrounding document defines a quarter-note beat, 120 beats per minute,
12-tone equal temperament referenced to A4=440 Hz, and a piano sound-profile
assignment. The expected onset is 0 seconds and logical duration is 0.5 seconds.
The provider displays C4 with the chosen clef/spelling. The sound backend selects
a sample or synthesizes the resolved frequency; it need not store MIDI in the score.

## Illustrative numerical gamelan event

```json
{
  "id": "gamelan-hit-1",
  "voiceId": "saron-line",
  "anchor": {"domainId": "cycle-main", "position": {"numerator": 0, "denominator": 1}},
  "extent": {"numerator": 1, "denominator": 1},
  "kind": "sound",
  "payload": {
    "pitch": {"kind": "degree", "contextId": "ensemble-test", "degree": "1", "register": 0}
  },
  "extensions": {}
}
```

The document defines a 16-pulse cycle, 60 pulses per minute, and a synthetic test
ensemble with degree 1 assigned 220 Hz. Expected onset is 0 seconds and logical
extent is 1 second. Numerical notation displays `1`; a mapped instrument sample
may ring past the logical extent unless damping semantics stop it. The synthetic
frequency is an engineering fixture, not an authoritative gamelan tuning.

Both fragments require surrounding parts, voices, contexts, views, and resources.
M0 creates complete fixtures and validates them against the agreed schema. These
fragments are deliberately not files the unimplemented app can already open.

## `.sdlk` persistence

Proposed first format: versioned UTF-8 JSON in a `.sdlk` file, containing the full
document and resource references. Use a mature serializer, with schema and
semantic validation as separate responsibilities. [Serde](https://serde.rs/)
provides Rust serialization machinery; it does not enforce musical correctness.

The file holds score content and presentation choices, not mandatory copies of
SoundFonts or sample libraries. Sound assets are normally installed separately.
Portable packaged projects are later scope: document plus manifest plus optional
redistributable assets in a vetted container. The extension/container policy is a
versioned decision, not a promise that plain JSON must remain the only encoding.

Required behaviors:

1. Read within declared size/depth/resource limits, validate, and report affected
   IDs/fields. A corrupt document must not replace the open valid document.
2. Reject unsupported core schema versions safely. Preserve unknown optional
   extension payloads on a semantic JSON round trip, even without their provider.
3. Required unavailable capabilities produce a clear restricted/read-only mode or
   block the affected operation. Preservation does not imply edit/play/render support.
4. Migrate explicitly from known older schemas. Keep the source intact until the
   migrated file is successfully saved. Test migration success and refusal paths.
5. Save to a temporary sibling file, verify completion, then perform a platform-
   appropriate atomic replacement. Establish durability/backup policy in M1.
6. A missing sound resource never prevents opening the score. Relink resources by
   stable identity and optional content hash; do not rely only on an absolute path.
7. Packaged assets require safe path handling, expansion limits, and licensing
   checks before any future container importer is enabled.

Do not add a database, event-sourced persistence, binary layout, or compression
just to anticipate millions of events. Measure ordinary JSON load/save first.

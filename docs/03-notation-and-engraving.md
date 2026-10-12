# Notation and engraving

Status: provider/layout proposal. M2 produces deliberately basic output; M4
expands the claimed engraving scope. Staff engraving is substantial product work,
not a generic renderer that becomes complete once it draws a note.

## Separate responsibilities

```text
ScoreDocument + view configuration + resource registry
  -> notation interpretation (symbols, grouping, labels, intent)
  -> layout constraints (spacing, collisions, anchors, page/system breaks)
  -> positioned primitives + document IDs + hit-test geometry
  -> screen SVG / export backend
```

The notation provider owns musical display rules. The common engraving layer
owns useful shared geometry, text placement, constraints, pagination mechanisms,
and output primitives. Different providers may use different layout strategies;
do not force a proportional graphic score through a staff-measure algorithm.

Use existing font/text/vector libraries. Sandikala's provider-specific rules are
original implementations informed by public standards and musician reference
cases. Using an established layout algorithm is valid; copying another project's
implementation is a separate licensing/provenance question.

## Initial provider interface

Define semantic interfaces in M0 before selecting Rust trait signatures:

| Operation | Input/output expectation |
| --- | --- |
| Describe capabilities | Supported contexts, event kinds, versions, and limitations |
| Interpret document/view | Produce notation objects with provenance and diagnostics |
| Interpret edit intent | Translate pitch/rhythm/symbol input into document commands |
| Produce constraints | Shared primitives or provider-specific layout requests |
| Explain conversion | Report preserved, ambiguous, omitted, and unsupported information |

The viewport interaction layer hit-tests layout geometry and returns stable IDs;
an ID then drives selection/edit commands. Document IDs and semantic providers
must not depend on React DOM node identities.

## Western notation

M2: two-staff piano fixture with basic clefs, spelled pitches, accidental display,
rests, durations, simple grouping, and ledger lines sufficient for the fixture.
M2 is not an arbitrary professional piano-score renderer.

M4 expands to multiple voices, chords, ties, slurs, tuplets, grace notes, beams,
dynamics, articulation marks, text, time/key signature changes, and page layout.
Add focused fixtures for each. M8 introduces advanced coverage such as cross-staff
notation, complex beams/tuplets, lyrics, percussion/tab systems, and detailed
part-extraction workflows after specific product decisions.

Use an appropriately licensed notation font and measured glyph metadata.
[SMuFL](https://www.w3.org/2021/03/smufl14/) defines musical glyph mapping. It does
not supply Sandikala's spacing, beam, collision, or pagination algorithms. Font
licenses and numerical-notation resources need independent checking.

## Numerical notation

M2: display degree identity, register markers, explicit rest conventions, basic
rhythmic grouping, and cycle anchors supported by the chosen reference convention.
Keep pitch-system identity separate from its visible tokens.

M4: expand rhythm marks, group boundaries, structural marks, annotations, page
layout, and relevant ensemble presentation. Validate the chosen convention with
a knowledgeable musician. Different numerical notation practices need configuration
or different providers; one convention is not all Indonesian or all Asian music.

## Switching views

An alternate view is created from the same composition, with its own notation
intent. Changes to semantic pitches/timing affect all views. Changes to a view's
manual spacing do not retune an instrument. Retain original spelling and meaning
even if a second view cannot display everything.

If a view cannot represent an event, show an explicit unsupported marker and
conversion report. Do not silently substitute the nearest equal-tempered note.
Unknown data may remain preserved while editing or output is unavailable.

## Editing and UI behavior

- Select a glyph/event or a logical time region; property editing targets IDs.
- Keyboard and pointer entry use the current voice, duration, notation, and context.
- Copy/paste carries context/relation intent; incompatible targets produce a
  meaningful choice or refusal rather than silently changing units.
- Zoom/pan changes viewport coordinates only. Selection survives a new layout.
- Properties distinguish musical changes from presentation overrides.
- Long jobs expose pending/error states. Revision tags prevent stale geometry.
- Core controls must work via keyboard; canvas/SVG selection needs an accessible
  companion event/properties representation rather than claiming glyphs alone
  provide screen-reader support.

## Publishing

Start with SVG as a debuggable vector output. Choose a PDF/text backend after an
M0/M4 spike demonstrates correct fonts, text shaping, page dimensions, and license
compatibility. Screen display and PDF should use the same semantic layout output;
do not implement a separate PDF music engine.

M4 includes page size, margins, orientation, system breaks, readable title/credits,
basic part views, and print/export preview. Native print behavior must be tested
on both platforms. Professional part extraction, transposition choices, linked
part overrides, and page turns are separately tracked M8 work.

## Acceptance evidence

Keep golden layout primitives for deterministic geometry and controlled SVG/image
fixtures for visual comparison. Test hit geometry, long text, dense voices, register
marks, accidentals, and page boundaries. Pin test fonts and renderer configuration;
cross-platform antialiasing differences need an explicit tolerance policy.

A screenshot baseline is not proof of musical correctness. Independently check
symbol/grouping semantics and have musicians inspect published examples. Treat
unsupported notation and partial conversion as tested product states.

## Deferred optimization

Full recomputation for small fixtures is acceptable. Only add incremental layout,
viewport culling, reusable scene graphs, or GPU drawing after profiling identifies
a specific bottleneck. Keep the layout result/backend boundary stable so these
changes do not require redesigning the composition model.

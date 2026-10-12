# Paged score studio and v0.4 delivery

The user authorized continuing through publishing with a page canvas resembling
the supplied scores, including piano. This advances a bounded Western subset
alongside the numerical provider. PR-03/05/07/10/11/13/20/21 are affected.

The new `sandikala.score.v1` provider requires the versioned
`sandikala.studio.v1` document extension. It retains the existing core event
shapes and exact rational quarter-unit timing. Q1 files retain their provider
and behavior; the new provider has separate capability checks and fixtures.

The initial contract includes multiple parts and two-staff instruments, fixed
global meters, quarter/eighth/sixteenth subdivision, durations, rests, degree
registers, spelled piano pitches, undo/redo, score/part pages, A4/Letter sizes,
landscape, manual system breaks, and native printing/Save as PDF. Unsupported
ties, grace notes, tuplets, mixed/additive meter changes, percussion strokes,
and professional collision coverage remain explicit later gates.

Layout is an original interval-column implementation: all voices share time
columns, minimum spacing responds to rhythmic density, a dynamic-programming
line-break pass minimizes ragged systems, and a separate page pass places
systems. Font assets are unmodified Leland and Edwin under OFL 1.1 with licenses.
No MuseScore source or engraving implementation is copied. Screen, part view,
SVG, and print consume the same Rust layout and embedded fonts.

`sandikala.mixer.v1` stores per-part gain/pan/mute/solo; event extension
`sandikala.event-marks.v1` stores bounded text and velocity. Both are validated
transactionally and preserved by undo and file round trips. Native playback
renders off the callback, then transfers immutable PCM to a lock-free atomic
transport for pause and seek. Float WAV exports preserve those rendered values.
Vector PDF uses the same SVG geometry through Krilla; its MSRV requires Rust
1.92 or newer. Installed workspace toolchain 1.97.1 satisfies that requirement.

Completing this implementation does not by itself declare stable v0.4 or Q4
complete. Native persistence, printed pages, musical review, and full claimed
playback coverage require recorded evidence before those gates close.

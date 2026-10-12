# Architecture

Status: baseline proposal. The principles below are stronger commitments than
the initial library choices. Read the [decision register](10-decisions-and-risks.md)
before treating an unresolved choice as settled.

## Overall structure

```text
React + TypeScript editor (Vite)
  notation viewport | properties | parts | transport | resource selection
                       |
               versioned command/query DTOs
                       |
Tauri desktop adapter (native window, menus, dialogs, permissions)
                       |
Rust application layer (document ownership, transactions, jobs, undo/redo)
                       |
              ScoreDocument + registry snapshots
                   /                  \
       Notation interpretation     Performance realization
                |                         |
       Engraving + hit geometry    Immutable performance plan
                |                   /                  \
        SVG / vector PDF       Offline rendering    Native scheduling
                                                    |
                                           Audio / MIDI / OSC
```

The native backend is local Rust code, not an HTTP server. A database is not
required to hold a score. File-based persistence is sufficient until a concrete
library-management requirement justifies an index/database.

## Frontend

Use React and TypeScript for toolbars, properties, selection feedback, resource
management, transport controls, and editor orchestration. Use Vite for the initial
frontend build. These are the proposed UI choices, not a claim that React is the
only suitable frontend.

React owns ephemeral UI state: focus, viewport, dialogs, local selection, pending
input, and user preferences. It holds a revisioned projection of the document,
not an independent authoritative composition. Selection uses stable document IDs.
Optimistic feedback is optional; reconciliation must handle rejected commands.

The first score viewport displays SVG/layout output produced by Rust, with a
thin interaction overlay. Rendering coordinates belong to the layout result.
Input gestures translate through the notation provider to document commands.
Do not calculate a second incompatible musical layout in JavaScript.

Use familiar icon controls, tooltips, restrained panels, keyboard focus, native
menus where appropriate, and an editor as the first screen. Avoid a marketing
landing page inside the application. Canvas/WebGL/virtualization are options
after measurements demonstrate a limit, not prerequisites.

## Native application and domain responsibilities

| Boundary | Owns | Must not own |
| --- | --- | --- |
| Domain | Events, relationships, context, validation, document invariants | Tauri calls, audio devices, UI coordinates |
| Application | Command transactions, revisions, undo/redo, job cancellation, resource resolution | Provider-specific glyph rules or callback DSP |
| Persistence | Document encoding, migrations, bounded decoding, atomic file operations | Hidden changes to musical meaning |
| Notation/engraving | Input interpretation, notation intent, layout and hit-test identity | Authoritative sound-library selection |
| Performance | Repeats/tempo/tuning interpretation and resolved action plans | Page breaks, beam coordinates, React state |
| Audio/adapters | Device buffers, sound voices, export encoding, external protocol mapping | Editing a live ScoreDocument inside a callback |
| Desktop adapter | Windows, dialogs, menu wiring, scoped native access | The music model itself |

## Start with a small workspace

Suggested eventual layout; create modules only when the assigned task needs them:

```text
apps/desktop/                 React frontend and Tauri adapter
crates/sandikala-core/         model, IDs, rational time, commands, validation
crates/sandikala-io/           persistence, migrations, later interchange modules
crates/sandikala-engine/       application jobs, notation, layout, performance
crates/sandikala-audio/        offline/native audio and sound backends
fixtures/                     documents, expected semantics, licensed assets
packs/                        instrument/tuning/notation-resource manifests
docs/                         product, decisions, interfaces, tasks
```

Keep time/pitch/notation as explicit modules initially. Split them into independent
crates when dependencies, reuse, build times, or ownership make that worthwhile.
Do not begin by creating a crate for every future concept. `sandikala-core`
depends only on general-purpose libraries; adapters depend inward toward it.

## Editing flow

1. The frontend submits a typed command with document ID and expected revision.
2. Rust validates the entire operation against a consistent document state.
3. Commit all changes as one transaction or reject all of them.
4. Record undo information and increment the revision. Runtime revisions are not
   file-format versions and do not have to be persisted as document semantics.
5. Publish a bounded change/projection and schedule relevant derived jobs.
6. Layout/performance jobs receive immutable input tagged with the revision.
7. Reject stale results instead of displaying or playing an older edit silently.

Initial input commands include insert/delete/update event, set context, edit a
relation, and grouped paste. Choose inverse commands or snapshots after a small
undo spike. The initial solution should be simple and correct, with a memory cap.

## Desktop boundary

Define a small interface such as open/save, apply-command, query-projection,
request-layout, request-export, load-resource, and transport-action. Return typed
errors containing affected IDs and a useful remedy. Validate IPC input in Rust;
TypeScript types alone are insufficient.

Send batches or revisioned projections appropriate to the workload. Never send
one IPC message per audio sample or schedule musical notes from React timers.
Throttle playhead updates for display; the UI clock does not control sound timing.
Long layout/export/library jobs support progress and cancellation without blocking
input or native audio. Single-owner document serialization is sufficient initially;
do not introduce distributed state or concurrent document mutation prematurely.

## Provider contracts and resources

Separate notation, pitch/tuning resolution, performance realization, and sound
rendering capabilities. A provider declares supported versions and event/context
types. An instrument pack composes available capabilities; it is not executable
behavior by itself. Persist stable identifiers and versions, never pointers or
machine-specific device handles.

Begin with built-in providers and validated data packs. Third-party executable
plugins are later work. A sandboxed runtime is a candidate, not an implemented
promise. Unknown payload preservation is handled by persistence even when no
provider is installed.

## Permissions and failure paths

Use narrowly scoped Tauri capabilities for user-chosen files and required native
features. Offline composing requires no network access. Missing sound devices
disable native playback while allowing editing and offline export. A font/library
error should identify the affected output and preserve the document. Never load
arbitrary remote code because a score names a provider.

## Evidence and sources

The frontend/backend message boundary is consistent with
[Tauri's architecture](https://v2.tauri.app/concept/architecture/). Its
[capability mechanism](https://v2.tauri.app/security/capabilities/) informs native
permission scoping. The module structure, ownership, and editing protocol above
are Sandikala proposals, not Tauri requirements. M0 includes a desktop/rendering
spike before these choices become implementation commitments.

# V2 information architecture

Updated: September 8, 2026  
Target: Windows laptops at 1280×800 and 1440×900.

## Product model

Local Store has three jobs:

1. Understand what is running or needs attention.
2. Open and manage apps already in the workspace.
3. Find, connect, or install something new.

The application should open on operational context, not on a store shelf. The
catalog remains important, but it is one destination inside a self-hosting
workspace rather than the product's entire identity.

## Top-level navigation

The final primary navigation is:

1. **Overview** — health, environment, recent changes, and attention.
2. **Discover** — browse the offline catalog and start Connect or Install.
3. **My Apps** — open and manage installed or linked apps.
4. **Activity** — operational history and diagnostics over time.

**Settings** is a bottom utility destination, visually separated from the four
workspace destinations. About/build/catalog information belongs inside Settings
or a small product menu, not as another primary destination.

Activity remains in V2 because it completes the intended operational model. It
must carry a `Concept data` indicator in the design lab until operation history
is persisted by the backend.

## Application shell

The laptop shell has three stable regions:

- A compact left navigation rail with the brand, primary destinations, Settings,
  and a Docker environment indicator.
- A screen header that identifies the current destination and holds only actions
  that apply to the whole screen.
- A flexible work surface. My Apps uses two columns; other destinations use a
  single content canvas with contextual drawers when needed.

The rail should be visually quieter than the content. It does not need the
current “Workspace / page” breadcrumb because the selected navigation item and
screen heading answer the same question. During a focused task such as Install,
the shell may recede, but the user must retain a clear route back.

## Screen ownership

| Information or action | Owner | Why |
| --- | --- | --- |
| Overall environment health | Overview | It answers “does anything need me?” |
| Docker and Compose readiness | Overview, with detail in Settings | Overview needs the signal; Settings owns diagnosis. |
| Recovery warning | Overview, with detail in Settings/Recovery | Urgent when present, invisible when absent. |
| App counts and attention summary | Overview | Cross-app information belongs above individual management. |
| App list and selected app details | My Apps | This is the operational workspace. |
| Open, Start, Stop, Logs, Shortcut, Remove, Uninstall | My Apps | They act on a specific saved app. |
| Catalog browsing and filtering | Discover | It acts on projects not yet in the workspace. |
| Catalog provenance and project details | Discover drawer | Keeps browse position while adding depth. |
| Connect form | Contextual modal/drawer launched from Discover, My Apps, Overview, or First Run | It is a short task with a clear return point. |
| Recipe review and install progress | Dedicated Install task view | Trust, consequences, progress, and recovery need more space than a modal. |
| Historical operations and metrics | Activity | Time-oriented evidence should not overload Overview. |
| Diagnostics, catalog snapshot, recovery detail, appearance facts | Settings | Infrequent utility and support material. |
| Onboarding | First Run flow outside the ordinary shell | It is a temporary state, not a permanent destination. |

## Overview versus My Apps

Overview answers “what changed, and does anything need attention?” It may show:

- overall healthy/attention state;
- Docker and Compose status;
- counts for running, stopped, linked, and unreachable apps;
- up to three apps that require action;
- recent additions or operations;
- recovery candidates;
- compact concept telemetry only when clearly labeled in the design lab.

Overview does not become a second app list. A healthy app may appear in a short
“running now” summary, but routine opening and lifecycle controls live in My Apps.

My Apps answers “what do I have, and what can I do with this app?” It owns the
complete saved-app list, selection, detail, actions, logs, and destructive flows.

## Managed and linked apps

Use these user-facing terms consistently:

| Backend term | V2 term | Meaning |
| --- | --- | --- |
| `RuntimeSpec::Compose` | **Managed app** | Installed by Local Store; lifecycle and data actions are available. |
| `RuntimeSpec::External` | **Linked app** | A saved address for something hosted elsewhere; Local Store opens it but does not run it. |
| `preview_install` | **Reviewed install** | One of the explicitly reviewed local recipes. |
| `connect` | **Connect** | Save the address of an existing web app. |
| `discover` | **Project page** | Learn about the project; no installation or connection claim. |

“Managed” and “Linked” should be visible as object types, not hidden in tooltip
copy. Runtime status is secondary to that type:

- Managed app: Running, Stopped, Starting, Stopping, Error, or Unreachable.
- Linked app: Ready, Unreachable, or Not checked.

Do not label a linked app “Running”; Local Store cannot know its server process.

## My Apps master/detail model

My Apps uses a persistent two-column workspace at both laptop widths:

- Left: searchable app list, compact status, object type, and selected state.
- Right: selected app header, primary action, status/readiness explanation,
  machine facts, and contextual sections.

At 1280×800 the list narrows and secondary text truncates; the detail remains
present rather than collapsing into a mobile navigation pattern.

The detail has three local sections:

1. **Overview** — status, URL, runtime, timestamps, data/path facts, and actions.
2. **Logs** — replaces the detail body without opening a modal; includes refresh,
   copy, empty, loading, and error states.
3. **Manage** — shortcut/remove for linked apps; stop/uninstall/data handling for
   managed apps.

Changing selection is immediate. Do not animate routine row-to-row navigation.
The selected app remains selected after Start, Stop, log refresh, and recoverable
errors. A removed app selects the nearest remaining row; an empty list shows the
Connect and Discover routes.

## Discover and project detail

Discover owns catalog search and faceted filtering. The default page prioritizes
scannability over editorial copy: reviewed installs may receive one compact
signature area, while the main catalog remains visible above the fold.

A project opens in a right contextual drawer rather than a centered modal. The
drawer preserves search, filters, paging, and scroll position. It contains:

- identity, description, category, license, architecture, and freshness;
- source provenance and warnings;
- exactly one primary capability action;
- an optional secondary Connect route for a reviewed install already running.

The primary action is determined by capability, never by visual guesswork:

- Reviewed install → `Review install`
- Connect → `Connect this app`
- Project page → `Visit project`

## Connect flow

Connect is a short contextual task and stays in a modal or narrow drawer. It
contains name, HTTP(S) address, an advisory address check, and the consequence:
the app is saved as a linked app and Local Store will not manage its server.

Entry points may prefill the catalog identity. On success, navigate to My Apps
with the new app selected. On failure, retain both fields and focus the invalid
one. Closing returns to the exact originating context.

## Install flow

Install is a dedicated task view, not a dialog. It has three states within the
same spatial frame:

1. **Review** — exact recipe, configuration, compatibility, trust evidence, and
   disclosed changes.
2. **Installing** — real stage list, active stage, truthful cancellation, and
   diagnostics.
3. **Result** — ready, cancelled, rolled back, recoverable failure, or unsafe
   cleanup failure.

The task header contains Back/Cancel context and the statement “Nothing has run
yet” while still in Review. The primary Install action stays adjacent to a short
summary of what will happen. Raw Compose and image audit evidence are optional
advanced disclosures, not the visual center.

If installation succeeds, the primary next action is `Open app`; a secondary
route opens My Apps with the app selected. Recoverable failures preserve valid
configuration. A failure requiring recovery links directly to the Recovery
surface and does not offer an unsafe retry.

## Logs placement

Logs belong inside the selected managed app's detail, not in a global modal or
top-level navigation. This keeps the app identity and lifecycle actions visible
while inspecting output. The log view uses mono type, preserves line breaks,
redacts backend diagnostics as today, and does not imply live streaming because
the current command returns a bounded snapshot.

Activity may link to the corresponding app's Logs section, but it does not own
the log reader.

## Recovery placement

Recovery is contextual:

- When candidates exist, Overview shows a needs-attention card.
- Settings contains **Diagnostics & Recovery**, the durable place to scan again
  and inspect candidates.
- Install failures that leave reviewable state link directly to that section.

Recovery does not occupy permanent primary navigation because it is exceptional
and usually empty. Its detail must show retained path, project identity,
ownership state, what cleanup will remove, and whether data will be kept.

The first prototype action is `Review recovery`. A future `Clear retained setup`
action is classified as derivable because the safe Rust implementation exists
but is not exposed through launcher IPC. “Adopt” is not offered.

## First Run

First Run appears only when a future completion flag says onboarding has not
finished. It is a focused flow outside the normal shell but uses the same visual
tokens and components.

Sequence:

1. **Welcome** — explain the product while Doctor runs in parallel.
2. **Choose** — reviewed install is primary; Connect and Browse are quieter
   alternate routes.
3. **Install** — reuse the real Install task states, not a simplified duplicate.
4. **Ready** — open the app or enter the workspace.

When Doctor passes, it never becomes a separate step. When it fails, an inline
Docker-required recovery state appears between Welcome and Choose. The user can
still choose Connect or Browse because neither requires Local Store to install a
container.

`Skip for now` marks onboarding complete and opens Overview. Settings offers
`Run introduction again`; this is a design requirement and therefore a concept
until the completion flag exists.

## Search and keyboard behavior

Search has two levels:

- `Ctrl+K` opens a global command/search palette from anywhere in the ordinary
  shell. It searches saved apps first, then catalog projects, and includes direct
  commands such as Connect, Open Settings, and Run Docker check. This is a V2
  concept requiring aggregation/orchestration work.
- Discover and My Apps retain visible, screen-scoped search fields. Discover
  searches the catalog; My Apps filters saved apps. These are available today.

Keyboard rules:

- `Ctrl+K` is immediate and receives no entrance animation.
- `Escape` closes the topmost drawer, dialog, or palette and restores its trigger.
- Arrow-key list navigation may be explored for My Apps, but Tab navigation must
  remain complete without it.
- Search and filter state survives opening and closing contextual detail.
- Install and destructive confirmations never share a keyboard shortcut.

## Route and surface map

```text
First Run
├── Connect ────────────────┐
├── Browse ─────────────┐   │
└── Reviewed install ─┐ │   │
                     │ │   │
Application shell    │ │   │
├── Overview          │ │   │
│   ├── attention app ├─┼───┤
│   ├── recovery ─────┼─┼──┐│
│   └── Docker check ─┼─┼──┼┼──> Settings / Diagnostics
├── Discover <────────┘ │  ││
│   ├── project drawer  │  ││
│   ├── Connect ────────┘  ││
│   └── Install task ──────┼┼──> My Apps / selected app
├── My Apps <───────────────┘│
│   ├── Overview detail      │
│   ├── Logs                 │
│   └── Manage               │
├── Activity ────────────────┘
│   └── app event ──────────────> My Apps / selected app
└── Settings
    ├── Docker diagnostics
    ├── Diagnostics & Recovery
    ├── Catalog snapshot
    └── About / build
```

## Krug trunk test

The toolkit asks, when dropped into a screen cold: where am I, what can I do
here, and what is one level up?

| Screen | Where am I? | What can I do? | What is one level up? | Result / required remedy |
| --- | --- | --- | --- | --- |
| Overview | Selected rail item and `Overview` heading | Inspect health, open an attention item, Connect | Application shell | Pass; avoid duplicating a full app list. |
| Discover | Selected rail item, heading, catalog count | Search/filter, inspect project, Connect, Review install | Application shell | Pass if catalog appears above the fold. |
| Project drawer | Project identity inside visible Discover context | Perform its one capability action, inspect evidence, close | Discover | Pass; drawer must not cover the selected nav or all catalog context. |
| My Apps | Selected rail item and `My Apps` heading | Filter/select apps, use selected app actions | Application shell | Pass if object type and selected row are unmistakable. |
| App Logs | Selected app remains visible; local `Logs` section selected | Refresh/copy/read logs, return to Overview section | Selected app in My Apps | Pass; do not open an identity-less global log modal. |
| Install Review | Recipe identity and `Review install` task heading | Change configuration, inspect evidence, install, go back | Originating project/First Run | Pass if “nothing has run yet” and Back destination are visible. |
| Install Progress | Same recipe/task frame and active stage | Cancel when truthful, read status | Install Review / originating context | Pass; never replace the whole frame with a generic spinner. |
| Activity | Selected rail item and time/filter context | Filter events, inspect an app/event | Application shell | Pass once concept-data labeling is unmissable in the prototype. |
| Settings | Utility selection and `Settings` heading | Diagnose environment, inspect recovery/catalog/build facts | Application shell | Pass; Settings is visually separate but not hidden. |
| Recovery detail | Settings context plus candidate identity | Review evidence, keep data, or clear when safely supported | Diagnostics & Recovery | Pass if snapshot versus action-time verification is explained. |
| First Run | Step rail and Local Store identity | Continue, Connect, Browse, or Skip | No ordinary parent; temporary entry flow | Pass if alternate routes remain available when Docker is missing. |
| Global palette | Query field over a still-recognizable shell | Open app/project/command or close | Current screen | Pass if results are grouped as My Apps, Discover, and Commands. |

## IA decisions deferred to visual prototyping

These do not block the architecture and should be resolved with screen variants:

- Exact rail width and whether labels compress at 1280×800.
- Dark-raised versus warm-light My Apps detail surface.
- Compact recipe strip versus grainy feature panel in Discover.
- Drawer width for catalog detail at both laptop viewports.
- Whether Overview shows healthy apps at all when nothing needs attention.
- Whether Activity concept metrics deserve a chart after the table/event design is
  proven useful without one.


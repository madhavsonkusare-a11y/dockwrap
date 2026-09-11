# V2 interactive prototype foundation

Status: Phase 6 complete. Open `index.html` directly or serve the repository
root with any static server. There is no framework, package install, or build
step.

## Structure

| File | Responsibility |
| --- | --- |
| `index.html` | Stable laptop shell, semantic landmarks, navigation, prototype panel, and local SVG icon sprite |
| `tokens.css` | Primitive and semantic design tokens established in Phase 3 |
| `components.css` | Shared controls, panels, state badges, fields, app icons, empty/loading states, focus, and reduced-motion behavior |
| `prototype.css` | Laptop shell and screen-composition layouts at 1280×800 and 1440×900 |
| `prototype-fixtures.js` | Explicitly separated `contract`, `derived`, and `concept` fixture namespaces |
| `prototype.js` | Hash navigation, renderers, forced conditions, keyboard behavior, local search, selection, and prototype feedback |

## Route and condition model

The ordinary shell exposes Overview, Discover, My Apps, Activity, and Settings.
The design-lab panel also jumps directly to Install, Recovery, and First Run.
All navigation uses URL fragments and replaces the work surface without a page
reload. Back/forward navigation remains meaningful.

Every route supports `default`, `loading`, `empty`, `busy`, `success`, and
`failure`. Loading and empty replace the route body; busy, success, and failure
preserve the screen context and user selection while adding immediate feedback.
This is a prototype state switcher, not simulated backend timing.

## Fixture truth contract

- `contract` shapes mirror the current Rust models and commands: `AppView`,
  `InstalledApp`, `RuntimeSpec`, `DoctorReport`, catalog records, recipe details,
  and `RecoveryCandidate`.
- `derived` contains only deterministic summaries from those values, such as
  counts by runtime type or status.
- `concept` contains the unimplemented persisted Activity history. Its screen
  always displays a `Concept data` warning; it is never styled as backend truth.

Prototype actions do not invoke Tauri, open project URLs, install containers, or
mutate retained files. Their immediate toast explains the intended behavior and
which later phase owns the complete flow.

## Interaction baseline

- Frequent navigation and selection are instant and preserve spatial context.
- Buttons use 120ms press feedback; the infrequent design-lab panel uses a 220ms
  transform/opacity transition from its trigger side.
- `Escape` closes the lab and returns focus to its trigger. `Ctrl+K` immediately
  exposes the future global-search concept without pretending it is implemented.
- Native links, buttons, fields, select controls, listbox semantics, focus rings,
  a skip link, live feedback, and reduced-motion handling are present from the
  first iteration.
- App artwork is bundled and uses containment without cropping. Fonts, the Satin
  Ember mark, controller code, and screen icons are all local.

Verify the foundation with:

```text
node scripts/check-v2-prototype.mjs
```

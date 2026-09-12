# Local Store V2 implementation handoff

Updated: September 12, 2026  
Status: **approved.** Madhav Sonkusare approved the prototype and this handoff on
September 12, 2026. The freeze is locked, and implementation can start.

This is the entry point for whoever implements V2 under `src/`. It assumes no memory of
the design phases.

## What this package is

| Document | Answers |
| --- | --- |
| `HANDOFF.md` (this file) | What to build, in what order, and how it is accepted |
| `freeze.json` + `scripts/check-v2-freeze.mjs` | Which token, font, and state values may not drift |
| `COMPONENT-SPECS.md` | What every component is made of, and its states |
| `COPY-RULES.md` | The words, the terminology, and machine-fact formats |
| `BACKEND-DEPENDENCIES.md` | What each surface needs from the backend, and every prototype annotation to strip |
| `PRODUCTION-ASSETS.md` | Fonts, brand mark, system icons, app logos |
| `VISUAL-SYSTEM.md` | Colour, type, spacing, geometry, grain, and density rules |
| `INFORMATION-ARCHITECTURE.md` | Why the screens are divided this way |
| `MOTION-SPEC.md` | Durations, easing, and reduced-motion substitutions |
| `evaluation.md`, `visual-verification.md` | What was tested, what was found, and how it was fixed |
| `index.html` and the prototype files | The working reference. Serve the repository root and open `docs/design/v2/index.html` |

The whole package is committed on the **`design/v2`** branch, so a branch switch in a
shared working tree cannot remove it. It is not pushed.

**Read in this order:** this file, then `COMPONENT-SPECS.md`, then
`BACKEND-DEPENDENCIES.md`. Open the prototype beside them.

## The freeze

`scripts/check-v2-freeze.mjs` records and guards 135 tokens across three contexts, five
font files by SHA-256, and 88 states across 9 screens. It fails on any drift, naming
what changed.

```bash
node scripts/check-v2-freeze.mjs           # verify
node scripts/check-v2-freeze.mjs --write   # re-record after an approved change
```

**Change control.** Tokens, fonts, the screen and state inventory, the component
specifications, and the copy rules change only through a design decision recorded in the
relevant document, followed by `--write` and a re-run of the phase suites. Production
copies `tokens.css` and `components.css` verbatim rather than re-deriving values, so the
same check guards the shipped product.

The freeze reports `status: "approved"`, with the approver and the date.

## Screens and navigation

Eight navigable destinations plus the modal surfaces. The freeze counts nine, because
it also records the prototype's design-lab panel, which production does not ship.

| Screen | Route | Job | States | In production today |
| --- | --- | --- | --- | --- |
| Overview | `#overview` | What changed, what runs, what needs attention | 7 | New |
| Discover | `#discover` | Browse the offline catalog; install reviewed recipes; connect apps | 10 | The Discover page, plus the filters and detail dialogs |
| My Apps | `#my-apps` | Open, inspect, and manage saved apps | 17 | The My Apps list, plus the logs, uninstall, and remove dialogs |
| Activity | `#activity` | Operations, newest first | 8 | New |
| Settings | `#settings` | Docker check, recovery, catalog facts, about | 6 | The settings and about dialogs |
| Install | `#install` | Focused task: review, progress, result | 21 | The install dialog |
| Recovery | `#recovery` | Focused task: review and clear a retained setup | 11 | Part of the settings dialog |
| First Run | `#first-run` | Introduction and first useful action | 7 | New |

Rail order is Overview, Discover, My Apps, Activity, then Settings and the Docker status
card at the bottom. Install and Recovery are focused tasks: they replace the workspace,
keep a back link, and mark their parent destination with `aria-current="location"`.

Two dialogs become focused screens (Install, Recovery) because they carry consequences
and need room. Confirmations stay dialogs: uninstall, delete app and data, and remove
linked app.

## Prototype to production map

Mapped against `2d7782e`. Module boundaries change slowly; re-check file names before
starting.

| Prototype | Production today | V2 target |
| --- | --- | --- |
| `tokens.css` | Custom properties at the top of `src/styles/app.css` | `src/styles/tokens.css`, copied verbatim |
| `components.css` | Primitive rules spread through `app.css` | `src/styles/components.css`, copied verbatim |
| `prototype.css` | `app.css` and `catalog.css` | One stylesheet per screen under `src/styles/`, plus `shell.css`. Drop every `.prototype-panel`, `.lab-tag`, and `.truth-label` rule |
| `index.html` shell: rail, workspace bar, icon sprite, toast region | `src/index.html` sidebar, topbar, and dialog elements | Rewritten `src/index.html`: rail, workspace bar, screen mount, sprite, toast region, and the three confirmation dialogs |
| Screen render functions (`overview()`, `discover()`, `myApps()`, …) | `render.js` card and row builders, called from `app.js` | `src/js/screens/<screen>.js`, each exporting a render and a bind function |
| `header()`, `appIcon()`, `icon()`, `escapeHtml()` | `render.js` | Keep in `render.js` |
| `bindLocalControls()`, `handle*Action()` | `app.js` | Per-screen bind functions; `app.js` keeps routing and shared state |
| `finishTransientExit()`, `@starting-style` rules | `motion.js` (dialogs only) | Extend `motion.js` with the drawer, popover, and panel exits |
| Fixture modules | `api.js` `invoke` | `api.js` unchanged; screens call commands directly |
| Preview URL parameters | — | Nothing. State comes from the backend |

Existing modules that carry product truth stay and are reused as they are:
`operations.js` (stage labels, cancellation, unsafe-retry rule), `readiness.js` (bounded
polling), `setup-form.js` (typed setup fields), `catalog-controls.js` (filters),
`recovery.js`, and `api.js`.

## Implementation sequence

Each step leaves the launcher fully usable, and its exit criteria must pass before the
next step starts.

**Step 0 — Prerequisites.** Land `feat/catalog-icons` so every catalog entry has a logo.
Copy the five WOFF2 files and both OFL texts into `src/fonts/` and update
`THIRD_PARTY_NOTICES.md`. Add `check-v2-freeze.mjs` to CI. *Exit:* the freeze check
passes against `src/styles/tokens.css` once it exists; no font request fails.

**Step 1 — Foundation.** Bring `tokens.css` and `components.css` into `src/styles/` and
restyle the current screens onto them, without changing structure. *Exit:* all 52
existing UI tests pass with updated screenshot baselines; axe is clean; the token and
legibility gates pass.

**Step 2 — Shell.** Rail, workspace bar, routing, and the toast region. Discover and My
Apps become routed screens inside the new shell; the dialogs keep working. *Exit:*
navigation and keyboard tests pass; focus moves to the screen heading on route change.

**Step 3 — Discover.** Toolbar, capability tabs, catalog cards, the filter popover, and
the project and connect drawers, replacing the filters, detail, and connect dialogs.
*Exit:* `catalog.spec.js` and the Discover half of `app.spec.js` pass, ported; V-10,
V-12, V-13, V-14, and V-15 behaviours hold.

**Step 4 — My Apps.** Master and detail, the Overview, Logs, and Manage tabs, and the
confirmation dialogs. *Exit:* `operations.spec.js` and `readiness.spec.js` pass, ported;
linked apps never show managed-only actions.

**Step 5 — Install as a focused screen.** Review, setup fields, staged progress with
cancellation, and the result matrix including rollback failure. *Exit:* `setup.spec.js`
and the install tests pass, ported; the commit pins below 760px of height (V-16).

**Step 6 — Settings and Recovery.** Settings becomes a screen; Recovery becomes a
focused task. The cleanup action ships only with a launcher command that re-verifies
ownership. *Exit:* `recovery.spec.js` passes, ported; without the command, Recovery is
read-only and says so.

**Step 7 — Overview.** Needs the summary projection. Attention list, counts, and
session-only recent activity. *Exit:* every number traces to a backend value.

**Step 8 — Activity.** Session-only, labelled as such, until durable history exists.

**Step 9 — First Run.** Needs the persisted completion flag. Until then it is reachable
from Settings but does not open by itself.

**Step 10 — Strip and accept.** Remove every annotation listed in
`BACKEND-DEPENDENCIES.md` and run the full acceptance suite.

## Playwright baselines

Ten screenshot baselines are replaced, not deleted. Re-record them at the step that
changes the screen.

| Baseline | Replaced in | Note |
| --- | --- | --- |
| `app.spec.js-snapshots/discover-1280x800.png` | Step 3 | Keep the viewport |
| `app.spec.js-snapshots/discover-800x600.png` | Step 3 | Only if the minimum window stays 800×600 (open decision) |
| `app.spec.js-snapshots/discover-400x860.png` | Step 3 | V2 has no layout this narrow; retire it unless the minimum window changes |
| `app.spec.js-snapshots/my-apps.png` | Step 4 | |
| `app.spec.js-snapshots/install-review.png` | Step 5 | Becomes a screen, not a dialog |
| `app.spec.js-snapshots/connect-dialog.png` | Step 3 | Becomes the connect drawer |
| `catalog.spec.js-snapshots/catalog-desktop.png`, `catalog-compact.png` | Step 3 | |
| `catalog.spec.js-snapshots/catalog-filters.png` | Step 3 | Becomes the filter popover |
| `catalog.spec.js-snapshots/settings.png` | Step 6 | Becomes a screen |

The 52 behaviour tests are ported, never dropped. They encode product truths the design
must keep: busy rows that hold focus, a cancel that is offered only before the commit, a
failed cleanup that blocks a blind retry, answers that do not survive their dialog, and
readiness that never invents a state it did not check.

## Production acceptance tests

Define these before implementation starts. They run against the real launcher UI through
the existing harness (`scripts/preview.mjs` with the mocked IPC in `tests/ui/fixtures.js`),
at 1440×900, 1280×800, and 1280×640 at 1.5×.

| # | Test | Reuses |
| --- | --- | --- |
| A | Zero axe violations (WCAG 2.0, 2.1, 2.2 A and AA) on every state | `check-v2-evaluation-gates.mjs` |
| B | A real Tab walk: every control reachable, focus visible at 3:1 or better, no escape from a modal | same |
| C | Text at 13px or above (mono labels at 11px only with tracking); form boundaries at 3:1 | same |
| D | Tokens, fonts, and the state inventory match the freeze | `check-v2-freeze.mjs` |
| E | Layout probe: no horizontal overflow, no unreadable truncation, no clipped glyphs, no overlapping controls, no logo outside its frame, no covered primary, every dialog and popover fits, and a focused task's commit is on screen at 1280×640 | `v2-layout-probe.mjs`, `check-v2-visual.mjs` |
| F | Screenshots are byte-deterministic across fresh browser contexts | `check-v2-visual.mjs` |
| G | Rest, hover, press, focus, selected, disabled, and busy are each visibly distinct | same |
| H | No prototype annotation ships: no `.lab-tag`, `.truth-label`, `#prototype-panel`, `[data-prototype-action]`, or preview parameter | new, from `BACKEND-DEPENDENCIES.md` |
| I | No failed asset request and no network request at run time, fonts included (V-17) | new |
| J | Every ported behaviour test, plus the V-register behaviours | `check-v2-visual-fixes.mjs`, `check-v2-evaluation-fixes.mjs` |
| K | Report: the launcher's default and minimum window sizes | `check-v2-real-windows.mjs` |

A and B are release blockers. So are E and H.

## Decisions

Settled on September 12, 2026 by Madhav Sonkusare.

1. **The prototype and this handoff are approved.** The freeze is locked.
2. **The launcher window.** Open at **1280×800**, clamped to the monitor's work area,
   with `min_inner_size` **1180×640** — the smallest width and height verified. Apply it
   in `src/main.rs` during Step 2; it is not a design-package change.
3. **Narrow layouts.** V2 has no layout below 1180px wide, and none will be built. The
   `discover-400x860` and `discover-800x600` baselines are retired rather than
   re-recorded.
4. **Activity ships session-only**, labelled "This session". Durable history comes
   later and does not change the screen's design.
5. **The Ctrl K command is omitted.** Catalog search and app filtering ship instead.
   The prototype keeps it flagged as a concept.

Still outstanding, and not a design decision: **land `feat/catalog-icons`** before or
with V2, so every catalog entry has a logo. Without it, 924 of 1,676 entries fall back
to a monogram.

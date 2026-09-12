# Local Store V2 component specifications

Updated: September 11, 2026  
Status: **frozen**. Approved by Madhav Sonkusare on September 12, 2026. Changes need a
recorded design decision (see `HANDOFF.md`).  
Normative sources: `tokens.css` for values, `components.css` for primitives, and
`prototype.css` for compositions. This document names every component, its parts, the
tokens it consumes, and its states. The CSS is the pixel-level specification, and
production copies it rather than re-deriving it.

## Rules every component follows

| Concern | Rule | Verified by |
| --- | --- | --- |
| Focus | `:focus-visible` only: 2px `--v2-action-focus` outline, 3px offset. The ring is at least 3:1 against every adjacent surface. | Phase 14 keyboard walk, 2,558 stops |
| Hover | Only on devices that hover (`@media (hover: hover) and (pointer: fine)`). Hover never carries meaning alone. | Phase 15 interaction states |
| Press | `scale(.97)` over `--v2-duration-press` (120ms), plus a pressed fill: `--v2-surface-active`, or `--v2-action-primary-pressed` on primary (dark text 4.61:1). Danger keeps its danger fill. The scale is removed under reduced motion; the fill stays. | Phase 13 motion suite, Phase 15 states |
| Disabled | The `disabled` attribute, not a class. Opacity .48, `cursor: not-allowed`, no press transform. | Phase 15 interaction states |
| Busy | The container gets `aria-busy="true"`, or the change is announced by `role="status"` or a live region. A busy button names the work in progress ("Checking…") and is disabled. | Phase 15 interaction states |
| Selected | `aria-current="page"` for navigation, `aria-selected` for list rows and tabs, `aria-pressed` for toggles. Selection always has a cue besides colour: the 3px ember inset bar, a stronger border, or weight. | Phase 14 axe, Phase 15 states |
| Type floor | 13px for anything read. `--v2-type-mono-label` (11px) only for short isolated tags, always with `--v2-tracking-label`. | Phase 14 legibility gate |
| Machine facts | Mono layer, `overflow-wrap: anywhere`. Never truncated. | Phase 15 V-01, V-06 |
| Truncation | Only scan-surface names and categories may ellipsize, and they carry a `title`. Operational messages get two lines, then clamp. Guidance always wraps. | Phase 15 V-03 to V-11 |
| Logos | A square frame with definite grid tracks. Contained at any aspect ratio, never cropped or stretched. | Phase 15 V-13 |
| Motion | Durations and easing come only from motion tokens; reduced motion shortens or removes them. See `MOTION-SPEC.md`. | Phase 13 motion suite |

## Primitives (`components.css`)

| Component | Parts | Size and tokens | Variants | States |
| --- | --- | --- | --- | --- |
| Button | Label, optional 16px leading or trailing icon | `--v2-control-height` (42px) min height, `0 --v2-space-5` padding, pill radius, 14px/600 | Default (elevated surface). **Primary**: ember fill, `--v2-text-on-accent` label, accent shadow; one per view. **Ghost**: transparent. **Danger**: danger text on danger-soft fill with danger border; never the default action | Hover, pressed, focus, disabled, busy |
| Icon button | 18px icon | 42×42px, `--v2-radius-md`, default border | — | Hover, pressed, focus, disabled. Requires `aria-label` |
| Text button | Label, optional 15px icon | 14px/600, `--v2-action-link` | Secondary card action | Hover (`--v2-action-primary-hover`), pressed, focus |
| Panel | Container | Default border, `--v2-radius-lg` (20px), raised surface, `--v2-shadow-raised` | Grain surface for the single signature region per viewport | — |
| Badge | Label | 26px, pill, 13px/600 | Neutral, success, warning, danger | Static; always a word, never colour alone |
| Type pill | Label | 24px, bordered, 13px/500 | "Managed app" or "Linked app" | Static |
| Status dot | 7px dot with a 4px halo | State colours | Neutral, success, warning, danger | Always paired with a state word |
| App icon | Logo image | 44px (`--v2-app-icon`); 64px large; 32px small | Neutral backing tile only when the logo lacks contrast | Contained at any aspect ratio |
| Field label | Label above the control | 13px/500, tertiary text | Visually hidden where the field has a visible name | — |
| Input and select | Control, optional select chevron | `--v2-control-height-lg` (46px), `--v2-border-control` (at least 3:1), sunk surface, `--v2-radius-md` | Search field: 17px leading icon, text inset `--v2-field-icon-inset` | Focus, invalid (`aria-invalid` plus a visible message under the field), disabled |
| Empty state | Symbol tile, heading, sentence, action | 54px symbol tile (`--v2-radius-icon-lg`), 350px min height | — | — |
| Condition banner | Icon, bold lead, sentence | `--v2-radius-md`, 13–14px | Default, success, failure, busy | Enters with a 4px rise (reduced motion: opacity only) |
| Skeleton | Block with a shimmer | 112px (232px hero), `--v2-radius-lg` | Stack, row of three | Static at .35 opacity under reduced motion |
| Toast | Bold lead and a sentence, in a fixed region | 360px max, elevated surface, `--v2-shadow-elevated` | — | Slides up; `role="status"`; never takes focus |
| Machine fact, kbd | Mono text | `--v2-type-mono` (13px); kbd uses the mono label | — | — |

## Application shell

| Component | Spec | States and behaviour |
| --- | --- | --- |
| App rail | `--v2-rail-width` (216px; 200px at ≤1320px wide), sunk surface. From top: brand mark, Workspace navigation, Settings, Docker status card | — |
| Navigation item | 42px min height; grid of 20px icon, label, and a mono count. `--v2-radius-md` | Current: elevated surface, default border, 3px ember inset, primary text, `aria-current="page"`. While a focused task (Install, Recovery) is open, its parent destination takes `aria-current="location"` with a neutral inset |
| Docker status card | A link to Settings. Status dot, state word, mono detail (engine version, or what happened), chevron. 61px min height | Ready, checking, unavailable. Detail wraps; it is never truncated (V-11) |
| Workspace bar | Sticky, 72px, blurred canvas at 88%. Breadcrumb on the left; the search command on the right | Content scrolls beneath it |
| Screen header | Eyebrow (only when it carries state; see `COPY-RULES.md`), 38px title, one-sentence description, actions on the right | One primary action at most |

## Browse surfaces

| Component | Spec | States and behaviour |
| --- | --- | --- |
| Catalog card | 190px min height, 16px padding. Logo line (44px logo, name, category), two-line description, mono facts (licence and architectures, with empty facts omitted), footer (Details text button plus a capability action) | The capability action is Review install (primary), Connect (default), or Explore (external). The name and category ellipsize with a `title`. The description clamps, and Details shows it in full |
| Featured recipe card | On the grain surface: logo tile, name, mono version, mono address, corner arrow | Hover lifts 2px on hover-capable devices. Press. Opens the install review |
| Discover toolbar | Search field; capability tabs (`aria-pressed`, 38px segments in a sunk track); Filters trigger with an active-count badge | — |
| Filter popover | 330px, anchored to the trigger. Category, licence, architecture, the maintenance-warning toggle, Clear | Opens with focus on the first field and reveals itself inside the window (V-12). Escape and Close return focus to the trigger |
| Pagination footer | Count plus "Load next 24" | Hidden when nothing matches. A narrowed list reads "Showing N matching projects" (V-10) |
| Project drawer | `--v2-drawer-width` (480px), full height. Identity, capability statement, warning, facts, provenance, commit row | Modal: focus trap, Escape, and a scrim to dismiss. The commit row is sticky with a divider (V-14) |
| Connect drawer | Name, HTTP(S) address, advisory note, feedback line, Check address and Save | The address starts empty, with an example placeholder. Validation is visible under the field. Saving works while the server is offline |
| Saved-app row | 68px. Logo, name (ellipsis and `title`), type, and a status word with a dot | Selected: elevated surface, strong border, 3px ember inset, `aria-selected="true"`. Arrow keys move through the list |
| Detail tabs | Overview, Logs, and Manage, with 24px gaps and an underline for the current tab | `aria-selected`; Left and Right arrows, Home, and End move between tabs and keep focus on the new tab |
| Log view | 286–370px, mono 13px at 1.75 line height, `--v2-surface-log` background with `--v2-text-log` text (12.1:1) | Bounded snapshot; Refresh and Copy; loading, empty, and failure states |
| Attention and recent rows | Logo, name, message, action | The message gets two lines, then clamps; its full value is the `title` |
| Metric card | Number, label, truth source | Four across; a "needs attention" card is tinted |
| Activity event | Row with expandable detail; filter chips (`aria-pressed`) | Expanded detail shows machine facts, wrapped |

## Trust surfaces and focused tasks

| Component | Spec | States and behaviour |
| --- | --- | --- |
| Install review | Identity, fact list, evidence grid (two columns, machine facts wrapped), setup fields, commit boundary | Setup fields are editable; required answers are asked for before Docker is |
| Install progress | Stage list: checking system, preparing files, validating recipe, starting containers, waiting for health, saving app, and rolling back on failure | Cancel is offered only before the commit boundary. The progress region is `aria-busy="true"` while work runs |
| Result panel | Result mark, eyebrow, 30px heading, sentence, status block, mono error code, result actions | Success, failure, cancelled, and unsafe (rollback failed: retry withheld). Below 760px of height the action row pins to the window bottom (V-16) |
| Recovery card and confirmation | Identity, ownership assurance, trust rows, consequences, typed confirmation for deletion | Deletion stays disabled until the app name is typed. Every action re-verifies ownership. Mismatch and scan failure block cleanup |
| Dialog | `--v2-dialog-width` (500px), constrained by the viewport, 32px padding, `--v2-radius-lg`, `--v2-shadow-dialog` | Focus trap, Escape, and a return-focus target. Irreversible actions use the danger variant and require typed confirmation |

## First Run

| Component | Spec | States and behaviour |
| --- | --- | --- |
| First Run shell | A 248px steps rail beside the content, in a raised panel. It clips with `overflow: clip`, so sticky footers work | Welcome, Choose, Install, Ready |
| Preflight checks | Engine and Compose rows, each a dot, word, and badge | Checking, ready, missing. Missing shows the recovery steps and browsing stays open |
| Starter card | 172px min height, 20px padding, logo, name, sentence, mono version and licence | A radio group (`role="radio"`, `aria-checked`). Selected uses the ember material with grain; unselected has a hover state (V-09) |
| Narrow state actions | Primary plus secondary | Below 760px of height the row pins to the window bottom (V-16) |

## Prototype-only (never ship)

The design-lab panel and its trigger, lab tags, truth labels, the search "Concept" flag,
`?stress=1` and every other preview parameter, and every "withheld" toast. The complete
list, with the production replacement for each, is in `BACKEND-DEPENDENCIES.md`.

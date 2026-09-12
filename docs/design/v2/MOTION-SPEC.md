# Local Store V2 motion specification

Updated: September 10, 2026  
Canonical tokens: `docs/design/v2/tokens.css`

## Principles

Motion exists to confirm input, preserve spatial origin, or make an asynchronous
state change legible. It is not applied to data simply to make the interface feel
active. Every implemented transition uses opacity and transform only; progress
indicators are predetermined CSS animations and never imply a fabricated percent.

## Inventory and gate decisions

| Surface or action | Frequency | Purpose | Decision |
| --- | --- | --- | --- |
| Primary navigation, app selection, tabs | Frequent | None beyond direct navigation | Instant content replacement; no entrance animation |
| Ctrl+K and keyboard list/tab navigation | Very frequent | Speed | Never animate |
| Search and catalog filtering | Frequent | Direct manipulation | Results update instantly; an already-open filter popover does not re-enter |
| Buttons and pressable cards | Frequent | Feedback | 120ms scale press; hover motion remains pointer-gated |
| Catalog filter popover | Occasional | Spatial consistency | Scale from the top-right trigger at 160ms |
| Catalog detail/connect drawer | Occasional | Spatial consistency | Slide from and return to the right edge at 220ms |
| Prototype controls panel | Occasional | Spatial consistency | Slide from and return to its right-edge trigger at 220ms |
| Uninstall/delete dialogs | Rare | Preventing a jarring change | Centered scale and fade at 220ms; backdrop moves with it |
| Toast | Occasional | Feedback | Enter and exit through the bottom at 220ms |
| Condition banners | Occasional | State indication | Fade and 4px settle at 160ms |
| Activity disclosure | Occasional | State indication | Detail settles 4px at 160ms; chevron rotates at 160ms |
| Install/recovery results | Rare | State indication | Fade and 6px settle at 160ms |
| First-run illustration | First-time | Explanation | Mark and workspace card settle at 220ms with a 40ms card delay |
| Skeletons | Loading only | State indication | 1.6s linear shimmer; disabled for reduced motion |
| Busy rings | Active operation only | State indication | 900ms linear rotation; stages, not percentages, communicate progress |
| Memory graph and grain surfaces | Readable data/decorative material | None | Static; values and textures never drift or pulse |

## Ingredients

| Token | Value | Use |
| --- | --- | --- |
| `--v2-duration-press` | 120ms | Physical press feedback |
| `--v2-duration-fast` | 160ms | Popovers, disclosures, and state changes |
| `--v2-duration-panel` | 220ms | Drawers, dialogs, panels, and toasts |
| `--v2-ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Entrances, exits, and feedback |
| `--v2-ease-drawer` | `cubic-bezier(0.32, 0.72, 0, 1)` | Right-edge drawer travel |
| Linear | CSS keyword | Continuous progress only |

Entry uses CSS `@starting-style`. Exit applies the matching closing state and
waits for `transitionend`, with a 280ms safety timeout before the DOM is replaced.
Rapidly updated controls suppress remount motion, so transitions retarget or the
content changes instantly rather than restarting a keyframe.

## Immediate feedback

Install, diagnostics, recovery, and confirmation actions render their busy or
working state in the same event task. The 160ms visual settle follows that DOM
update; it never delays the status text or disables the control late.

## Reduced motion

With `prefers-reduced-motion: reduce`, positional transforms are removed. Overlays,
dialogs, banners, disclosures, and results retain a short 100–120ms opacity change
for continuity. Press scaling, onboarding travel, skeleton shimmer, and busy-ring
rotation are disabled. Navigation and keyboard actions remain instant in both modes.


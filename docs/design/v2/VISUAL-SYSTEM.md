# Local Store V2 visual system

Updated: September 9, 2026  
Canonical tokens: `docs/design/v2/tokens.css`  
Rendered reference: `docs/design/v2/specimen.html`
Motion reference: `docs/design/v2/MOTION-SPEC.md`

## Direction

Local Store should feel like warm, trustworthy machinery: technically explicit
without looking like an admin template. Near-black surfaces form the structure,
an ember orange identifies the product and primary commitment, and a mono layer
distinguishes facts asserted by the machine from prose written by the product.

The system is dark-only and laptop-first. It is designed for 1280×800 as the
minimum working canvas and 1440×900 as the primary canvas.

## Color

The neutral ramp is warm enough to belong beside orange but not so brown that
white content appears dirty. Surfaces separate through small lightness changes,
hairlines, and occasional elevation—not repeated gradients.

| Role | Token | Value | Use |
| --- | --- | --- | --- |
| Canvas | `--v2-surface-canvas` | `#0c0a09` | Application ground |
| Sunk | `--v2-surface-sunk` | `#100d0c` | Rail, inputs, logs |
| Raised | `--v2-surface-raised` | `#161211` | Cards and panes |
| Elevated | `--v2-surface-elevated` | `#1d1816` | Selected controls and nested task regions |
| Hover | `--v2-surface-hover` | `#251e1b` | Pointer hover only |
| Primary text | `--v2-text-primary` | `#f7f1e9` | Titles and important content |
| Secondary text | `--v2-text-secondary` | `#c3b8ae` | Ordinary supporting prose |
| Tertiary text | `--v2-text-tertiary` | `#998b80` | Labels and captions |
| Ember | `--v2-action-primary` | `#f26419` | One primary commitment per task region |
| Focus | `--v2-action-focus` | `#ff9b5c` | Keyboard focus and high-visibility accent text |
| Success | `--v2-state-success` | `#62c79a` | Healthy/running state |
| Warning | `--v2-state-warning` | `#f5c451` | Attention without failure |
| Danger | `--v2-state-danger` | `#e8614c` | Failure and destructive intent |

Measured contrast against the canvas:

| Foreground | Ratio | Decision |
| --- | ---: | --- |
| Primary text | 17.60:1 | AAA |
| Secondary text | 10.15:1 | AAA |
| Tertiary text | 5.98:1 | AA |
| Ember | 6.22:1 | AA |
| Focus orange | 9.50:1 | AAA |
| Success | 9.55:1 | AAA |
| Warning | 12.13:1 | AAA |
| Danger | 5.88:1 | AA |

`--v2-text-quiet` measures 3.72:1 and is reserved for disabled, decorative, or
non-text marks. It must not render readable product copy. The specimen initially
used it for captions; the visual audit corrected those captions to tertiary.

Orange-filled controls always use `--v2-text-on-accent` (`#2a0f03`). Contrast is
5.66:1 on the default ember, 6.66:1 on hover, and 4.61:1 on pressed ember. White
or cream text is prohibited on orange button fills.

## Signature ember material

The grainy ember gradient is a signature surface, not a generic card style. It is
reserved for three high-value moments: the reviewed-recipe editorial banner, the
optional future-telemetry study, and the currently selected first-run recipe. All
other structure stays neutral so the material retains meaning and visual impact.

The grain is generated from fine fractal noise layered over an asymmetric
red-to-orange field. It must read as film texture at laptop scale—not as discrete
dots, a tiled pattern, or a paper simulation. `Ember Paper` is not part of V2.

Charts follow the same truth rules as the rest of the product. The current backend
does not expose container memory, so the memory graph is hidden behind the
prototype's Future signals switch and carries both `Concept telemetry` and
`Backend work` labeling. It must not appear as live product data until a bounded
telemetry contract exists.

## Typography

### Instrument Sans

Instrument Sans is the product voice. Use it for navigation, headings, body
copy, explanations, buttons, labels, and status words. The prototype bundles
Regular 400, Medium 500, and SemiBold 600 locally under the SIL Open Font
License. Do not simulate unavailable weights.

- Display and screen titles: 600
- Section and component headings: 600
- Buttons and selected navigation: 600
- Labels and ordinary emphasis: 500
- Body and supporting prose: 400

### IBM Plex Mono

IBM Plex Mono is evidence, not decoration. Use it only when the interface is
showing a value asserted or consumed by the machine:

- image names, versions, and digests;
- ports, URLs, paths, and project IDs;
- Docker/Compose versions;
- health targets and timeouts;
- operation IDs, exit codes, log output, timestamps, sizes, and durations.

Do not use mono for navigation, category names, buttons, marketing statements,
or uppercase eyebrows. A status word such as “Running” remains sans; its exact
uptime, when available, is mono.

### Type scale

| Role | Size | Line height | Notes |
| --- | ---: | ---: | --- |
| Display | 52px | 0.98 | Rare screen-level statement, maximum two lines |
| Screen title | 38px | 1.05 | Primary destination heading |
| Large heading | 30px | 1.1 | Focused-task results: install outcomes, recovery confirmation |
| Heading | 27px | 1.15 | Major section/task heading |
| Subheading | 20px | 1.15 | Card group or detail heading |
| Large body | 17px | 1.55 | Introductory or consequential prose |
| Body | 15px | 1.55 | Default reading size |
| Small body | 14px | 1.55 | Dense browse descriptions |
| Label | 13px | 1.3 | Controls and metadata labels |
| Caption | 13px | 1.3 | Supporting text; still reading text |
| Machine fact | 13px | 1.3 | Mono values: ports, versions, paths, digests, times |
| Machine label | 11px | 1 | Mono, uppercase, **always** tracked at `--v2-tracking-label` (0.08em). Short isolated tags only: status, counts, key legends, eyebrows |

**Thirteen pixels is the floor for anything read as text or as a value.** A
dark-only interface uses negative contrast polarity, whose legibility penalty grows
as characters shrink, so the floor sits higher than a light UI would need. The only
exception is the machine label: a short, isolated, letter-spaced tag, which reads
as a shape rather than as prose. Nothing else goes below 13px, including the mono
layer — the facts users most need to trust must not be the smallest text on
screen. The evaluation gate enforces this across every prototype state.

This section previously set a 12px floor while 51 `font:` shorthands shipped mono
at 9–11px. The Phase 14 evaluation (`evaluation.md`, F-01) records the correction.
The design must survive real strings at 100%, 125%, and 150% Windows display
scaling without shrinking type.

**Truncation rules** (Phase 15, `visual-verification.md`):

- Machine facts (images, digests, paths, ports, addresses, errors) wrap with
  `overflow-wrap: anywhere`. They never truncate.
- Instructions and status guidance wrap. A step the user must follow is never cut.
- Operational messages in scan rows get two lines, then clamp, with the full message
  as the row's `title` and shown whole in the detail view.
- Only names and categories on scan surfaces may ellipsize on one line. They carry a
  `title`, and the detail view shows them whole.

## Layout

| Element | 1440×900 | 1280×800 |
| --- | ---: | ---: |
| Navigation rail | 216px | 200px |
| Inline page padding | 48px | 32–43px via `clamp()` |
| Block page padding | 36px | 36px |
| Content maximum | 1220px | Available canvas width |
| My Apps list | 340px | 340px — app names stay whole at the 13px floor |
| Preferred detail pane | 520px | 480px |
| Catalog drawer | 440px | 410px |
| Small dialog | 480px | 480px maximum, constrained by viewport |
| Install task maximum | 1080px | Available canvas width |

Spacing uses a four-pixel base: 4, 8, 12, 16, 20, 24, 28, 32, 40, 48, and 64px.
Zero is permitted for reset and axis-specific declarations. Arbitrary gaps and
padding do not enter component CSS.

Every token a declaration references must exist. An undefined custom property voids
its whole declaration — six margins referenced a missing `--v2-space-7` and silently
rendered as 0 until Phase 14 added it. The evaluation gate now rejects undefined
references.

Three documented exceptions sit below or beside the scale: `1px` gaps that draw
divider grids, `2–3px` optical baseline nudges, and one `54px` column alignment in
Activity. Text that must clear an inline field icon uses `--v2-field-icon-inset`.

## Geometry

The system uses four radius roles:

- `8px` — small internal elements and compact icon tiles;
- `12px` — controls, selected navigation, and compact panels;
- `20px` — destination cards and large content panes;
- `999px` — action buttons, chips, and status pills.

Three supporting geometries complete the set:

- `4px` (`--v2-radius-xs`) — key caps and skeleton lines;
- **icon containers** keep a proportional corner of roughly 0.3 × their size,
  quantised to `--v2-radius-icon-sm` (10px, 30–38px boxes), `-md` (13px, 42–48px)
  and `-lg` (18px, 54–58px);
- `50%` — true circles: status dots, spinners, orbit rings.

The distinction is structural: pills are things the user can press or scan as a
state; rounded rectangles are places that contain content. Do not put pill
radii on text inputs or large cards.

## Surface and elevation hierarchy

1. **Canvas** — flat ground; no shadow or gradient.
2. **Sunk** — navigation, log readers, and input wells.
3. **Raised** — cards and persistent panes with one hairline.
4. **Elevated** — drawers, dialogs, and selected task regions with a stronger
   border or elevation shadow.
5. **Scrim** — only behind modal dialogs; drawers keep application context visible.

Inset highlights and gradients are not the default surface recipe. The primary
button may use a subtle tonal change on hover, and the later grain treatment may
contain a dedicated gradient inside its one signature region.

## Borders and shadows

- Soft borders separate nested rows or machine facts.
- Default borders define cards and panes.
- Text inputs and selects use `--v2-border-control`, which measures at least 3:1
  against every surface it sits on (WCAG 1.4.11). The translucent borders measure
  1.3–1.5:1 and cannot mark a field on their own.
- Strong borders define secondary buttons.
- Accent/state borders appear only when meaning requires them.
- Raised cards receive at most a one-pixel optical highlight.
- Full elevation shadows are reserved for drawers and dialogs.
- Orange glow is restricted to the primary commitment and brand signature.

The interface must remain understandable with shadows disabled. Structure comes
from spacing and borders first.

## Application and system icons

Application logos retain their original colors, aspect ratios, and silhouettes.
They are not recolored, masked into circles, or placed on a different saturated
background for every app.

- 32px: compact list logo
- 44px: default card/row logo
- 64px: selected-app or install identity

Every logo is contained inside its square frame, whatever its aspect ratio. In the
catalog, 137 logos are taller than wide and 23 are at least twice as wide as tall.
The frame's grid tracks are definite (`grid-template: 100% / 100%`), so
`max-height: 100%` holds.

A neutral backing tile is optional only when a logo needs contrast. Its use must
be consistent within that component. Logos with intrinsic backgrounds remain
unframed.

System icons are monochrome and may sit inside a 32px neutral tile. They never
compete with app artwork. State should not be communicated by icon color alone.

## Density

### Browse surfaces

Discover and the My Apps list are scan surfaces. They use 14px descriptions,
compact metadata, 16–20px card padding, short cards, and restrained actions.
The default 1440×900 view should show at least six catalog projects without
requiring scroll. Repeated prose and decorative wrappers are removed.

### Trust and confirmation surfaces

Install, recovery, destructive confirmation, and setup configuration use 15–17px
prose, 24–32px internal spacing, explicit labels, and stable two-column facts.
They may show fewer objects because the task is comprehension, not scanning.
Consequences appear before the commitment action.

### Operational surfaces

Overview and Activity sit between those modes: compact rows for facts, generous
space around attention and next actions. Healthy state should be quiet; errors
should be locatable rather than visually loud across the whole screen.

## Ember and grain

Grain is a product signature, not a surface texture.

Allowed:

- inside the orange square of the approved brand mark;
- one bounded signature region on Discover or First Run;
- a restrained empty-state or completion illustration when no other bloom is on
  that screen.

Prohibited:

- behind body text, forms, logs, tables, or machine facts;
- on ordinary cards, navigation items, dialogs, and detail panes;
- as a full-window background;
- more than once in the same visible viewport;
- as a replacement for state color or hierarchy.

The bloom must remain low-frequency and the grain fine enough not to sparkle or
look animated. Phase 4 owns the final brand-mark implementation and intensity.

## Controls and feedback

- Primary action: ember fill, dark text, one per task region.
- Secondary action: raised neutral fill and strong border.
- Tertiary action: transparent until hover/focus.
- Destructive action: danger text and soft danger surface; never orange.
- Press feedback: `scale(.97)` over 120ms for pointer presses.
- Hover/color feedback: 160ms.
- Drawers and dialogs: 220ms maximum, strong ease-out.
- Keyboard-triggered palette and routine list selection: immediate.
- Reduced motion removes transforms while retaining useful color changes.

The full transition inventory remains a Phase 13 deliverable. These values are
the visual-system defaults, not permission to animate every component.

## Implementation rule

Component CSS consumes semantic tokens. Raw colors, font sizes, radii, and
nonzero spacing values belong only in `tokens.css`. If a component cannot be
expressed with the system, revise the system deliberately or document the
exception; do not add an unexplained literal.

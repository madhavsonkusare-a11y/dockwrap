# Local Store V2 visual verification

Updated: September 11, 2026  
Status: **Phase 15 complete** — every hard check passes at 1280×800 and 1440×900; one
window-size decision handed to Phase 16 (V-16)  
Targets: `index.html`, `prototype.css`, `components.css`, `prototype.js`, `stress-fixtures.js`

## Result

| Check | Coverage | Defects found | Final |
| --- | --- | --- | --- |
| Deterministic captures | 88 states × 1280×800 and 1440×900 = 176 PNGs, SHA-256 in `visual-manifest.json` | — | 176 captured |
| Determinism | 13 states re-captured in a fresh browser context at both viewports | none | **0 differing bytes** in 26 pairs |
| Horizontal overflow | 176 normal + 176 stress renders | none | **0** |
| Text cut with no way to read it | same | V-01 to V-07, V-11 | **0** |
| Glyphs shaved by tight line-height | same | V-08 | **0** |
| Overlapping controls, spilled logos | same | V-13 (and V-15 by eye) | **0** |
| Primary action covered | same | none | **0** |
| Dialogs, drawers, popovers that don't fit or hide a commit | same, plus 1280×640 | V-12, V-14 | **0** |
| Interaction states distinct from rest | 12 controls × rest/hover/active/focus, 5 selections, disabled, 3 busy = 57 | V-09 | **0** |
| Display scaling | 13 key screens at 1280×800 @1.25×/1.5×, plus 4 real laptop panels | none new | **0 new** |
| Human review of every 1280×800 capture | 88 captures, the stress catalog scrolled to its end | V-10, V-13, V-15 | resolved |
| Real window sizes (report only) | 88 states × 3 sizes below the design minimum | V-14, V-16 | V-16 open for Phase 16 |

Probe and harness errors were corrected before any result was recorded (see
*Probe corrections*); no count above includes a known false positive.

**How to run it.** Serve the repository root with
`python -m http.server 8765 --bind 127.0.0.1`, then:

```bash
node scripts/check-v2-visual.mjs              # hard checks + every capture, ~16 minutes
node scripts/check-v2-visual.mjs --quick      # every sixth state, for iteration
node scripts/check-v2-visual-fixes.mjs        # behavioural proof of each V- correction
node scripts/check-v2-real-windows.mjs        # report: launcher default and maximised laptop windows
python scripts/build-v2-contact-sheet.py      # contact sheet + curated approval JPEGs
```

The Phase 14 gates (`check-v2-evaluation-gates.mjs`, `check-v2-evaluation-fixes.mjs`)
and the five phase suites still pass after every Phase 15 correction.

## Method

- **One state matrix.** `scripts/v2-states.mjs` holds the 88 states the Phase 14 gates
  use; Phase 15 imports the same list, so evaluation and visual verification can't drift.
- **Deterministic capture.** Headless Chrome, `reducedMotion: "reduce"`, fonts awaited,
  image cache warmed per context, every image decoded, two animation frames settled, `animations: "disabled"`, caret hidden, a fresh
  `about:blank` between states (a hash-only change is a same-document navigation and
  would carry state). Proof is byte equality, not a pixel threshold.
- **Layout probe** (`scripts/v2-layout-probe.mjs`, run in the page). Horizontal
  overflow; text cut by its own box on either axis, and whether the full value is still
  readable (a `title` or `aria-label`, the same text shown whole elsewhere, or a
  catalog card's Details); glyph clipping as its own class; overlapping controls
  within one layer; logos that spill out of their frame; primaries covered
  (`elementFromPoint`) or below the fold; and every dialog, drawer, and popover checked
  for fit and for a commit or irreversible action hidden by a scroll container.
- **Worst-case real content** (`?stress=1`, fixtures from
  `scripts/build-v2-stress-fixtures.py`). Saved-app names of 46–50 characters, an 82-character
  linked-app URL, a 117-character digest-pinned image
  (`ghcr.io/usememos/memos:0.30.0-rc.2-alpine3.20@sha256:…`), a verbatim 182-character
  Docker daemon error, and 12 real catalog entries chosen from all 1,672: the three
  longest descriptions, three longest names, three widest logos (3.6–3.9:1), and three
  tallest (0.43–0.51:1).
- **Display scaling.** The same 1280×800 CSS viewport at 125% and 150%, and the CSS
  viewports real panels produce at Windows' default scaling: 1920×1080 at 125%
  (1536×864) and 150% (1280×720), 2560×1600 at 150% (1707×1067), 2880×1800 at 200%
  (1440×900).
- **Interaction states.** Real pointer hover and press, and a real keyboard Tab walk for
  focus (programmatic `.focus()` doesn't trigger `:focus-visible`). Each state is
  compared with rest across background, border, text colour, shadow, outline,
  transform, opacity, and decoration. The danger button is tested after its
  confirmation is typed, because until then it is correctly disabled.
- **Human review.** Every 1280×800 capture viewed in contact-sheet montages, and the
  stress catalog viewed scrolled to its end. That review found V-10, V-13, and V-15;
  the probe was then extended so it catches the spill class on its own.

## Findings register

Severity uses the Phase 14 scale (0 none … 4 blocks the task).

| ID | Sev | Where | Finding | Correction | Status |
| --- | --- | --- | --- | --- | --- |
| V-01 | 3 | Install review, evidence and change list | Image, path, and port facts were ellipsized with ordinary content at 1280. The trust surface was hiding the machine facts it exists to show. | Machine facts wrap, never truncate: `overflow-wrap: anywhere` on the evidence grid and seven other fact rules (provenance, facts, path disclosure, activity detail, change list, summary and result facts). | Resolved |
| V-02 | 2 | First Run, starter cards | The version and licence line lost the licence. | Wraps inside the card. | Resolved |
| V-03 | 2 | Overview, Needs attention | Operational messages were cut to one line. | Two lines, then clamp; the full message is the row's `title`. | Resolved |
| V-04 | 2 | My Apps, saved-app list | A long name was ellipsized with no way to read it. | The full name is the `title`; the detail header shows it whole. | Resolved |
| V-05 | 3 | Overview, Needs attention error row | A verbatim Docker daemon error lost its cause. | Same two-line clamp and `title`; the My Apps detail shows the error whole. | Resolved |
| V-06 | 3 | Install evidence | A digest-pinned image lost its digest, which is the part that proves the pin. | Covered by the V-01 wrap rule; asserted with a real `@sha256:` reference. | Resolved |
| V-07 | 1 | Discover cards | The longest real names and categories were ellipsized. | `title` on both; Details shows them whole. | Resolved |
| V-08 | 1 | Discover, verified-recipe cards | Mono version and address lines had `line-height: 1` and lost 2px of glyph. | Compact leading token. | Resolved |
| V-09 | 2 | First Run, starter cards | An unselected, pressable card had no hover state. | Strong border and hover surface. | Resolved |
| V-10 | 2 | Discover, no results | Zero matches still showed "Showing 6 of 1,672 · Page 1" and "Load next 24", and the copy said "restore the catalog fixtures". | Pagination hides at zero; a narrowed list reads "Showing N matching projects"; product copy ("…to browse all 1,672 projects"). | Resolved |
| V-11 | 2 | Rail Docker card, Overview calm state, Settings checks | Status and recovery guidance truncated: "Prior result retained", "No repair started", "Start Docker Desktop, then run the check again." | Guidance wraps. An instruction the user must follow is never truncated. | Resolved |
| V-12 | 2 | Discover, More filters | The popover ended 53px below the window at 1440×900 and 166px below at 1280×800, with the warnings toggle and Clear off screen. | Opening the popover scrolls the workspace only as far as needed (`block: "nearest"`, 24px margin, instant under reduced motion). Focus still lands on the first field. | Resolved |
| V-13 | 3 | Every app logo | `max-height: 100%` resolved against an auto grid track, so any logo taller than wide overflowed its frame and painted over the card copy. 137 of the 1,671 catalog logos (8%) are taller than wide, and Paperless-ngx spilled 9px in the default fixtures. | `.app-icon` gets definite tracks (`grid-template: 100% / 100%`). The probe now reports any logo outside its frame. | Resolved |
| V-14 | 2 | Discover, project drawer | In windows 760px tall or shorter, the drawer's commit ("Review install", "Save linked app") scrolled out of view. It passed at 1280×800; the real-window sweep found it. | The drawer's action row is sticky, with a divider, and is always visible. | Resolved |
| V-15 | 1 | Discover cards, real catalog data | A project with no stated architecture rendered a dangling separator ("WTFPL ·"). | Empty facts are omitted. | Resolved |
| V-16 | 3 | Launcher window vs design range | `src/main.rs` opens the launcher at 1180×760 with a minimum of 800×600, below the 1280×800 design minimum. Realistic maximised windows are smaller still: 1366×688 (1366×768 panel) and 1280×640 (1920×1080 at 150%, Windows' default for that panel). At those sizes, focused-task commits fall below the fold (list below). There is no overflow, truncation, overlap, or unfit dialog at any of the three sizes. | Phase 16 decision. The recommended fix is in the next section. | **Open → Phase 16** |

### Probe corrections

These were probe or harness errors, fixed before any number above was recorded.

- A popover or sticky bar that the page scrolls under was reported as control overlap.
  Only controls in the same layer are compared now.
- Vertical glyph clipping was reported as lost text. It's a separate class now.
- Focus was first tested with `.focus()`, which doesn't trigger `:focus-visible`. It's
  a real Tab walk now.
- The danger button was reported unreachable. It is correctly disabled until the
  confirmation is typed, so the test types the confirmation first.
- Chrome decodes images off-thread. From a cold cache, a late-decoding logo made
  Chrome repaint a strip of the Discover grain surface. The result was stable but
  landed on one of two renders, differing by 1–2 colour levels, so it wasn't
  byte-identical. Software rasterization didn't change this. Each browser context now
  warms the image cache first, and captures wait for every image to decode. With that,
  10 of 10 fresh contexts produced one render. The app loads bundled logos from local
  assets, so the warm render is the representative one.
- The two 1.5× laptop panels shared one pass-count key, so the report under-counted
  renders (417 rather than 430). Counts accumulate now.
- Interaction-state crops were cut at the element box, which hid focus rings drawn
  outside it. Crops now carry a 10px margin at 2× pixels.

## Real window sizes (V-16)

`scripts/check-v2-real-windows.mjs` → `visual-real-windows.json`. These sizes are
outside the approved range, so this is a report, not a gate.

| Window | CSS viewport | Overflow | Unreadable truncation | Overlap | Unfit dialogs | Focused-task commit below the fold |
| --- | --- | --- | --- | --- | --- | --- |
| Launcher default (`inner_size(1180, 760)`) | 1180×760 | 0 | 0 | 0 | 0 | 0 |
| 1366×768 laptop, maximised | 1366×688 | 0 | 0 | 0 | 0 | 3 |
| 1920×1080 laptop at 150%, maximised | 1280×640 @1.5× | 0 | 0 | 0 | 0 | 7 |

Commits below the fold at 1280×640: Install failure "Review new port" (starts at 724px),
Recovery success "Return to Settings" (640px, keep and delete), Recovery mismatch and
scan failure "Scan again" (682px), First Run missing Docker "Check again" (645px). At
1366×688, the two Install failure states (generic and port conflict) and First Run
missing Docker. Each can still be
reached by scrolling, but on a focused task the commit belongs on screen.

**Recommendation for Phase 16:**

1. Open the launcher at 1280×800, clamped to the monitor's work area, and set
   `min_inner_size` to at least 1024×640.
2. Apply the V-14 pattern to focused tasks (Install, Recovery, First Run): below 760px
   of height, the commit row becomes a sticky footer. That covers a maximised
   1920×1080 laptop at Windows' default 150% without redesigning the screens.
3. Add 1280×640 to the verified range once the footer lands.

## Catalog density with real content

- **Descriptions** clamp at two lines on every card, and Details shows the full text.
  With the three longest descriptions in the catalog, rows keep a common height, and
  the card grid holds three columns at 1280 and 1440.
- **Names and categories** ellipsize on one line with a `title`. "Calibre Web Automated
  Book Downloader" is the worst case.
- **Logos.** 1,531 of 1,671 (92%) are near-square (0.8–1.25:1), 23 are 2:1 or wider, and
  8 are 0.6:1 or narrower. After V-13, every aspect ratio sits inside its 44px frame.
  Wordmark-only logos wider than 3:1 (Appsmith, Neon, Martin) stay legible but small at
  44px. The icon pipeline should prefer a square symbol variant where upstream
  publishes one (Phase 16, icon import requirements).
- **Unstated licence.** The catalog says "See project" when no licence is stated. It
  reads as a fact but isn't one. Phase 16's machine-fact formatting rules should give it
  a product phrase such as "Licence not stated".

## Dialogs and panels at 1280×800

Every `role="dialog"` surface, plus the filter popover, fits the window with its commit
visible. That covers the uninstall, delete-data, and remove-link confirmations; the
project and connect drawers; the prototype panel; and More filters (after V-12). The
irreversible actions (Delete permanently, Remove linked app) are never inside a clipped
scroll area at 1280×800 or 1440×900. Recovery's typed confirmation is an in-page card,
not a dialog; its Delete setup and data button sits at 724–766px, on screen at both
sizes. After V-14 the drawers also pass at 1280×640.

## Interaction states

All 57 checks pass: Primary, Ghost, Text, Icon, and Danger buttons, navigation item,
Docker status card, search command, catalog compact action, filter chip, saved-app row,
and starter card, each at rest, hover, active, and keyboard focus; selected
navigation, saved-app row, filter chip, starter card, and detail tab; the disabled
delete confirmation; and busy states on Install, My Apps, and Recovery, each announced
through `aria-busy`, `role="status"`, or a live region. Crops are in the contact sheet
under *Interaction states*.

## Display scaling

At 125%, 150%, and 200%, the probe finds no new overflow, truncation, overlap, obscured
primaries, or unfit dialogs at any panel. At 1.5× the captures render crisp: type,
1px hairlines, mono facts, and SVG logos stay sharp. One review item is outside the
approved range: at 1920×1080 and 150% in full screen (1280×720), the Install
port-conflict commit starts at 724px. It's covered by V-16.

## Evidence and storage

- `visual-manifest.json` has SHA-256 hashes for the 176 normal captures.
- `visual-report.json` has every finding and review item from the last full run.
- `visual-real-windows.json` has the V-16 report.
- `.cache/v2-visual/` holds the full-resolution PNGs. It's gitignored and regenerated
  by the script (about 75 MB).
- `visual-contact-sheet.html` links every capture from the cache, so run the capture
  script first.
- `screenshots/phase15/` holds the curated approval set: 161 JPEGs, 2.4 MB. That's every
  1280×800 state, the stress highlights, the 150% scaling cases, and the
  interaction-state crops.

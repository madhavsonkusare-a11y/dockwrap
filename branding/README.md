# Local Store visual identity

The open **L** gives independent apps a home on the desktop. A single coral tile
is the focal point. The wordmark pairs a heavier “Local” with a quieter “Store”.
This refinement retains the original five-panel Penpot deck's concept.

## Review the system

Open `brand-deck.html` in a browser for the updated five-panel gallery:
foundation, geometry, reduction, color/type, and product/motion.
The product panel uses the actual reviewed Playwright screenshot.

## Canonical files

- `local-store-mark.svg` — flat dark mark for incidental light backgrounds.
- `local-store-mark-reversed.svg` — flat paper/coral mark used by the dark UI.
- `local-store-mark-mono.svg` — one-color reduction.
- `local-store-app-icon.svg` — charcoal/warm gradient source for OS exports.
- `../src/assets/mark.svg` — identical copy of the reversed mark.

The product UI is dark-only. A flat dark mark is retained for external documents
and integrations that supply a light background.

## Geometry

The approved 64 × 64 mark uses an 8-unit L stroke. Its visible bounds are
x = 9–55, y = 9–55. The 32 × 32 tile starts at (23, 9), with radius 8.
The top edges align at y = 9 and both facing gaps are exactly 6 units.

The tile's lower-left arc and the L's facing arc share center (31, 33).
Their radii are 8 and 14: the difference is the 6-unit gap. The L centerline
radius is 18 and its outside radius is 22.

At 512 × 512, apply scale 7 and offset 32. The tile is 224 × 224 at (193, 95),
radius 56. Both top edges align at y = 95; both facing gaps are 42 pixels.
The shared corner center is (249, 263), with facing radii 56 and 98.
Overall margins are 95 pixels on every side. The whole mark is centered;
the tile is positioned by its alignment with the L.

`node scripts/generate-brand.mjs --check` verifies the source assets and the
constant separation across 10,001 quarter-circle samples. Change the generator
to revise geometry; regenerate every variant together.

## Palette

| Token | Value | Use |
| --- | --- | --- |
| Charcoal | `#111214` | Workspace canvas |
| Surface | `#1B1D20` | Cards and panels |
| Graphite | `#232529` | Raised controls |
| Paper | `#F8F7F3` | Primary text and L |
| Signal | `#FF623E` | Flat identity tile |
| Warmth | `#FFC1A9` → `#FF9E7E` | Primary action material |
| Muted | `#A3A5AC` | Supporting text |
| Mint | `#9AD8B6` | Recipe capability and running states |

Keep gradients static and local to surfaces. Use weight, spacing, and contrast
for hierarchy. Preserve app logos' original colors and shapes inside their tiles.

## Type, shape, and motion

Inter Variable with optical sizing; Lucide for interface icons. Use the existing
4/8/12/16/24/32/48 spacing rhythm. Cards use 15-pixel corners, dialogs 20, inputs
10. Smaller controls retain proportionally smaller corners.

Following [Emil Kowalski's skills](https://github.com/emilkowalski/skills):
pointer press feedback at 0.97 scale; occasional dialog transitions at 250 ms
between 0.96 and 1, using `cubic-bezier(.23, 1, .32, 1)`. Interrupt from the current
visual value. Keyboard actions, catalog changes, and reduced-motion preferences
stay immediate. Native dialogs retain focus trapping, Escape, and restoration.
Support reduced transparency and increased contrast.

## Exports and verification

Run `node scripts/generate-brand.mjs`, then regenerate; don't retouch raster exports:

```sh
cargo tauri icon branding/local-store-app-icon.svg
node scripts/render-brand-deck.mjs
```

Only desktop PNG/ICO/ICNS outputs listed in `tauri.conf.json` are shipped.
The CLI may also generate unused mobile/Appx exports; these are not part of
the desktop asset set. Inspect the mark at 16/32/64 pixels after changes.

The original external Penpot export stays unchanged. Its refined successor is
versioned here alongside the production icon sources and interface.

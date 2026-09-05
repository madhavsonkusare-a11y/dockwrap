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

The mark uses a 64 × 64 canvas. The L has an 8-unit stroke, round ends, and a
7-unit inner corner. Its visible bounds are x = 11–53, y = 9–55.

The orange tile is 22 × 22, positioned at (21, 21), with a 5.5-unit corner radius.
Its center is **(32, 32)**, the exact canvas center. Its enlarged area balances
the L's visual weight. Keep the 2-unit side gap and 4-unit shelf gap intact.

The 512 × 512 application icon uses the same mark at 7× scale with a 32-pixel
offset. The orange square is 154 × 154 at (179, 179), centered at **(256, 256)**.
Do not independently move the tile in platform exports.

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
| Mint | `#9AD8B6` | Verified capability and running states |

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

Edit the SVG sources, then regenerate; don't retouch raster exports:

```sh
cargo tauri icon branding/local-store-app-icon.svg
node scripts/render-brand-deck.mjs
```

Only desktop PNG/ICO/ICNS outputs listed in `tauri.conf.json` are shipped.
The CLI may also generate unused mobile/Appx exports; these are not part of
the desktop asset set. Inspect the mark at 16/32/64 pixels after changes.

The original external Penpot export stays unchanged. Its refined successor is
versioned here alongside the production icon sources and interface.

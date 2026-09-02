# Local Store visual identity

## Concept

A bold **L-frame** represents a local desktop home for self-hosted apps; the
single signal-orange tile represents an app selected from the store.

## Canonical files

- `local-store-mark.svg` — transparent product mark for light surfaces.
- `local-store-app-icon.svg` — filled application icon source for platform exports.

## Palette

| Token | Hex | Use |
| --- | --- | --- |
| Ink | `#111827` | App-icon field and primary mark |
| Signal | `#FF623E` | App tile / primary accent |
| Paper | `#F8F7F3` | L-frame on dark icon field |

Do not redraw or raster-edit the source SVGs. Regenerate files in `icons/` from
`local-store-app-icon.svg` with `cargo tauri icon branding/local-store-app-icon.svg`.

## Small-size rule

At 16 px, the L-frame and orange tile must remain visually separate. Keep the
icon source unmasked; Tauri generates platform-specific sizes and container
formats.

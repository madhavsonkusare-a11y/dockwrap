# Local Store V2 production assets

Updated: September 11, 2026  
Status: **approved** by Madhav Sonkusare on September 12, 2026.  
Everything below ships inside the Tauri bundle. The product makes no network request
for fonts, icons, or logos at run time.

## Fonts

Five WOFF2 files replace the current `src/fonts/InterVariable.woff2`. The files, weights,
and SHA-256 hashes are frozen in `freeze.json`, and `scripts/check-v2-freeze.mjs` fails
if any of them changes.

| Family | Weights | Files (`docs/design/v2/assets/fonts/`) | Licence |
| --- | --- | --- | --- |
| Instrument Sans | 400, 500, 600 | `InstrumentSans-Regular.woff2`, `-Medium.woff2`, `-SemiBold.woff2` | SIL OFL 1.1, `InstrumentSans-OFL.txt` |
| IBM Plex Mono | 400, 500 | `IBMPlexMono-Regular.woff2`, `-Medium.woff2` | SIL OFL 1.1, `IBMPlex-OFL.txt` |

- Copy the files and both licence texts into `src/fonts/`, and add both families to
  `THIRD_PARTY_NOTICES.md`.
- Keep the `@font-face` blocks from `tokens.css` unchanged, including
  `font-display: swap`.
- Inter can be retired once no production stylesheet references it. Keep its OFL text
  until then.
- Acceptance: no request for a font file fails. The missing SemiBold face (Phase 15,
  V-17) went unnoticed because the browser synthesizes bold silently, so the
  acceptance suite checks for this explicitly (see `HANDOFF.md`).

## Brand mark

Use `BRAND-MARK.md`'s export map as written: the grain mark at 32px and above, the flat
small marks at 16 and 24px, the light variants on light surfaces, and the baked PNG for
filter-constrained renderers. The files are in `docs/design/v2/assets/brand/`.
Production replaces `src/assets/mark.svg` and keeps the existing mono marks for
restricted reproduction.

## System icons

The prototype's 15 system icons are an inline sprite in `index.html`: overview,
discover, grid, activity, settings, search, sliders, chevron, close, arrow, plus, check,
alert, external, and docker. They are original drawings on a 24px grid with a 1.7px
round stroke (`components.css` `svg` rule), so no third-party licence applies. Ship the
sprite as the V2 system set. Where production needs an icon the sprite lacks, draw it on
the same grid, or use the already-licensed Lucide set (`src/assets/LUCIDE-LICENSE`)
drawn at the same stroke weight.

## App logos

| Requirement | Detail |
| --- | --- |
| Coverage | Every catalog entry has a logo. The complete, validated set is committed on `feat/catalog-icons` (ba16e0e and 33ddb0f: 1,673 of 1,673). Upstream artwork comes first (Homarr dashboard icons, Coolify, and the Umbrel gallery); a generated Local Store monogram covers the rest. Land that branch before, or together with, V2 |
| Pipeline | `ICON-PIPELINE.md`: `scripts/cache-catalog-icons.py` and `scripts/catalog_pipeline.py`. A refresh needs the network; every check and render is offline |
| Manifest | `catalog/icons.json` records, for each logo, its source, repository, revision, licence, notice, and SHA-256. Attribution follows it and `THIRD_PARTY_NOTICES.md` |
| Rendering | A square frame with definite grid tracks (`grid-template: 100% / 100%`), `object-fit: contain`. Never cropped, recoloured, masked, or stretched (V-13) |
| Missing file | If a logo fails to load, the card stays usable and shows the monogram. Production already tests this ("image failure leaves a usable card") |
| Wordmark logos | Six logos are at least 3:1 wide, including Appsmith, Neon, and Martin. They stay legible but small at 44px. Where upstream publishes a square symbol, the pipeline should prefer it, recorded as a reviewed alias |
| Prototype copies | `docs/design/v2/assets/logos/` holds the 21 logos the prototype shows, SHA-verified against the manifest (`scripts/vendor-v2-prototype-logos.py`). These are design fixtures, not production assets |

## Grain and ember material

The grain is an inline SVG `feTurbulence` data URI in `prototype.css`
(`.grain-surface::after`) and the selected starter card. It needs no files. Rules:
one signature region per viewport, and never behind body text, forms, logs, tables, or
machine facts (`VISUAL-SYSTEM.md`). Under reduced motion it is static, as it already is.

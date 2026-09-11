# V2 brand mark treatment

Status: approved for prototype use  
Scope: Phase 4 visual direction; production assets in `src/` remain unchanged

## Decision

Use the **satin-ember** treatment for the Local Store mark at 32 px and larger. It keeps the approved 64-unit geometry intact, builds material from separate tone, micro-grain, bloom, and rim layers, and leaves the cream path completely outside the filter stack.

Use the **flat optical-size variant** at 16 and 24 px. At those sizes the grain resolves as unstable subpixel contrast rather than material, so removing it produces the more faithful mark.

The former Ember Paper experiment has been removed. Do not use, export, reference, or reintroduce it in product UI, campaign art, documentation, or generated assets. Satin Ember is the only approved textured treatment.

## Export map

| Context | Asset | Rule |
|---|---|---|
| Dark, base, raised, or deep-ember surface; 32 px+ | `assets/brand/mark-grain-approved.svg` | Default V2 product mark |
| Light surface; 32 px+ | `assets/brand/mark-grain-approved-light.svg` | Near-black path; identical tile treatment |
| Dark surface; 16 or 24 px | `assets/brand/mark-small.svg` | Flat optical-size export |
| Light surface; 16 or 24 px | `assets/brand/mark-small-light.svg` | Flat optical-size export with near-black path |
| High-density or filter-constrained renderer | `assets/brand/mark-grain-baked.png` | Transparent 512 px render bake; use at 32 px+ |
| One-ink, print, engraving, or restricted reproduction | Existing flat/mono brand assets | Grain is prohibited |

The retained comparison source assets are `mark-flat.svg` and `mark-grain-subtle.svg`. The approved file intentionally duplicates the subtle treatment so downstream code can target a stable semantic filename while the study remains inspectable.

## Geometry and visual constraints

- Preserve the existing `0 0 64 64` view box and all path/rectangle coordinates.
- The tile remains `x=23`, `y=9`, `width=32`, `height=32`, `rx=8`.
- The path remains an 8-unit round stroke. Do not apply grain, blur, glow, opacity, or filters to it.
- Keep the tile in front of the path at the overlap; this is part of the approved silhouette.
- Do not place the cream-path export on a light surface. Use the light-surface asset instead.
- Do not add an outer card or forced container to the standalone mark.

## Reference study

The revised treatment draws on four implementation patterns found in open-source GitHub work:

- [sohumsuthar/liquid-glass](https://github.com/sohumsuthar/liquid-glass) separates tint, static grain, and directional edge light instead of making one filter simulate the whole material.
- [alexmwalker’s organic SVG texture study](https://gist.github.com/alexmwalker/b404eaa69c458076b7662ddb11fc244f) clips texture to the source shape and uses tonal blend modes so grain follows the material rather than sitting above it as dots.
- [mouse-lin/finesse-skill](https://github.com/mouse-lin/finesse-skill/blob/main/skills/finesse-ui/SKILL.md) treats premium grain as a low-strength fixed substrate rather than a visible decorative motif.
- [MelodicBloom/svg-filter-lab](https://github.com/MelodicBloom/svg-filter-lab/blob/main/docs/how-to-implement-performant-svg-filters-without-killing-your-frame-rate.md) recommends tightly scoped filter regions, small graphs, and no more than three octaves for ordinary production use.

These are design and rendering references, not copied assets. The Local Store filter graph and color treatment are original and retain the project’s existing geometry and palette.

## Grain recipe

The primary asset uses a fixed `seed="23"`, `baseFrequency="1.05"`, two octaves, and `stitchTiles="stitch"`. The noise is desaturated, tonally compressed with `feComponentTransfer`, and blended using `soft-light` inside a tightly bounded 33-unit filter region. A separate warm bloom gives the tile a light source, while a one-unit gradient rim supplies the fine top highlight and bottom falloff. There is no animation or external resource.

The baked fallback is a 512 × 512 transparent PNG captured directly from the approved Edge render by `bake-brand-mark.mjs`. It has no runtime SVG filter, script, embedded font, external request, repeated dot pattern, or visible tiling. Regenerate it whenever the approved SVG changes; do not hand-edit it.

## Offline and determinism verification

The mark lab is `brand-mark-lab.html`. Verification is performed against a localhost-only server with all external requests rejected, using Microsoft Edge as the closest available match to the Windows Tauri WebView2 rendering engine.

Acceptance checks:

1. Every SVG loads from the repository with no network dependency.
2. Two clean screenshots of the same loaded page are pixel-identical.
3. The `seed`, view box, gradient coordinates, filter bounds, and optical-size switch are fixed in source.
4. The cream path remains filter-free in every export.
5. The 16, 24, 32, 48, 64, and 128 px ladder remains recognizable with no clipped stroke or tile.

Record the exact verification command, browser version, screenshot hashes, request log, and timing results in this document after each material change to the assets.

### Verification record — 2026-09-09

Run with `node docs/design/v2/verify-brand-mark.mjs` against the repository served only on `127.0.0.1`.

| Check | Result |
|---|---|
| Rendering engine | Microsoft Edge 152.0.4191.66; the installed Microsoft Edge WebView2 Runtime reports the identical 152.0.4191.66 build |
| Offline boundary | Passed; every non-`127.0.0.1` request was blocked and the external-request log stayed empty |
| Asset loading | Passed; all 17 page images loaded at non-zero intrinsic dimensions |
| Determinism | Passed; two clean full-page renders were byte-identical |
| Render SHA-256 | `9a54951fc0c37f16b73584ce4251808d92da86eb8f3d88af1c2158f1ab945a02` for both captures |
| Layout | Passed at 1440 × 900; no horizontal overflow |
| Lab full-page screenshot | 497.3 ms |
| 256-mark stress load | 36.3 ms |
| 256-mark stress screenshot | 247.7 ms |
| Baked PNG reproducibility | Passed; two independent bake runs produced SHA-256 `97275e90726f37edb4a48448d46f9b3e8a24d9833276630285e00fa9e508581d` |
| Console/page/request failures | None |

The original engine render exposed invalid filter-coordinate assumptions: user-space bounds had been supplied without `filterUnits="userSpaceOnUse"`. The revised assets retain that explicit coordinate system and composite the filtered output back into the source tile, preventing noise leakage outside the rounded square.

## Performance policy

The procedural filter is acceptable for a single brand mark in navigation, onboarding, and branded empty states. It should not be repeated as decoration or used as a per-row catalog icon. In a dense or repeated context, use `mark-grain-baked.png` or the flat small mark.

Performance acceptance is based on a 256-mark stress page rendered through the Edge channel. Record cold load-to-idle, screenshot time, and any failed requests. The stress test is deliberately far above normal product usage; it is a regression tripwire, not a target layout.

## Design review

| Before | After | Why |
|---|---|---|
| Single high-alpha noise blend | Separate ember tone, compressed micro-grain, bloom, and optical rim | Creates directional material depth without making noise the subject |
| Reversed mark used on all surfaces | Cream and near-black path roles | Maintains contrast without adding a container |
| Same rendering at every size | Flat 16/24 px optical export | Prevents muddy subpixel grain |
| Visible vector-dot fallback | Transparent 512 px render bake | Preserves the approved appearance without a repeated pattern or runtime filter |

## Production handoff note

Phase 4 is a visual-design artifact only. When the V2 implementation phase begins, copy the approved exports into the production asset pipeline, use the 32 px size switch in the logo component, retain the existing mono marks for restricted reproduction, and rerun the deterministic and performance checks inside the packaged Tauri build.

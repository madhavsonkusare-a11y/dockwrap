# Phase 3 visual-system audit

Updated: September 9, 2026  
Targets: `tokens.css`, `specimen.css`, and `specimen.html`

## Anti-generic/fingerprint audit

This audit applies the evaluation toolkit's anti-generic prompt and the V2
product model. Familiar desktop structure is retained where it improves trust;
identity comes from typography, color discipline, icon treatment, factual mono
type, and the later grain signature.

| Before | After | Why |
| --- | --- | --- |
| Inter is used for product prose and machine facts alike | Instrument Sans for product voice; IBM Plex Mono only for asserted machine values | Creates a meaningful typographic distinction instead of changing fonts decoratively |
| Eleven radii from 4px through 23px | Four structural roles: 8px, 12px, 20px, and pill | Radius now communicates component type |
| Similar diagonal gradients and inset highlights across cards, dialogs, rows, and avatars | Flat warm surfaces separated by lightness, border, and limited elevation | Removes the “same object everywhere” fingerprint |
| Pale peach is the main action color | Saturated ember with dark foreground and explicit hover/pressed values | Restores a confident, accessible product signal |
| Editorial shelf consumes the first viewport | Browse density and operational content receive priority; signature art is bounded | The application reads as a useful self-hosting workspace, not a consumer storefront |
| App logos are repeatedly framed in colorful rounded tiles | Original logo silhouette first; neutral backing only when contrast requires it | Preserves recognition and reduces decorative container soup |
| Status presentation tends toward a colored dot beside every label | State words and restrained soft surfaces; color is secondary | Avoids dashboard cliché and improves clarity |
| Three-column feature-card patterns are used as default composition | Grids are reserved for scan sets; tasks use asymmetric fact/action layouts | Prevents every page becoming a generic bento board |
| Uppercase micro-labels carry substantial navigation and content meaning | Uppercase is restricted to short section kickers at 12px; primary labels remain sentence case | Retains rhythm without sacrificing legibility |
| Grain and glow can become an all-purpose dark-mode effect | One bounded signature region per viewport, prohibited on operational surfaces | Makes the effect identifiable because it is rare |
| Generic success dashboards reward “all good” with maximum decoration | Healthy state is visually quiet; attention receives structure and a next action | Optimizes for the question users actually have |

### Accepted familiar patterns

- A left navigation rail is retained because it is category-appropriate for a
  desktop operational workspace.
- Cards remain in Discover because independent catalog projects must be scanned
  and compared.
- Install and destructive confirmation stay visually conventional because trust
  is more important than novelty at the point of commitment.
- Pills are retained for actions and state chips, but not used on containers,
  inputs, or every label.

## Token consistency audit

Automated literal scans were run after the rendered review.

| Check | Result | Interpretation |
| --- | --- | --- |
| Raw hex/RGB colors in `specimen.css` and `specimen.html` | 0 | Components consume semantic color tokens only |
| Numeric `border-radius` values outside `tokens.css` | 0 | All radii use the four geometry tokens |
| Numeric `font-size` values outside `tokens.css` | 0 | All type uses the canonical scale |
| Nonzero raw `padding`, `margin`, or `gap` values | 0 | Reported numeric values were only zero resets/axis declarations |
| Remote font or stylesheet dependencies | 0 | The specimen is fully offline |
| Webfont file signatures | 5 valid `wOF2` files | Downloaded files are valid WOFF2 containers |

Primitive literals are intentionally allowed in `tokens.css`, where they are
named and documented. Inline specimen swatches set only a semantic custom
property value such as `var(--v2-action-primary)`; no new literal enters the
component layer.

## Rendered review

The full specimen was rendered in Chrome at 1440×900 and 1280×800. Both widths
preserve the rail, type hierarchy, component layouts, three-card scan set, and
trust-panel structure without horizontal clipping.

Two issues found in the first pass were corrected:

| Before | After | Why |
| --- | --- | --- |
| 12px rail/caption text used `--v2-text-quiet` at 3.72:1 | Readable captions use `--v2-text-tertiary` at 5.98:1 | Small text must meet AA; quiet is now reserved for disabled/non-text use |
| Cream labels sat on the bright ember and green swatches | Bright swatches use the dark on-accent foreground | Light text lacked sufficient contrast on those demonstration surfaces |

## Typeface assets

| File | SHA-256 |
| --- | --- |
| `InstrumentSans-Regular.woff2` | `f28af62faa9eec1e5482cf6e2a3e06bc865fa2fa6937bd56c14d2be23c9b4c46` |
| `InstrumentSans-Medium.woff2` | `65d25fc111c40fd6b481d9db860af365730752228dfd8a246e79648d8a01f02a` |
| `InstrumentSans-SemiBold.woff2` | `04235f235483213906d69cafb7a87ee197adaffacb081c8a7f809d00f9b353cd` |
| `IBMPlexMono-Regular.woff2` | `ba204497f16b6d334cee9d1e963a831b73e3a56e1d6300a8489d18df7214b350` |
| `IBMPlexMono-Medium.woff2` | `33faf307fa6031fb4062276d7320a6d632de890cbb347576fd80cfa01077bc25` |

Both families are accompanied by their upstream SIL Open Font License text.

## Phase 3 result

The visual system is approved as the baseline for prototype construction. Phase
4 may refine the brand mark and grain implementation without changing the
semantic color, type, spacing, radius, or surface hierarchy established here.


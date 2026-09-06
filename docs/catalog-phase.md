# Catalog and approved identity phase

Implemented September 6, 2026, following the September 5 audit of Hermes's plan.
This phase completes catalog breadth, visual discovery and the approved identity;
runtime and distribution gates remain in [the task ledger](upgrade-status.md).

| Before | After | Why |
| --- | --- | --- |
| Earlier centered-tile logo revision | Approved top-aligned tile, equal 42 px gaps, concentric facing corners | Preserve the approved proportions in every vector and desktop export |
| 1,259 discovery rows assembled from a legacy snapshot and recipes | 1,672 deduplicated projects with frozen IDs and pinned provenance | Reuse four existing catalogs and make updates reviewable |
| Five bundled identities, remote icon/favicons elsewhere | 508 validated catalog SVGs plus the featured identities, local fallback | Reliable offline browsing and consistent artwork framing |
| Long category dropdown | Searchable category dialog, combined capability/license/architecture filters, counts and interest collections | Make the larger catalog easy to browse |
| Introductory sections crowded out app results | Compact featured shelf, connection guidance after the catalog, short-window layout | Put project cards into view sooner |
| Recipe badges said Verified | Three explicit install previews | Configuration checks do not prove a full real-container lifecycle |
| Doctor accessible during installation | Settings with standalone Docker diagnostics | Make setup problems discoverable before attempting installation |
| Shortcuts depended on a CLI path that refused native open | Protocol shortcuts, native CLI dispatch and stable window IDs | Restore the intended native-window path |

Reused Tauri, serde, Inter, Lucide, native dialogs and the Web Animations API.
The Emil Kowalski and Apple design skills guide spacing, restrained warm
materials and interruptible pointer transitions. Search/filter/navigation and
keyboard/reduced-motion interactions remain immediate. No new UI framework.

The importer uses pinned PyYAML for existing upstream structures. Build-time
artwork caching deliberately replaces runtime retrieval, preserving offline
coverage and avoiding another production network subsystem.

## Validation boundaries

Local verification: 72 Rust tests, six importer/input-security tests, 19
Playwright tests with axe and ten visual baselines; formatting and clippy with
warnings denied; deterministic catalog/icon/brand checks; all three Compose
configuration checks. The five brand-deck panels were rendered and the geometry
and 16/32/64 px reductions inspected.
The Windows release binary builds with `cargo build --release --locked`.

Rust tests cover catalog paging, combined filters, provenance, malformed data,
native CLI dispatch and shortcut escaping alongside existing storage/runtime
tests. Importer tests cover stable renames, deduplication, checksums and unsafe
YAML/SVG inputs. Browser tests use the real generated catalog and a controlled
Tauri adapter, with axe, Tab/Shift-Tab, failure paths and visual baselines.

The recipe checks validate Compose configuration only. Native shortcut activation,
clean installers and the full container lifecycle are still separate gates.
GitHub's quality and OS build results are recorded on draft PR #2; no release
tag or recipe verification promotion is part of this phase.

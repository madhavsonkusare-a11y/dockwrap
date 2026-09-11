# V2 catalog icon pipeline

Status: Phase 5 complete. The generated manifest resolves all 1,672 catalog
entries to a local, validated icon without runtime network access.

## Coverage and source order

The pre-V2 manifest contained 752 cached upstream icons and 920 entries without
an asset. The V2 manifest in `catalog/icons.json` now records 100% coverage:

| Source | Resolved | Selection rule |
| --- | ---: | --- |
| Homarr dashboard icons | 669 | Exact/normalized catalog ID or reviewed alias/path |
| Coolify catalog | 81 | URL attached to the same pinned upstream record |
| Umbrel Apps Gallery | 8 | Exact catalog ID or reviewed alias |
| Local Store Satin monogram | 914 | Deterministic fallback after every upstream candidate fails |

Selection is deterministic. It uses catalog IDs first and only the explicit
maps in `catalog/icon-aliases.json` and `catalog/umbrel-icon-aliases.json` for
non-exact matches. There is no fuzzy name matching. `catalog/icons.json` lists
every generated fallback and its reason, so the remaining upstream gaps are
reviewable rather than implicit.

## Official Umbrel audit

Umbrel application packages live in `getumbrel/umbrel-apps`; the gallery artwork
is maintained separately in `getumbrel/umbrel-apps-gallery`. The importer pins
the gallery to revision
`7655ed83a9b26c50dc5f16888591da9d46d294a6`. At that revision the gallery tree
contains 398 app icon paths. It has 185 exact IDs in common with this catalog,
but most already had a preferred pinned source. Seventeen exact matches could
fill an original coverage gap.

Eight clean assets passed the validator: `chatbot-ui`, `databag`, `electrs`,
`fx`, `jupyterlab`, `kan`, `matter-server`, and `strix`. Nine SVGs were refused
because they contain embedded or external content: `blinko`, `domain-locker`,
`leafwiki`, `localai`, `openhands`, `openreader`, `pastefy`, `transmute`, and
`trip`. Those nine receive generated monograms.

The Umbrel gallery repository declares no repository-wide license or artwork
license at the pinned revision. Its manifest records `NOASSERTION`, and
`catalog/notices/umbrel-apps-gallery/NOTICE.md` is a release gate: commercial or
broad redistribution requires permission or an asset-by-asset license review.
Names and marks remain the property of their owners; inclusion does not imply
endorsement.

## Validation and rendering contract

`scripts/cache-catalog-icons.py` delegates its CLI to
`scripts/v2_catalog_icons.py`. A refresh inventories only pinned Git trees,
downloads from approved GitHub hosts, and enforces:

- a 512 KiB response/file ceiling, allowed MIME types, and matching `.svg` or
  `.png` extensions;
- static PNG signatures and dimensions no larger than 2,048 px;
- parseable SVG dimensions no larger than 4,096 units and at least one visible
  graphic element;
- rejection of doctypes, entities, scripts, event handlers, external URLs,
  embedded images, iframes, foreign objects, and active animation elements;
- rejection of artwork more extreme than a 5:1 or 1:5 aspect ratio for a compact
  app tile;
- SHA-256 verification and exact repository, revision, license, notice, MIME,
  byte-size, and dimension metadata during the offline check.

Original aspect ratios are preserved. Product surfaces must render icon images
with `object-fit: contain`; destructive clipping, forced square distortion, and
logo recoloring are prohibited. Vector artwork is preferred when it passes the
validator. Generated monograms are path-only SVGs with no runtime font or
external dependency, a restrained dark field, and the approved Satin Ember
accent. The removed Ember Paper experiment must not be referenced or restored.

## Audit disposition

`catalog/icon-audit.json` is regenerated from the manifest. The current audit
reports no missing, extra, suspicious, blank, corrupt, or incorrectly shaped
assets. Nine duplicate upstream hashes are intentional:

| IDs | Disposition |
| --- | --- |
| `adguard`, `adguard-home` | Same application alias |
| `booklore`, `grimmory` | Grimmory is a community fork retaining BookLore identity artwork |
| `firefly`, `firefly-iii` | Same application alias |
| `flowise-ai`, `flowise-with-databases` | Same application, different package |
| `forgejo`, `forgejo-with-mariadb` | Same application, different package |
| `freshrss`, `freshrss-with-mariadb` | Same application, different package |
| `gitea`, `gitea-with-mariadb` | Same application, different package |
| `mixpost`, `mixpost-pro` | Shared product identity artwork |
| `nextcloud`, `nextcloud-with-mariadb` | Same application, different package |

The full human-review surface is `docs/design/v2/icon-contact-sheet.html`. It
groups all 1,672 assets by source, shows dimensions and IDs, and uses uncropped
containment at laptop density.

## Reproduction and implementation handoff

Run from the repository root after installing `scripts/requirements-catalog.txt`:

```text
python scripts/cache-catalog-icons.py --refresh
python scripts/cache-catalog-icons.py --check
python scripts/catalog_pipeline.py
python scripts/catalog_pipeline.py --check
python scripts/render-icon-contact-sheet.py
```

Refresh requires network access. All check and product-rendering paths are
offline. Any source revision update must be deliberate: update
`catalog/icon-sources.lock.json`, regenerate the assets/manifest/audit/contact
sheet, review attribution, inspect the duplicate and suspicious lists, and run
the full checks before committing.

References: [official Umbrel app packages](https://github.com/getumbrel/umbrel-apps),
[official Umbrel gallery](https://github.com/getumbrel/umbrel-apps-gallery), and
[GitHub's repository licensing guidance](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository).

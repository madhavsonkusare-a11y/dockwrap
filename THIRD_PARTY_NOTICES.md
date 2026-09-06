# Third-party material

Local Store's original launcher code is MIT licensed. Imported catalog text,
artwork, fonts and interface icons retain their own licenses and copyright
notices; the root MIT license does not replace those terms.

## Catalog material

Local Store adapts and combines project descriptions and metadata, normalizes
categories and identities, and adds local capability information. Source files,
source revisions and transformation scripts are included in this repository.
Every generated project includes original listing URLs and upstream revisions.

| Attribution | License and preserved notices |
| --- | --- |
| awesome-selfhosted contributors, [structured data repository](https://github.com/awesome-selfhosted/awesome-selfhosted-data) | [CC-BY-SA-3.0](catalog/notices/awesome-selfhosted/LICENSE), [authors](catalog/notices/awesome-selfhosted/AUTHORS) |
| Runtipi app-store contributors, [repository](https://github.com/runtipi/runtipi-appstore) | [GPL-3.0](catalog/notices/runtipi/LICENSE) |
| IceWhale Technology and CasaOS/ZimaOS app-store contributors, [repository](https://github.com/IceWhaleTech/CasaOS-AppStore) | [Apache-2.0](catalog/notices/casaos/LICENSE) |
| Coolify contributors, [repository](https://github.com/coollabsio/coolify) | [Apache-2.0](catalog/notices/coolify/LICENSE) |

Adapted awesome-selfhosted material remains available under CC-BY-SA-3.0;
adapted Runtipi material remains available under GPL-3.0. Consult each preserved
license for its terms. `catalog/sources.lock.json` records the exact revisions.
`catalog/legacy.json` retains the previous awesome-selfhosted-derived snapshot
for compatibility and attribution. App software-license labels describe the
upstream app, independently of these data licenses.

## Visual assets

- App artwork: [Homarr dashboard-icons contributors](https://github.com/homarr-labs/dashboard-icons),
  [Apache-2.0](src/assets/apps/LICENSE). Original artwork is redistributed without
  visual modification; the UI supplies an outer frame. The catalog icon manifest
  records source URLs, immutable revision and checksums. Existing featured app
  identities use the same upstream icon collection. Project names and logos
  remain the property of their respective owners; inclusion implies no endorsement.
- Interface icons: Lucide Icons and Contributors, [ISC license](src/assets/LUCIDE-LICENSE).
- Inter font: The Inter Project Authors, [SIL Open Font License 1.1](src/fonts/OFL.txt),
  [upstream source](https://github.com/rsms/inter).

Installer resources include this document and the referenced license texts.
Rust/Node dependency manifests and lockfiles provide the dependency inventory.
Automated dependency-license reporting and release SBOMs remain planned work.

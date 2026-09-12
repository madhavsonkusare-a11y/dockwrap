# V2 product and backend capability matrix

Updated: September 8, 2026  
Scope: evidence for the V2 visual prototype. This document describes what the
application can truthfully show now, what can be derived without a new product
capability, and what is an intentionally speculative design concept.

## Status vocabulary

| Status | Meaning in the V2 prototype |
| --- | --- |
| `available` | A current Tauri command, event, model, or bundled dataset returns the value directly. |
| `derivable` | Existing data can produce the value deterministically, but a typed projection or small adapter may still be needed. |
| `concept` | The current product does not collect or persist the information. Fixture data must be visibly labeled in the design lab. |

The prototype may show all three classes. Production UI must not present a
`concept` value as real until the corresponding backend contract exists.

## Backend surfaces reviewed

- `src/commands.rs` — launcher IPC commands and serialized views
- `src/model.rs` — installed-app and runtime identity
- `src/runtime/mod.rs` — Doctor, status, readiness, lifecycle, install, and rollback
- `src/operations.rs` — live operation events
- `src/recovery.rs` — interrupted-install inventory and safe discard behavior
- `src/catalog_index.rs` and `src/catalog_schema.rs` — discovery data and filters
- `src/recipes/*.json` and `src/recipes/mod.rs` — reviewed install recipes
- `src/setup/review.rs` — safe setup-field projection
- `src/js/*.js` — what the shipped launcher currently exposes

## Inventory snapshot

| Measure | Current value |
| --- | ---: |
| Catalog projects | 1,672 |
| Bundled project icons | 752 |
| Projects missing a dedicated icon | 920 |
| Projects marked as having a web UI | 529 |
| Reviewed install recipes | 3 |
| Catalog categories | 96 |
| Distinct license values | 63 |
| Distinct architecture values | 6 |
| Projects with an upstream update date | 1,142 |
| Projects carrying a warning | 8 |
| Pinned catalog sources | awesome-selfhosted, CasaOS, Coolify, Runtipi, legacy snapshot |

## Installed applications

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Stable app ID | `InstalledApp.id` | Internal only | Stable row/detail selection and operation correlation | `available` |
| Catalog association | `InstalledApp.catalog_id` | Not shown | Link an installed app back to its catalog record and artwork | `available` |
| Display name | `InstalledApp.display_name` | Row title | Row and detail identity | `available` |
| Launch address | `InstalledApp.launch_url` | Row text | Machine-fact line in mono type | `available` |
| Stored icon path | `InstalledApp.icon_path` | Avatar when present | App identity | `available` |
| Managed versus external runtime | `RuntimeSpec::Compose` / `External` | Small type label | Primary object-type distinction | `available` |
| Compose project directory and file | `RuntimeSpec::Compose` | Not shown | Advanced detail and recovery context | `available` |
| Creation time | `created_at_unix` | Not shown | Detail metadata or recent-addition context | `available` |
| Last record update time | `updated_at_unix` | Not shown | Detail metadata; must not be called runtime activity | `available` |
| Installed app count | Length of `list_apps` | Sidebar badge | Overview and My Apps summary | `derivable` |
| Counts by runtime/status | `list_apps` result | Not summarized | Overview health strip | `derivable` |
| User-defined sorting/pinning | No model field | Absent | Do not imply in V2 unless explicitly conceptual | `concept` |

`updated_at_unix` is registry metadata, not evidence that the container or app
was recently used. V2 copy must not label it “last active.”

## Runtime status and readiness

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Connected, running, stopped, error | `AppStatus` | Status on My Apps rows | Row, detail, and Overview health | `available` |
| Ready, unreachable, unknown | `Readiness` | Overrides row label for ready/unreachable | Separate reachability signal | `available` |
| Status diagnostic | `AppView.status_error` | Inline error | Needs-attention detail | `available` |
| Bounded readiness polling | `createReadinessMonitor` | Visible My Apps only | Preserve this low-cost behavior | `available` |
| Address check before saving | `check_address` | Connect dialog feedback | Inline advisory feedback | `available` |
| Container uptime | Not queried | Absent | Overview/detail concept | `concept` |
| Restart count and exit code | Not queried or modeled | Absent | Detail/Activity concept | `concept` |
| CPU and memory | Not queried | Absent | Overview/Activity concept | `concept` |
| Per-app and total disk usage | Not queried | Absent | Overview/Activity concept | `concept` |
| Historical availability | No persisted samples | Absent | Activity concept | `concept` |

Container status and HTTP readiness are different signals. A running container
may still be starting or unreachable; an external connection has no locally
managed container state. V2 must keep those meanings distinct.

## Docker Doctor

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Overall ready state | `DoctorReport.ready` | Settings and install review | Overview, First Run, Install | `available` |
| Docker Engine version/detail | Doctor check `docker` | Settings and install review | Compact environment status | `available` |
| Docker Compose version/detail | Doctor check `compose` | Settings and install review | Compact environment status | `available` |
| Typed process failure detail | `DoctorCheck.error` | Reduced to text | Expandable diagnostic detail | `available` |
| Disk-space preflight | No Doctor check | Mockup only | Must remain conceptual | `concept` |
| Port-availability preflight result | Checked inside install transaction | Failure only | A pre-install “free” check needs a new typed preview contract | `concept` |
| Image cache/download size | Not queried | Mockup only | Concept until image inspection/pull progress exists | `concept` |

Doctor runs exactly two checks. The visual design must not present a longer
checklist as current functionality.

## Install recipes and review

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Name, description, category, license | `Recipe` | Name plus selected risk copy | Install identity and context | `available` |
| Pinned image and version | `Recipe.image`, `version` | Summary | Prominent machine fact | `available` |
| Source and documentation links | `source_url`, `documentation_url` | Not shown in review | Evidence and help links | `available` |
| Verification date | `verified_at` | Not shown | Recipe evidence | `available` |
| Launch and health addresses | `launch_url`, `health_url` | Launch address only | Port and health review | `available` |
| Host and container ports | `host_port`, `container_port` | Editable host port | Explicit port mapping | `available` |
| Editable published port | `install_app(host_port)` and `Recipe::with_host_port` | Implemented | Preserve in V2 | `available` |
| Data directories and storage description | `data_directories`, `data_storage` | Storage description | Consequence disclosure | `available` |
| Risk notes | `risk_notes` | Bulleted list | Consequence disclosure | `available` |
| Platform requirements | `requirements.container_platforms` | Not shown | Advanced compatibility detail | `available` |
| Image audit URL/date/digest | `requirements.image_audit` | Not shown | Expandable evidence/provenance | `available` |
| Setup fields | `SetupReview.fields` | Generic controls supported | Install configuration | `available` |
| Generated credential count | `SetupReview.generated_credential_count` | Not emphasized | Honest secret-generation notice | `available` |
| Service count | `SetupReview.service_count` | Not shown | “What will run” summary | `available` |
| Restart policy | Present inside current Compose text | Not shown | Needs a typed backend projection before production | `derivable` |
| Container name | Present in Compose and risk notes | Risk note only | Consequence summary | `derivable` |
| Download size and rate | Not available | Absent | Progress concept | `concept` |
| Percent download progress | Not available | Absent | Do not show as real | `concept` |

The backend serializes more recipe material than the current review uses. V2
should show the useful trust evidence while keeping the raw Compose document
behind an optional advanced disclosure.

## Operations and lifecycle

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Open managed or linked app | `open_app` | Row action | Primary My Apps action | `available` |
| Start and stop managed app | `start_app`, `stop_app` | Row actions | Row and detail actions | `available` |
| Read recent Compose logs | `app_logs` | Modal | Detail subview | `available` |
| Create shortcut | Existing invoke path | Linked-app row action | Secondary detail action | `available` |
| Remove external connection | `remove_app_cmd` | Confirmation | Management action | `available` |
| Uninstall managed app | `uninstall_app` | Confirmation | Management action | `available` |
| Preserve data on uninstall | `delete_data: false` | Default | Safe default | `available` |
| Delete managed data | `delete_data: true` | Extra native confirmation | Explicit destructive branch | `available` |
| Live operation ID, app ID, kind, state | `OperationEvent` | Busy reconciliation | Detail status and ephemeral feedback | `available` |
| Install stage | `OperationEvent.stage` | Install progress label | Full staged progress | `available` |
| Install cancellation token | `cancel_id` | Stop control | Truthful cancel affordance | `available` |
| Durable activity history | Events are not persisted | Absent | Required for Activity | `concept` |
| Operation duration | No start/end timestamps in event | Absent | Activity concept | `concept` |

The seven install stages are `checking_system`, `preparing_files`,
`validating_recipe`, `starting_containers`, `waiting_for_health`, `saving_app`,
and `rolling_back`. The shipped UI currently describes six happy-path stages;
V2 must also design rollback as an explicit state.

## Recovery

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Read-only candidate scan | `inspect_recovery` | Settings scan | Overview attention and Recovery detail | `available` |
| Recipe name and retained Compose path | `RecoveryCandidate` | Shown in Settings | Recovery identity and advanced details | `available` |
| Project name | `RecoveryCandidate.project_name` | Not shown | Advanced evidence | `available` |
| Ownership state | `not_checked`, `no_containers`, `verified`, `mismatch` | Explanatory text | Clear safety state | `available` |
| Ownership verified boolean | `docker_ownership_verified` | Implied | Avoid duplicating enum in visual design | `available` |
| Safe discard implementation | `recovery::discard` exists in Rust/CLI | Launcher has no mutation command | UI action requires new launcher command | `derivable` |
| Adopt interrupted install | Explicitly unsupported | Absent | Do not offer | `concept` |

Recovery should never imply that a scan proves permission to delete. The
mutation rechecks ownership under a lock; a future UI command must retain that
behavior rather than acting on the displayed snapshot.

## Catalog and discovery

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Stable ID, aliases, name | `Project` | Name | Search and identity | `available` |
| Description, category, tags | `Project` | Card/detail subset | Browse and filtering | `available` |
| Licenses | `Project.licenses` and joined `license` | Detail/filter | Card fact and filter | `available` |
| Platforms and architectures | `Project` | Architecture filter/detail | Compatibility | `available` |
| Source and website URLs | `Project` | Detail links | Provenance and project action | `available` |
| Updated date | `updated_at` | Detail | Freshness, where known | `available` |
| Warning, maintenance, archived | `Project` | Warning and snapshot wording | Caution state | `available` |
| Web UI | `web_ui` | Converted to capability | Connect eligibility | `available` |
| Local icon path | `icon` | Avatar when present | Required app identity | `available` for 752 |
| Multiple pinned sources | `sources[]` | Detail disclosure | Trust/provenance | `available` |
| Capability | `preview_install`, `connect`, `discover` | Card labels/tabs | Primary browse distinction | `derivable` |
| Search and bounded paging | `search_catalog` | Implemented | Preserve | `available` |
| Category facets | `category_counts` | Filter dialog | Filter counts | `available` |
| License, architecture, collection, warning filters | `Filters` | Implemented | Preserve with improved hierarchy | `available` |
| Complete dedicated icon coverage | 920 missing | Monogram fallback | Phase 5 asset work | `concept` until imported |
| Ratings, popularity, install count | No data | Absent | Do not invent as product facts | `concept` |

Capability meanings are fixed:

- `Install preview` — one of three explicitly reviewed local recipes.
- `Connect` — project has a web UI but Local Store does not install it.
- `Explore project` — discovery only; no supported local connection is claimed.

## First Run and application settings

| Capability or fact | Backend evidence | Current UI | V2 use | Status |
| --- | --- | --- | --- | --- |
| Docker preflight | `doctor` | Manually invoked | Can run during First Run | `available` |
| Install, Connect, Browse routes | Existing commands/navigation | Available separately | First Run choices | `derivable` |
| Completed-onboarding flag | No stored field | No First Run | Needed to gate automatic display | `concept` |
| Dark appearance | Product decision | Settings text only | Fixed V2 mode | `available` |
| Offline catalog snapshot facts | `CatalogPage` | Settings text | About/Settings | `available` |
| Runtime preferences | No settings model | Absent | Do not fabricate controls | `concept` |
| Update channel/status | Explicitly deferred | Absent | Do not show active update controls | `concept` |

## Prototype feature classification

| Prototype surface | Available now | Derivable with a projection | Concept fixtures |
| --- | --- | --- | --- |
| Overview | App/status counts, Doctor, recovery candidates | Recently added apps from creation timestamps | CPU, memory, disk use, uptime, event history |
| Discover | Full catalog, filters, provenance, warnings | Capability summaries | Popularity, ratings, install counts |
| My Apps | Identity, status, readiness, actions, logs, paths, timestamps | Grouped health summary | Resource graphs, durable last-used history |
| Install | Recipe facts, setup fields, Doctor, stages, cancel, rollback | Typed restart/container projection | Image size, transfer rate, percent download |
| Activity | Live operation events only | Session-only event list | Durable history, duration, metrics, availability timeline |
| First Run | Doctor plus existing install/connect/browse routes | Single flow orchestration | Completion flag and automatic launch behavior |
| Settings/Recovery | Catalog facts, Doctor, recovery inventory | Safe discard exposed as launcher IPC | Appearance choices, updater controls |

## Product constraints carried into V2

- Windows laptop is the current target; design at 1280×800 and 1440×900.
- Local Store remains dark-only and offline-first.
- Project pages are sources, never treated as running instances.
- Only three reviewed recipes may claim automatic installation.
- Linked apps are not locally managed and must not receive Start, Stop, Logs, or
  Uninstall controls.
- HTTPS readiness is `unknown`, not `unreachable`, because the current probe is
  plain HTTP only.
- Destructive deletion is never the default.
- Recovery mutation must re-verify ownership at action time.
- Machine facts use the mono layer; prose and navigation use the sans layer.


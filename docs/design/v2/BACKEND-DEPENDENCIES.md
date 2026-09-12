# Local Store V2 backend dependencies and prototype annotations

Updated: September 11, 2026  
Status: **approved** by Madhav Sonkusare on September 12, 2026.  
Source of evidence: `CAPABILITY-MATRIX.md`, which lists every backend field, command,
and event behind each claim. The status words are the same: **available** (a current
command or model returns it), **derivable** (existing data plus a small typed
projection), and **concept** (not collected or persisted today).

The capability matrix was written against `main`. Later branches added backend work,
including readiness, setup fields, and cancel. On the checked-out branch (`2d7782e`) the
launcher exposes 16 commands: `list_apps`, `add_app`, `remove_app_cmd`,
`search_catalog`, `open_project`, `doctor`, `recipe_details`, `app_readiness`,
`check_address`, `cancel_app_setup`, `install_app`, `start_app`, `stop_app`,
`app_logs`, `uninstall_app`, and `inspect_recovery`. Re-check each **derivable** and
**concept** row against the branch being implemented before building it.

## Per component

| V2 component | What it shows | Status | Backend source | Production rule |
| --- | --- | --- | --- | --- |
| Rail Docker status card | Ready, checking, unavailable; engine version | available | `doctor` (`DoctorReport.ready`, `docker`, `compose` checks) | Ship |
| Rail counts | Discover total, My Apps count | available / derivable | `search_catalog` total, length of `list_apps` | Ship |
| Search command (Ctrl K) | Search across apps, projects, settings | concept | No aggregation contract | **Decided 2026-09-12: omit.** Catalog search on Discover and filtering in My Apps ship instead. Revisit if a search contract is built |
| Overview status summary | Headline, prerequisites, counts by runtime and status | available / derivable | `list_apps`, `AppStatus`, `Readiness`, `doctor` | Ship the counts through a typed summary projection |
| Overview Needs attention | Stopped, error, and unreachable apps; retained setups | available | `AppStatus`, `status_error`, `Readiness`, `inspect_recovery` | Ship |
| Overview Recent activity | Session operations | derivable (session only) | `OperationEvent` stream | Ship as "This session". Durable history is a concept |
| Overview concept telemetry | Memory, CPU, disk, uptime graph | concept | Not queried | **Omit** |
| Discover featured recipes | The three reviewed recipes, with version and address | available | `Recipe` | Ship |
| Discover catalog cards and filters | Name, category, description, licence, architectures, capability, warning | available | `search_catalog`, `category_counts`, `Filters` | Ship |
| Discover logos | One logo per project | available on `feat/catalog-icons` | `catalog/icons.json` (1,673 of 1,673) | Ship after that branch lands. Until then, use the monogram fallback |
| Project drawer | Facts, provenance, sources, capability statement | available | `Project`, `sources[]` | Ship |
| Connect drawer | Name, address, advisory check | available | `check_address`, `add_app` | Ship. The check never blocks saving |
| My Apps list and detail | Identity, type, status, readiness, address, paths, timestamps | available | `list_apps`, `InstalledApp`, `RuntimeSpec`, `Readiness` | Ship. `updated_at_unix` is "Last changed", never "last active" |
| My Apps Logs | Bounded recent output, refresh, copy | available | `app_logs` | Ship |
| My Apps Manage | Start, stop, uninstall (keep data), delete app and data, remove link, shortcut | available | `start_app`, `stop_app`, `uninstall_app(delete_data)`, `remove_app_cmd`, shortcut path | Ship. Linked apps get only Open, Remove, and Shortcut |
| Install review | Recipe facts, ports, storage, risks, image audit, setup fields, credentials notice | available | `Recipe`, `SetupReview` | Ship |
| Install review: restart policy, container name | Typed facts | derivable | Present only inside the Compose text | Needs a typed projection; until then, show them in the Compose disclosure only |
| Install progress | Seven stages including rollback; cancel before the commit | available | `OperationEvent.stage`, `cancel_id` | Ship. No percentages or download sizes (concept) |
| Install results | Success, failure matrix, rollback failed | available | Typed install errors | Ship. Retry stays withheld after a rollback failure |
| Activity | Operation list, filters, expandable detail | concept (durable) / derivable (session) | `OperationEvent` is not persisted | **Decided 2026-09-12:** ship session-only, labelled "This session". Durable history, duration, and metrics come later and need a persistence contract |
| Settings Docker check | Engine and Compose rows, re-run | available | `doctor` | Ship |
| Settings catalog facts | Counts, snapshot, local icons | available | `CatalogPage` | Ship |
| Settings Introduction card | Re-open the introduction | concept | No completion flag | Ship only with the First Run completion flag |
| Recovery scan and review | Candidate, paths, ownership state | available | `inspect_recovery`, `RecoveryCandidate` | Ship |
| Recovery stop and delete | Re-verified cleanup under a lock | derivable | `recovery::discard` re-verifies ownership and is reachable from the CLI (`src/cli.rs`). There is no launcher command | Ship only with a launcher command that calls `discard`, never acting on the displayed snapshot |
| First Run | Preflight, choose, install or connect, ready | available / derivable | `doctor`, existing install and connect commands | Ship the flow |
| First Run automatic display | Show once, then never again | concept | No completion flag | Needs a persisted completion flag before First Run can open by itself |

## Backend contracts V2 needs

These are the only new backend requirements. Everything else ships on current commands.

1. **First Run completion flag.** Persisted; read at startup; reset from Settings.
2. **Recovery action command.** Stop, or stop and delete, with ownership re-verified
   under the operation lock at action time, never from the displayed snapshot.
3. **Typed recipe projection.** Restart policy and container names as fields.
4. **Overview summary projection.** Counts by runtime type, status, and readiness in
   one call, so Overview doesn't recompute them from rows.
5. **Durable activity history** — deferred past the first release by the 2026-09-12
   decision. Persisted operation events with start and end times. Until then, Activity
   is session-only.
6. **Search aggregation** — not being built. The Ctrl K command is omitted.

## Prototype annotations: remove every one (F-22)

The prototype marks design-only content with two classes, `.lab-tag` and
`.truth-label`, and with preview machinery. Production ships none of it. The
acceptance test `no prototype annotations` (see `HANDOFF.md`) fails on any match.

| Annotation | Where (render function) | Production replacement |
| --- | --- | --- |
| "Design lab" panel, its trigger button, and every preview control | `index.html` `#prototype-panel`, `.lab-trigger`; `setPanel()` | Nothing. Remove the trigger from the workspace bar |
| "Concept" flag on the search command | `index.html` `.concept-flag` | Remove the command until search aggregation exists |
| "Concept telemetry · backend work" | `overview()` | Remove the telemetry panel |
| "Current data" | `overview()` | Nothing. The data is real |
| "Catalog facts · available" | `projectDrawer()` | Nothing |
| "Editable now" | `installReview()` | Nothing |
| "Component study" | `installSetupFields()` | Nothing. The real setup form replaces the study |
| "Current session", "Live event" | `activity()` | "This session" as the list heading, the one truthful label kept |
| "Backend work" | `activity()` | Remove, or ship durable history |
| "Available now" and "Concept" | `settings()` | Nothing; remove the Introduction card until the completion flag exists |
| "Inspection available now", "Cleanup needs launcher wiring", "Implementation boundary" | `recovery()` | Nothing, once the recovery action command exists |
| `data-prototype-action` toasts ("… withheld", "Prototype action", "Global search concept") | `bindLocalControls()` | The real command and its real result toast |
| Preview URL parameters (`?state`, `?stress`, `?section`, `?dialog`, `?recovery`, `?step`, and the rest) | Bottom of `prototype.js` | None. Production state comes from the backend |
| `stress-fixtures.js`, `prototype-fixtures.js` | Fixture modules | Real commands |

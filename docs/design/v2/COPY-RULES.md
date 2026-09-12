# Local Store V2 copy rules

Updated: September 11, 2026  
Status: **frozen**. Approved by Madhav Sonkusare on September 12, 2026.  
Scope: every word in the product UI, including labels, statuses, errors, and machine
facts. The prototype's strings are the reference, except where a rule below corrects
them.

## Voice

- Write from the user's side of the screen. People manage *apps*, not Compose projects,
  and *addresses*, not endpoints.
- Say what happened and what to do next. An error names its cause and its recovery path
  ("Port 5230 is already in use. Choose an unused port or stop the app using it").
  Errors carry no apology and nothing vague.
- A control names its outcome: "Review install", "Save linked app", "Uninstall, keep
  data", "Delete app and data". The confirmation afterwards repeats the verb ("Linked
  app saved").
- Say what did **not** change whenever a user might fear it did: "Nothing was changed",
  "Saved apps are unaffected", "The server and its data stay untouched".
- Only the product itself makes claims. No marketing adjectives on operational screens.
  The ember signature region may carry one line of voice.
- Sentence case everywhere, except short tags in the uppercase mono label style.

## Terminology

| Use | For | Never |
| --- | --- | --- |
| Managed app | An app Local Store installed from a reviewed recipe and runs through a Compose project it owns | "Container", "stack", "deployment" |
| Linked app | A saved address for an app the user runs elsewhere. Local Store opens and checks it; it does not manage it | "Connected server", "remote app" |
| Reviewed install | Installing one of the reviewed recipes, after the review screen | "One-click install", "Auto install" |
| Recipe | The reviewed definition behind a managed app | "Template", "manifest" (in UI) |
| Project | A catalog entry. It is a source to explore, never a running instance | "App" (until it is saved) |
| Catalog | The offline list of projects bundled with this build | "Store", "marketplace" |
| Connect | Saving an address as a linked app | "Add server", "Import" |
| Explore | Opening a project's own page outside Local Store | "Install" |
| Docker check | The Engine and Compose readiness check in Settings | "Doctor" (internal name only) |
| Retained setup | Files and containers an interrupted install left behind | "Orphan", "junk" |
| Recovery | Reviewing and clearing a retained setup | "Cleanup" as a noun |
| Activity | The list of operations: install, start, stop, uninstall, open | "Logs" (logs are an app's own output) |
| Uninstall, keep data | Removing a managed app and preserving its data folder (the default) | "Delete" |
| Delete app and data | Permanent removal of the app and its data folder. Always needs typed confirmation | "Reset", "Clean" |

**Status words** are capitalised and paired with a dot: Running, Stopped, Error, Ready,
Unreachable, Checking. An HTTPS address the probe cannot check is **Unknown**, never
Unreachable. Linked apps never show Start, Stop, Logs, or Uninstall.

## Headers and eyebrows (F-21)

An eyebrow appears only when it carries state or a fact the title does not. Removing
one never removes information.

| Screen | Eyebrow | Decision |
| --- | --- | --- |
| Overview | "Workspace health" | Remove. It restates the screen |
| Discover | "Offline catalog" | Keep. It states a fact: nothing is fetched |
| My Apps | "Your workspace" | Remove. It restates the navigation |
| Activity | "Operations over time" | Remove. It restates the description |
| Settings | "System & support" | Remove. It restates the navigation |
| Install | "Focused task · {app}" | Keep. It names the task and its subject |
| Recovery | "Focused task · retained setup" | Keep |

Inside a screen, result and state kickers stay: "Install in progress", "Recovery
complete", "Scan complete", "Docker required for local installs". Section kickers stay
where they classify facts the heading doesn't ("Provenance", "Machine changes").
The prototype applies this table.

## Machine facts

Machine facts are values a user may copy, compare, or type elsewhere. They use the mono
layer, wrap with `overflow-wrap: anywhere`, and are never truncated.

| Fact | Format | Example |
| --- | --- | --- |
| Address | Exactly as Local Store stores and opens it. Protocol included in detail views; omitted on compact recipe cards, which show host and port only | `http://127.0.0.1:5230` · `localhost:5230` |
| Port | Number only; the protocol belongs to the address | `5230` |
| Version | The upstream tag, verbatim | `0.30.0`, `2.37.10` |
| Image | Full reference, including registry and digest when pinned | `ghcr.io/usememos/memos:0.30.0@sha256:…`, with the whole digest shown |
| Path | Absolute, exactly as the backend returns it. The prototype shortens the data root to `Local Store/`; production does not | `Local Store/apps/memos/compose.yaml` (prototype) |
| Error code | Code, then operation and subject, joined by a middle dot | `port_in_use · install · memos` |
| Docker error | Verbatim, in full, in detail views. Scan rows clamp it to two lines with the full text in a `title` | — |
| Counts | Grouped thousands | `1,672 projects` |

**Dates and times:**

- Facts use an absolute date without zero padding: `Sep 4, 2026`.
- Activity rows use a day and a 24-hour clock: `Today · 09:42`, `Yesterday · 11:28`,
  `Sep 6 · 10:04`. The prototype's `Sep 06` is corrected to `Sep 6`.
- Relative phrases ("2 days ago") appear only in Overview's recent list.

**Missing values** are omitted, not shown as placeholders. A card with no stated
architecture shows only its licence (V-15). The catalog's licence placeholder "See
project" reads as a fact but isn't one. Production shows **Licence not stated**, and
Details links to the project.

## Truth language

Production never presents a value the backend cannot supply. Where the prototype marks a
value as a concept (see `BACKEND-DEPENDENCIES.md`), production either ships the backend
contract or omits the surface. It never ships the value with a caveat. Advisory checks
say so ("The address check is advisory").

## Numbers in copy

Use numerals for counts, ports, and versions ("3 apps need a quick look"). Spell out
nothing that a user might compare. Don't use percentages Local Store cannot measure;
install progress shows stages, never a percentage.

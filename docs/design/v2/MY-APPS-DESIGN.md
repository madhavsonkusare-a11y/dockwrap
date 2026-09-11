# My Apps design — Phase 9

My Apps is the operational center for saved applications. It answers two
questions without changing context: **what do I have?** and **what can I safely
do with the selected app?** The selected direction keeps a persistent list at
the left and a dark-raised detail surface at the right at both supported laptop
widths.

## Chosen direction

The dark-raised detail panel was selected over a warm-light panel. A light
panel made the details prominent, but it also felt like a separate product and
reduced the legibility of status and log surfaces. The raised neutral surface
keeps the workspace coherent with the shell; a single orange inset edge marks
the selected managed app. Linked apps use a neutral inset edge and a dashed
type pill so object ownership is not communicated by color alone.

| Before | After | Why |
| --- | --- | --- |
| Rows were visually interchangeable | Every row exposes object type plus a written status, with a selected-state ownership cue | A user can scan the list without guessing from icon color |
| One generic detail treatment | Status-specific Overview, Logs, and Manage sections | Keeps operational information attached to the selected app |
| Managed and linked apps implied the same control | Managed apps own lifecycle/log/data actions; linked apps own Open, address check, shortcut, and record removal | Matches the backend boundary and never claims Local Store runs an external server |
| Removal actions were undifferentiated | “Uninstall and keep data” is the safe default; permanent data deletion is isolated and name-confirmed | Consequences are visible before commitment |
| Errors displaced useful data | Refresh failures retain the last-known list, status, and selection | Recovery does not destroy context |
| Tabs and rows were mouse-oriented placeholders | Listbox and tablist semantics support arrow keys, Home/End, retained focus, Escape, and a trapped confirmation dialog | The master/detail model remains efficient from the keyboard |

## Information shown

The default Overview uses only fields that help identify or operate the app:

- address, runtime/object type, catalog association, created date, and registry
  update date;
- current runtime or readiness explanation;
- managed project paths inside a disclosure rather than in the default scan;
- an explicit server-boundary note for linked apps.

The fixtures cover managed Running, Stopped, Error, and Busy states plus linked
Ready and Unreachable states. A linked app is never described as Running.

## Actions and safety

Primary actions adapt to the selected app and status: Open, Start, Stop, View
logs, or Check address. The Manage section contains lower-frequency actions.
Managed apps provide lifecycle control, safe uninstall, and an isolated danger
zone. Linked apps provide shortcut creation and removal of the Local Store
record while promising that the external server remains untouched.

The safe uninstall dialog states that the managed data folder remains on disk.
The irreversible path states exactly which three resources are deleted and
keeps its confirmation disabled until the user types the app name. Prototype
actions intentionally show feedback without changing apps or files.

## Logs

Logs remain inside the selected app rather than opening an identity-less modal.
The surface uses 11px IBM Plex Mono at 1.75 line-height, preserves wrapping, and
labels the data as a bounded on-request Compose snapshot. Loading, empty, and
failure states retain the app identity and offer a truthful refresh action.
Linked-app logs remain disabled because the backend does not own that server.

## Responsive and keyboard behavior

- 1440×900: 340px list with a flexible detail area.
- 1280×800 minimum: the same master/detail relationship remains present; detail
  padding tightens and long values truncate or disclose rather than reflowing
  the entire workspace.
- Up/Down and Home/End move selection in the visible list and retain focus.
- Left/Right and Home/End move between enabled detail tabs.
- Search filters in place and has a recoverable filtered-empty state.
- Dialog focus starts on Cancel, cycles within the modal, returns to its opener,
  and Escape closes it.

## Evaluation toolkit evidence

The toolkit’s interaction-level rubric was applied after driving the rendered
prototype, not just reviewing screenshots:

- **Missing feedback:** every prototype action produces an immediate toast;
  busy and success banners retain the selected app.
- **Silent validation:** destructive confirmation visibly remains disabled until
  the exact app name is entered.
- **Hidden consequences:** uninstall, linked removal, and data deletion each
  enumerate a distinct data/server consequence.
- **Weak recovery:** refresh failure, log failure, no results, and empty logs all
  preserve identity and expose a direct recovery action.

Automated axe scans are clean for default, loading, empty, busy, refresh-failed,
logs default/loading/empty/failure, managed Manage, linked Manage, and the
delete-data dialog. Browser interaction verified filtering, Arrow-key selection,
tab movement, exact-name validation, Escape, and focus return. The foundation
checker also verifies the app/log fixtures and offline icons.

## Deterministic review URLs

- Default: `index.html#my-apps`
- Busy: `index.html?state=busy#my-apps`
- Refresh failed: `index.html?state=failure#my-apps`
- Empty: `index.html?state=empty#my-apps`
- Logs: `index.html?app=memos&section=logs#my-apps`
- Log failure: `index.html?app=memos&section=logs&substate=failure#my-apps`
- Managed controls: `index.html?app=memos&section=manage#my-apps`
- Linked controls: `index.html?app=immich&section=manage#my-apps`
- Safe uninstall: `index.html?app=memos&section=manage&dialog=uninstall#my-apps`
- Permanent deletion: `index.html?app=memos&section=manage&dialog=delete-data#my-apps`

Phase 9 remains a visual prototype. No production frontend, Docker project,
application record, shortcut, log source, or managed data was mutated.

# Local Store: agent handoff

## September 10 — Node-RED offered, and a reviewed way to move a pin

Node-RED is approved and offered. Two apps are now installable that came from
an upstream catalogue rather than a hand-written recipe.

**A correction worth carrying forward: Node-RED does not exercise the setup
form.** The previous handoff said it would. Its Runtipi definition declares
`form_fields: []`, exactly like PrivateBin, so the setup form has *still* never
rendered a real app's fields. The candidates that would are in the queue and
measured: **121 importable definitions declare fields, 34 of them required**.
Best first choices are `runtipi:flatnotes` (three required answers, two
generated credentials, one service) and `runtipi:planka` (four required, one
generated, two services). That is the next gap, and nothing so far has closed
it.

**`ImagePin`: a review may move a tag, and nothing else.** The Runtipi
definition pins 5.0.6; Node-RED released 5.0.7 a week later, migrating to a
patched JSONata and updating body-parser — dependency fixes worth having in an
app whose purpose is evaluating expressions somebody writes. The template model
had no way to say that: definitions are kept verbatim, so the only options were
to ship the older release or withhold the app. Upstream catalogues lag upstream
projects constantly, so this would have recurred for every app.

The override is deliberately narrow, with a test for each way it could stop
being an override and become a second definition wearing the first one's
provenance: the same repository only, a tag that actually changes, a stated
reason, a tag that exists at all, and a pin that names an image the definition
really runs. Pins are applied *before* the image audit, so the audit is forced
to be about what runs — auditing 5.0.6 while running 5.0.7 fails.

**First use, proven in a real browser and against the real admin API.**
`scripts/nodered-probe.mjs` deploys a flow through Node-RED's own API, then
checks what only a running app can show: the editor renders its workspace and
palette, opens and holds its `/comms` websocket, and draws the deployed
function node on the canvas; a `GET /probe` returns `local-store-42`, a value
the function computes rather than echoes, so a reply proves the runtime
executed user-authored JavaScript. Credentials are asserted encrypted at rest
and never written in plain text. Passed in 46.92s.

**Verified non-vacuous.** Making the keep-data uninstall destructive turns the
post-restart flow check into a 404 rather than passing anyway.

**The judgement call, stated plainly: the Node-RED editor has no sign-in.**
Anyone who can reach it can change flows and run code inside the container.
That is inherent to Node-RED and normal for a local tool, it is bound to
loopback only — asserted, including that nothing binds `0.0.0.0` — and it is
the first of the risk notes the install review shows. If that is the wrong
call, `promotion.state` in `src/templates/nodered.json` is the one line that
reverses it.

**An identity mismatch the tests caught.** The catalog lists Node-RED as
`node-red`; the Runtipi definition describing it is `nodered`. Discovery
matched offerings by raw id, so an approved app was installable by id and
findable by nobody. `catalog_index` now resolves through `catalog::catalog_id`,
which is the alias table that already existed for exactly this.

**Both evidence files stated a promotion state that had gone stale.** They said
`withheld` while both apps were approved. That field now reads from the
manifest, so it cannot disagree with the thing it describes.

Proven through the product: `local-store recipes` lists five apps, `install
nodered` committed `catalog_id: "node-red"` with the editor answering HTTP 200,
and `uninstall --delete-data` left no container, network or volume.

Validation: 251 Rust tests across 24 binaries, strict Clippy, `cargo fmt
--check`, thirteen offline gates, 52 Playwright. Scope is unchanged and worth
repeating: one host, one architecture actually executed.


## September 10 — the first imported app a person can actually install

PrivateBin is approved and reachable. Before this, approving anything would
have been a no-op: `ReviewedTemplate::offerable` was called by nothing but its
own guard test, and every user-facing path — the catalog listing,
`recipe_details`, `install_app`, the CLI — resolved through
`recipes::recipe(id)`, which never looked at a template. The allowlist was a
shelf.

**`src/offerings.rs` is the one lookup now.** An `Offering` is either a
reviewed recipe or an approved reviewed template, and `offerings()` filters the
withheld ones out once rather than asking every call site to remember. Both the
review a person is shown and the install that follows resolve through it, so
they cannot describe different things — the same reason `template_for` was made
a single function when the setup form was built. `OfferingSummary` keeps the
field names the launcher already read off a recipe, so the review dialog needed
no new vocabulary to describe an imported app.

Recipes are unchanged where it matters: `Offering::recipe()` still hands
`begin_install` the recipe's own Compose file, byte for byte. Only an app with
no such file is rendered from a plan.

**The guard test was narrowed, not deleted.** It used to assert nothing was
approved, which was true and useful right up until it wasn't. It now asserts
the approved set equals a named `APPROVED` list, so approving a *second* app
still has to appear in a diff, and a companion test requires an approved
template to carry the evidence its approval claims. The catalog, CLI-query and
facet tests were changed to derive their expected set from `offerings()` rather
than hardcode three ids — approving an app should update them, not break them.

**Order mattered.** The flag was flipped last, after the wiring, with the guard
already demanding it. Flipping first would have meant doing the whole refactor
with a broken guard and no way to tell an accidental approval from the intended
one.

**An icon fix the test caught.** Imported apps committed with `catalog_id:
None`, so they installed without an icon while recipes had one. It now resolves
through the offering named by the plan's id. That is sound only while a plan
carries its offering's own id, which nothing in the type system enforces, so
`an_offering_maps_to_a_plan_that_carries_its_own_id` asserts it. The lifecycle
test renames the plan for isolation and therefore gets no icon — that is the
guard working, and it asserts exactly that rather than the production value.

**Proven through the product, not around it.**
`tests/privatebin_lifecycle.rs` now resolves through `offerings::offering`, so
it fails if the review is withheld or unwired; passed in 32.37s. Then the real
CLI, against an isolated config root: `local-store install privatebin`
committed `catalog_id: "privatebin"`, the app answered HTTP 200 on
`http://localhost:8080`, and `uninstall --delete-data` removed it leaving no
container, network or volume.

Scope, unchanged by any of this: one host, one architecture actually executed;
the seven-architecture claim is registry metadata. **Neither offered app
declares setup fields, so the setup form has still never rendered a real app's
fields.** Node-RED is the next candidate, and it is the one that would close
that gap — review the 5.0.7 patch before pinning.


## September 10 — PrivateBin qualified, and one decision left for the owner

The B2 gate the last handoff described is met, and the review now ships as
`src/templates/privatebin.json` rather than living as a test fixture. That move
is the point: in `tests/fixtures/` the manifest was covered by two unit tests
and nothing else. In the allowlist it is covered by
`every_reviewed_template_resolves_to_a_valid_plan` — pinned upstream commit,
audited images, per-architecture digests, a named lifecycle proof that has to
exist, a recorded promotion decision — and by
`scripts/check-template-platforms.py`, which verified its seven published
architectures against live registry metadata and now re-verifies them offline.

**Browser first use is proven, and the evidence file no longer says otherwise.**
`tests/privatebin_lifecycle.rs` called `browser_probe` three times while the
evidence it wrote declared `browser_first_use: "not_tested"` — a stale string
from a run that predated those calls. The claim is now derived: each probe is an
assert, so reaching the evidence write means all three passed, and the checks it
lists say what they were. Passed in 33.68s against the shipped manifest.

What the probes actually establish, which an HTTP readiness check cannot: the
loopback origin is a secure context exposing `window.crypto.subtle`; a paste
typed into the real page encrypts, saves and decrypts again from its own link;
that link still decrypts after a stop/start; and it still decrypts after a
keep-data uninstall and reinstall.

**Verified non-vacuous.** Turning the keep-data uninstall into a destructive one
makes the second read fail on `expect(locator).toContainText` rather than
passing anyway, so the persistence claim is doing work. The tree was restored
from a byte-identical backup afterwards and the run repeated.

Storage side, unchanged from the previous batch and re-confirmed: writes as the
documented UID 65534 / GID 82 rather than root, one loopback port binding, the
data survives restart and keep-data reinstall, delete-data removes every owned
container, network and volume, and unrelated containers are counted before and
after and survive. Evidence:
`docs/evidence/privatebin-storage-windows-2026-09-09.json`.

**Promotion stays withheld, and that is not a gap in the evidence.** The image
was rebuilt 2026-08-08, publishes linux/amd64 and arm64 among seven
architectures, and installs and works. What is missing is a decision, not a
measurement: offering an app to people is the owner's call, and
`no_reviewed_template_is_offerable_without_an_explicit_approval` exists so that
call appears in a diff instead of arriving as a side effect of a passing test.
Flipping `promotion.state` to `"approved"` is the whole change, and it will make
that guard test fail until somebody rewrites it deliberately.

Scope limits worth keeping in front of whoever picks this up: one host (Windows
with Docker's Linux engine), one definition, one architecture actually executed.
The seven-architecture claim is registry metadata, not seven runs. PrivateBin
declares no setup fields, so **the setup form still has never rendered a real
app's fields** — approving this app would not close that gap either, and
Node-RED remains the next candidate after it.

Validation for this batch: 183 library tests, 23 test binaries green, strict
all-target/all-feature Clippy clean, `cargo fmt --check` clean, and fourteen
offline gates passing including the refreshed template-platform check. No
leftover container, network or volume; scratch directories removed. No commit
or push in this batch.


## September 9 — B1 candidate inventory

Implemented `scripts/build-candidate-queue.py` and `examples/candidate_queue.rs`.
Run `python scripts/build-candidate-queue.py` with the pinned ZIPs in
`.cache/catalog`; it runs both production Rust importers without installing apps.
Outputs: `catalog/candidate-queue.json` and `docs/candidate-queue-baseline.md`.
Baseline: 356 CapRover definitions / 117 expressible; 250 Runtipi / 85 expressible.
After shortlist reconciliation there are 233 repository groups (source-declared
or explicitly reviewed) and 365 unresolved definitions. The original baseline
was 232 groups / 369 unresolved; three cross-source duplicates were reconciled
and linkding gained its own repository identity.
These are NOT counts of verified unique installable apps. Image-family matches
are reconciliation hints only. Blocked candidates retain blockers; normalized
images and field requirements are available where the importer produces a template.
No defaults, answers or generated credential values are exported.

Source ZIP hashes live in `catalog/import-audit-sources.json`. The Runtipi
`sources.lock.json` hash covers normalized discovery data, not its ZIP. A fresh
codeload download matched the cached ZIP; the separate archive pin preserves
both meanings without altering the discovery lock.

Validation: full queue generation succeeded with the baseline above;
`cargo clippy --example candidate_queue --locked -- -D warnings` passed;
`python scripts/test_candidate_queue.py` passed (3 identity/hint/stale-review tests).
B1 bounded shortlist screening is now delivered in `docs/candidate-screening.md`,
with public release/platform evidence in `catalog/candidate-screening-evidence.json`
and revision/image-bound decisions in `catalog/candidate-reviews.json`.
Use `python scripts/screen-candidate-images.py --refresh` for metadata refresh;
without the flag it prints saved evidence. Refresh does not approve anything.
PrivateBin 2.0.6 is first for B2 qualification, Node-RED second (review newer
5.0.7 patch before choosing its final pin). Linkding's current definition pins
a 2023 image; Joplin's pins a 2022 database image. Both are withheld for updates.
All four have linux/amd64 metadata, but none has first-use proof from this batch.
Next agent: add reviewed-template support for Runtipi config/provenance through
the existing importer (the review module currently handles CapRover only), then
qualify PrivateBin browser paste creation/reopening and lifecycle on Windows.
Check managed-folder ownership and loopback browser crypto before promotion.
Do not expose arbitrary-template execution or infer readiness from HTTP 200.
Full-catalog maintenance screening and identity reconciliation remain ongoing.
Do not mistake
old CodiMD lifecycle proof for promotion approval. No candidate was promoted.
No Docker resources, production UI edits, commits or pushes in this batch.
V2 work remains owned by the other session. Other-session icon provenance files
also appeared during this work and were left untouched. The usage window reset;
latest check was 34% used / 66% remaining. Recheck before starting another batch.

September 9 planning update: the owner requested an Umbrel architecture review
and a new backend plan for maximum one-click app coverage. The result is
`docs/backend-app-coverage-plan.md`. Next implementation cycle: deduplicated
candidate queue, one maintained candidate with complete promotion evidence,
then reusable batch verification. No candidate was promoted or implementation
changed in that research/planning turn. V2 design is being handled in another
session and must remain untouched by this backend work.

Updated September 8, 2026. This is the current starting point for any coding
agent. [Upgrade status](upgrade-status.md) tracks all 33 original deliverables;
[archived checkpoints](agent-history-2026-09-07.md) preserve the complete prior
handoff, rationale and diagnostics. Private Hermes files and chat history are
not needed to continue.

## Working agreement

- Product: **Local Store**, repository https://github.com/madhavsonkusare-a11y/local-store.
- **Local review only. Do not commit or push without a new owner instruction.**
  Branch is `main`, last known commit `49bf970`. The large mixed uncommitted tree
  contains owner/other-agent work, 244 added icons and these continuation batches.
  Preserve all of it; do not reset, clean or force-push. Coordinate before another
  agent edits this checkout. Earlier direct-to-main permission is superseded.
- Query current usage before sizing a batch; reserve enough to validate and
  hand off. The last check in this continuation was 91% of the five-hour window
  used; it can reset independently of the chat.
- Dark-only UI. Preserve the approved identity, spacing, gradients, keyboard
  behavior and reduced motion. Reuse existing modules; apply Emil/Apple design
  guidance for UI changes. No new UI dependencies were added in this continuation.
- Icon sourcing is deferred. There are 1,672 discovery projects, 752 offline
  icons, and three install previews. Discovery count is not verified-install count.
- Do not rename the checkout directory while tools use it. Legacy constants and
  the old URL scheme intentionally support migration and existing shortcuts.

## Current outcome

**23 complete, 8 partial, one not implemented, one final gate not run. Ten of
33 deliverables still need work.** This counts deliverables, not effort.

Latest continuation: **setup-review backend contract implemented** in
`src/setup/review.rs`. `PlanTemplate::setup_review()` validates the template and
returns an explicit serializable projection. Sensitive defaults/options, raw
regexes, template environment and generated credential identities/values are
excluded. Sensitive fields use password controls; non-sensitive booleans,
numbers and choices retain their input types. All validation remains in Rust.

`recipe_details` now returns the existing flattened reviewed recipe plus this
`setup_review` object. The same launcher guard and three-recipe allowlist apply.
Current recipes have no setup fields; the frontend continues reading its existing
recipe fields unchanged. Two focused projection tests verify redaction, default
preservation, unknown-answer refusal and deployment validation.
Both projection tests, all three launcher capability tests and strict all-target
Clippy passed. No commits or pushes; the mixed working tree is preserved.

**Imported installation UI is NOT complete.** No imported candidate was promoted,
no template-install IPC was added, and no form was rendered in this batch. The
remaining usage was insufficient to review a candidate and finish that entire
flow. Next agent should use this projection, not serialize a full template.
Before wiring a candidate: complete recipe-equivalent provenance/platform and
lifecycle review (the old CodiMD proof alone is not catalog approval), add a
reviewed-template allowlist, then reuse begin_template_install and the existing
operation/progress/commit flow. Render fields with text-safe DOM operations,
mask sensitive controls, explain that blank uses a default, and keep submitted
answers out of saved UI state, logs and diagnostic reports. Test rejection,
retry/close races and successful installation together before promotion.

Previous continuation: **health-dependent startup ordering is implemented and
proven with real Docker**. Both importers retain service_started/service_healthy
conditions; unknown options and completion-job conditions remain named refusals.
Runtipi healthCheck now maps startPeriod/startInterval explicitly. Unresolved
health-check dollar expressions remain blocked. Plans reject dependency cycles,
dangling health conditions and dependencies on explicitly disabled probes.

Fixed an import-validation gap exposed by the cycle regression: PlanTemplate
validation checked inputs but did not validate its underlying DeploymentPlan.
It now does both. Final reports: **CapRover 117/356; Runtipi 85/250**. Health
support added seven Runtipi candidates (95 to 102), then full plan-policy checks
excluded 17 (102 to 85). Those were false-positive importability claims, not
previously shipped installs. The catalog's reviewed install offerings are unchanged.

Validation: 169 library tests passed after the template fix, plus the final
focused graph-invariant regression. Compose config confirms service_healthy
rendering; strict Clippy passes. Real Adminer/Postgres lifecycle passed in 58.03s
with a deliberately five-second probe: database healthy at
12:10:48.682458335Z, web started at 12:10:49.102724975Z. It also passed credential
authentication, retained-data reinstall and scoped cleanup. Evidence:
`docs/evidence/health-dependent-startup-windows-2026-09-08.json`.
No test deployment remains, no UI changes, no commits or pushes.

Next: choose a reviewed imported candidate and expose typed setup through the
existing install transaction. Platform constraints and health-check variable
expansion need explicit modelling before broader source coverage. Completion-job
dependency semantics remain unsupported. See updated pinned importer reports.

Previous continuation: reviewed the owner's completed CapRover adapter, CodiMD
proof, generated-secret rule checks, startup overrides and own-address rewrite.
The current counts are **CapRover 117/356 and Runtipi 95/250 expressible plans**.
One imported app (CodiMD) has a prior Docker lifecycle proof; these counts do
not promote additional apps to reviewed install availability.

Added `PlanHealthcheck` in `src/plan/healthcheck.rs`, exposed through
`PlanOverrides`, and mapped Compose healthcheck objects in the CapRover adapter.
It preserves CMD/CMD-SHELL, disabled checks, inherited image tests, timing and
retry fields. Unknown properties, invalid modes and unresolved dollar expressions
are named refusals. Setup currently resolves environment values only, so health
check expansion is deliberately not guessed. The launcher HTTP readiness probe
is unchanged; dependency `service_healthy` conditions still need modelling.

Validation: **169 library tests pass**, Docker Compose config parsing confirms
exec/shell forms and timing survive rendering, strict all-target/all-feature
Clippy passes. Refreshed pinned CapRover report remains 117/356 because affected
definitions have other blockers. No containers started in this batch, no new
dependencies, no UI changes, commits or pushes.

Read Umbrel's pinned store API, install review and lifecycle implementation.
See `docs/umbrel-implementation-reference.md` for source links and practical
implications. The main Umbrel code is PolyForm Noncommercial; that is distinct
from the unresolved packaging permission in umbrel-apps. No code/assets copied.
Next: explicitly map Runtipi healthCheck, model health-dependent startup, and
prove delayed database readiness. Preserve the existing transaction engine.

### Earlier continuation checkpoints (historical counts below)

Earlier continuation: Recovery inspection UI passed its Playwright test
(empty/verified/error/busy/late replies), launcher capabilities passed three
Rust tests, and strict Clippy passed. Copy now limits the empty-state claim to
supported apps and does not claim Docker labels prove data integrity. Automatic
repair remains unimplemented. The owner's real Adminer/Postgres multi-service
proof is preserved; its scope remains plan transaction, not a reviewed app.

Import-source study follow-up: `docs/caprover-source-audit.md` provides an offline,
checksum-pinned audit of 356 CapRover definitions. See the updated study for
Umbrel findings and source links; Umbrel remains research-only pending packaging
reuse permission and platform adaptation. No catalog installability was promoted.
Typed setup now supports explicit hex secret generation and bounded ASCII regex
fields. Existing generated secrets remain alphanumeric. Stored secrets are
validated without replacement. CapRover's generated-secret declaration parser
is implemented in `src/importers/caprover.rs`; the deployment adapter is NOT.
The setup-variable mapper now preserves literal defaults and rules, maps generated
hex secrets, masks unreviewed answers and refuses platform expressions. It maps
2,057 variables and refuses 520, with named reasons in the reproducible report.
Six CapRover unit tests pass, as does strict all-target Clippy.
The strict regex-literal translator also refuses flags, lookaround,
backreferences and unsupported escapes. `python scripts/caprover_setup_report.py`
runs the actual Rust parsers over the checksum-pinned archive: 1,377 patterns
and 222 secret declarations supported; 87 patterns and 52 secret declarations
refused across 75 apps. See `docs/caprover-setup-report.md`. These are primitive
counts, not working apps. Full library suite passed 149 tests before the final
translator/mapper additions; the final six CapRover tests passed separately.
Strict Clippy, MSRV and dependency-license checks pass.
Next: implement a conservative version-4 adapter using these primitives, preserve
all constraints or report a named limitation, then produce real importer counts.
The implementation and validation sequence is in
`docs/caprover-adapter-checklist.md`; no new install engine is needed.
Do not use the study's 180-app estimate as a verified count. Pattern conversion
from JavaScript must explicitly refuse unsupported syntax/semantics.

Latest real-Docker recheck after secret validation changes: Adminer/Postgres
multi-service lifecycle passed in 38.60 seconds; sanitized evidence is
`docs/evidence/plan-multi-service-windows-2026-09-08.json`. Data, credentials,
reinstall and scoped cleanup passed. No test deployment remains. Refreshed
Runtipi report reflects the owner's port mapping: 83/250 expressible plans,
167 blocked. These remain candidates rather than reviewed installable apps.

Previous work: **foundation task 14: Docker ownership verification**.
`local-store recovery [--json] --docker` now checks returned container IDs and
Compose labels through bounded read-only Docker commands, without executing a
retained Compose file or reading container environment. Every matched container
must have the expected project, service, non-one-off flag, canonical absolute
Compose path and working directory. The report distinguishes not checked, no
matching containers, mismatch and verified. Errors/truncation cannot set verified.
The default inventory remains offline and does not write files.

Three inventory tests and strict Clippy pass. Extended the real interrupted-Memos
test to exercise the CLI verification; it passed in 17.14 seconds, then completed
data-preserving recovery/reinstall/deletion. Evidence root:
`.cache/install-interruption-27292-1788837129845341700`. No test deployment remains.
This only verifies label/path association, not unchanged Compose content or
permission to execute it. A recovery action must recheck under its lock and
revalidate the deployment definition. Inspection UI is now validated; recovery
actions remain open. No commit/push.
Latest usage: 86% five-hour, 92% weekly consumed; refresh before sizing more work.

Previous batch: **locked plan transaction and registry integration**.
`begin_template_install` and `install_template` now reuse the recipe path's
`PendingInstall` through one shared begin helper. Resolution, container startup,
registry commit and rollback retain the matching operation lock; mismatched app
locks are rejected for both recipe and template entry points. No new IPC exposed.

Extended the real n8n test to use private production registry storage, assert
the pending lock is held, commit successfully, and deliberately corrupt only
its test registry before a preserved-data reinstall commit. Rollback removed
the container while preserving the original secret file and volume. Restoring
the test registry allowed retry with the same encryption key and data marker;
final uninstall removed resources and the registry entry. Passed in 96.79 seconds.
Evidence: [plan transaction](evidence/plan-transaction-n8n-windows-2026-09-08.json).
All 33 runtime tests and strict Clippy pass. No UI changes, commits or pushes.

Previous batch: **real plan-install lifecycle proof**. The opt-in
`tests/plan_lifecycle.rs` passed on Windows/Docker Linux-amd64 with cached n8n
2.37.10 in 70.35 seconds. It renders the reviewed n8n plan under a unique project
name and unused port, adds a generated `N8N_ENCRYPTION_KEY`, then calls the real
`install_template_with` path. It proves HTTP readiness, the persisted key reaches
the container, stop/status/start, keep-data uninstall, byte-identical secret reuse
on reinstall, mounted-volume marker persistence, and complete explicit deletion.
No credentials are logged or included in the report. Evidence:
[n8n plan lifecycle](evidence/plan-lifecycle-n8n-windows-2026-09-08.json).
Strict Clippy, formatting and diff checks pass. Test-owned resources were removed;
no unrelated containers, registry entries or UI files changed. No commit/push.
This is runtime proof, not GUI/registry integration, multiple-service startup,
encrypted-workflow migration or cross-platform proof.

Reproduce with `LOCAL_STORE_RUN_DOCKER_TEST=1` and
`cargo test --locked --test plan_lifecycle -- --ignored --nocapture`.
The test requires the n8n image already cached and retains diagnostics on failure.

Previous batch: **timezone import support and credential preservation review**.
Reviewed the new plan/template/importer/install path and preserved its shared
`InstallSource` design. Runtipi `${TZ}` references now synthesize one optional
text field with default `UTC`, only when used and not already declared upstream.
Users can override it; upstream defaults and labels stay intact. This is an
explicit per-app default, not host-timezone detection or IANA database validation.
Regenerated the pinned report: **45 -> 60 importable out of 250**; platform
placeholder blockers **96 -> 26**. Importability is not lifecycle verification.

Found and fixed a credential reuse fault: unreadable/corrupt retained secret
files previously became an empty map and triggered new credentials. They now
stop before Docker or file writes, with a redacted error. Only a missing file
retains the existing new-secret behavior. A regression proves corrupt content
is preserved and no Docker call occurs. All **136 library tests** and strict
all-target/all-feature Clippy pass; formatting/diff checks pass. No new crates,
UI changes, real Docker runs, commit or push in this batch. Windows Tauri resource
build was denied in the sandbox; normal process permissions completed validation.

Previous batch: **plans can install (roadmap phase 2)**.
`runtime::install_template_with` resolves a `PlanTemplate` with the answers
given, reuses or generates its secrets, renders the Compose file and hands the
result to the same install path recipes use.

**One install path, not two.** A reviewed recipe and a resolved plan both reduce
to an `InstallSource`, so rollback, the confinement check, port preflight and
cancellation are shared rather than duplicated into a copy that drifts. The
refactor is behaviour-preserving — every existing runtime test passed unchanged.

**Where a generated secret lives**, the question the previous handoff left open:
`local-store-secrets.json` in the app's own project directory, beside the data
it unlocks, rather than in the registry that lists every app. Uninstalling while
keeping data keeps the file, so a reinstall **reuses the credential** instead of
minting a password the preserved data cannot be opened with. Verified by
disabling reuse and watching the test fail. A secret the plan no longer declares
is dropped; a missing answer stops the install before Docker is touched.
`docs/security-and-privacy.md` records the storage location and that the values
also appear in that app's Compose file, as in any Compose deployment.

**Not reachable by a user.** No command, no interface, and the three reviewed
recipes still install from their own Compose file. Switching them over is
provably behaviour-preserving — a plan renders their Compose byte-for-byte — but
it is a separate change needing its own lifecycle evidence. 183 Rust tests, 44
Playwright and all Python and licence/MSRV gates pass. No new crate, commit or
push.

Previous batch: **the Runtipi importer now emits `PlanTemplate`s**. It reads
`config.json` form fields — `type: random` becomes a generated secret, other
types become typed fields, `type: password` is marked sensitive so an interface
masks it — and supplies the platform values a loopback install can answer for
itself (`APP_PROTOCOL` is `http`, `APP_PORT` the published port,
`APP_DOMAIN`/`APP_HOST`/`LOCAL_DOMAIN` are `localhost:<port>`). Fields an app
declares but never uses are dropped rather than shown.

**Measured, not asserted: importable apps went 18 -> 45 of 250.** The old
181-app `environment placeholder` blocker is gone, replaced by `platform
placeholder` at 96 — of which **`TZ` alone is 76**. Regenerate
`docs/runtipi-import-report.md` with `python scripts/runtipi_import_report.py`;
the blocked count moving is how progress is reported here, not a count of apps
called supported. 179 Rust tests, 44 Playwright and all Python and licence/MSRV
gates pass. No new crate, commit or push.

Previous batch: **typed setup fields and generated secrets (roadmap phase 2)**.
`src/setup.rs` adds a `PlanTemplate` pairing a plan that carries `${KEY}`
placeholders with the fields a person answers and the secrets generated for
them, resolving both into an installable `DeploymentPlan`.

The random-source decision the previous handoff asked for: **`getrandom`**, the
OS CSPRNG. It was already in the graph through Tauri, is MIT OR Apache-2.0 and
its MSRV of 1.85 is below ours, so neither the licence count nor the crate count
moved. Secrets use rejection sampling; `byte % 62` would favour early letters.
Resolution is all-or-nothing, so a literal `${...}` can never reach a container;
secrets can be **supplied rather than regenerated**, which is what keeps a
preserved-data reinstall working; a default its own field would reject fails
validation rather than one unlucky install; and a secret value never appears in
an error message, which is tested. 176 Rust tests, 44 Playwright and all Python
and licence/MSRV gates pass. No new crate, commit or push.

**Not yet wired:** the importer still reports `${...}` as *needs input* rather
than emitting a template, so the 181-app figure in the import report is
unchanged. The capability exists; the mapping does not.

Previous batch: **read-only Runtipi importer (roadmap phase 3)**.
`src/importers/runtipi.rs` maps Runtipi's normalized `docker-compose.json` onto
a `DeploymentPlan` or reports precisely why it cannot, split into *refused*
(deliberately not expressible), *not modelled* (a plan-model gap) and *needs
input* (typed fields and secrets that do not exist yet).
`scripts/runtipi_import_report.py` regenerates
[the report](runtipi-import-report.md) from the archive already pinned and
cached for the catalog build; nothing is downloaded, installed or promoted.
**18 of 250 definitions map cleanly**, and the blocker table is the point: 181
apps are blocked on supplied values alone, which makes typed setup fields and
secrets the single biggest lever — more than every unmodelled container setting
combined. 167 Rust tests, 44 Playwright and all Python and licence/MSRV gates
pass. No new dependency, commit or push.

Previous batch: **normalized multi-service deployment plan (roadmap phase 2)**.
`src/plan.rs` models services, images, environment, storage and one published
endpoint, and renders Compose instead of carrying it. Its gate is that a plan
built from each of the three shipped recipes renders **byte-for-byte** identical
Compose, so the model is proven against known-good data rather than asserted;
`tests/plan_compose.rs` additionally has Docker resolve a web-app-plus-database
plan and confirms the database is published nowhere. Structural rules replace
re-reading YAML: exactly one published port per plan, host port 1024 or above,
pinned tags, mounts confined to the project directory, declared-and-used volumes
and real dependency targets. `choose_free_port` keeps a documented port when
free. **Nothing installs from a plan yet** — recipes still use their own Compose
file, and internal secrets and typed setup fields remain. 161 Rust tests, 44
Playwright, all Python and licence/MSRV gates pass. No commit or push.

Previous batch: **read-only recovery inventory (14/15)**. `src/recovery.rs`
and `local-store recovery [--json]` list retained Compose files for the three
supported recipes absent from the registry. No Docker calls, migration or writes.
Candidates are not confirmed orphans; ownership is explicitly unverified. The
scanner excludes registered IDs, fails on corrupt/legacy-only registry state,
and checks resolved project/file confinement. Two integration tests pass,
including a real CLI run with empty PATH and no config directory creation;
strict Clippy passes. The full UI and broad test suite were not rerun. No push.

Previous batch: **interrupted-install recovery evidence (14/15)**. The opt-in
`tests/install_interruption.rs` passed against real Memos twice, including the
final recovery message assertion. It kills the actual installer after healthy
startup while registry commit is held, proves the unregistered container survives,
then recovers using the retained Compose file without deleting data. Reinstall
and explicit deletion pass. Clippy and 27 runtime tests pass. See
[interrupted install recovery](interrupted-install-recovery.md). The port error
now points to retained setup files when present. Automatic discovery/recovery of
unregistered containers remains unimplemented. No commit or push.

Previous batch: **cross-process lifecycle locks and uninstall commit safety
(14/15)**, detailed below. Real subprocess contention tests, 27 runtime tests,
strict Clippy and a freshly built Memos lifecycle rerun pass. Evidence:
[locking rerun](evidence/recipe-lifecycle-locking-windows-2026-09-07.json).
No new dependency, UI changes, commit or push in this batch.

Previous batch: **real container lifecycle proof (12/14/15)**. All three
recipes passed on Windows with Docker Desktop's Linux/amd64 engine. See
[recipe lifecycle proof](recipe-lifecycle-proof.md) and its checked-in JSON
report for the exact image IDs, binary hash and checkpoints. An IPv6-first
localhost bug was found and fixed in the health probe, with a real listener
regression test. Version upgrades and other host platforms remain unverified.

The owner's connection checks, guarded retry/removal behavior, port selection,
recipe schema, data-confinement tests, MSRV and license gates, and security
write-up were reviewed and preserved. This continuation completed:

1. **My Apps lifecycle UX (25).** `src/js/readiness.js` checks eligible displayed
   apps 15 seconds after a round completes, with at most four requests active.
   It pauses on navigation/hidden documents, skips busy or stopped managed apps,
   and ignores old app/address/operation replies. It reuses `app_readiness`,
   changes status text in place, and does not poll Docker container state.
   Unknown/errors fall back to the existing status. Removing a row restores focus
   to a neighboring action or the empty-state action; failed connection removal
   can retry without duplicate requests or destructive uninstall calls.
2. **Recipe manifests (11).** Schema 3 declares Linux Docker, Compose v2, local
   storage and container platforms, with upstream tag metadata cached under
   `catalog/recipe-platforms/`. The offline checker rejects tag/platform/digest
   drift. `--refresh` explicitly fetches public metadata; it never pulls layers
   or silently approves a changed image. See [recipe requirements](recipe-requirements.md).
   These are compatibility declarations, not host/lifecycle proof. Compose still
   uses the existing version tags; index digests are recorded audit evidence.
   URL validation now checks the actual loopback port (52300 cannot pass for
   5230), and remapping preserves matching numbers in health paths/queries.
3. **Release workflow hardening (29, partial).** Build jobs now depend on quality
   **and minimum-rust**. Explicit targets: Windows x64 on windows-2022, macOS
   Apple Silicon on macos-15, Linux x64 on ubuntu-24.04. Tests/builds use the same
   target; Tauri CLI is pinned to 2.11.4. `scripts/prepare-release.py` stages
   installers, target-named CLI binaries, SHA-256 lists and unsigned build JSON.
   Missing installers, duplicate names, stale staging directories and out-of-root
   inputs are refused. A tag must match the configured version. Only the release
   job has write permission. **The changed matrix has not run remotely.** See
   [PUBLISH.md](../PUBLISH.md); do not call unsigned metadata an attestation.
4. **Accurate privacy scope.** The offline launcher is distinguished from remote
   app pages, which can make their own requests. WebView profiles, Docker storage
   and OS registrations are acknowledged. Native privilege review remains partial.

## Existing foundations: reuse these

- `src/activation.rs`, `src/native.rs`: shared strict activation parsing; official
  single-instance 2.4.4 and deep-link 2.4.10 plugins. Single-instance stays first.
  URI delivery is already handled by its deep-link feature; do not dispatch it
  twice. Native work runs off the UI loop, coalesces opens and queues errors for
  the launcher. Ordinary CLI commands stay independent processes.
- Every IPC command needs its handler, `build.rs` declaration, capability grant
  and runtime launcher guard. `tests/capabilities.rs` checks consistency.
- Bounded process runner, redaction, transaction-wide per-app operation token,
  checkpoint cancellation through commit, rollback preservation, port preflight,
  Compose validation, lifecycle diagnostics, and confined deletion already exist.
  Cross-process registry and per-app Docker operation locking are implemented
  for processes sharing the same configuration root.
- Cancellation cannot interrupt an in-flight Compose command; its deadline still
  applies. Once saving/rollback begins, the Stop setup control is withdrawn.
  A failed cleanup blocks blind reinstall retry. Arbitrary external data locations
  would require redesigning ownership guards and are not a small setup option.
- The MSRV is honestly 1.88.0. Earlier work verified all targets on that toolchain;
  CI enforces it. License checking uses the owner's existing SPDX evaluator.
- Approved 512px logo: tile `(193,95)`, size 224, radius 56; L stroke 56; gaps 42;
  shared facing-corner center `(249,263)`, radii 56/98; aligned top y=95.

## Completed since the last handoff

### The recovery action (14/15)

Recovery could inspect but not act: the interruption test cleared its own
orphan by typing `docker compose down`, which meant the product had no answer
for a person in the same position. `recovery::discard` is that answer, exposed
as `local-store recover <recipe-id> [--delete-data]`.

Everything the inventory is careful about applies here twice over, because the
candidate list a person was shown is a snapshot. By the time they click, the
app may have been installed by another window, the containers may have gone, or
something else may have taken the project name. So none of it is trusted: the
app's operation lock is taken first, the candidate is re-derived under it, and
Docker ownership is re-verified before anything is removed.

What it refuses, each with a test:

- **An ownership mismatch.** Containers under this project name whose labels
  point at a different Compose file belong to somebody else; Docker is not
  asked to stop them.
- **An app that is properly installed.** `inspect_at` skips anything the
  registry lists, so a real app is not a candidate and cannot be discarded by
  this path — it says to uninstall instead.
- **A busy app.** The lock is taken before anything is read, so Docker is not
  touched at all.
- **A removal Docker did not complete.** A zero exit with containers still
  present is reported rather than claimed as success.
- **Deleting outside the managed root.** Data deletion goes through
  `confined_to_managed_root`, and containers are removed before files so a
  failure leaves the recoverable order.

Discard, not adopt. These files come from a transaction that never committed —
no health confirmed, no registry entry written — so the honest exit is to clear
them and let an ordinary install proceed, not to register an app whose install
nobody finished. Adoption is still open and would need its own health story.

Seven unit tests plus the real interruption test, which now recovers through
this command: it removes exactly one container, keeps the data by default with
the marker file unchanged, then refuses the same command once the app is
properly installed. Passed in 22.69 seconds; unrelated containers on this
machine were untouched. Evidence:
`docs/evidence/recovery-action-memos-windows-2026-09-08.json`.

`recovery` itself stays read-only. An inventory command that could also delete
things is too easy to run by accident, so the action is a separate verb.

### Task 12 closed: an upgrade that carries the person's data

`tests/recipe_upgrade.rs` installs Memos 0.29.1, creates an account and a note
**through the running application's own API**, does a keep-data uninstall,
installs the shipped 0.30.0 over the preserved data directory, then signs in
with the same credentials and reads that note back. Passed in 24.46 seconds.

The account surviving proves the user table migrated; the note surviving proves
the content did. Neither is implied by a health probe — a Memos that quietly
created an empty database answers on its port exactly like one that migrated an
existing file, which is why "the new image starts" was never enough for this
task.

Verified non-vacuous: changing the keep-data uninstall to discard data makes
the test fail on the missing note rather than passing anyway.

Evidence: `docs/evidence/recipe-upgrade-memos-windows-2026-09-08.json`. Scope is
one recipe across one minor version; major-version upgrades and the other two
recipes are not covered, and the file says so.

### Task 16 closed; 17, 27, 29 and 31 advanced by a real installer run

**A clean Windows installer, from a machine with nothing installed.** Built the
NSIS bundle, verified the baseline had no prior install and no registered
scheme, installed silently, and checked what actually landed: the binary, the
uninstaller and every bundled notice, licence and catalog resource; a Start
Menu shortcut that launches the launcher; both `localstore://` and the legacy
scheme registered to the installed binary. Cold-start activation launches one
process and opens the named app. A second activation on either scheme reuses
that process. A malformed deep link is refused with the expected form and exit
2. Silent uninstall removes the tree, the shortcut and the Add/Remove entry,
and leaves the user's app registry alone. Evidence:
`docs/evidence/windows-installer-2026-09-08.json`.

The packaged build also read a registry migrated from the pre-rename legacy
format and took `registry.lock` — task 31's migration proof, done by the
installed binary rather than a test harness.

One correction is recorded in that evidence file: the schemes appeared to
survive uninstall, and they do not. This session's shell reads a virtualised
HKCU (paths resolve under `Packages\Claude_*\LocalCache`), so registry state
after uninstall could not be trusted. `installer.nsi` deletes both keys in its
uninstall section; that is read from the script, not observed running, and the
file says so.

**Task 16 is complete.** Navigation policy was a closure inside the window
builder, which is why it had no evidence. It is now `decide_navigation`, a pure
function with tests: an app window may move within its own origin, any other
origin opens in the OS browser instead of inside a window titled after the
person's app, and `file:`, `javascript:`, `data:` and custom schemes are
refused outright rather than handed to the OS. App windows hold no IPC
permission — the capability grant names only `launcher`.

**Advisory scanning (29).** `scripts/check-advisories.py` runs cargo-audit and
refuses to pass on a missing tool or an unfetched database, because a scan that
checked nothing reads like evidence that it did. Every accepted finding needs a
recorded reason, and an acceptance that no longer matches a real finding fails
the gate — verified by adding a fake one. Seven advisories are accepted in
`catalog/advisory-policy.json`, all transitive through Tauri's GTK and
urlpattern chains, none a vulnerability class, none at a version this project
can choose.

**Authenticated provenance (29).** The release job now attests every published
file with `actions/attest-build-provenance` and the OIDC permissions it needs.
PUBLISH.md documents `gh attestation verify` and says plainly what it is not:
provenance proves which workflow built a file, not that an identified publisher
vouches for it, and Windows will still warn.

**Task 30 stays not implemented, deliberately.** The prerequisites are written
down in PUBLISH.md with the exact commands. No placeholder key was added — a
fake `pubkey` produces update artifacts nobody can verify, which is worse than
having none — and an agent should not generate the key that becomes the release
identity.

### The reviewed-template allowlist, and a review that said no

`src/templates/` is the allowlist the last two handoffs asked for. A reviewed
template carries the pinned upstream definition verbatim, the provenance to
fetch it again, an audit of every image it runs, a named lifecycle proof, and
the review decisions somebody made about it. `ReviewedTemplate::plan_template()`
maps it through the importer and applies those decisions, or refuses.

**What the review is actually for.** `caprover::setup_variable` marks every
field sensitive, because an upstream catalogue has no way to say which of its
variables is a credential and masking a time zone is the safer mistake. The
consequence, once the setup form existed, was that a time zone would render as
a masked box with its default withheld. A review is where somebody who read the
app says which fields are credentials — `a_review_decides_which_answers_are_credentials`
asserts CodiMD's time zone comes back as a text control showing `Europe/London`.

Both directions are checked, so neither half can drift: a field upstream adds
arrives unreviewed and stops the template, a review left behind by a removed
field stops it too, and the same holds for images.

**The review said no.** Both CodiMD images were last rebuilt in **August 2020**.
Upstream development moved to HedgeDoc, so no fix is coming. The template is
correct and its lifecycle is proven; the app is not one to hand somebody. That
outcome is recorded in the manifest as `promotion.state: "withheld"` with the
reason, and `no_reviewed_template_is_offerable_without_an_explicit_approval`
fails the moment anything is marked approved — so a promotion has to appear in
a diff rather than happening quietly.

A review that can only say yes is not a review. The mechanism now records
either answer.

**Images could not be audited the way recipes are.** The recipe checker pins a
tag's index digest from Docker Hub. These repositories publish no index digest
for a tag, only one per architecture, and a multi-service template has several
images rather than one. `scripts/check-template-platforms.py` checks what is
actually published: every architecture's digest, the platform list, and the
tag's last rebuild date, all offline against a cached copy. It also fails when
a manifest names a lifecycle proof that is not in the tree.

**The lifecycle proof now proves the reviewed artefact.**
`tests/caprover_lifecycle.rs` no longer carries a transcribed definition; it
loads `templates::reviewed_template("codimd")` and installs whatever that
resolves to. Passed in 70.58 seconds, no leftover container, network or volume.
Evidence: `docs/evidence/caprover-codimd-windows-2026-09-08.json`.

Full run: 178 library tests, 20 test binaries, 52 Playwright tests, strict
Clippy, every offline gate including the new template check, recipes
revalidated through Docker's own parser.

### The setup form, and answers carried into the install

The previous handoff left the setup-review projection built and nothing
rendering it. `src/js/setup-form.js` now renders it, and `install_app` carries
what a person types.

**The form.** Built with DOM nodes rather than markup, because every label,
option and default in it is upstream text and interpolating that into HTML
would let an app's own definition write the page.
`an_app_label_cannot_write_markup_into_the_review` puts an `onerror` payload in
a label and fails if it executes; reintroducing an `innerHTML` assignment was
confirmed to fail it. Sensitive fields render as password controls with
autocomplete off, and their defaults are described rather than shown, because
the projection never sends them. Each optional field says what leaving it blank
does, in the words a person would use: *Leave blank to use admin@example.com*.
Generated credentials are stated as a count and never displayed or asked for.

**The answers.** Read from the form at the moment of sending and kept nowhere
else — not in `state`, not in a saved dialog, not in a diagnostic. Blanks are
omitted rather than sent as empty strings, so the backend applies its own
default instead of being handed a value that means something different.
`answers_do_not_survive_the_dialog_they_were_typed_into` types a password,
cancels, reopens, and fails if that value can be found anywhere reachable.

**The backend.** `install_app` gained an optional `answers` map and validates it
through `PlanTemplate::accept_answers` before any lock is taken or file
written — which refuses a key no field asked for, so a client cannot supply an
environment value the review never showed anyone. A recipe with no typed setup
still installs from its own reviewed Compose file byte for byte, through
`begin_install`; only a template that actually declares fields goes through
`begin_template_install`. `template_for` is one function so the setup a person
reviews and the setup an install enforces cannot drift apart.

Seven Playwright tests and one Rust test at the command boundary. Full run: 173
library tests, 20 test binaries, 52 Playwright tests, strict Clippy, every
offline gate, recipes revalidated through Docker's own parser. No reviewed
baseline changed, because no reviewed recipe declares a field yet.

### A regression fixed, and the public-domain question answered in part

**The regression, which this session introduced.** Making the host port a
preference rather than a promise (so a busy port no longer fails an install)
broke an assumption the Runtipi importer was built on: it baked
`localhost:<port>` into environment values at *import* time, using the port the
plan asked for. Once the installer could choose a different one, 59 Runtipi
definitions could tell an app it lives at an address it does not answer on. The
app installs, reports healthy, and hands out links that go nowhere — the kind
of failure nobody finds until a user clicks one.

The address is now settled where it is actually known. `src/setup.rs` reserves
`LOCAL_STORE_URL`, `LOCAL_STORE_HOST` and `LOCAL_STORE_PORT`; importers emit
those placeholders instead of a guess, and `install_template_with` fills them
after the host port is chosen and before any answer is resolved. A field or
secret may not claim one of those names, and `PlanTemplate::validate` treats
them as sourced rather than demanding a declaration.
`an_app_is_told_the_port_it_actually_got_not_the_one_it_asked_for` installs
onto a busy preferred port and fails if the app is told the wrong one;
reintroducing the bug was confirmed to fail it.

**The public-domain question, decided on evidence.** It was the largest single
blocker at 101 apps and was written up as needing a product decision. Looking
at what the 202 references actually do split them cleanly:

- **145 are the app's own address** — `https://$$cap_appname.$$cap_root_domain`
  or the bare host. That is precisely what a loopback install has, once the
  port is known, so it is now rewritten to the placeholder above. No decision
  about hostnames or certificates was needed for these.
- **The rest name a sibling's public address** (`$$cap_appname-web`,
  `$$cap_appname-storage`), or carry a scheme we cannot honour (`wss://` over
  plain loopback), or a mail domain, or a comma-separated host list. Those stay
  refused. Rewriting them would point an app at itself or silently change a
  scheme, and being wrong there is worse than importing fewer apps.

The rewrite is deliberately all-or-nothing: it fires only when the whole value
is the app's own address, and only for an environment value. A `hostname:` of
`localhost:8080` is not a valid host name, so those stay refused.

**CapRover 95 to 117**, and `public domain` fell from 101 apps to 46 — the 46
that genuinely need a second routable endpoint. Runtipi stays at 95: the change
there bought correctness, not coverage.

Both Docker lifecycle proofs were re-run afterwards, because
`install_template_with` changed: CodiMD in 64.79s and Adminer/Postgres in
33.19s, no leftover container, network or volume.

### The three measured import wins: 73 to 95, and Runtipi 83 to 95

All three items the previous handoff listed under priority 1 are done, each
measured by regenerating the reports rather than estimated.

**Generated secrets carrying a validation rule (+18 apps).** Upstream attaches
a `validRegex` to a generated credential in 213 variables, and the pair used to
be refused outright because nothing proved the generated value would satisfy
the rule. `SecretSpec::every_value_matches` now decides that question:
`src/setup.rs` reasons about the generator's own alphabet and answers only for
a narrow shape — one character class carrying one quantifier, optionally
anchored — refusing everything else as unproven. It is sound by construction
and deliberately incomplete, because a wrong yes would hand someone a
credential their own app rejects. `/^\d+$/` stays refused: a hex value can be
all letters. `a_proven_rule_accepts_every_value_the_generator_can_produce`
takes the eight rules that actually appear upstream and puts 2,000 real
generated values through each, so the proof is checked against 16,000 values it
claims to cover.

**Hyphenated variable identifiers (+6 apps).** `$$cap_mariadb-db` folds to
`CAP_MARIADB_DB`. Collision detection runs on the *folded* key, so
`$$cap_a-b` and `$$cap_a_b` are caught rather than one silently overwriting the
other.

**`command`, `entrypoint` and `hostname` are modelled (+6 CapRover, +12
Runtipi).** `PlanService` gained one `overrides: PlanOverrides` field.
`PlanArgs` keeps Compose's two argument forms apart on purpose: a bare string
is split the way a shell would split it and a list is not, so collapsing them
would change what the container runs. Both importers map them; neither drops
them.

Reports after this batch: **CapRover 95 of 356, Runtipi 95 of 250.** CapRover
variables refused fell from 372 to 190.

One reporting fix went with it: 27 of the 30 upstream `hostname` values are
`$$cap_appname.$$cap_root_domain`, and they were being reported as an
unreadable hostname rather than as the public-domain boundary they are.
`public domain` is now correctly the largest single blocker at 101 apps.

`tests/caprover_lifecycle.rs` was re-run after the plan model change and passed
in 71.78 seconds with no leftover container, network or volume.

### The CapRover deployment adapter (checklist items 1-9)

`src/importers/caprover.rs` now imports a whole definition, not only its setup
variables. `caprover::import(id, definition)` returns a validated
`PlanTemplate` or a list of named limitations. `Limitation` and `ImportOutcome`
were lifted into `src/importers/mod.rs` so both importers share one vocabulary
and one report shape, and `preferred_host_port` moved with them.

**Measured by the real adapter over all 356 pinned definitions: 73 expressible,
283 blocked.** `python scripts/caprover_import_report.py` regenerates
[the report](caprover-import-report.md) from the checksum-verified archive.
Thirteen unit tests cover the simple case, the web-plus-database case, eight
named refusals and the quoted-boolean regression below.

Four things the implementation found that the study's structural scan could not:

- **`$$cap_root_domain` is the largest single boundary**, in 217 of 356
  definitions. CapRover gives every app a routable public hostname and apps
  build URLs, callbacks and cookie domains out of it; a loopback install has no
  such name. Substituting `localhost` would produce links that resolve for
  nobody and break silently, so those apps are refused under their own
  `public domain` heading rather than approximated. That is a product decision
  about hostnames, certificates and proxying — not a parser gap, and not
  something an importer should decide.
- **`notExposeAsWebApp` is written as the quoted string `'true'`**, not a
  boolean. Reading only the boolean form made every database look like a second
  web endpoint. Had the endpoint rule not refused those apps first, that would
  have published a database to the host. An unrecognised value is now refused
  rather than assumed, and
  `web_exposure_is_read_whether_it_is_a_boolean_or_the_quoted_string` fails if
  the assumption comes back.
- **Endpoint refusals were cascading.** A service dropped for an earlier reason
  took its endpoint with it, and the app was then *also* reported as having no
  endpoint — one problem counted twice, with the real cause hidden. Endpoint
  refusals fell from 142 to 51, which matches the 35 apps with no web service
  and 23 with more than one.
- **157 upstream defaults are YAML numbers or booleans**, not strings, and the
  setup primitive refused all of them. A port or a version is the same
  environment string either way, so integers and booleans are now accepted.
  Fractional numbers still are not: YAML reads `13.10` as `13.1`, and
  installing that as an image tag would fetch a different release. Refused
  variables fell from 520 to 372.

The shared plan rules earned their keep: `PlanTemplate::validate` caught the
adapter emitting a setup field for a version variable that only ever reached
the image tag, where no answer could have changed anything. Fields and secrets
that nothing references are now dropped, which is also what the checklist asked
for in item 5.

### One imported app, installed for real

`tests/caprover_lifecycle.rs` takes the pinned CodiMD definition through the
adapter and then through the shared install transaction, with real containers.
**Passed in 134.54 seconds**; report at `.cache/<id>/report.json`, no leftover
container, network or volume.

CodiMD was chosen because it exercises what the adapter actually had to get
right: two services; one generated credential that MariaDB bakes into its
volume on first start and CodiMD must present on every start after; a
`srv-captain--` reference that only resolves if it was rewritten to a Compose
service name; and `notExposeAsWebApp: 'true'` as a quoted string. Checked with
containers running: the typed timezone answer reaches the container, both
services present the same credential, that credential authenticates against
MariaDB over TCP, the database has no host port binding, a row written under
the first credential survives stop/start, a keep-data uninstall preserves both
volume and credential, the reinstall reuses the credential and reads that row
back, and explicit deletion removes everything owned while unrelated containers
are counted before and after and survive.

Scope: one imported definition through the shared plan transaction. It is not a
reviewed recipe, and 73 expressible definitions remain 73 candidates.

### Multi-service plan lifecycle, proven against real Docker

`tests/plan_multi_service.rs` is the phase 2 gate from the roadmap: "a web app
plus database installs without hand-editing Compose and survives
restart/removal tests". It installs Adminer beside Postgres from a single
`PlanTemplate` and checks, with real containers:

- the typed setup answer reaches the database (`printenv POSTGRES_DB`);
- the database is **not** published to the host, while both services still
  share the project network;
- the generated password authenticates over TCP, not via the image's local
  socket trust rule;
- a row written under the first password survives stop/start;
- a keep-data uninstall preserves both the volume and the credential;
- the reinstall reuses the password and reads that row back — the failure that
  would otherwise look like data loss;
- explicit deletion removes every owned container, network and volume, and
  containers owned by anything else are counted before and after and survive.

Opt-in like the other Docker tests (`#[ignore]` plus
`LOCAL_STORE_RUN_DOCKER_TEST=1`). It runs in 36 seconds and cleans up on the
way out even when an assertion fails part-way, through a label-scoped `Drop`
guard. Report written to `.cache/<id>/report.json`.

### Install-time port assignment

A plan's host port is now a preference rather than a promise. A first install
steps past a busy port (`plan::choose_free_port`, which existed and was wired
to nothing); a reinstall reuses the port recorded in the retained Compose file
so a saved link keeps working, and reports a conflict rather than quietly
starting a second copy beside a running one. Four unit tests cover those cases;
reintroducing the bug was confirmed to fail
`a_reinstall_keeps_the_address_the_app_already_had`.

This is what makes privileged container ports importable: the container keeps
port 80 and the host gets 8080.

### Two importer gaps closed

- **`readOnly` mounts.** `PlanMount` carries `read_only`, rendered as `:ro`.
  This removed a named gap for 35 Runtipi definitions but unblocked **zero** on
  its own — every one of them had another blocker too. Recorded honestly
  because the count in the old handoff implied otherwise.
- **Privileged container ports.** Runtipi importable went **60 → 83**.

### The source study

[docs/import-sources-study.md](import-sources-study.md) answers "where do the
next few hundred apps come from" with measurements rather than reputation.
Headlines: CapRover is the best-fitting source by a wide margin; Umbrel and
Easypanel have **no licence file** and cannot be used at all; Portainer's 684
templates pin only 27 of 431 images; and 37 of 178 CasaOS manifests ship
hardcoded credentials that would be identical on every installation, so any
CasaOS importer must treat credential-shaped literals as secrets to generate.

### Cross-process registry locking (14)

Saving the registry was already atomic, so the file never tore — but atomicity
does not prevent a **lost update**. `insert_installed_app`,
`remove_installed_app` and `update_installed_app` each read the registry,
changed it and saved it; two processes doing that at once meant whichever saved
second silently discarded the other's app.

**This was reproduced, not assumed.** With the lock removed, eight concurrent
`local-store add` processes left five apps in the registry — `app-1`, `app-3`
and `app-6` were gone. With it, all eight survive. `tests/registry_contention.rs`
holds that evidence and fails if the lock is taken out again.

- `fs4 = "=1.1.0"` provides the lock: MIT OR Apache-2.0, MSRV 1.75, and its
  default feature is `sync`, so no async runtime is pulled in. Both the licence
  and MSRV gates pass with it added.
- The lock is a **sidecar** `registry.lock`, never the registry file itself:
  saving replaces that file by rename, which would discard a lock taken on it.
- The wait is bounded at ten seconds and then reports a busy registry — mapped
  to the existing `operation_busy` code, which the launcher already explains —
  rather than hanging. The operating system releases the lock if a holder dies.
- Read paths lock **only when a registry directory already exists**. An earlier
  version locked unconditionally, which made a failed activation create config
  on disk and broke
  `native_open_missing_app_fails_without_creating_registry_or_starting_gui`.
  That test was right: looking up an app that does not exist must leave the disk
  untouched. Do not reintroduce unconditional locking on read.

Three subprocess tests cover what was asked: contention across eight processes,
mutual exclusion while a lock is held, and release after the holder is killed
outright. The crash case runs the test binary as its own helper — an ignored
test invoked by name — because a lock's release on process death cannot be
observed from inside the process holding it.

**Lifecycle lock continuation:** `OperationLock` now also holds an fs4 file lock
at `<config>/local-store/operation-locks/<app-id>.lock`. IDs are validated before
filesystem access. Sidecars are outside app directories and never unlinked, so
deletion/reinstall cannot replace the locked file. Acquisition is nonblocking;
competing processes receive `operation_busy`. Different app IDs remain independent.
The existing cancellation token and operation ID stay process-local.

`uninstall_and_remove` keeps the lock through Docker cleanup **and registry
removal**, used by both the CLI and desktop command. Previously the registry
removal happened after unlock, allowing a reinstall to commit in between.
Installation already keeps `PendingInstall` locked through commit and rollback.

`tests/operation_contention.rs` uses real subprocesses to prove same-app
exclusion, independent apps, CLI install/start/stop/uninstall rejection before
Docker, unchanged registry on refusal, and OS release after killing the holder.
The helper is ignored during normal discovery and invoked explicitly by the test.
This proves lock release, not recovery from killing a real in-flight Docker
command. Separate configuration roots and manual Docker commands are outside
this coordination boundary.

## Validation and limits

Latest local checks on Windows:

- Latest batch, run in full on Windows: 20 Rust test binaries green (166 library
  tests), 45 Playwright tests, strict Clippy clean, `cargo fmt --check` clean,
  every offline gate script passing, all three recipes revalidated through
  Docker's own parser, and `tests/plan_multi_service.rs` passing against real
  Docker in 36 seconds with no leaked containers, networks or volumes.
- Latest Docker batch: all three recipes passed 13 real CLI/container checkpoints
  each; the IPv4 localhost regression and 27 runtime tests pass, strict Clippy
  passes, formatting and Python syntax checks pass. No UI files changed.

- **147 Rust tests**, strict all-target/all-feature Clippy and formatting pass.
  Three of those are the new subprocess registry-contention tests, which take
  about ten seconds because two of them wait on real process lifetimes.
- One new runtime dependency since the last handoff: `fs4 = "=1.1.0"` for
  cross-process file locking. Licence and MSRV gates pass with it.
- **21 Node tests and 44 Playwright tests** pass together; existing visual
  baselines unchanged. One old test assumed Open was the last IPC call; it now
  asserts the exact Open request independently of background readiness traffic.
- **18 Python tests** pass: eight catalog, four recipe-platform, six release
  staging tests. MSRV, SPDX self-test/graph checks, offline catalog/icons/brand,
  image metadata and all three Docker Compose configuration checks pass.
- Windows sandbox process restrictions can hang Playwright preview teardown.
  Running with normal process permissions completed cleanly in 45 seconds.
  Do not add retries or weaken assertions; stop only a verified test-owned preview
  process if sandbox cleanup is blocked.
- Earlier Windows release-image smoke proved startup, second launch forwarding,
  window reuse, both registered protocols and remote ACL denial for four commands.
  `scripts/smoke-native-windows.ps1` backs up/restores protocol handlers and uses
  private app data; close Local Store before running it. Report details are in
  the archive. That evidence does not prove a clean installer installation.
- **The multi-service proof covers a plan transaction, not an application.**
  It proves Postgres keeps its password and its data across a reinstall; it does
  not prove any particular app migrates its schema, and it exercises one
  two-service shape rather than the 3-to-7-service stacks common in the Coolify
  and CapRover catalogues.
- **The setup form has never rendered a real app's fields.** No reviewed
  recipe declares typed setup, so every test of it drives a projection the
  fixture supplies. The form, the validation and the answer plumbing are
  exercised end to end; what has not happened is a person installing an app
  that actually asks a question. That needs a reviewed template, which needs
  the provenance and platform review below.
- **The allowlist exists and is deliberately empty of approved templates.**
  CodiMD is reviewed, audited, lifecycle-proven and **withheld**: its images
  were last rebuilt in August 2020. Nothing is offerable, so `recipe_details`
  and `install_app` were left accepting reviewed recipes only. That is a
  decision, not an omission — a code path no user can reach would be worse.
- **A withheld template is still loaded and resolved.** `reviewed_templates()`
  returns CodiMD and its plan is built and validated on every test run. That is
  what keeps the mechanism honest, but it does mean the bundled binary carries
  a definition for an app nobody may install. If that is unwanted, the fix is
  to drop the manifest, not to mark it approved.
- **The CapRover count of 117 is expressibility measured by real code**, unlike
  the study's earlier 180, which was a structural scan and is now corrected in
  place. It is not a count of working apps: exactly one imported definition
  (CodiMD) has been installed and run, and the other 116 have had no lifecycle
  run at all. The same applies to Runtipi's 95. None of them is a reviewed
  recipe.
- **`every_value_matches` is sound but incomplete on purpose.** It proves a
  narrow shape and refuses everything else, so a rule it cannot decide is
  reported as unproven rather than accepted. It is checked against 16,000 real
  generated values, but that is a cross-check of the proof, not the proof
  itself: the guarantee comes from the reasoning about the alphabet, and the
  values are there to catch a mistake in it.
- **The CodiMD proof uses a transcribed definition, not the archive.** This
  crate has no YAML reader, so `tests/caprover_lifecycle.rs` embeds the pinned
  `codimd.yml` normalized to JSON. The adapter is checked against real upstream
  files by the report; the test checks that the adapter's output runs. If the
  pinned revision moves, that fixture needs re-transcribing.
- **An app that imports is not an app that works.** The own-address rewrite
  makes 55 more CapRover apps expressible, and some of them will still be
  wrong in ways only a real run finds: an app that sets a secure cookie, mails
  a link, or registers an OAuth callback needs more than a loopback URL. The
  lifecycle gate is what catches that, and it has been run for exactly one
  imported app.
- **The source study measures expressibility, not working apps.** Its counts say
  a definition can be turned into a plan under our safety rules. Three of the
  four sources were measured by a throwaway scan rather than a real importer, so
  those numbers will move once an importer applies the rules exactly; only the
  Runtipi figure (83) comes from production code. No app named in it has had a
  lifecycle run, and none should be offered as an install on the strength of it.
- **No installer was rebuilt after the latest owner and continuation changes.**
  No remote CI, commits, pushes or new signing credentials. Real container runs
  now pass for all three recipes; see the lifecycle report.
  Earlier `connect-dialog.png` and `install-review.png` owner changes still need
  visual owner review; this continuation did not regenerate them.

## Next work, in priority order

**Updated product direction:** follow [the one-click roadmap](one-click-roadmap.md).
The owner wants broad one-click catalog installation and GitHub-URL import,
with catalog expansion. The original 33 tasks are foundations, not the complete
product goal. Prioritize reusable deployment adapters, multi-service plans and
automated recipe verification; do not build a separate installer for each app.

1. **Windows is the shipping target.** macOS and Linux proof is deferred by
   owner decision as of 8 September 2026 and is listed at the end of
   [the ledger](upgrade-status.md) rather than counted as outstanding. Do not
   reopen it without being asked.

   **Task 30 is deferred too**, by the same decision. Do not generate a signing
   key, do not add the updater plugin, and do not wire an update UI. The owner
   will generate the key after full deployment and keep it secret. Nothing in
   the build references an updater today, which is the state to preserve: a
   placeholder key produces update artifacts nobody can verify. Task 33 cannot
   pass until 30 lands, so it is blocked rather than pending.

   That leaves one owner-blocked item: **29** needs a push to verify the build
   matrix and the provenance step remotely, and publication is paused.

   Everything achievable on this host has been done. Do not mark these complete
   from a Windows machine or an unpushed branch.

2. **Find a candidate worth approving.** The machinery is done and proven end
   to end; what is missing is an app that deserves to be offered. CodiMD was
   reviewed and withheld — correct template, proven lifecycle, images five
   years stale — which is the mechanism working, not a setback.

   Image recency is the criterion that decided it, and it should be applied
   first next time rather than last. `scripts/check-template-platforms.py`
   prints each tag's last rebuild date; a candidate whose images have not been
   rebuilt within a year or so is not worth the rest of the review.

   [The import report](caprover-import-report.md) lists what each candidate
   asks for, so an app can be judged on whether its questions are answerable.
   Prefer a maintained single-service app with one or two meaningful fields —
   `linkding`, `node-red` and `joplin` all look plausible and none has a
   required field. Then: manifest, audit, lifecycle run, evidence, and a
   promotion decision with its reason.

   **Approval is the project owner's call.** Prepare the evidence and ask.
   Marking `promotion.state` approved is what makes an app reachable, and the
   test that currently guards it is meant to be argued with, not edited around.

3. **Wire the commands once something is approved.** `recipe_details` and
   `install_app` still only accept reviewed recipes, deliberately: a code path
   no user can reach is worse than none. `template_for` in `src/commands.rs` is
   the seam, and `PlanTemplate` is already what both sides speak. The setup
   form, its validation and the answer path are built and tested against a
   fixture projection; they have never rendered a real app's fields, and the
   first approved template is what changes that.

4. **Health-check expansion and platform constraints.** Health checks now map
   from CapRover and Runtipi, service_healthy conditions survive both importers,
   and delayed dependency readiness has a real Docker proof. Next address
   container-variable expansion without confusing it with setup substitution,
   then explicit platform constraints. Completion-job dependencies remain
   unsupported. There is still no Coolify importer.

5. **Decide the floating-tag question.** A third of Coolify's image references
   (269 of 813) use `latest`, `main` or no tag, and our pinning rule refuses
   the whole app when one service is unpinned — that single rule accounts for
   217 of Coolify's 239 refusals, more than every privilege refusal combined.
   Resolving a floating tag to a digest once, at import time, would roughly
   triple Coolify's yield. This is a policy decision about who owns the pin,
   not an engineering problem, and it should be made deliberately.
6. **Product recovery flow (14/15).** Healthy pre-commit installer interruption
   and explicit data-preserving recovery now pass with real Memos. Next add
   the recovery action/UI on the now-verified retained-file inventory.
   Recheck labels, absolute Compose paths and current registry under lock. Present an
   explicit recovery action; do not stop a listener merely because its port
   matches. Keep locks through recovery and registration. Interruption during
   image pull/Compose startup and other applications remains untested.
7. **Native and package proof (31, Windows only).** Clean Windows installer, shortcuts,
   packaged migration, complete navigation boundaries, then macOS/Linux. Image
   platform metadata does not establish desktop installer support.
8. **Release gates (29 only; 30 and 33 are deferred).** After owner publication authorization, inspect
   actual quality, MSRV and all three target jobs. Advisory scanning, authenticated
   provenance, signing/updater and the combined release gate remain. Signing
   needs owner-managed credentials; do not invent them or claim release readiness.
   Also confirm private vulnerability reporting is enabled on GitHub before
   relying on the reporting route in the security document; it remains unverified.

Docker is now operational: Desktop 4.68.0, engine 29.3.1, Linux/amd64. The owner
started it before this batch. Sandbox access still requires normal Docker process
permissions. All test-created containers, networks, volumes and managed app
directories were removed by the successful harness; downloaded images remain
cached. Existing unrelated containers were not changed. The earlier failed
Memos install rolled itself back. Historical socket diagnosis stays in the archive.

## V2 icon pipeline requirements

The V2 catalog icon contract is frozen in `docs/design/v2/ICON-PIPELINE.md`.
Every catalog entry must resolve to a local manifest asset; runtime network
icons and CSS-only initials are not acceptable. Preserve source precedence,
explicit aliases, checksums, validation limits, attribution, original aspect
ratio, `object-fit: contain`, and the deterministic Satin monogram fallback.
Never use or reintroduce the removed Ember Paper treatment. Treat Umbrel gallery
artwork as `NOASSERTION` until permissions or asset-specific licenses are
verified; `catalog/notices/umbrel-apps-gallery/NOTICE.md` is a release gate.
Regenerate and review the audit and contact sheet whenever a pin or mapping
changes.

## Commands for the next agent

Run from the repository root. Use Python with `scripts/requirements-catalog.txt`,
Node/npm and the platform Tauri dependencies. On Windows, Rust scratch can use
`.cache/rust-tmp` via this shell's TEMP/TMP. Keep POSIX browser temp paths short;
do not restore the checkout-relative override that broke Linux Chromium.

```text
git status --short --branch
python scripts/check-msrv.py
python scripts/check-licenses.py --self-test
python scripts/check-licenses.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo test --locked --test registry_contention
cargo test --locked --test plan_compose
python scripts/runtipi_import_report.py    # regenerates docs/runtipi-import-report.md
python scripts/caprover_import_report.py  # regenerates docs/caprover-import-report.md
python scripts/caprover_setup_report.py   # regenerates docs/caprover-setup-report.md
cargo test --locked --release --test cli_packaged
python scripts/test_catalog.py
python scripts/test_recipe_platforms.py
python scripts/check-recipe-platforms.py
python scripts/check-template-platforms.py
python scripts/check-advisories.py   # needs: cargo install cargo-audit
python scripts/test_release.py
python scripts/catalog_pipeline.py --check
python scripts/cache-catalog-icons.py --check
node scripts/generate-brand.mjs --check
python scripts/validate-recipes.py
npm test                                  # includes tests/ui/setup.spec.js
git diff --check
```

The Docker lifecycle proofs are opt-in and start real containers, so run them
deliberately rather than as part of a sweep. Each cleans up what it created and
leaves unrelated containers alone.

```text
LOCAL_STORE_RUN_DOCKER_TEST=1 cargo test --test plan_lifecycle -- --ignored --nocapture
LOCAL_STORE_RUN_DOCKER_TEST=1 cargo test --test plan_multi_service -- --ignored --nocapture
LOCAL_STORE_RUN_DOCKER_TEST=1 cargo test --test caprover_lifecycle -- --ignored --nocapture
LOCAL_STORE_RUN_DOCKER_TEST=1 cargo test --test recipe_upgrade -- --ignored --nocapture
LOCAL_STORE_RUN_DOCKER_TEST=1 cargo test --test install_interruption -- --ignored --nocapture
```

Run UI tests apart from Rust compilation where possible. Windows owns the pixel
baselines; non-Windows runs use `npm test -- --ignore-snapshots`. Test only as
broadly as a concrete remaining risk requires. Update this current summary and
the ledger after a completed batch; append detailed history to the archive only
when it explains an important boundary or tradeoff.

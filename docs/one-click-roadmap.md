# One-click catalog and GitHub installation roadmap

Product direction agreed September 7, 2026: make catalog apps usable with one
click, expand installable coverage, and support installing a project from its
GitHub URL. This extends the original 33-task reliability plan; completing that
plan alone does not deliver broad installation coverage.

## Product contract

After the one-time runtime setup, a compatible verified app should require
Install, then Open. Local Store selects unused ports, prepares persistent
storage, generates internal service secrets, starts dependencies and checks
readiness. Settings remain available without being required for normal installs.

Some apps require an external API key, account setup, license, GPU or device.
Those get a short setup form with the reason stated before downloading. A GitHub
repository may be a library, documentation or an unsupported application; a URL
alone is not sufficient evidence that it can run. Never label an untested import
as one-click verified, or execute README shell commands automatically.

Catalog states: **Install**, **Set up**, **Connect**, and **Unsupported here**.
Show the device-specific reason and last verification evidence. Keep discovery
coverage separate from verified-install coverage and allow filtering to apps
that can run on this computer.

## Architecture: reuse first

1. **Deployment adapters.** Import existing Runtipi, Coolify and CasaOS deployment
   definitions, retaining source revisions and notices. Current catalog imports
   mostly describe projects; extend them to produce deployment candidates.
   Start with Runtipi's Compose metadata and form fields, then add other formats.
2. **One normalized deployment plan.** Extend the current Recipe model to support
   multiple services, dependencies, named storage, generated secrets, required
   user inputs, health checks and an explicit web entry point. Plans carry source
   provenance and verification status. Do not write an installer for every app.
3. **Existing runtime.** Reuse the Docker/Compose runner, registry, cancellation,
   progress, readiness and rollback code. Replace the current single-image,
   single-published-port assumptions deliberately; retain data-ownership guards.
4. **GitHub resolver.** Inspect repository metadata and supported files first.
   Prefer a matching verified catalog recipe, then upstream Compose, then a
   Dockerfile, then a supported source-build provider. Evaluate Railpack for
   source detection/builds rather than implementing language-specific builders.
5. **Verification pipeline.** Imported definitions are candidates until isolated
   lifecycle checks pass. Store outcomes by source revision, image digest,
   architecture and relevant runtime version. Upstream changes require retesting.

References: [Runtipi Dynamic Compose](https://runtipi.io/docs/reference/dynamic-compose),
[Coolify applications](https://coolify.io/docs/applications/),
[Railpack](https://railpack.com/). These are integration candidates; this roadmap
does not claim their formats are already supported by Local Store.

## Phases and acceptance gates

| Phase | Work | Done when |
| --- | --- | --- |
| 1. Prove the foundation | Restore Docker, add cross-process coordination, exercise the three original recipes | Install, readiness, restart, preserved-data reinstall and explicit deletion are proven with isolated test data |
| 2. Generalize deployment plans | Multi-service Compose, automatic free ports, internal secrets, typed setup fields and ownership-aware storage | A web app plus database installs without hand-editing Compose and survives restart/removal tests |
| 3. Import deployment definitions | Runtipi adapter first, with unsupported-feature reports; Coolify/CasaOS follow | Existing upstream definitions become reproducible candidates without per-app installer code |
| 4. Verify 10, then 25 apps | Prioritize common web apps with simple storage and no external credentials | Every promoted app has recorded lifecycle evidence; catalog Install buttons reflect it |
| 5. GitHub import MVP | Public repository/ref parsing; catalog matching; reviewed Compose/Dockerfile paths | Paste URL, inspect detected plan, supply only necessary fields, install and open with clear failure recovery |
| 6. Broaden GitHub builds | Evaluate/integrate Railpack; add monorepo root selection and explicit build settings | Supported source-only web apps build in isolation with reproducible inputs; unsupported repos fail clearly |
| 7. Scale catalog coverage | More adapters, deduplication, scheduled source refresh and automated retesting | Grow toward 100 verified apps, then further based on measured pass rates; retain broad discovery separately |
| 8. Maintain installed apps | Update detection, backup-aware upgrades, rollback and signed launcher delivery | Users can safely keep their apps usable across updates, not just install once |

## GitHub flow

Paste repository URL -> resolve project/ref -> inspect deployable files ->
show detected app, required inputs and resource access -> Install -> build/pull
progress -> health check -> Open. The first import reviews unfamiliar code;
subsequent installs of a verified recipe can use the direct one-click path.

Support public repositories first. Private repositories need a scoped GitHub
authorization flow and secure credential storage later. Pin the selected commit;
do not silently build a moving branch. Builds and apps must not inherit host
credentials, arbitrary host paths, privileged access or the Docker socket by
default. Keep build isolation distinct from merely running the final container.

## Measuring progress

- Verified installable apps per supported architecture, not just catalog size.
- Percentage needing no user configuration after runtime setup.
- Fresh-install and preserved-data upgrade success rates.
- Median time to first usable screen and actionable failure coverage.
- Percentage of GitHub imports resolved through existing deployment definitions.

## Phase 2 progress: the normalized plan exists

`src/plan.rs` is the normalized multi-service model phase 2 asks for. A
`DeploymentPlan` describes services, images, environment, storage and one
published endpoint, and renders the Compose file rather than carrying one.

**Its acceptance gate is that it can express what already ships.** A plan built
from each of the three reviewed recipes renders Compose **byte-for-byte**
identical to the file that recipe ships, which is what makes the model
trustworthy rather than merely plausible. Those three are already put through
Docker by `scripts/validate-recipes.py`, so the single-service shape is
transitively proven.

The multi-service shape has no shipped example, so `tests/plan_compose.rs`
renders a web-app-plus-database plan and checks Docker's own resolution of it:
two services, only the web one published and only on loopback, the database
reachable from nowhere on the host, `depends_on` preserved, the named volume
declared, and numeric or boolean-looking environment values still strings.

Structural rules replace re-reading hand-written YAML: exactly one published
port per plan — so a database can never be exposed — a host port of 1024 or
above, pinned image tags, bind mounts confined to the project directory,
declared-and-used named volumes, and dependencies that name a real service.
`choose_free_port` keeps a recipe's documented port when it is free and steps
aside when it is not.

Still to do in phase 2: internal secrets and typed setup fields. Secrets need a
decision on a random source — there is no CSPRNG dependency yet, and inventing
one quietly would be the wrong way to generate a database password. Nothing
installs from a plan yet; recipes still use their own Compose file.

## Phase 3 progress: the Runtipi importer reads, and reports

`src/importers/runtipi.rs` maps Runtipi's normalized `docker-compose.json` onto
a `DeploymentPlan`, or explains precisely why it cannot. It is read-only: no
install, no write, and no app becomes a reviewed recipe by being importable.
`scripts/runtipi_import_report.py` regenerates
[the report](runtipi-import-report.md) from the archive already pinned and
cached for the catalog build — nothing is downloaded.

Across **250 definitions, 45 map cleanly** — up from 18 before typed fields and
secrets existed. That movement is the measurement; the named blockers are the
remaining work queue:

| Blocker | Apps | What it needs |
| --- | --- | --- |
| `platform placeholder` | 96 | Values Runtipi's platform supplies. **`TZ` alone is 76 of these** |
| `host path` (refused) | 73 | Upstream mounts outside the app's own storage |
| `privileged internal port` | 48 | Assigning a host port at install; the plan model already separates host from container |
| `command` / `entrypoint` / `user` | ~55 | Small additions to the plan model |
| `healthCheck` / `dependsOn condition` | ~54 | Readiness modelling |
| `privileged`, `capAdd`, `devices`, `networkMode`, `securityOpt` | ~33 | Deliberately refused; needs an upstream change, not more importer |

The importer already supplies the platform values a loopback install can answer
for itself — `APP_PROTOCOL` is `http`, `APP_PORT` is the published port, and
`APP_DOMAIN`/`APP_HOST`/`LOCAL_DOMAIN` are `localhost:<port>`. What remains is
mostly **`TZ` (76 apps)**, plus `INTERNAL_IP` and `GID`.

**`TZ` is the clearest next lever.** It is a user preference with a safe
default, so it could become an optional field defaulting to `UTC`. That means
synthesizing a field the upstream did not declare, which is a deliberate
decision rather than a mechanical mapping — take it knowingly, and say so in the
report, because a silently wrong clock is worse than a visible setting.

An importable app is a *candidate*, not a verified one. Each still needs a real
install, health, restart, preserved-data reinstall and deletion run before it
could be offered as an install.

## Phase 2 progress: typed setup fields and generated secrets

`src/setup.rs` closes the gap the import report measured. A `PlanTemplate`
pairs a plan carrying `${KEY}` placeholders with the **fields** a person answers
and the **secrets** generated for them, and resolves the two into an installable
`DeploymentPlan`.

**Random source decision, as the handoff asked for before any code:**
`getrandom` — the OS CSPRNG (BCryptGenRandom on Windows, `getrandom(2)` on
Linux, arc4random on the BSDs). It was already in the dependency graph through
Tauri, is MIT OR Apache-2.0, and its MSRV of 1.85 sits below ours, so the
licence and crate counts did not move. A generated database password must come
from the operating system, never a seeded PRNG. Secrets use rejection sampling,
because `byte % 62` would quietly favour the first few letters of the alphabet.

Properties worth keeping:

- **Resolution is all-or-nothing.** A placeholder with no declared source is an
  error; a literal `${...}` never reaches a container. A declared field nothing
  uses is also an error, so templates cannot rot.
- **Secrets can be supplied rather than regenerated**, which is what lets a
  preserved-data reinstall keep working — a fresh database password would leave
  the app unable to open its own data.
- **A default its own field would reject is caught at validation**, not on the
  one install where someone leaves the field alone.
- **Secret values never appear in an error message**, tested explicitly.
- Every answer problem is reported at once, so a setup form can show them all.

Alphabet is letters and digits only: the value passes through YAML, a Compose
environment and often a connection string, and punctuation invites quoting bugs
in all three. Length carries the strength.

**Now wired.** The importer reads Runtipi's `config.json` form fields and emits
a `PlanTemplate`: `type: random` becomes a generated secret, everything else a
typed field, and `type: password` is marked sensitive so an interface masks it.
Fields an app declares but never references are dropped rather than shown.

## Phase 2 progress: plans can install

`runtime::install_template_with` installs from a `PlanTemplate`: it resolves the
answers a person gave, reuses or generates the secrets the plan declares, writes
the rendered Compose file, and hands the result to the same install path
recipes use.

**There is one install path, not two.** Both a reviewed recipe and a resolved
plan reduce to an `InstallSource`, so rollback, the confinement check, port
preflight and cancellation are shared rather than duplicated into a second copy
that would drift. The refactor is behaviour-preserving: every existing runtime
test passed unchanged.

**Where a generated secret lives** — the question this batch existed to answer:
`local-store-secrets.json` in the app's own project directory, beside the data
it unlocks, not in the registry that lists every app. That placement is what
makes the important property work: uninstalling while keeping data keeps the
file, so a reinstall **reuses the same credential** rather than minting a fresh
database password the preserved data cannot be opened with. A test proves it,
and fails if reuse is removed. A secret the plan no longer declares is dropped
rather than left lying around, and a missing answer stops the install before
Docker is touched at all.

**Still not wired to anything a user can reach.** No command, no interface, and
the three reviewed recipes still install from their own Compose file. Switching
them over is provably behaviour-preserving — a plan renders their Compose
byte-for-byte — but it is a separate change with its own lifecycle evidence.

## Next implementation batch

The three-recipe lifecycle harness, cross-process registry safety, the
normalized multi-service plan and the read-only Runtipi importer are done.

Next: **a real container lifecycle run for a plan-installed app**. Everything so
far is proven against a fake process runner; the three reviewed recipes earned
their status through real install, health, restart, preserved-data reinstall and
deletion runs, and a plan install deserves the same before anything is offered.
Reuse `scripts/`-level harness patterns and an isolated data root and port. The
preserved-data reinstall is the case that matters most, because it is where a
regenerated secret would show up as an app that can no longer open its data.

Cheaper wins available in parallel, each measured by regenerating the report:
`TZ` as an optional field (76 apps), `readOnly` mounts (35), `command` and
`entrypoint` (41), and assigning a host port for privileged container ports
(48). Do not mistake any of these for verification: an importable app still
needs a real install, health, restart, preserved-data reinstall and deletion
run before it earns an install button.
Do not start mass catalog expansion by relabeling discovery entries as installs.
Preserve local-review-only publication instructions in the agent handoff.

# Managed container engine for V1

Required by the owner September 12, 2026. Status and dependencies are
E01–E04 in [V1_TASKS.md](../V1_TASKS.md). Moby/Compose in WSL is the proposed
implementation; packaging and licensing still require fresh verification.

Local Store asks the person to install Docker Desktop before it can install
anything. That is the largest dependency in the product, it carries a licence
that costs money for larger companies, and an upstream update to it can change
behaviour underneath a proof. This is how to remove it.

## Goal, and what is not the goal

**Goal.** No third-party application to install. Local Store ships and manages
its own container engine, starts it when needed, and applies an explicit background-app policy. Closing
the launcher must not silently stop installed applications.

**Not the goal.** Running Linux images without a Linux kernel. On Windows that
means WSL2 or a virtual machine whatever the engine is called, and WSL2 needs a
Windows feature enabled once, with admin. The honest claim is "nothing else to
install", never "no dependencies".

**Also not the goal.** Forcing anybody to migrate. An existing Docker Desktop
should still be detected and used.

## What an engine has to satisfy

Everything goes through one function — `CommandSpec::docker` in
`src/runtime/process.rs:96` — which hardcodes the program name. Everything else
is arguments, and the surface is small:

| Used for | Commands |
| --- | --- |
| Readiness | `version --format`, `compose version --short` |
| Lifecycle | `compose -f … -p … up -d / stop / down / ps [--status running --quiet\|--all] / logs --tail --no-color / config --quiet` |
| Images | `pull`, `inspect --format {{.Image}}` |
| Ownership, cleanup | `container inspect --format {{json .Config.Labels}}`, `ps -aq` / `network ls -q` / `volume ls -q` with filters, `container rm -f`, `network rm`, `volume rm` |
| Probes | `ps --filter publish=… --format …`, `exec` |

Harder than the commands are the Compose semantics the plan renderer emits
(`src/plan.rs`, `to_compose`): `depends_on` with all three conditions including
`service_completed_successfully`, healthchecks with `start_period`, long-syntax
ports with `host_ip`, `stop_signal`, `restart: "no"`, named volumes, and
`internal: true` networks. Dify needs nearly all of them at once.

One user-facing string mentions Docker Desktop by name (`src/js/app.js:217`).

## Options considered

**A. Bundle Docker Engine (moby) and the Compose v2 plugin in a WSL distro
Local Store manages.** Both are Apache-2.0; the licence that costs money covers
Docker *Desktop*, not the engine. Compatibility is exact, so the plan model,
importers, evidence and probes do not change. Rancher Desktop and Podman
Desktop do this, minus the desktop application.

**B. containerd and nerdctl.** Leaner, fully CNCF, but `nerdctl compose`
implements a subset of Compose and the gaps are in the features this project
just added. The risk is not that it fails; it is that it differs quietly, which
for a product whose claim is *proven* is the expensive kind of risk.

**C. Podman, rootless.** Better isolation and a Docker-compatible API socket,
but compose support is either podman-compose (less complete) or the API path,
and `podman machine` on Windows is WSL2 anyway. Rootless changes bind-mount
ownership, which is where several past failures already lived.

**D. Drop Compose and drive the engine API from Rust.** Local Store already
owns a structured plan, so this would give exact control — including the
container-health assertion the qualification roadmap wants. But it means
re-implementing dependency ordering, health waiting, restart policies and
cleanup, and the retained `compose.yaml` is a real recovery artifact:
`src/recovery.rs` reads the published port back out of it.

**Chosen: A, behind a seam that leaves B, C and D open.**

## Phases

### Phase 1 — the seam

Replace direct `CommandSpec::docker` calls with a `ContainerEngine`
abstraction: which program to run, and any argument differences. Detection
order: an engine Local Store manages, else whatever is already installed.

*Acceptance.* The existing suite passes unchanged against Docker Desktop, with
the engine chosen through the abstraction; a test drives a fake engine whose
program name is not `docker`.

*Effort.* Small, mechanical, covered by the existing fake process runner.

### Phase 2 — the managed engine

A distro image containing `dockerd`, `containerd`, `runc`, the CLI and the
Compose plugin, imported with `wsl --import` under a name Local Store owns.
Start it on demand, preserve explicitly allowed background operation, and expose it on a socket
only Local Store uses. Version pinned by Local Store, so an upstream update
cannot change behaviour under a proof.

*Acceptance.* On a machine with no Docker Desktop installed, an app installs,
opens, restarts and uninstalls.

*Effort.* The bulk of the work, most of it packaging and lifecycle supervision
rather than Local Store's own code.

### Phase 3 — prove the swap with the suite that already exists

Re-run the offered qualification suite against the bundled engine. This is a
real conformance test that almost nobody else has: it answers "is this engine
compatible?" with a whole catalogue's worth of facts instead of release notes.
The same run is how B or C would be evaluated later, if ever.

*Acceptance.* Every offered app passes against the bundled engine, and the
evidence records which engine and version proved it.

*Note.* Evidence should gain an engine field; that is a manifest-visible change
and needs a full re-proof, so land it together with the qualification
roadmap's items 1 and 2 rather than separately.

### Phase 4 — the things owning the engine makes possible

- Compact the disk image automatically. The virtual disk grew to 94 GB against
  26 GB of real data on 2026-09-12 and had to be compacted by hand with
  diskpart; a managed engine can do this on a schedule or when space runs low, only after an explicit safe maintenance design. Never
  compact or unregister another product's distro/disk.
- Report engine health and disk use in the doctor check, instead of "start
  Docker Desktop".
- Pin the engine version per release, so a proof names the exact stack.

## Costs and risks

- **WSL2 is still required**: a Windows feature, enabled once with admin, and
  occasionally a BIOS virtualization toggle. Say so plainly in the first-run
  screen.
- **Installer size**: a rootfs with the engine is roughly 150–250 MB
  compressed, which probably wants to be a first-run download rather than part
  of the installer.
- **Security updates become ours.** Pinning the engine means owning the
  responsibility to move the pin.
- **Support burden** when WSL itself misbehaves — today that is Docker
  Desktop's problem.
- **Trademark**: shipping the binaries is fine under Apache-2.0, but call it a
  container engine, or moby, and do not imply Docker endorsement.
- Bind-mount behaviour does not improve: a self-managed distro sees Windows
  paths through the same `/mnt/<drive>` translation, so the MySQL 8
  `lower_case_table_names` refusal and Nextcloud's 18-minute first start stay
  exactly as they are.

## Open questions to settle before committing

1. **The licence.** Does Docker Desktop's current licence make distributing
   Local Store with an "install Docker Desktop first" step a problem for the
   people likely to use it? This decides whether this plan is polish or a
   blocker. Settle it first.
2. Current feature coverage of `nerdctl compose` and podman-compose against the
   Compose features listed above — only matters if A is rejected.
3. WSL distro packaging mechanics: rootfs construction, registration under a
   private name, upgrade path when the engine pin moves, and what happens when
   the person also runs Docker Desktop at the same time.

None of these was checked when this plan was written. Verify before building.

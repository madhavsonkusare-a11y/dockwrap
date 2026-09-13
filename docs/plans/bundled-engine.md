# Managed container engine for V1

Updated September 13, 2026. Required by the owner; task status belongs to
E01–E04 in [V1_TASKS.md](../V1_TASKS.md).

## Selected development architecture

Build a Linux/amd64 Ubuntu 24.04 rootfs containing Docker Engine, CLI and the
maintained Compose plugin, then import it into a Local Store-owned WSL2 distro.
Reuse upstream packages, Compose semantics and our bounded process runner.
The [development payload](../../engine/README.md) now builds and exports with
five checksum-pinned Docker packages, a full installed-package inventory and
retained notices. That page records exact versions, provenance and build limits.

WSL2 still requires virtualization and Windows prerequisites, potentially with
administrator consent and a restart. The managed-engine promise removes a
separate Docker Desktop installation; it does not remove platform prerequisites.
Keep the existing Docker Desktop path available without silently migrating apps.

Rancher Desktop's rootfs construction and checksum verification are useful
references. Its complete Alpine/OpenRC/Kubernetes service stack is not our
payload. containerd/nerdctl and Podman remain possible future backends, but they
would need independent Compose and storage conformance proof. Driving the engine
API directly would require recreating dependency ordering and recovery behavior
that Compose already supplies.

## E01 verification checkpoint — September 13, 2026

A development rootfs was built and exported; its binaries report the locked
versions. [Evidence](../evidence/engine-payload-development-2026-09-13.json) is
explicitly unsigned and not WSL-boot-tested. The 128-package inventory is exact
for that build, but Ubuntu transitive dependencies still resolve at build time.
Finish the full dependency lock, signed-index provenance and package source/
notice obligations before approving a distributable payload.

Microsoft documents custom distro imports and systemd support. Docker recommends
maintained packages over static binaries for production updates. Moby/Compose
project licenses alone do not cover the complete distro's distribution duties.

- [WSL import](https://learn.microsoft.com/en-us/windows/wsl/use-custom-distro)
- [WSL systemd](https://learn.microsoft.com/en-us/windows/wsl/systemd)
- [Docker package installation](https://docs.docker.com/engine/install/ubuntu/)
- [Static binary update limits](https://docs.docker.com/engine/install/binaries/)
- [Moby license](https://github.com/moby/moby/blob/master/LICENSE)
- [Compose license](https://github.com/docker/compose/blob/main/LICENSE)

## E02 — One selected engine for every operation

`CommandSpec::docker` in `src/runtime/process.rs` is a useful constructor seam,
but qualification also constructs Docker commands directly, and recovery has
independent calls. Audit every caller before declaring the seam complete.

Persist engine identity with each installation and its retained project, so a
restart, reinstall or interrupted-install recovery cannot follow a changed Docker
context. Define explicit adoption for old records whose engine is unknown.
Discovery of an available engine is separate from selecting or migrating to it.

A WSL program prefix alone is insufficient: define distro and user selection,
working directory, Compose file translation and bind paths. Preserve argument
arrays, cancellation, deadlines and bounded output. Use the selected backend for
install, doctor, qualification, probes, rollback, recovery and resource cleanup.
Tests must use a different executable/context and detect every fallback to the
ambient Docker context. No engine selector is wired into production yet.

## E03 — Bootstrap and payload integrity

Define an owned distro/data-disk layout and explicit Windows/WSL support matrix.
Verify available disk, virtualization and WSL before downloading. Verify an
authenticated manifest and payload digest before import; checksum equality alone
does not authenticate a remotely supplied manifest. Downloads and imports need
bounded execution, progress, cancellation and resumable failures.

An existing WSL installation is not a clean-machine test. Prove bootstrap on a
Windows x64 host without Docker Desktop, including consent/restart cases. Check
loopback forwarding, bind permissions, all Compose dependency conditions and
coexistence with an existing Docker Desktop installation.

## E04 — Background apps, repair and updates

Systemd services alone do not keep a WSL instance alive. Design and test an owned
supervisor that preserves background apps after the launcher closes. Exercise
sleep/wake, crash, restart and failed engine upgrade. Never use global WSL shutdown
or unregister another product's distro. Removing the engine and deleting app data
must remain separate, explicit actions.

Local Store maintainers own engine security updates: review upstream advisories,
refresh locks, rebuild and requalify, then sign and deliver the payload through
the updater. Keep replaceable engine binaries separate from durable app data.
Live restore is configured in the development daemon but is not evidence of a
working update/rollback system. Disk compaction needs an explicit offline
maintenance design and must target only the owned disk.

## Proof boundary

Requalify current offerings on the managed engine after E04 and Q01/Q02. Existing
Docker Desktop evidence cannot establish compatibility with the new daemon,
filesystem or network. All-service health checks now exist in qualification;
full engine identity, resource measurements and useful app tasks remain required.
V3 integration, signing credentials and release publication are separate gates.

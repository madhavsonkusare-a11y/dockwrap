# Recover an interrupted installation

If Local Store or its CLI is killed after Docker starts a container but before
the registry is saved, the container can remain running without appearing in
My Apps. Retrying installation then reports an occupied port. Where retained
setup files exist, that error now includes their path.

Run `local-store recovery` (or `local-store recovery --json`) to inventory
retained setup files for unregistered supported recipes. This reads the current
registry and filesystem without migration, Docker calls or writes. A candidate
may be an intentional keep-data uninstall, a current install, or an interrupted
setup. Add `--docker` to inspect container IDs and Compose labels using bounded,
read-only Docker queries. `ownership_status` is `not_checked`, `no_containers`,
`mismatch`, or `verified`; only `verified` sets `docker_ownership_verified` true.
Verification requires all returned containers to match the expected project,
service, absolute Compose file and working directory, and excludes one-off
containers. Truncated, failed or malformed Docker output cannot verify ownership.
The default command still runs without Docker. Verification does not execute the
Compose file or inspect environment values. It does not establish that the file
is unchanged since startup or safe to execute. A future recovery action must
recheck ownership and registry state under its operation lock.
The result is a snapshot, not authorization to stop or delete anything. A corrupt
registry fails inspection instead of treating its apps as absent. Legacy-only
registries must be migrated by opening Local Store first.

For this specific case:

1. Confirm the app is absent from My Apps and that no Local Store installation
   is still running. Do not stop an unrelated service just because its port matches.
2. Locate the retained `compose.yaml` identified in the error. Inspect the Docker
   container's `com.docker.compose.project.config_files` label and verify it
   refers to that exact file before stopping the project.
3. Run `docker compose -f "<verified absolute compose.yaml path>" -p "<verified project name>" down`.
   Use the project's actual name (for example, `local-store-memos`). Leave out
   `--volumes`, and keep the app's data directory.
4. Retry installation in Local Store. Existing data is reused. If the preserved
   folder contains unexpected files, review them before retrying; do not delete
   the folder merely to bypass the check.

The app does not yet discover these unregistered containers automatically or
provide a dedicated recovery action. An occupied port alone does not establish
ownership, so Local Store does not automatically stop its listener.

## Evidence and reproduction

Passed September 7, 2026 on Windows with Docker Desktop Linux/amd64 and Memos
0.30.0. The final run completed in 13.73 seconds. Its private evidence root was
`.cache/install-interruption-13712-1788803163410605000`; test-owned app files,
containers and networks were removed. The test's assertion sequence is the
reproducible evidence; no release installer or other desktop platform was tested.

`tests/install_interruption.rs` is an opt-in real Docker test. It uses a private
configuration root, refuses existing Memos resources, requires the cached Memos
0.30.0 image, and holds the registry lock to create a deterministic pre-commit
interruption window. It kills only the installer process it spawned after the
container becomes healthy. The test proves:

- The daemon-owned container survives while the registry stays empty.
- A fresh install refuses the occupied port and preserves setup files.
- Explicit `compose down` preserves a marker in mounted storage.
- Reinstallation preserves that marker, and explicit uninstall removes the
  registry entry, managed directory, containers and network.

On PowerShell, from the repository:

```powershell
$env:LOCAL_STORE_RUN_DOCKER_TEST = '1'
cargo test --locked --test install_interruption -- --ignored --nocapture
```

The test reports a unique `.cache/install-interruption-*` evidence directory.
On failure, review its retained files and Docker project before cleanup. This
test covers interruption after healthy startup, not interruption during an image
pull, arbitrary daemon crashes, registry corruption, or database version upgrades.

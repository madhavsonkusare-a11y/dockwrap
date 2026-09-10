# What this project learned the hard way

Things that cost real time, found by running the code rather than reading it.
Each one bit at least once and several bit repeatedly. If you are about to
touch the plan model, the importers or anything that talks to Docker, this is
the page worth reading first.

## Windows canonical paths cannot be mounted

`Path::canonicalize` on Windows returns an extended-length path,
`\\?\D:\Data\...`. Docker Compose builds a bind-mount source out of whatever it
is given, and that form makes it refuse the whole file with **"too many
colons"** — the drive colon inside the prefix is read as a mount separator.

It is worth canonicalizing anyway: that is what stops a path merely *pointing*
at somewhere refused. So resolve first, then strip the prefix before the path
reaches Docker or the registry.

This bit three times in one day:

1. Adoption started an app from a canonicalized Compose path and failed.
   Crucially `compose down` never resolves mounts, so `discard` had always
   worked and the bug looked like it was in adoption.
2. The path was also being written into the registry, which would have made the
   adopted app fail to start for the rest of its life rather than once.
3. A shared folder is a mount source too, and `docker_path` had been rewritten
   with a three-character prefix instead of four — silently disarming it.

`crate::folders::docker_path` is the one implementation. Use it; do not write
another.

## "Is this declared value used?" must look at mounts

Three separate places asked whether a declared setup field was referenced, and
all three looked only at environment values:

- `PlanTemplate::validate` called a folder field declared-but-unused.
- The Runtipi importer's field filter dropped it, then refused the template for
  referring to a key nothing declared.
- Earlier, the same shape dropped a time-zone field as inert, leaving an app in
  UTC with no way to change it.

A value can reach a container through an environment entry **or** a mount
source. Anything answering "is this used?" has to consider both.

## A guard has to compare at the right granularity

The check that stops a reinstall running over unrecognised files compared the
directory listing against each *full* declared data path. An app keeping its
data in `data/.ollama` puts a single `data` directory on disk, so the listing
and the list never matched and **every keep-data reinstall of any app with a
nested data path was refused**. It would have hit a real person on their second
install of Ollama or Metabase.

Compare what is actually on disk against what would actually be on disk.

## Timeouts tuned for the simple case fail the real one

The first-start health wait was sixty seconds. That is fine for something that
only has to open a port, and not enough for WordPress running its own
installer or Metabase initialising a schema. Both were rolled back **while
still working**.

Giving up early throws away a working app; waiting longer costs time that is
bounded, visible and cancellable. Prefer the second.

## Our own rules can be stricter than the thing they model

The plan required every environment name to be `NAME_LIKE_THIS`. Uppercase is a
convention, not a rule: Ghost configures itself with `database__client`,
Jellyfin with `JELLYFIN_PublishedServerUrl`. Ten apps were refused for a house
style wearing a safety rule's clothes.

Before refusing something, check whether the thing you are modelling refuses
it. Then write the check against what actually matters — here, that a name
cannot break out of the line it renders into.

## Sources do not agree on format

CapRover ships YAML; its importer takes JSON, and the import report converts in
Python. A batch runner that read the files directly reported four of eight apps
as "no longer importable" when they were fine.

Normalise at extraction, once, where the conversion is already understood.

## Evidence must be derived, not declared

Two evidence files claimed things that had stopped being true:

- One said `browser_first_use: "not_tested"` while the test ran the browser
  probe three times. The string was left over from a run that predated those
  calls.
- Two said `promotion: "withheld"` after their apps were approved.

Both now read from the thing they describe. A related trap: a check that passes
*trivially* must say so. The credential step reported "keeps the credentials it
generated" for apps that generate none — true, and useless, and
indistinguishable from the real thing.

## Measure before choosing, and check the measurement

Ranking candidates by Docker Hub pull count scored every app that ships a
Postgres or Redis sidecar identically, in the billions, because the first image
in a definition is often not the app. Falling back to the least-pulled image did
the same thing more quietly for apps on ghcr.io, which publishes no pull count
at all. A pull count now counts only when the image is identifiably the app's
own: **borrowing a database's popularity is worse than having no number.**

Also: `NOASSERTION` from GitHub means "no licence file detected", not "not open
source". Reading it as a refusal excluded WordPress and qBittorrent, both GPL.

And a cache keyed on the wrong field never refills: stars and licence came from
one call, but the skip was keyed on stars, so every project cached before
licences were collected kept a null licence permanently.

## Prove a test can fail

Every claim worth making has been checked by reintroducing the bug it guards
against — a destructive keep-data uninstall, a review that says a password is
not sensitive, a removed platform placeholder. A test that has never failed has
not been shown to test anything.

## A refusal is a finding, not a failure

The review mechanism records "no" as readily as "yes". CodiMD is withheld
because its images were last rebuilt in 2020; Planka because it is proprietary,
pins a release a year old, runs its database with `trust` authentication, and
uses a floating tag. Writing that down is more useful than a longer catalogue.

## Qualifying apps in bulk costs system-drive space permanently

Docker Desktop on Windows keeps its images in a virtual disk under
`%LOCALAPPDATA%`, on C:. That file **only grows**. Deleting images frees space
*inside* it and returns nothing to the drive, and compacting it needs
Administrator rights.

A batch that pulls a hundred images therefore consumes tens of gigabytes of the
system drive whatever it cleans up afterwards. On this machine it took C: from
6 GB to 331 MB, at which point Docker could no longer create its own sockets and
crashed on every start with "The file cannot be accessed by the system" — an
error that looks nothing like "the disk is full".

Two things that follow:

- **Check free space before a long batch, not after.** An option added to make
  re-runs fast (`--keep-images`) is the same option that fills a drive.
- **Move Docker's disk off the system drive.** Stop Docker, `wsl --shutdown`,
  move `%LOCALAPPDATA%\Docker\wsl\disk` to another drive, and junction the old
  path to the new one:

      New-Item -ItemType Junction -Path "$env:LOCALAPPDATA\Docker\wsl\disk" -Target "D:\DockerData\disk"

  No elevation, no data loss, and every image and container survives. That
  turned 0.3 GB free into 47 GB here.

Recovering a Docker Desktop that will not start, in order: quit it and
`com.docker.backend`, remove or rename `%LOCALAPPDATA%\Dockerun` and
`%LOCALAPPDATA%\docker-secrets-engine` (their socket files become undeletable
when the disk fills), `wsl --shutdown`, then start Docker Desktop and let *it*
boot the VM — starting the distro by hand leaves the data disk unmounted and
every API call returns 500.

## What the batch is for

Running candidates in bulk found three real defects in one afternoon — the
nested data path, the sixty-second timeout, and the YAML that could not be
read. None was visible from reading the code. If a change claims to unlock
apps, run the apps.

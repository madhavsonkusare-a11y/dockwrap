"""Rank every queued candidate by how many people actually use it.

The v1 goal is one hundred apps chosen for audience reach, which means the
order has to come from evidence rather than from taste. Three public signals,
each fetched once and cached so a ranking is reproducible offline:

* Docker Hub pull count for the image we would actually ship. This is the
  strongest signal available, because it measures the artefact rather than the
  project's reputation. It does not exist for ghcr.io, lscr.io or quay.io.
* GitHub stars for the project, where the queue resolved a repository identity.
  A proxy for reputation rather than use, and easily gamed, so it is weighted
  below pulls.
* How many upstream catalogues list the app. An app that CapRover, Runtipi,
  CasaOS and Coolify all package is mainstream almost by definition.

Nothing here decides what gets offered. It decides what is worth looking at
first, and records why each app is or is not installable today so the
capability work can be aimed rather than guessed.

Run with --refresh to fetch; without it, the saved evidence is used.
"""

import argparse
import json
import math
import subprocess
import re
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUEUE = ROOT / "catalog" / "candidate-queue.json"
CATALOG = ROOT / "src" / "generated" / "catalog.json"
CACHE = ROOT / "catalog" / "reach-signals.json"
REPORT = ROOT / "docs" / "candidate-ranking.md"
RANKED = ROOT / "catalog" / "candidate-ranking.json"

# Registries that publish no public pull count. Popular apps are
# disproportionately here, which is worth seeing in the output rather than
# silently scoring them zero.
PRIVATE_REGISTRIES = ("ghcr.io/", "lscr.io/", "quay.io/", "gcr.io/", "registry.gitlab.com/")

IMAGE_PATTERN = re.compile(r'"image"\s*:\s*"([^"\s]+)"')


# Pinned source archives, opened once. Blocked candidates record no images at
# all, and those are precisely the apps this ranking exists to find — so the
# images come from the definition itself.
_ARCHIVES = {}


def archive_for(revision):
    if revision not in _ARCHIVES:
        found = sorted((ROOT / ".cache" / "catalog").glob(f"*{revision}*.zip"))
        _ARCHIVES[revision] = zipfile.ZipFile(found[0]) if found else None
    return _ARCHIVES[revision]


def images_from_definition(candidate):
    """Every image a definition names, read from the pinned archive."""
    provenance = candidate.get("provenance") or {}
    archive = archive_for(provenance.get("revision", ""))
    path = provenance.get("path")
    if not archive or not path:
        return []
    matches = [name for name in archive.namelist() if name.endswith("/" + path)]
    if not matches:
        return []
    try:
        text = archive.read(matches[0]).decode("utf-8", "replace")
    except KeyError:
        return []
    # Both source formats spell it the same way, and reading the strings is
    # enough here: this is a popularity signal, not an install plan.
    return sorted({match.group(1) for match in IMAGE_PATTERN.finditer(text)})


def candidate_images(candidate):
    """Images to score by, whether or not the candidate is importable."""
    images = candidate.get("images")
    if images:
        return list(images)
    family = candidate.get("image_family")
    if family:
        return [family]
    return images_from_definition(candidate)


def image_repository(image):
    """The Docker Hub repository path for an image, or None if it is elsewhere."""
    if image.startswith(PRIVATE_REGISTRIES):
        return None
    name = image.rsplit(":", 1)[0]
    if name.startswith("index.docker.io/"):
        name = name[len("index.docker.io/") :]
    if name.startswith("docker.io/"):
        name = name[len("docker.io/") :]
    # A bare name is an official image, which lives under `library`.
    return name if "/" in name else f"library/{name}"


def fetch_json(url, timeout=20):
    request = urllib.request.Request(url, headers={"User-Agent": "Local-Store-reach"})
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read(2 * 1024 * 1024))
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, OSError):
        return None


def fetch_stars(repo):
    """Stars via the gh CLI, which is already authenticated for this repo."""
    try:
        done = subprocess.run(
            ["gh", "api", f"repos/{repo}", "--jq", ".stargazers_count"],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if done.returncode != 0:
        return None
    try:
        return int(done.stdout.strip())
    except ValueError:
        return None


def load_signals():
    if CACHE.exists():
        return json.loads(CACHE.read_text(encoding="utf-8"))
    return {"pulls": {}, "stars": {}}


def refresh(candidates, signals):
    repos = sorted(
        {
            repository
            for candidate in candidates
            for image in candidate_images(candidate)
            if (repository := image_repository(image))
        }
    )
    print(f"fetching pull counts for {len(repos)} Docker Hub repositories")
    for index, repository in enumerate(repos, 1):
        if repository in signals["pulls"]:
            continue
        data = fetch_json(f"https://hub.docker.com/v2/repositories/{repository}")
        signals["pulls"][repository] = data.get("pull_count") if data else None
        if index % 25 == 0:
            print(f"  {index}/{len(repos)}")
            CACHE.write_text(json.dumps(signals, indent=1, sort_keys=True), encoding="utf-8")

    projects = sorted(
        {
            candidate["identity"][len("github:") :]
            for candidate in candidates
            if str(candidate.get("identity", "")).startswith("github:")
        }
    )
    print(f"fetching stars for {len(projects)} GitHub projects")
    for index, repo in enumerate(projects, 1):
        if repo in signals["stars"]:
            continue
        signals["stars"][repo] = fetch_stars(repo)
        if index % 25 == 0:
            print(f"  {index}/{len(projects)}")
            CACHE.write_text(json.dumps(signals, indent=1, sort_keys=True), encoding="utf-8")

    CACHE.write_text(json.dumps(signals, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    return signals


def own_image(candidate, images):
    """The image that is the app itself, not something it depends on.

    Getting this wrong is what made every app shipping a Postgres sidecar score
    identically in the billions. When it cannot be identified, no pull count is
    used at all — borrowing a database's popularity would be worse than having
    no number.
    """
    primary = candidate.get("primary_image")
    if primary:
        return primary
    identity = str(candidate.get("identity", ""))
    owner = identity[len("github:") :].split("/")[0] if identity.startswith("github:") else ""
    project = identity[len("github:") :].split("/")[-1] if identity.startswith("github:") else ""
    wanted = {
        candidate["id"].lower().replace("-", "").replace("_", ""),
        owner.lower().replace("-", ""),
        project.lower().replace("-", ""),
    }
    wanted.discard("")
    for image in images:
        flattened = image.rsplit(":", 1)[0].lower().replace("-", "").replace("_", "")
        if any(name in flattened for name in wanted):
            return image
    return None


def reach(candidate, signals, listings):
    """A single comparable number, and the parts it came from.

    Logarithms because these distributions span orders of magnitude. Stars are
    weighted above pulls despite being the weaker measure of use, because they
    are the only signal available for the ghcr.io images that many of the most
    popular apps ship — scoring on pulls alone would rank those apps last for a
    reason that has nothing to do with how many people run them.
    """
    images = candidate_images(candidate)
    mine = own_image(candidate, images)
    repository = image_repository(mine) if mine else None
    pulls = signals["pulls"].get(repository) if repository else None

    stars = None
    identity = str(candidate.get("identity", ""))
    if identity.startswith("github:"):
        stars = signals["stars"].get(identity[len("github:") :])

    sources = listings.get(candidate["id"], 1)
    score = 1.5 * sources
    if pulls:
        score += 4.0 * math.log10(pulls + 1)
    if stars:
        score += 5.0 * math.log10(stars + 1)
    return round(score, 2), pulls, stars, sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--top", type=int, default=150)
    args = parser.parse_args()

    queue = json.loads(QUEUE.read_text(encoding="utf-8"))
    candidates = queue["candidates"]
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    listings = {
        entry["id"]: max(1, len({source["source"] for source in entry.get("sources", [])}))
        for entry in catalog["entries"]
    }

    signals = load_signals()
    if args.refresh:
        signals = refresh(candidates, signals)
    if not signals["pulls"]:
        raise SystemExit("no cached signals; run with --refresh")

    ranked = []
    for candidate in candidates:
        score, pulls, stars, sources = reach(candidate, signals, listings)
        blockers = sorted({blocker["feature"] for blocker in candidate["blockers"]})
        ranked.append(
            {
                "id": candidate["id"],
                "source": candidate["source"],
                "reach": score,
                "pulls": pulls,
                "stars": stars,
                "catalogues": sources,
                "importable": candidate["importable"],
                "blockers": blockers,
                "services": candidate.get("service_count"),
                "required_inputs": candidate.get("required_inputs"),
                "images": candidate_images(candidate),
            }
        )
    # Highest reach first; ties broken by id so the file is stable.
    ranked.sort(key=lambda row: (-row["reach"], row["id"]))
    RANKED.write_text(
        json.dumps({"schema_version": 1, "candidates": ranked}, indent=1) + "\n",
        encoding="utf-8",
    )

    top = ranked[: args.top]
    installable = [row for row in top if row["importable"]]
    lines = [
        "# Candidates by audience reach",
        "",
        "Generated by `python scripts/rank-candidates.py`. Reach combines the",
        "Docker Hub pull count of the app's own image, GitHub stars where the",
        "queue resolved a repository, and how many upstream catalogues list it.",
        "It orders what to look at first; it approves nothing.",
        "",
        f"**Top {len(top)}: {len(installable)} importable today, "
        f"{len(top) - len(installable)} blocked.**",
        "",
        "| # | App | Reach | Pulls | Stars | Importable | Blocked on |",
        "| --- | --- | --- | --- | --- | --- | --- |",
    ]
    for position, row in enumerate(top, 1):
        pulls = f"{row['pulls']:,}" if row["pulls"] else "—"
        stars = f"{row['stars']:,}" if row["stars"] else "—"
        blocked = ", ".join(row["blockers"][:3]) if row["blockers"] else ""
        lines.append(
            f"| {position} | {row['id']} ({row['source']}) | {row['reach']} | {pulls} | "
            f"{stars} | {'yes' if row['importable'] else 'no'} | {blocked} |"
        )

    # What to build next, weighted by the reach of what it unblocks.
    gain = {}
    for row in top:
        if row["importable"]:
            continue
        if len(row["blockers"]) == 1:
            gain.setdefault(row["blockers"][0], []).append(row)
    lines += [
        "",
        "## What one capability would unlock, within this top list",
        "",
        "Counting only apps whose *sole* remaining blocker is that feature.",
        "",
        "| Feature | Apps | Combined reach | Examples |",
        "| --- | --- | --- | --- |",
    ]
    for feature, rows in sorted(
        gain.items(), key=lambda item: -sum(row["reach"] for row in item[1])
    ):
        examples = ", ".join(row["id"] for row in rows[:4])
        lines.append(
            f"| {feature} | {len(rows)} | {round(sum(row['reach'] for row in rows), 1)} | {examples} |"
        )
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"ranked {len(ranked)} candidates")
    print(f"top {len(top)}: {len(installable)} importable, {len(top) - len(installable)} blocked")
    print(f"wrote {REPORT.relative_to(ROOT)} and {RANKED.relative_to(ROOT)}")


if __name__ == "__main__":
    main()

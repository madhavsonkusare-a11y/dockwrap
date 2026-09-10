"""Assemble a reviewed template from a candidate that passed qualification.

Writing one of these by hand takes about seventy lines, most of which are
copied from somewhere: the identity from the catalog, the provenance from the
queue, the image audit from the registry, the field list from the importer.
Doing that four times was the sign it should be generated.

What is generated is everything mechanical. What is left blank, deliberately,
is everything that is a judgement:

* `promotion.state` is `withheld` with a reason saying nobody has decided yet.
* Every setup field gets a review entry copying the importer's guess about
  whether it is a credential, marked `REVIEW:` so it cannot be mistaken for a
  decision somebody made.
* `risk_notes` gets what can be derived — how many containers, whether a
  folder is shared, whether credentials are generated — and a `REVIEW:` line
  for whatever is specific to the app.

A manifest still needs a person to read it before it is offered. This just
means they start from the facts rather than from an empty file.

    python scripts/generate-template.py <app-id>
"""

import argparse
import datetime
import json
import subprocess
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REVIEW = "REVIEW: "


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "Local-Store-template"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read(2 * 1024 * 1024))


def hub_repository(image):
    name = image.rsplit(":", 1)[0]
    if any(name.startswith(host) for host in ("ghcr.io/", "lscr.io/", "quay.io/", "gcr.io/")):
        return None
    return name if "/" in name else f"library/{name}"


def image_audit(image):
    """Digests and rebuild date for one image, from the registry itself."""
    repository = hub_repository(image)
    if not repository:
        raise SystemExit(
            f"{image}: not on Docker Hub, and the platform gate cannot audit other registries yet"
        )
    tag = image.rsplit(":", 1)[1]
    url = f"https://hub.docker.com/v2/repositories/{repository}/tags/{tag}"
    data = fetch(url)
    platforms = {}
    for entry in data.get("images", []):
        if entry.get("os") in (None, "unknown") or entry.get("architecture") in (None, "unknown"):
            continue
        key = "/".join(
            part for part in (entry["os"], entry["architecture"], entry.get("variant")) if part
        )
        platforms[key] = entry["digest"]
    if not platforms:
        raise SystemExit(f"{image}: the registry lists no platforms for this tag")
    return {
        "image": image,
        "source_url": url,
        "checked_at": data["last_updated"][:10],
        "last_updated": data["last_updated"][:10],
        "container_platforms": sorted(platforms),
        "digests": dict(sorted(platforms.items())),
    }


def facts_for(candidate):
    """Ask the importer what this definition actually becomes."""
    provenance = candidate["provenance"]
    path = ROOT / ".cache" / "definitions" / provenance["revision"] / provenance["path"]
    if path.suffix in (".yml", ".yaml"):
        path = path.with_suffix(".json")
    if not path.is_file():
        raise SystemExit(f"{path} is missing; run scripts/extract-definitions.py first")
    done = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "--example",
            "template_facts",
            "--",
            candidate["source"],
            candidate["id"],
            str(path),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        raise SystemExit(done.stderr.strip() or "the importer refused this definition")
    return json.loads(done.stdout)


def risk_notes(facts, catalog):
    notes = []
    services = facts["services"]
    notes.append(
        f"Creates {services} Docker container{'s' if services != 1 else ''} and publishes one "
        "port on this computer's loopback interface only."
    )
    if facts["shared_folders"]:
        notes.append(
            "Reads and writes a folder on this computer that you choose during setup. Nothing "
            "outside that folder is shared with it."
        )
    if facts["generated_credentials"]:
        count = facts["generated_credentials"]
        notes.append(
            f"Generates {count} credential{'s' if count != 1 else ''} of its own, keeps "
            f"{'them' if count != 1 else 'it'} beside the app's data, and reuses "
            f"{'them' if count != 1 else 'it'} if you reinstall."
        )
    notes.append(
        "Keeps its data in this app's managed folder. A keep-data uninstall leaves it; deleting "
        "data removes it permanently."
    )
    notes.append(
        "Proven on Windows with Docker's Linux engine. macOS and Linux hosts are unverified."
    )
    notes.append(
        REVIEW + "say what this app does that somebody should know before installing it, and "
        "remove this line."
    )
    return notes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app")
    parser.add_argument("--source", help="runtipi or caprover, when an app is in both")
    args = parser.parse_args()

    queue = json.loads((ROOT / "catalog" / "candidate-queue.json").read_text(encoding="utf-8"))
    matches = [
        candidate
        for candidate in queue["candidates"]
        if candidate["id"] == args.app
        and candidate["importable"]
        and (args.source is None or candidate["source"] == args.source)
    ]
    if not matches:
        raise SystemExit(f"no importable candidate named {args.app}")
    if len({candidate["source"] for candidate in matches}) > 1:
        raise SystemExit(
            f"{args.app} is in more than one source; choose with --source "
            + ", ".join(sorted({candidate['source'] for candidate in matches}))
        )
    candidate = matches[0]

    # A manifest for an app nobody has run is a manifest that cannot be
    # completed: the platform gate requires a lifecycle proof, and there is
    # nothing honest to point it at. Requiring the run first also means the
    # generated file is about an app that demonstrably works.
    result_path = ROOT / ".cache" / "qualification" / f"{args.app}.json"
    if not result_path.is_file():
        raise SystemExit(
            f"{args.app} has no qualification result. Run it first with "
            "LOCAL_STORE_RUN_DOCKER_TEST=1 cargo run --release --example qualify_batch"
        )
    result = json.loads(result_path.read_text(encoding="utf-8"))
    if not result.get("passed"):
        failed = next((s for s in result.get("steps", []) if not s["passed"]), {})
        raise SystemExit(
            f"{args.app} did not pass qualification (failed at {failed.get('step')!r}). "
            "Fix that before writing a manifest for it."
        )

    facts = facts_for(candidate)

    catalog = json.loads((ROOT / "src" / "generated" / "catalog.json").read_text(encoding="utf-8"))
    entry = next((e for e in catalog["entries"] if e["id"] == args.app), None)
    ranking = json.loads(
        (ROOT / "catalog" / "candidate-ranking.json").read_text(encoding="utf-8")
    )
    ranked = next((r for r in ranking["candidates"] if r["id"] == args.app), {})

    provenance = candidate["provenance"]
    definition_path = ROOT / ".cache" / "definitions" / provenance["revision"] / provenance["path"]
    if definition_path.suffix in (".yml", ".yaml"):
        definition_path = definition_path.with_suffix(".json")
    definition = definition_path.read_text(encoding="utf-8")
    config_path = definition_path.parent / "config.json"

    manifest = {
        "schema_version": 1,
        "id": args.app,
        "display_name": (entry or {}).get("name") or args.app,
        "catalog_name": (entry or {}).get("name") or args.app,
        "description": (entry or {}).get("description")
        or (REVIEW + "one sentence describing this app"),
        "category": (entry or {}).get("category") or (REVIEW + "a catalog category"),
        "license": ((ranked.get("licenses") or [None])[0]) or (REVIEW + "the upstream licence"),
        "source_url": (entry or {}).get("source_url") or (REVIEW + "the project's repository"),
        "documentation_url": (entry or {}).get("website_url")
        or (entry or {}).get("source_url")
        or (REVIEW + "where its own setup instructions live"),
        "verified_at": result["steps"] and datetime.date.today().isoformat(),
        "lifecycle_proof": f"docs/evidence/{args.app}-qualification.json",
        "origin": {
            "importer": candidate["source"],
            "repository": provenance["repository"],
            "revision": provenance["revision"],
            "path": provenance["path"],
            "license": REVIEW + "the licence of the definition, not of the app",
        },
        "requirements": {
            "docker_engine_os": "linux",
            "compose_major": 2,
            "local_storage_required": True,
            "images": [image_audit(image) for image in facts["images"]],
        },
        "promotion": {
            "state": "withheld",
            "reason": REVIEW
            + "nobody has decided whether to offer this app. Record the decision and why.",
        },
        "data_storage": f"Local Store managed folder / {args.app}"
        + (", plus a folder you choose" if facts["shared_folders"] else ""),
        "risk_notes": risk_notes(facts, entry),
        # The importer's guess, marked as a guess. A review says which of these
        # answers is genuinely a credential; upstream cannot.
        "fields": {
            field["key"]: {
                "label": REVIEW + field["upstream_label"],
                "sensitive": field["importer_thinks_sensitive"],
            }
            for field in facts["fields"]
        },
        "definition": definition,
    }
    if candidate["source"] == "runtipi" and config_path.is_file():
        manifest["config"] = {
            "path": str(Path(provenance["path"]).parent / "config.json").replace("\\", "/"),
            "content": config_path.read_text(encoding="utf-8"),
        }

    # The proof travels with the manifest that names it.
    evidence = ROOT / "docs" / "evidence" / f"{args.app}-qualification.json"
    evidence.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")

    target = ROOT / "src" / "templates" / f"{args.app}.json"
    target.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    outstanding = json.dumps(manifest).count(REVIEW)
    print(f"wrote {target.relative_to(ROOT)} and {evidence.relative_to(ROOT)}")
    print(f"{outstanding} thing(s) marked {REVIEW.strip()} still need a person")
    print("then: record a lifecycle proof, set verified_at, and decide promotion")


if __name__ == "__main__":
    main()

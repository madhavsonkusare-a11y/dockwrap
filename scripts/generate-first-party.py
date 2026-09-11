"""Assemble a reviewed template from one of Local Store's own definitions.

`generate-template.py` works from a candidate an importer found in a pinned
app store. A definition Local Store wrote itself lives in this repository at
`definitions/apps/<id>/`, and its provenance is the commit that last changed
that folder — so the manifest can be generated from the tree alone.

Like the other generator, this fills in only what is mechanical: identity from
the catalogue, the image audit and digest from the registry, the fields and
storage from the importer, the files verbatim. Anything that is a judgement —
what a person should know before installing, whether to offer it — is left as
a `REVIEW:` marker, and `promotion` is left pending until a qualification as
offered has run.

    python scripts/generate-first-party.py <app-id> [<app-id> ...]
"""

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("generate_template", ROOT / "scripts" / "generate-template.py")
g = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(g)


def facts(app, definition):
    built = ROOT / "target" / "release" / "examples" / "template_facts.exe"
    if not built.is_file():
        built = ROOT / "target" / "release" / "examples" / "template_facts"
    done = subprocess.run([str(built), "runtipi", app, str(definition)], capture_output=True, text=True)
    if done.returncode:
        raise SystemExit(done.stderr.strip() or f"{app}: the importer refused its definition")
    return json.loads(done.stdout)


def revision(folder):
    out = subprocess.run(["git", "log", "-1", "--format=%H", "--", folder], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout.strip()
    if not out:
        raise SystemExit(f"{folder} is not committed, so there is no revision for the manifest to cite")
    dirty = subprocess.run(["git", "status", "--porcelain", "--", folder], cwd=ROOT,
                           capture_output=True, text=True, check=True).stdout.strip()
    if dirty:
        raise SystemExit(f"{folder} has uncommitted changes; commit them so the manifest cites what it carries")
    return out


def generate(app):
    folder = f"definitions/apps/{app}"
    definition_path = ROOT / folder / "docker-compose.json"
    config_path = ROOT / folder / "config.json"
    config = json.loads(config_path.read_text(encoding="utf-8"))
    found = facts(app, definition_path)
    if found["limitations"]:
        raise SystemExit(f"{app}: not importable: {found['limitations']}")
    if found.get("binary_seeds"):
        raise SystemExit(f"{app}: binary seed files {found['binary_seeds']} cannot be carried")

    catalog = json.loads((ROOT / "src" / "generated" / "catalog.json").read_text(encoding="utf-8"))
    entry = g.catalog_entry(catalog, app) or g.catalog_entry(catalog, config["name"])
    if not entry:
        raise SystemExit(f"{app}: no catalogue entry; run scripts/catalog_pipeline.py after committing the definition")

    images = []
    for image in found["images"]:
        audit = g.image_audit(image)
        audit["index_digest"] = g.index_digest(image)
        if not audit["index_digest"]:
            raise SystemExit(f"{image}: the registry returned no digest to pin")
        images.append(audit)

    notes = g.risk_notes(found, entry)
    assert notes[-1].startswith(g.REVIEW)
    notes.insert(-1, "Written by Local Store from the project's own deployment instructions, not taken from an app store.")

    labels = {f["env_variable"]: f.get("label", f["env_variable"]) for f in config.get("form_fields", [])}
    manifest = {
        "schema_version": 1,
        "id": app,
        "display_name": config["name"],
        "catalog_name": entry["name"],
        "description": config.get("short_desc") or entry["description"],
        "category": (config.get("categories") or [entry["category"]])[0],
        "license": config.get("license") or (entry.get("licenses") or [g.REVIEW + "the upstream licence"])[0],
        "source_url": config.get("source") or entry["source_url"],
        "documentation_url": config.get("documentation") or config.get("website") or entry["website_url"],
        "verified_at": "",
        "lifecycle_proof": f"docs/evidence/{app}-qualification.json",
        "origin": {
            "importer": "runtipi",
            "repository": "madhavsonkusare-a11y/local-store",
            "revision": revision(folder),
            "path": f"{folder}/docker-compose.json",
            "license": "MIT",
        },
        "requirements": {"docker_engine_os": "linux", "compose_major": 2, "local_storage_required": True, "images": images},
        "promotion": {"state": "approved", "reason": "PENDING: qualify as offered, then record the outcome here."},
        "data_storage": g.data_storage(found, app),
        "risk_notes": notes,
        "fields": {
            field["key"]: {"label": labels.get(field["key"], field["upstream_label"]), "sensitive": field["importer_thinks_sensitive"]}
            for field in found["fields"]
        },
        "definition": definition_path.read_text(encoding="utf-8"),
        "config": {"path": f"{folder}/config.json", "content": config_path.read_text(encoding="utf-8")},
    }
    seeds = [{"path": p, "content": (ROOT / folder / p).read_text(encoding="utf-8")} for p in found.get("seed_files", [])]
    if seeds:
        manifest["seeds"] = seeds
    target = ROOT / "src" / "templates" / f"{app}.json"
    target.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    oldest = min(i["last_updated"] for i in images)
    print(f"{app}: {len(images)} image(s), oldest built {oldest}; {json.dumps(manifest).count(g.REVIEW)} REVIEW marker(s)")


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    for app in sys.argv[1:]:
        generate(app)


if __name__ == "__main__":
    main()

"""Extract pinned source definitions so a batch run can read them as files.

The archives are checksum-pinned and already cached for the catalog build.
Unpacking them costs nothing and keeps the batch runner free of an archive
dependency it would otherwise need only to read two files per app.
"""

import json
import zipfile
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
ARCHIVES = ROOT / ".cache" / "catalog"
OUT = ROOT / ".cache" / "definitions"


def main():
    queue = json.loads((ROOT / "catalog" / "candidate-queue.json").read_text(encoding="utf-8"))
    wanted = {}
    for candidate in queue["candidates"]:
        provenance = candidate.get("provenance") or {}
        revision, path = provenance.get("revision"), provenance.get("path")
        if revision and path:
            wanted.setdefault(revision, set()).add(path)

    written = 0
    seeds = 0
    for revision, paths in wanted.items():
        archives = sorted(ARCHIVES.glob(f"*{revision}*.zip"))
        if not archives:
            print(f"no archive for {revision}")
            continue
        with zipfile.ZipFile(archives[0]) as archive:
            names = archive.namelist()
            for path in sorted(paths):
                for name in names:
                    if not name.endswith("/" + path):
                        continue
                    target = OUT / revision / path
                    target.parent.mkdir(parents=True, exist_ok=True)
                    raw = archive.read(name)
                    if target.suffix in (".yml", ".yaml"):
                        # CapRover ships YAML; the Rust importer reads JSON, the
                        # same conversion the import report already does. Doing
                        # it here keeps the batch runner free of a YAML
                        # dependency it would need for two files per app.
                        target = target.with_suffix(".json")
                        raw = json.dumps(yaml.safe_load(raw.decode("utf-8"))).encode("utf-8")
                    target.write_bytes(raw)
                    written += 1
                    # A companion config sits beside the definition.
                    folder = name.rsplit("/", 1)[0]
                    companion = folder + "/config.json"
                    if companion in names:
                        (target.parent / "config.json").write_bytes(archive.read(companion))
                    # And so do the files Runtipi copies into the app's data
                    # folder on install — an nginx config, a starter settings
                    # file. A definition that mounts one is broken without it.
                    for seed in names:
                        if seed.startswith(folder + "/data/") and not seed.endswith("/"):
                            out = target.parent / seed[len(folder) + 1:]
                            out.parent.mkdir(parents=True, exist_ok=True)
                            out.write_bytes(archive.read(seed))
                            seeds += 1
                    break
    print(f"extracted {written} definition(s) and {seeds} seed file(s) to {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()

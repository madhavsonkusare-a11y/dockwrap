"""Check reviewed-template image claims against cached registry metadata.

The recipe checker assumes one image per app and an index digest for its tag.
Neither holds for a multi-service template built from an imported definition:
there are several images, and these repositories publish a digest per
architecture rather than one for the tag. This checks what is actually there.

Offline by default. `--refresh` fetches only public Docker Hub tag metadata,
never image layers, and needs no Docker daemon. Refreshing the cache does not
approve a change: a digest that moves still fails the check afterwards, which
is the point.
"""
import argparse
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TEMPLATES = ROOT / "src" / "templates"
CACHE = ROOT / "catalog" / "template-platforms"


def cache_name(image):
    repository, tag = image.rsplit(":", 1)
    return f"{repository.replace('/', '-')}-{tag}.json"


def published(metadata):
    """Platform to digest, for every architecture the tag really carries."""
    found = {}
    for image in metadata["images"]:
        if image.get("os") in (None, "unknown"):
            continue
        if image.get("architecture") in (None, "unknown"):
            continue
        key = "/".join(
            part
            for part in (image["os"], image["architecture"], image.get("variant"))
            if part
        )
        found[key] = image["digest"]
    return dict(sorted(found.items()))


def validate(template_id, audit, metadata):
    tag = audit["image"].rsplit(":", 1)[1]
    actual = published(metadata)
    for valid, detail in [
        (metadata["name"] == tag, "tag differs"),
        (
            audit["source_url"].endswith(f"/{tag}"),
            "source url does not name the pinned tag",
        ),
        (
            audit["last_updated"] == metadata["last_updated"][:10],
            "the registry rebuilt this tag since it was reviewed",
        ),
        (
            audit["container_platforms"] == sorted(actual),
            "container platforms differ",
        ),
        (audit["digests"] == actual, "a digest moved; review the upstream change"),
    ]:
        if not valid:
            raise ValueError(f"{template_id}: {audit['image']}: {detail}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true")
    args = parser.parse_args()

    manifests = sorted(TEMPLATES.glob("*.json"))
    if not manifests:
        raise SystemExit("no reviewed templates to check")
    for path in manifests:
        template = json.loads(path.read_text(encoding="utf-8"))
        for audit in template["requirements"]["images"]:
            cache = CACHE / cache_name(audit["image"])
            if args.refresh:
                request = urllib.request.Request(
                    audit["source_url"],
                    headers={"User-Agent": "Local-Store-recipe-audit"},
                )
                with urllib.request.urlopen(request, timeout=30) as response:
                    raw = response.read(1024 * 1024 + 1)
                if len(raw) > 1024 * 1024:
                    raise SystemExit(f"{audit['image']}: tag metadata exceeds 1 MiB")
                CACHE.mkdir(parents=True, exist_ok=True)
                cache.write_text(
                    json.dumps(json.loads(raw), indent=2) + "\n", encoding="utf-8"
                )
            if not cache.exists():
                raise SystemExit(
                    f"{audit['image']}: no cached metadata; run with --refresh"
                )
            metadata = json.loads(cache.read_text(encoding="utf-8"))
            try:
                validate(template["id"], audit, metadata)
            except ValueError as error:
                raise SystemExit(str(error)) from error
        proof = ROOT / template["lifecycle_proof"]
        if not proof.is_file():
            raise SystemExit(
                f"{template['id']}: names a lifecycle proof that is not there: "
                f"{template['lifecycle_proof']}"
            )
        state = template["promotion"]["state"]
        oldest = min(a["last_updated"] for a in template["requirements"]["images"])
        print(
            f"verified {path.stem}: {len(template['requirements']['images'])} images, "
            f"oldest rebuilt {oldest}, promotion {state}"
        )


if __name__ == "__main__":
    main()

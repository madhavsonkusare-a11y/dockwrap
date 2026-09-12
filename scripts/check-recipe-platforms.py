"""Check recipe platform claims against cached upstream tag metadata.

Default operation is offline. --refresh fetches only public Docker Hub tag
metadata, never image layers, and does not require a Docker daemon. Updating
the cache does not silently approve new recipe requirements or image digests.
"""
import argparse
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / "catalog" / "recipe-platforms"


def platforms(metadata):
    return sorted({
        "/".join(part for part in (image["os"], image["architecture"], image.get("variant")) if part)
        for image in metadata["images"]
        if image.get("os") not in (None, "unknown")
        and image.get("architecture") not in (None, "unknown")
    })


def metadata_url(recipe):
    repository, tag = recipe["image"].removeprefix("docker.n8n.io/").rsplit(":", 1)
    return f"https://hub.docker.com/v2/repositories/{repository}/tags/{tag}"


def validate(recipe, metadata):
    tag = recipe["image"].rsplit(":", 1)[1]
    requirements = recipe["requirements"]
    audit = requirements["image_audit"]
    for valid, detail in [
        (metadata["name"] == tag, "tag differs"),
        (audit["source_url"] == metadata_url(recipe), "source does not match the pinned image"),
        (audit["index_digest"] == metadata["digest"], "index digest differs; review the upstream change"),
        (requirements["container_platforms"] == platforms(metadata), "container platforms differ"),
    ]:
        if not valid:
            raise ValueError(f"{recipe['id']}: {detail}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true")
    args = parser.parse_args()
    for path in sorted((ROOT / "src" / "recipes").glob("*.json")):
        recipe = json.loads(path.read_text(encoding="utf-8"))
        # n8n's official hostname challenges for this same Docker Hub namespace.
        url = metadata_url(recipe)
        cache = CACHE / path.name
        if args.refresh:
            request = urllib.request.Request(url, headers={"User-Agent": "Local-Store-recipe-audit"})
            with urllib.request.urlopen(request, timeout=30) as response:
                raw = response.read(1024 * 1024 + 1)
            if len(raw) > 1024 * 1024:
                raise SystemExit(f"{path.stem}: tag metadata exceeds 1 MiB")
            metadata = json.loads(raw)
            CACHE.mkdir(parents=True, exist_ok=True)
            cache.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
        else:
            metadata = json.loads(cache.read_text(encoding="utf-8"))
        try:
            validate(recipe, metadata)
        except ValueError as error:
            raise SystemExit(str(error)) from error
        print(f"verified {path.stem}: {', '.join(recipe['requirements']['container_platforms'])}")


if __name__ == "__main__":
    main()

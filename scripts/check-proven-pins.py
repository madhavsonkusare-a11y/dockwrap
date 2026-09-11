"""Check that an approved template pins exactly the images its proof ran.

For each image: the local image ID must equal the one the qualification
recorded, and the manifest's index digest must be the digest Docker pulled
it by. Run after `qualify_batch --offered`, while the images are still local.

    python scripts/check-proven-pins.py <app-id> [<app-id> ...]
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def docker(*args):
    return subprocess.run(["docker", *args], capture_output=True, text=True).stdout.strip()


def main():
    ok = True
    for app in sys.argv[1:]:
        manifest = json.loads((ROOT / "src" / "templates" / f"{app}.json").read_text(encoding="utf-8"))
        proof = json.loads((ROOT / manifest["lifecycle_proof"]).read_text(encoding="utf-8"))
        for audit in manifest["requirements"]["images"]:
            image = audit["image"]
            repo = image.rsplit(":", 1)[0]
            local = docker("image", "inspect", image, "--format", "{{.Id}}")
            pulled = None
            for line in docker("image", "inspect", image, "--format", "{{range .RepoDigests}}{{println .}}{{end}}").split():
                name, _, digest = line.partition("@")
                if name in (repo, "docker.io/" + repo, "docker.io/library/" + repo):
                    pulled = digest
            good = bool(local) and proof.get("image_ids", {}).get(image) == local and audit.get("index_digest") == pulled
            ok &= good
            print(f"{app:12} {image:48} {'proven and pinned' if good else 'MISMATCH'}")
        print(f"{app:12} proof passed: {proof.get('passed')}  scope: {proof.get('scope', '')[:40]}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

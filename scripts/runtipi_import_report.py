"""Generate the Runtipi import report from the pinned upstream archive.

Reads the app-store archive already cached for the catalog build, extracts the
`docker-compose.json` definitions to a temporary directory, and runs the
read-only Rust importer over them. Nothing is downloaded, installed or written
outside the report path.
"""
import json
import pathlib
import subprocess
import sys
import tempfile
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "runtipi-import-report.md"

lock = json.loads((ROOT / "catalog" / "sources.lock.json").read_text(encoding="utf-8"))
source = next((s for s in lock["sources"] if s["id"] == "runtipi"), None)
if source is None:
    raise SystemExit("no runtipi source is pinned in catalog/sources.lock.json")

revision = source["revision"]
archive = ROOT / ".cache" / "catalog" / f"runtipi-{revision}.zip"
if not archive.exists():
    raise SystemExit(
        f"cached archive missing: {archive}\n"
        "Run the catalog pipeline first; this script never downloads."
    )

with tempfile.TemporaryDirectory(prefix="runtipi-definitions-") as directory:
    target = pathlib.Path(directory)
    extracted = 0
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            parts = member.filename.split("/")
            # <repo>-<revision>/apps/<app id>/docker-compose.json
            if len(parts) != 4 or parts[1] != "apps":
                continue
            if parts[3] not in ("docker-compose.json", "config.json"):
                continue
            app = target / parts[2]
            app.mkdir(parents=True, exist_ok=True)
            (app / parts[3]).write_bytes(bundle.read(member))
            if parts[3] == "docker-compose.json":
                extracted += 1
    if extracted == 0:
        raise SystemExit("archive contained no app definitions; check its layout")

    result = subprocess.run(
        [
            "cargo", "run", "--quiet", "--example", "runtipi_report",
            "--", str(target), revision,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

if result.returncode != 0:
    raise SystemExit(f"importer failed:\n{result.stderr.strip()}")

REPORT.write_text(result.stdout, encoding="utf-8", newline="\n")
summary = [line for line in result.stdout.splitlines() if line.startswith("| ")][:5]
print(f"wrote {REPORT.relative_to(ROOT)} from {extracted} definitions at {revision[:12]}")
for line in summary:
    print(f"  {line}")
sys.exit(0)

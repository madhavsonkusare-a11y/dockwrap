"""Generate the CapRover import report from the checksum-pinned archive.

Normalizes each pinned YAML definition to JSON and runs the real Rust adapter
over it. Nothing is downloaded, installed, evaluated or written outside the
report path; the archive is verified against its recorded checksum first.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

import yaml

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "caprover-import-report.md"
MAX_MEMBER_BYTES = 1024 * 1024


def main():
    source = json.loads(
        (ROOT / "catalog/import-audit-sources.json").read_text(encoding="utf-8")
    )["caprover"]
    archive = ROOT / ".cache/catalog" / f"caprover-{source['revision']}.zip"
    if not archive.exists():
        raise SystemExit(
            f"cached archive missing: {archive}\n"
            "Run scripts/audit-caprover-source.py first; this script never downloads."
        )
    if hashlib.sha256(archive.read_bytes()).hexdigest() != source["sha256"]:
        raise SystemExit("Pinned archive checksum mismatch")

    definitions = {}
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            if "/public/v4/apps/" not in member.filename:
                continue
            if not member.filename.endswith((".yml", ".yaml")):
                continue
            if member.file_size > MAX_MEMBER_BYTES:
                raise SystemExit(f"Oversized app definition: {member.filename}")
            # safe_load never constructs arbitrary objects, and no path from the
            # archive is ever used to write anything.
            definitions[Path(member.filename).stem] = yaml.safe_load(
                bundle.read(member)
            )

    result = subprocess.run(
        [
            "cargo",
            "run",
            "--locked",
            "--quiet",
            "--example",
            "caprover_report",
            "--",
            source["revision"],
        ],
        cwd=ROOT,
        input=json.dumps(definitions),
        text=True,
        capture_output=True,
        check=True,
    )
    REPORT.write_text(result.stdout, encoding="utf-8")
    summary = [
        line
        for line in result.stdout.splitlines()
        if line.startswith("| Definitions read")
        or line.startswith("| Fully expressible")
        or line.startswith("| Blocked by")
    ]
    print(f"wrote {REPORT.relative_to(ROOT)} from {len(definitions)} definitions")
    print("\n".join(summary))


if __name__ == "__main__":
    main()

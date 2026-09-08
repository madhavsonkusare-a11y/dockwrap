"""Measure real Rust setup primitives against pinned YAML; never installs apps."""
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile
import yaml

ROOT = Path(__file__).resolve().parents[1]


def main():
    source = json.loads((ROOT / "catalog/import-audit-sources.json").read_text())["caprover"]
    archive = ROOT / ".cache/catalog" / f"caprover-{source['revision']}.zip"
    if hashlib.sha256(archive.read_bytes()).hexdigest() != source["sha256"]:
        raise SystemExit("Pinned archive checksum mismatch")
    apps = {}
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            if "/public/v4/apps/" not in member.filename or not member.filename.endswith(".yml"):
                continue
            if member.file_size > 1024 * 1024:
                raise SystemExit("Oversized app definition")
            app = yaml.safe_load(bundle.read(member))
            apps[Path(member.filename).stem] = app.get("caproverOneClickApp", {}).get("variables", [])
    result = subprocess.run(
        ["cargo", "run", "--locked", "--quiet", "--example", "caprover_setup_report"],
        cwd=ROOT, input=json.dumps(apps), text=True, capture_output=True, check=True,
    )
    counts = json.loads(result.stdout)
    refused_apps = counts.pop("refused_apps")
    mapping_reasons = counts.pop("mapping_reasons")
    lines = ["# CapRover setup primitive compatibility", "",
             f"Pinned revision: `{source['revision']}`; {len(apps)} definitions.", "",
             "Reproduce: `python scripts/caprover_setup_report.py` (cached archive required).",
             "Runs the actual Rust parsers on normalized upstream variables. No installs,",
             "credential generation, source-expression execution or catalog promotion.", "",
             "| Measurement | Count |", "| --- | --- |"]
    lines.extend(f"| {key.replace('_', ' ')} | {value} |" for key, value in counts.items())
    lines.extend(["", "Supported means only that the declaration can be represented by these primitives.",
                  "Variable mapping checks literal defaults and rules together; the separate primitive",
                  "counts do not. Neither checks variable substitution, service topology, platform",
                  "requirements, image support or successful installation. ASCII-only pattern fields",
                  "are intentionally narrower than JavaScript regexes. A full adapter must report",
                  "these limitations and preserve every other upstream constraint.", ""])
    lines.extend(["## Setup-variable mapping refusals", "",
                  "First refusal per variable; categories can hide additional limitations.", "",
                  "| Reason | Variables |", "| --- | --- |"])
    lines.extend(f"| {reason} | {count} |" for reason, count in sorted(mapping_reasons.items()))
    lines.extend(["", "## Apps needing primitive follow-up", "",
                  "Counts only; upstream values are not included in diagnostics.", "",
                  "| App | Refused patterns | Refused secret declarations |", "| --- | --- | --- |"])
    lines.extend(f"| {app} | {values[0]} | {values[1]} |" for app, values in sorted(refused_apps.items()))
    lines.append("")
    (ROOT / "docs/caprover-setup-report.md").write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps(counts, sort_keys=True))


if __name__ == "__main__":
    main()

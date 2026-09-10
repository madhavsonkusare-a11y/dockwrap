"""Rewrite the reviewed-template list from the manifests that exist.

`reviewed_templates()` names each manifest with `include_str!`, which needs a
literal path, so the list cannot simply read the directory. Maintaining it by
hand was fine for one template and is not fine for twenty: a manifest that
exists but is not listed is silently absent, which is the worst of both states.

This changes nothing about what is *offered*. That is still `promotion.state`
in each manifest, and the guard test's `APPROVED` list still has to name every
approved app by hand — deliberately, so an approval appears in a diff.
"""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE = ROOT / "src" / "templates" / "mod.rs"
INCLUDE = re.compile(r'^const [A-Z0-9_]+: &str = include_str!\("[^"]+\.json"\);$')
ARRAY = re.compile(r"^    \[[A-Z0-9_, ]+\]$")


def constant(name):
    return re.sub(r"[^A-Za-z0-9]", "_", name).upper()


def main():
    manifests = sorted(path.stem for path in (ROOT / "src" / "templates").glob("*.json"))
    if not manifests:
        raise SystemExit("no manifests in src/templates")

    lines = MODULE.read_text(encoding="utf-8").splitlines()
    includes = [index for index, line in enumerate(lines) if INCLUDE.match(line)]
    if not includes:
        raise SystemExit("could not find the include block in src/templates/mod.rs")
    arrays = [index for index, line in enumerate(lines) if ARRAY.match(line)]
    if not arrays:
        raise SystemExit("could not find the template array in src/templates/mod.rs")

    # Replace the whole contiguous include block in one go, rather than editing
    # line by line — doing that left duplicate constants behind.
    first, last = includes[0], includes[-1]
    replacement = [f'const {constant(name)}: &str = include_str!("{name}.json");'
                   for name in manifests]
    lines[first : last + 1] = replacement

    arrays = [index for index, line in enumerate(lines) if ARRAY.match(line)]
    lines[arrays[0]] = "    [" + ", ".join(constant(name) for name in manifests) + "]"

    MODULE.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"listed {len(manifests)} manifest(s): {', '.join(manifests)}")


if __name__ == "__main__":
    main()

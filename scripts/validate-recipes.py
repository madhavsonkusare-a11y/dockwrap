"""Validate reviewed recipe JSON and Compose syntax without starting containers."""
import json
import pathlib
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
recipes = sorted((ROOT / "src" / "recipes").glob("*.json"))
if len(recipes) < 3:
    raise SystemExit("expected at least three reviewed recipes")

with tempfile.TemporaryDirectory(prefix="local-store-recipes-") as directory:
    target = pathlib.Path(directory)
    for path in recipes:
        recipe = json.loads(path.read_text(encoding="utf-8"))
        compose = target / f"{recipe['id']}.yaml"
        compose.write_text(recipe["compose"], encoding="utf-8")
        subprocess.run(
            ["docker", "compose", "-f", str(compose), "config", "--quiet"],
            check=True,
        )
        print(f"validated {recipe['id']} ({recipe['image']})")

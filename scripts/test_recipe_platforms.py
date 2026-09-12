"""Offline regressions for recipe image compatibility evidence."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("platforms", Path(__file__).with_name("check-recipe-platforms.py"))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class PlatformTests(unittest.TestCase):
    def setUp(self):
        self.recipe = json.loads((audit.ROOT / "src/recipes/n8n.json").read_text())
        self.metadata = json.loads((audit.CACHE / "n8n.json").read_text())

    def test_attestations_are_not_cpu_platforms(self):
        self.assertEqual(audit.platforms(self.metadata), ["linux/amd64", "linux/arm64"])
        self.assertTrue(any(image["os"] == "unknown" for image in self.metadata["images"]))

    def test_reviewed_evidence_agrees(self):
        for path in (audit.ROOT / "src/recipes").glob("*.json"):
            audit.validate(json.loads(path.read_text()), json.loads((audit.CACHE / path.name).read_text()))

    def test_changed_digest_or_tag_needs_review(self):
        for key in ("digest", "name"):
            changed = copy.deepcopy(self.metadata)
            changed[key] = "different"
            with self.subTest(key=key), self.assertRaises(ValueError):
                audit.validate(self.recipe, changed)

    def test_platform_and_source_claims_cannot_drift(self):
        changed = copy.deepcopy(self.recipe)
        changed["requirements"]["container_platforms"].append("linux/arm/v7")
        with self.assertRaisesRegex(ValueError, "platforms differ"):
            audit.validate(changed, self.metadata)
        changed = copy.deepcopy(self.recipe)
        changed["requirements"]["image_audit"]["source_url"] = "https://example.org/other-image"
        with self.assertRaisesRegex(ValueError, "source does not match"):
            audit.validate(changed, self.metadata)


if __name__ == "__main__":
    unittest.main()

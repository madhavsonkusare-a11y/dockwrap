"""Offline regressions for registry evidence used by template CI."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("platforms", Path(__file__).with_name("check-template-platforms.py"))
platforms = importlib.util.module_from_spec(spec)
spec.loader.exec_module(platforms)


class TemplatePlatformsTests(unittest.TestCase):
    def test_every_committed_review_has_registry_evidence(self):
        for path in platforms.TEMPLATES.glob("*.json"):
            template = json.loads(path.read_text(encoding="utf-8"))
            for audit in template["requirements"]["images"]:
                metadata = json.loads((platforms.CACHE / platforms.cache_name(audit["image"])).read_text())
                with self.subTest(app=template["id"], image=audit["image"]):
                    platforms.validate(template["id"], audit, metadata)

    def test_changed_registry_evidence_and_install_pin_are_refused(self):
        audit = json.loads((platforms.TEMPLATES / "tautulli.json").read_text())["requirements"]["images"][0]
        metadata = json.loads((platforms.CACHE / platforms.cache_name(audit["image"])).read_text())
        for key in ("image", "source_url", "last_updated", "container_platforms", "digests"):
            changed = copy.deepcopy(metadata)
            changed["audit"][key] = "changed"
            with self.subTest(key=key), self.assertRaises(ValueError):
                platforms.validate("tautulli", audit, changed)
        changed = copy.deepcopy(metadata)
        changed["reference"] = "sha256:" + "0" * 64
        with self.assertRaisesRegex(ValueError, "install pin"):
            platforms.validate("tautulli", audit, changed)


if __name__ == "__main__":
    unittest.main()

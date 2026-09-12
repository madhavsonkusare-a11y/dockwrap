"""The Compose validator must enforce the exact reviewed image, including its pin."""
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("recipes", Path(__file__).with_name("validate-recipes.py"))
recipes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recipes)


class RecipeImagesTests(unittest.TestCase):
    def test_exact_digest_is_required(self):
        for path in recipes.recipes:
            recipe = json.loads(path.read_text())
            image = recipe["image"]
            pinned = image + "@" + recipe["requirements"]["image_audit"]["index_digest"]
            config = {"services": {"app": {"image": pinned}}}
            recipes.validate_images(recipe, config)
            for invalid in (image, image + "@sha256:" + "0" * 64, "other:1@" + pinned.split("@")[1]):
                config["services"]["app"]["image"] = invalid
                with self.subTest(app=recipe["id"], image=invalid), self.assertRaises(ValueError):
                    recipes.validate_images(recipe, config)


if __name__ == "__main__":
    unittest.main()

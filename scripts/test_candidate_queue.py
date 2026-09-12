"""Identity rules must not turn image similarity into verified app coverage."""
import importlib.util
from pathlib import Path
import unittest
import copy

spec = importlib.util.spec_from_file_location('queue_builder', Path(__file__).with_name('build-candidate-queue.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class IdentityTests(unittest.TestCase):
    def test_review_is_invalidated_by_source_or_image_change(self):
        row = {'provenance': {'revision': 'abc'}, 'images': ['team/app:1'],
               'verification_status': 'not_qualified_by_this_queue'}
        review = {'revision': 'abc', 'images': ['team/app:1'],
                  'identity': 'github:team/app', 'maintenance_status': 'not_screened'}
        module.apply_review(row, review)
        self.assertEqual(row['identity'], 'github:team/app')
        self.assertEqual(row['verification_status'], 'not_qualified_by_this_queue')
        for changed in [dict(review, revision='def'), dict(review, images=['team/app:2'])]:
            with self.assertRaises(ValueError):
                module.apply_review(copy.deepcopy(row), changed)

    def test_repository_identity(self):
        self.assertEqual(module.canonical_repo('https://github.com/Owner/App.git'), 'github:owner/app')
        for url in ['https://github.com/owner/app/tree/main', 'https://evil.test/owner/app',
                    'https://github.com/owner/app?ref=x', 'https://github.com/owner/a%20b']:
            self.assertIsNone(module.canonical_repo(url))

    def test_image_hint_preserves_registry_and_repository(self):
        self.assertEqual(module.image_family('registry.test:5000/team/app:2'), 'registry.test:5000/team/app')
        self.assertEqual(module.image_family('docker.io/team/app@sha256:abc'), 'team/app')
        self.assertNotEqual(module.image_family('team/app:1'), module.image_family('other/app:1'))


if __name__ == '__main__':
    unittest.main()

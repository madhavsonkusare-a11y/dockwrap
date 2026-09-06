"""Offline importer regression tests: identity, provenance and untrusted inputs."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml
import catalog_pipeline as pipeline

spec = importlib.util.spec_from_file_location('icons', Path(__file__).with_name('cache-catalog-icons.py'))
icons = importlib.util.module_from_spec(spec)
spec.loader.exec_module(icons)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'catalog/sources').mkdir(parents=True)
        (self.root / 'catalog/legacy.json').write_text('[]')
        self.root_patch = patch.object(pipeline, 'ROOT', self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)

    def generate(self, records):
        source = {'id': 'awesome-selfhosted', 'repository': 'example/catalog', 'revision': 'a' * 40}
        data = pipeline.encode({'source': source['id'], 'revision': source['revision'], 'records': records, 'excluded': []})
        (self.root / 'catalog/sources/awesome-selfhosted.json').write_bytes(data)
        source['sha256'] = pipeline.hashlib.sha256(data).hexdigest()
        self.lock = {'snapshot_date': '2026-09-05', 'sources': [source]}
        return pipeline.generate(self.lock)

    def row(self, name, source, website=None):
        return pipeline.row(name, source, website, 'A useful project.', ['analytics'], upstream_id=name)

    def test_distinct_projects_do_not_merge_by_name(self):
        catalog, _, _ = self.generate([
            self.row('Memex', 'https://github.com/one/memex'),
            self.row('Memex', 'https://github.com/two/memex'),
        ])
        self.assertEqual(len(catalog['entries']), 2)
        self.assertEqual(len({entry['id'] for entry in catalog['entries']}), 2)

    def test_repository_and_documentation_variants_merge_with_provenance(self):
        catalog, report, _ = self.generate([
            self.row('Notes', 'https://github.com/team/notes', 'https://notes.example.com'),
            self.row('Notes', 'https://docs.notes.example.com/setup'),
            self.row('Notes Docker', 'https://github.com/team/notes/tree/main/docker'),
        ])
        self.assertEqual(len(catalog['entries']), 1)
        self.assertEqual(len(catalog['entries'][0]['sources']), 3)
        self.assertEqual(len(report['merges']), 1)

    def test_stable_identity_survives_project_rename(self):
        catalog, _, ids = self.generate([self.row('Old Name', 'https://github.com/team/tool')])
        (self.root / 'catalog/ids.json').write_bytes(pipeline.encode(ids))
        renamed, _, _ = self.generate([self.row('New Name', 'https://github.com/team/tool')])
        self.assertEqual(catalog['entries'][0]['id'], renamed['entries'][0]['id'])
        self.assertEqual(renamed['entries'][0]['name'], 'New Name')

    def test_modified_snapshot_is_rejected(self):
        self.generate([self.row('Notes', 'https://github.com/team/notes')])
        (self.root / 'catalog/sources/awesome-selfhosted.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
            pipeline.generate(self.lock)

    def test_categories_and_urls_are_normalized_without_guessing_instances(self):
        self.assertEqual(pipeline.normalize_category('analytics'), 'Analytics')
        self.assertEqual(pipeline.normalize_category('AI'), 'Artificial Intelligence')
        self.assertEqual(pipeline.normalize_category('Money, Budgeting & Management'), 'Money, Budgeting & Management')
        self.assertNotEqual(pipeline.identity('https://gitlab.com/team/group/one'), pipeline.identity('https://gitlab.com/team/group/two'))
        for url in ['file:///tmp/icon', 'https://user:secret@example.com', 'https://exa mple.com']:
            self.assertIsNone(pipeline.http_url(url))
        with self.assertRaises(yaml.constructor.ConstructorError):
            pipeline.adapter({'id': 'awesome-selfhosted'}, 'bad.yml', '!!python/object/apply:os.system [echo nope]')

    def test_icons_reject_active_external_and_oversized_content(self):
        icons.validate_svg(b'<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>')
        for svg in [b'<svg><script>bad()</script></svg>', b'<svg onload="bad()"/>',
                    b'<svg><use href="https://example.com/a.svg"/></svg>',
                    b'<!DOCTYPE svg [<!ENTITY a "x">]><svg/>',
                    b'<svg><style>@import "https://example.com";</style></svg>',
                    b' ' * (icons.MAX_BYTES + 1)]:
            with self.assertRaises(ValueError):
                icons.validate_svg(svg)
        with self.assertRaisesRegex(ValueError, 'Unapproved icon host'):
            icons.fetch('http://127.0.0.1/private', 100)


if __name__ == '__main__':
    unittest.main()

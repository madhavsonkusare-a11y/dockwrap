"""Collect bounded public metadata for the B1 shortlist; never pull or run images.

Offline by default. --refresh updates evidence, not approval. Reuses the reviewed
template platform parser. Fetch failures remain explicit, not positive results.
"""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import urllib.request
import urllib.error

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('platforms', Path(__file__).with_name('check-template-platforms.py'))
PLATFORMS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PLATFORMS)
TARGETS = {
    'caprover:linkding': 'sissbruecker/linkding',
    'runtipi:nodered': 'node-red/node-red',
    'runtipi:privatebin': 'PrivateBin/PrivateBin',
    'runtipi:joplin': 'laurent22/joplin',
}
OUTPUT = ROOT / 'catalog/candidate-screening-evidence.json'


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'Local-Store-Candidate-Audit/1'})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            raw = response.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError('metadata exceeds 1 MiB')
        return json.loads(raw)
    except (urllib.error.URLError, ValueError) as error:
        return {'fetch_error': type(error).__name__}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh', action='store_true')
    args = parser.parse_args()
    if args.refresh:
        queue = json.loads((ROOT / 'catalog/candidate-queue.json').read_text())
        rows = {r['source'] + ':' + r['id']: r for r in queue['candidates']}
        results = []
        for key, project in TARGETS.items():
            row = rows[key]
            repo_url = f'https://api.github.com/repos/{project}'
            repo = fetch(repo_url)
            releases_url = repo_url + '/releases/latest'
            release = fetch(releases_url)
            item = {'candidate': key, 'source_revision': row['provenance']['revision'],
                    'repository': project, 'repository_evidence_url': repo_url,
                    'repository_metadata': {k: repo.get(k) for k in ('full_name', 'archived', 'disabled', 'pushed_at', 'fetch_error')},
                    'latest_project_release_url': releases_url,
                    'latest_project_release': {k: release.get(k) for k in ('tag_name', 'published_at', 'html_url', 'fetch_error')},
                    'images': [], 'promotion': 'not_approved'}
            for image in row['images']:
                repository, tag = image.rsplit(':', 1)
                if '/' not in repository:
                    repository = 'library/' + repository
                url = f'https://hub.docker.com/v2/repositories/{repository}/tags/{tag}'
                metadata = fetch(url)
                item['images'].append({'image': image, 'evidence_url': url,
                    'last_updated': metadata.get('last_updated'),
                    'platform_digests': PLATFORMS.published(metadata) if 'images' in metadata else {},
                    'fetch_error': metadata.get('fetch_error')})
            results.append(item)
        OUTPUT.write_text(json.dumps({'schema_version': 1, 'checked_at': datetime.now(timezone.utc).isoformat(),
            'scope': 'Metadata only; release freshness does not prove image safety or first use. Joplin project releases are not necessarily server releases.',
            'candidates': results}, indent=2) + '\n', encoding='utf-8')
    evidence = json.loads(OUTPUT.read_text())
    for row in evidence['candidates']:
        print(row['candidate'], json.dumps({'release': row['latest_project_release'], 'images': row['images']}))


if __name__ == '__main__':
    main()

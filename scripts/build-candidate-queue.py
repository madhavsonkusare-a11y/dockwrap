"""Offline queue from checksum-pinned sources and the actual Rust importers.
No installs, downloads, credential generation or automatic promotion.
"""
import hashlib
import json
import re
from pathlib import Path
import subprocess
from urllib.parse import urlsplit
import zipfile
import yaml

ROOT = Path(__file__).resolve().parents[1]


def canonical_repo(value):
    if not isinstance(value, str):
        return None
    parsed = urlsplit(value)
    parts = parsed.path.strip('/').split('/')
    if parsed.scheme != 'https' or parsed.netloc.lower() != 'github.com' or len(parts) != 2:
        return None
    if not all(re.fullmatch(r'[A-Za-z0-9_.-]+', p) for p in parts) or parsed.query or parsed.fragment:
        return None
    return 'github:' + '/'.join(parts).removesuffix('.git').lower()


def image_family(image):
    if not image:
        return None
    image = image.split('@')[0]
    head, sep, tail = image.rpartition('/')
    tail = tail.split(':')[0]
    return (head + sep + tail).removeprefix('docker.io/').lower()


def apply_review(row, review):
    if review['revision'] != row['provenance']['revision'] or review['images'] != row.get('images'):
        raise ValueError('stale candidate review: revision or images changed')
    if canonical_repo(review['identity'].replace('github:', 'https://github.com/')) != review['identity']:
        raise ValueError('invalid reviewed repository identity')
    row['identity'] = review['identity']
    row['identity_status'] = 'reviewed_repository_match'
    row['maintenance_status'] = review['maintenance_status']
    row['screening_review'] = review


def read_sources():
    lock = json.loads((ROOT / 'catalog/import-audit-sources.json').read_text())
    sources = {source: lock[source] for source in ('runtipi', 'caprover')}
    records, provenance = [], {}
    for source, pin in sources.items():
        archive = ROOT / '.cache/catalog' / f"{source}-{pin['revision']}.zip"
        if hashlib.sha256(archive.read_bytes()).hexdigest() != pin['sha256']:
            raise ValueError(f'{source}: archive checksum mismatch')
        with zipfile.ZipFile(archive) as bundle:
            for member in sorted(bundle.infolist(), key=lambda m: m.filename):
                parts = member.filename.split('/')
                selected = (source == 'runtipi' and len(parts) == 4 and parts[1] == 'apps' and parts[3] == 'docker-compose.json') or (source == 'caprover' and '/public/v4/apps/' in member.filename and member.filename.endswith(('.yml', '.yaml')))
                if not selected:
                    continue
                if member.file_size > 1024 * 1024:
                    raise ValueError('oversized definition')
                app_id = parts[2] if source == 'runtipi' else Path(member.filename).stem
                record = {'source':source, 'id':app_id, 'definition':yaml.safe_load(bundle.read(member))}
                if source == 'runtipi':
                    config_path = '/'.join(parts[:-1]) + '/config.json'
                    if bundle.getinfo(config_path).file_size > 1024 * 1024:
                        raise ValueError('oversized config')
                    record['config'] = json.loads(bundle.read(config_path))
                records.append(record)
                key = source + ':' + app_id
                if key in provenance:
                    raise ValueError('duplicate source identity')
                provenance[key] = {'revision':pin['revision'], 'path':'/'.join(parts[1:]), 'archive_sha256':pin['sha256'], 'repository':pin.get('repository'), 'declared_project':canonical_repo(record.get('config', {}).get('source'))}
    return records, provenance


def main():
    records, provenance = read_sources()
    result = subprocess.run(['cargo','run','--locked','--quiet','--example','candidate_queue'], cwd=ROOT, input=json.dumps(records), capture_output=True, text=True, check=True)
    rows = json.loads(result.stdout)
    if len(rows) != len(records):
        raise ValueError('adapter omitted records')
    groups = {}
    reviews = json.loads((ROOT / 'catalog/candidate-reviews.json').read_text())['reviews']
    known_keys = {row['source'] + ':' + row['id'] for row in rows}
    if reviews.keys() - known_keys:
        raise ValueError('review references missing candidate')
    for row in rows:
        key = row['source'] + ':' + row['id']
        row['provenance'] = provenance[key]
        identity = provenance[key]['declared_project']
        row['identity'] = identity or 'unresolved:' + key
        row['identity_status'] = 'source_declared_repository' if identity else 'unresolved'
        row['image_family'] = image_family(row.get('primary_image'))
        row['maintenance_status'] = 'not_screened'
        row['verification_status'] = 'not_qualified_by_this_queue'
        if key in reviews:
            apply_review(row, reviews[key])
        groups.setdefault(row['identity'], []).append(key)
    for row in rows:
        row['possible_matches'] = [r['source']+':'+r['id'] for r in rows if r is not row and row.get('image_family') and r.get('image_family') == row['image_family'] and r['identity'] != row['identity']]
    # Reviewed shortlist first; remaining rows have structural order only.
    rows.sort(key=lambda r: (r.get('screening_review', {}).get('screening_rank', 999), not r['importable'], r.get('required_inputs', 999), r.get('service_count', 999), len(r.get('fields', [])), r['identity'], r['source']))
    summary = {s: {'definitions':sum(r['source']==s for r in rows), 'expressible':sum(r['source']==s and r['importable'] for r in rows)} for s in ['caprover','runtipi']}
    summary['resolved_project_groups'] = sum(not k.startswith('unresolved:') for k in groups)
    summary['unresolved_definitions'] = sum(r['identity_status']=='unresolved' for r in rows)
    summary['confirmed_unique_verified_apps'] = None
    payload = {'schema_version':1, 'summary':summary, 'identity_groups':groups, 'candidates':rows}
    (ROOT / 'catalog/candidate-queue.json').write_text(json.dumps(payload, indent=2)+'\n', encoding='utf-8')
    lines = ['# Candidate queue baseline', '', 'Generated offline by `python scripts/build-candidate-queue.py` using actual Rust adapters.', '', '```json', json.dumps(summary, indent=2), '```', '', 'Repository URLs are source-declared identities, not independently verified matches.', 'Unknown identities remain source-qualified; equal image repositories are possible matches,', 'never automatic app merges. Consequently no unique verified-app total is claimed.', '', 'Ranking is for structural screening only: expressible, fewer required inputs, fewer services,', 'then fewer fields. Maintenance, image age, host compatibility and first use still need review.', 'No candidate is promoted. Field values/defaults and generated credentials are not exported.', '', '## First 20 structural screening candidates', '', '| Source | App | Services | Required inputs | Identity |', '| --- | --- | --- | --- | --- |']
    for row in [r for r in rows if r['importable']][:20]:
        lines.append(f"| {row['source']} | {row['id']} | {row['service_count']} | {row['required_inputs']} | {row['identity_status']} |")
    lines.extend(['', '## Reviewed shortlist', '', 'The first four rows use the explicit maintenance/first-use screening order in', '`catalog/candidate-reviews.json`; remaining rows retain structural ordering.', 'See `docs/candidate-screening.md` for the decision and qualification gates.', 'Reviewed repository matches merge identity only, never deployment definitions or approval.'])
    report = '\n'.join(lines).replace(
        'Ranking is for structural screening only: expressible, fewer required inputs, fewer services,\nthen fewer fields. Maintenance, image age, host compatibility and first use still need review.',
        'Four reviewed candidates lead the queue; remaining rows use structural screening:\nexpressible, fewer required inputs, services and fields. First use remains unverified.'
    ).replace('## First 20 structural screening candidates', '## First 20 screening candidates')
    (ROOT / 'docs/candidate-queue-baseline.md').write_text(report+'\n', encoding='utf-8')
    print(json.dumps(summary))


if __name__ == '__main__':
    main()

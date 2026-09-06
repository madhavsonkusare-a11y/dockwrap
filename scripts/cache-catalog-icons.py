"""Bundle validated icons from a pinned Homarr snapshot. No runtime network requests."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'catalog/icons.json'
REPOSITORY = 'homarr-labs/dashboard-icons'
MAX_BYTES = 512 * 1024


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError('Icon redirects are not allowed')


def fetch(url, limit):
    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme != 'https' or parsed.hostname not in ('api.github.com', 'raw.githubusercontent.com'):
        raise ValueError('Unapproved icon host')
    request = urllib.request.Request(url, headers={'User-Agent': 'Local-Store-Icons/1'})
    with urllib.request.build_opener(NoRedirect()).open(request, timeout=20) as response:
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError('Icon exceeds size limit')
    return data


def validate_svg(data):
    if len(data) > MAX_BYTES or re.search(br'<!DOCTYPE|<!ENTITY', data, re.I):
        raise ValueError('Unsafe SVG declaration or size')
    root = ET.fromstring(data)
    if root.tag.split('}')[-1] != 'svg':
        raise ValueError('Expected SVG root')
    for element in root.iter():
        if element.tag.split('}')[-1] in ('script', 'foreignObject', 'image', 'iframe', 'animate', 'animateTransform', 'set'):
            raise ValueError('SVG contains active or external content')
        for key, value in element.attrib.items():
            name = key.split('}')[-1].lower()
            if name.startswith('on') or (name in ('href', 'src') and not value.startswith('#')):
                raise ValueError('SVG contains an event or external reference')
            if re.search(r'url\(\s*["\']?(?!#)', value, re.I) or 'javascript:' in value.lower():
                raise ValueError('SVG contains external styling')
        if element.tag.split('}')[-1] == 'style' and element.text:
            if '@import' in element.text.lower() or re.search(r'url\(\s*["\']?(?!#)', element.text, re.I):
                raise ValueError('SVG style references external content')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh', action='store_true')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        manifest = json.loads(MANIFEST.read_text())
        for record in manifest['icons'].values():
            path = record['path']
            if not re.fullmatch(r'assets/catalog/[a-z0-9-]+\.svg', path):
                raise ValueError('Invalid cached icon path')
            data = (ROOT / 'src' / path).read_bytes()
            validate_svg(data)
            if hashlib.sha256(data).hexdigest() != record['sha256']:
                raise ValueError(f'Icon checksum mismatch: {path}')
        print(f"Verified {len(manifest['icons'])} cached SVGs offline.")
        return
    if not args.refresh:
        parser.error('Choose --refresh or --check')
    previous = json.loads(MANIFEST.read_text()) if MANIFEST.exists() else {}
    revision = previous.get('revision') or json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/commits/main', 2 * 1024 * 1024))['sha']
    tree = json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/git/trees/{revision}?recursive=1', 8 * 1024 * 1024))
    if tree.get('truncated'):
        raise ValueError('Incomplete icon inventory')
    paths = {Path(item['path']).stem: item['path'] for item in tree['tree'] if re.fullmatch(r'svg/[^/]+\.svg', item['path'])}
    projects = json.loads((ROOT / 'src/generated/catalog.json').read_text(encoding='utf-8'))['entries']
    (ROOT / 'src/assets/catalog').mkdir(exist_ok=True)
    aliases = {'actual-budget': 'actual', 'uptime-kuma': 'uptime-kuma', 'changedetection-io': 'changedetection', 'wiki-js': 'wikijs', 'esp-home': 'esphome'}
    def cache(entry):
        candidates = [entry['id'], aliases.get(entry['id']), entry['id'].replace('-', '')]
        match = next((paths[key] for key in candidates if key in paths), None)
        if not match:
            return entry['id'], None, 'No matching upstream SVG'
        url = f'https://raw.githubusercontent.com/{REPOSITORY}/{revision}/{match}'
        output = ROOT / 'src/assets/catalog' / f"{entry['id']}.svg"
        try:
            old = previous.get('icons', {}).get(entry['id'])
            data = output.read_bytes() if old and old['url'] == url and output.exists() else fetch(url, MAX_BYTES)
            validate_svg(data)
            output.write_bytes(data)
            return entry['id'], {'path': f"assets/catalog/{entry['id']}.svg", 'url': url, 'sha256': hashlib.sha256(data).hexdigest()}, None
        except (ValueError, ET.ParseError, OSError) as error:
            return entry['id'], None, str(error)
    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(cache, projects))
    manifest = {'repository': REPOSITORY, 'revision': revision, 'license': 'Apache-2.0',
                'icons': {key: record for key, record, _ in results if record},
                'fallbacks': {key: reason for key, record, reason in results if not record}}
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(f"Cached {len(manifest['icons'])} app icons; {len(manifest['fallbacks'])} use monograms.")


if __name__ == '__main__':
    main()

"""Bundle validated SVG/PNG icons from pinned upstreams. No runtime requests."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import io
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
ICON_PATH = r'assets/catalog/[a-z0-9-]+\.(svg|png)'


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


def validate_icon(data, suffix):
    if suffix == '.svg':
        validate_svg(data)
        return
    if suffix != '.png' or len(data) > MAX_BYTES or not data.startswith(b'\x89PNG\r\n\x1a\n'):
        raise ValueError('Invalid PNG signature, format or size')
    from PIL import Image
    # Inspect dimensions before decoding; verify chunks, then actually decode.
    with Image.open(io.BytesIO(data)) as image:
        if image.format != 'PNG' or not (0 < image.width <= 2048 and 0 < image.height <= 2048):
            raise ValueError('Invalid PNG dimensions')
        if getattr(image, 'is_animated', False):
            raise ValueError('Animated icons are not allowed')
        image.verify()
    with Image.open(io.BytesIO(data)) as image:
        image.load()


def homarr_candidates(project_id, paths, aliases):
    # Reviewed aliases only: fuzzy matches can assign an unrelated app's logo.
    preferred = aliases.get(project_id)
    # Some upstreams label theme variants inconsistently. An exact reviewed
    # asset path takes precedence over suffix conventions and prior choices.
    if preferred and '/' in preferred:
        return [preferred] if preferred in paths else []
    keys = [aliases.get(project_id), project_id, project_id.replace('-', '')]
    return list(dict.fromkeys(
        path for key in keys if key
        for variant in (key + '-light', key)
        for suffix in ('svg', 'png')
        if (path := f'{suffix}/{variant}.{suffix}') in paths
    ))


def coolify_icons():
    lock = json.loads((ROOT / 'catalog/sources.lock.json').read_text())
    source = next(item for item in lock['sources'] if item['id'] == 'coolify')
    data = (ROOT / 'catalog/sources/coolify.json').read_bytes()
    if hashlib.sha256(data).hexdigest() != source['sha256']:
        raise ValueError('Coolify source checksum mismatch')
    snapshot = json.loads(data)
    if snapshot['revision'] != source['revision']:
        raise ValueError('Coolify source revision mismatch')
    prefix = f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/"
    # Compose headers use web-root paths; in the source tree those files live
    # under public/. Keep the same immutable revision and listing association.
    icons = {row['upstream_id']: [prefix + 'public/' + row['icon'][len(prefix):], row['icon']] for row in snapshot['records']
             if row.get('icon') and row['icon'].startswith(prefix)
             and re.fullmatch(r'svgs/[a-zA-Z0-9_-]+\.(svg|png)', row['icon'][len(prefix):])}
    return source, icons


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh', action='store_true')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        manifest = json.loads(MANIFEST.read_text())
        for record in manifest['icons'].values():
            path = record['path']
            if not re.fullmatch(ICON_PATH, path):
                raise ValueError('Invalid cached icon path')
            data = (ROOT / 'src' / path).read_bytes()
            validate_icon(data, Path(path).suffix)
            if hashlib.sha256(data).hexdigest() != record['sha256']:
                raise ValueError(f'Icon checksum mismatch: {path}')
        print(f"Verified {len(manifest['icons'])} cached SVG/PNG icons offline.")
        return
    if not args.refresh:
        parser.error('Choose --refresh or --check')
    previous = json.loads(MANIFEST.read_text()) if MANIFEST.exists() else {}
    revision = previous.get('revision') or json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/commits/main', 2 * 1024 * 1024))['sha']
    tree = json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/git/trees/{revision}?recursive=1', 8 * 1024 * 1024))
    if tree.get('truncated'):
        raise ValueError('Incomplete icon inventory')
    paths = {item['path'] for item in tree['tree'] if re.fullmatch(r'(svg|png)/[^/]+\.(svg|png)', item['path'])}
    projects = json.loads((ROOT / 'src/generated/catalog.json').read_text(encoding='utf-8'))['entries']
    (ROOT / 'src/assets/catalog').mkdir(exist_ok=True)
    aliases = json.loads((ROOT / 'catalog/icon-aliases.json').read_text())
    coolify, supplemental = coolify_icons()
    def cache(entry):
        old = previous.get('icons', {}).get(entry['id'])
        candidates = [f'https://raw.githubusercontent.com/{REPOSITORY}/{revision}/{path}'
                      for path in homarr_candidates(entry['id'], paths, aliases)]
        candidates.extend(url for s in entry['sources']
                          if s['source'] == 'coolify' and s['revision'] == coolify['revision']
                          and s['upstream_id'] in supplemental
                          for url in supplemental[s['upstream_id']])
        # Preserve existing, valid artwork at the same pin; only fill gaps.
        if old and old['url'] in candidates:
            candidates.insert(0, old['url'])
        errors = []
        for url in dict.fromkeys(candidates):
            suffix = Path(urllib.parse.urlsplit(url).path).suffix
            path = f"assets/catalog/{entry['id']}{suffix}"
            output = ROOT / 'src' / path
            try:
                data = output.read_bytes() if old and old['url'] == url and output.exists() else None
                if data is None or hashlib.sha256(data).hexdigest() != old['sha256']:
                    data = fetch(url, MAX_BYTES)
                validate_icon(data, suffix)
                output.write_bytes(data)
                return entry['id'], {'path': path, 'url': url, 'sha256': hashlib.sha256(data).hexdigest()}, None
            except (ValueError, ET.ParseError, OSError, SyntaxError) as error:
                errors.append(f'{Path(urllib.parse.urlsplit(url).path).name}: {error}')
        return entry['id'], None, '; '.join(errors) if errors else 'No matching pinned upstream artwork'
    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(cache, projects))
    manifest = {'repository': REPOSITORY, 'revision': revision, 'license': 'Apache-2.0',
                'additional_sources': {'coolify': {key: coolify[key] for key in ('repository', 'revision', 'license')}},
                'icons': {key: record for key, record, _ in results if record},
                'fallbacks': {key: reason for key, record, reason in results if not record}}
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(f"Cached {len(manifest['icons'])} app icons; {len(manifest['fallbacks'])} use monograms.")


if __name__ == '__main__':
    main()

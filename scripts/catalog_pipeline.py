"""Reproducible, data-only imports. Network access occurs only with --refresh.

Upstream archives are read in memory without extracting or executing their files.
Committed normalized inputs support offline generation and CI --check.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import subprocess
from pathlib import Path
import re
import unicodedata
import urllib.parse
import urllib.request
import zipfile

import yaml

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "catalog/sources.lock.json"
OUTPUT = ROOT / "src/generated/catalog.json"


def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def slug(value):
    value = unicodedata.normalize("NFKD", str(value)).encode("ascii", "ignore").decode().lower()
    return re.sub(r"[^a-z0-9]+", "-", value).strip("-")[:55].rstrip("-")


def http_url(value):
    if not isinstance(value, str) or len(value) > 2048 or re.search(r"[\s\x00-\x1f]", value):
        return None
    try:
        url = urllib.parse.urlsplit(value)
        if url.scheme not in ("http", "https") or not url.hostname or url.username or url.password:
            return None
        return value
    except ValueError:
        return None


def identity(value):
    value = http_url(value)
    if not value:
        return None
    url = urllib.parse.urlsplit(value)
    host = url.hostname.lower().removeprefix("www.")
    path = url.path.rstrip("/").removesuffix(".git")
    if host == "gitlab.com":
        path = path.split("/-/", 1)[0].lower()
    if host in ("github.com", "codeberg.org"):
        path = "/" + "/".join(path.strip("/").split("/")[:2]).lower()
        if path.count("/") < 2:
            return None
    return host + path


def strings(value):
    if isinstance(value, str):
        return [value] if value else []
    return [str(item) for item in value] if isinstance(value, list) else []


def english(value):
    return value.get("en_US", "") if isinstance(value, dict) else str(value or "")


def row(name, source_url, website_url, description, categories, **extra):
    return dict(name=name, source_url=http_url(source_url), website_url=http_url(website_url),
                description=str(description or "").strip(), categories=strings(categories), **extra)


def adapter(source, path, content):
    kind = source["id"]
    if kind == "awesome-selfhosted":
        item = yaml.safe_load(content)
        return row(item["name"], item.get("source_code_url") or item.get("website_url"),
                   item.get("website_url"), item.get("description"), item.get("tags"),
                   licenses=strings(item.get("licenses")), platforms=strings(item.get("platforms")),
                   updated_at=str(item.get("updated_at") or ""), archived=bool(item.get("archived")),
                   # Upstream deployment languages are not a desktop compatibility claim.
                   web_ui=None)
    if kind == "runtipi":
        item = json.loads(content)
        return row(item["name"], item.get("source"), item.get("website"), item.get("short_desc") or item.get("description"),
                   item.get("categories"), architectures=strings(item.get("supported_architectures")),
                   available=item.get("available", True), web_ui=bool(item.get("exposable")))
    if kind == "casaos":
        item = yaml.safe_load(content)
        meta = item.get("x-casaos", {})
        return row(english(meta.get("title")) or path.split("/")[1], meta.get("repo") or meta.get("website"),
                   meta.get("website"), english(meta.get("tagline")) or english(meta.get("description")),
                   meta.get("category"), architectures=strings(meta.get("architectures")),
                   icon=http_url(meta.get("icon")), web_ui=bool(meta.get("port_map")))
    # Coolify stores discoverable metadata in structured header comments.
    header = dict(re.findall(r"(?m)^# ([a-z_]+):\s*(.*)$", content))
    doc = http_url(header.get("documentation"))
    item = yaml.safe_load(content)
    name = header.get("name") or Path(path).stem.replace("-", " ").title()
    icon_path = header.get("logo", "")
    icon = f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/{icon_path}" if icon_path.startswith("svgs/") else None
    return row(name, doc, doc, header.get("slogan"), header.get("category"),
               keywords=header.get("tags", "").split(","), icon=icon,
               web_ui=bool(header.get("port")), variants=list((item or {}).get("services", {})))


PATTERNS = {
    "awesome-selfhosted": r"software/[^/]+\.yml",
    "runtipi": r"apps/[^/]+/config\.json",
    "casaos": r"Apps/[^/]+/docker-compose\.ya?ml",
    "coolify": r"templates/compose/[^/]+\.ya?ml",
}


def refresh_source(source):
    url = f"https://codeload.github.com/{source['repository']}/zip/{source['revision']}"
    cache = ROOT / ".cache/catalog" / f"{source['id']}-{source['revision']}.zip"
    cache.parent.mkdir(parents=True, exist_ok=True)
    selected_cache = cache.with_suffix('.manifests.json')
    selected = {}
    if selected_cache.exists():
        selected = json.loads(selected_cache.read_text(encoding='utf-8'))
    elif not cache.exists() and source['id'] in ('casaos', 'coolify'):
        def fetch(url, bound):
            request = urllib.request.Request(url, headers={'User-Agent': 'Local-Store-Catalog/1'})
            with urllib.request.urlopen(request, timeout=45) as response:
                data = response.read(bound + 1)
            if len(data) > bound:
                raise ValueError('upstream response exceeds its size limit')
            return data
        tree = json.loads(fetch(f"https://api.github.com/repos/{source['repository']}/git/trees/{source['revision']}?recursive=1", 8 * 1024 * 1024))
        if tree.get('truncated'):
            raise ValueError('upstream tree is incomplete')
        paths = [entry['path'] for entry in tree['tree'] if re.fullmatch(PATTERNS[source['id']], entry['path']) or entry['path'] in ('LICENSE', 'LICENSE.md', 'LICENSE.txt', 'AUTHORS', 'NOTICE')]
        def fetch_path(path):
            url = f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/{path}"
            return path, fetch(url, 1024 * 1024).decode('utf-8-sig')
        with ThreadPoolExecutor(max_workers=6) as pool:
            selected = dict(pool.map(fetch_path, sorted(paths)))
        selected_cache.write_bytes(encode(selected))
    elif not cache.exists():
        request = urllib.request.Request(url, headers={"User-Agent": "Local-Store-Catalog/1"})
        with urllib.request.urlopen(request, timeout=45) as response:
            data = response.read(180 * 1024 * 1024 + 1)
        if len(data) > 180 * 1024 * 1024:
            raise ValueError("upstream archive exceeds 180 MiB")
        cache.write_bytes(data)
    records, excluded, notices = [], [], {}
    if not selected:
        with zipfile.ZipFile(cache) as archive:
            for member in sorted(archive.infolist(), key=lambda entry: entry.filename):
                path = member.filename.partition("/")[2]
                if not re.fullmatch(PATTERNS[source["id"]], path) and path not in ("LICENSE", "LICENSE.md", "LICENSE.txt", "AUTHORS", "NOTICE"):
                    continue
                if member.file_size > 1024 * 1024:
                    raise ValueError(f"manifest too large: {path}")
                selected[path] = archive.read(member).decode("utf-8-sig")
    for path, content in sorted(selected.items()):
        if path in ("LICENSE", "LICENSE.md", "LICENSE.txt", "AUTHORS", "NOTICE"):
            notices[path] = content
            continue
        try:
            item = adapter(source, path, content)
        except (yaml.YAMLError, json.JSONDecodeError, KeyError, TypeError, AttributeError) as error:
            excluded.append({"path": path, "reason": f"Invalid upstream manifest: {type(error).__name__}"})
            continue
        if not item.get("source_url") or not item.get("name") or not item.get("description"):
            excluded.append({"path": path, "reason": "Missing name, description or valid project URL"})
            continue
        item["upstream_id"] = path
        records.append(item)
    output = ROOT / "catalog/sources" / f"{source['id']}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    data = encode({"source": source["id"], "revision": source["revision"], "records": records, "excluded": excluded})
    output.write_bytes(data)
    notice_dir = ROOT / "catalog/notices" / source["id"]
    notice_dir.mkdir(parents=True, exist_ok=True)
    for name, text in notices.items():
        (notice_dir / name).write_text(text, encoding="utf-8")
    print(f"{source['id']}: {len(records)} usable records, {len(excluded)} exclusions", flush=True)
    return hashlib.sha256(data).hexdigest()


# Upstream taxonomies differ. Retain detailed editorial categories, normalize broad aliases.
CATEGORY_ALIASES = {
    "ai": "Artificial Intelligence", "finance": "Money, Budgeting & Management",
    "rss": "Feed Readers", "mail": "Communication - Email", "email": "Communication - Email",
    "books": "Document Management - E-books", "cms": "Content Management Systems (CMS)",
    "git": "Software Development - Source Control", "ci": "Software Development - Testing",
    "developer": "Software Development - IDE & Tools", "development": "Software Development - IDE & Tools",
    "devtools": "Software Development - IDE & Tools", "api": "Software Development - API Management",
    "database": "Database Management", "databases": "Database Management", "data": "Database Management",
    "gaming": "Games", "network": "Network Utilities", "networking": "Network Utilities",
    "health": "Health and Fitness", "helpdesk": "Ticketing", "social": "Communication - Social Networks and Forums",
    "monitoring": "Monitoring", "mcp": "Artificial Intelligence", "auth": "Identity and Access Management",
    "messaging": "Communication - Custom Communication Systems", "communication": "Communication - Custom Communication Systems",
    "storage": "File Transfer & Synchronization", "home": "Internet of Things (IoT)",
}


def normalize_category(value):
    value = value.strip()
    if value == 'database,observability,developer-tools':
        value = 'database'
    return CATEGORY_ALIASES.get(value.lower(), value[:1].upper() + value[1:])


# Curated identity overrides are explicit: deployments/forks never merge by name alone.
def load_overrides():
    path = ROOT / "catalog/overrides.json"
    return json.loads(path.read_text()) if path.exists() else {"identity_aliases": {}, "names": {}, "web_ui": []}


def generate(lock):
    overrides = load_overrides()
    groups, index, excluded, counts = [], {}, [], {}
    def canonical(url):
        key = identity(url)
        if key is None and http_url(url):
            parsed = urllib.parse.urlsplit(url)
            candidate = parsed.hostname.lower() + parsed.path.rstrip('/').lower()
            if candidate in overrides['identity_aliases']:
                key = candidate
        return overrides["identity_aliases"].get(key, key)
    def add(item, provenance):
        keys = {canonical(item.get(key)) for key in ("source_url", "website_url")} - {None}
        matches = {index[key] for key in keys if key in index}
        # Documentation and homepages may use different paths/subdomains. Require
        # BOTH the same normalized name and a related first-party hostname.
        # Names alone never join different projects (e.g. the two Memex projects).
        def host_family(url):
            if not http_url(url):
                return None
            host = urllib.parse.urlsplit(url).hostname.lower().removeprefix('www.')
            for prefix in ('docs.', 'doc.', 'documentation.', 'manual.', 'help.', 'support.'):
                host = host.removeprefix(prefix)
            if host in ('github.com', 'gitlab.com', 'codeberg.org', 'hub.docker.com', 'readthedocs.io', 'linuxserver.io', 'coolify.io'):
                return None
            return host
        name_key = slug(item['name']).replace('-', '')
        if not matches:
            hosts = {host_family(item.get(key)) for key in ('source_url', 'website_url')} - {None}
            for number, group in enumerate(groups):
                if hosts and any(slug(other['name']).replace('-', '') == name_key and hosts.intersection({host_family(other.get(key)) for key in ('source_url', 'website_url')} - {None}) for other, _ in group):
                    matches.add(number)
        if len(matches) > 1:
            # A project can introduce a missing bridge between previously distinct records.
            first = min(matches)
            for other in sorted(matches - {first}):
                groups[first].extend(groups[other]); groups[other] = []
                index.update({key: first for key, value in index.items() if value == other})
            group_id = first
        else:
            group_id = next(iter(matches), len(groups))
        if group_id == len(groups):
            groups.append([])
        groups[group_id].append((item, provenance))
        index.update({key: group_id for key in keys})
    for source in lock["sources"]:
        data = (ROOT / "catalog/sources" / f"{source['id']}.json").read_bytes()
        if hashlib.sha256(data).hexdigest() != source["sha256"]:
            raise ValueError(f"source checksum mismatch: {source['id']}")
        snapshot = json.loads(data)
        if snapshot["revision"] != source["revision"]:
            raise ValueError("source revision mismatch")
        counts[source["id"]] = len(snapshot["records"])
        excluded.extend(dict(source=source["id"], **item) for item in snapshot["excluded"])
        for item in snapshot["records"]:
            add(item, {"source": source["id"], "upstream_id": item["upstream_id"], "revision": source["revision"],
                       "url": f"https://github.com/{source['repository']}/blob/{source['revision']}/{item['upstream_id']}"})
    legacy = json.loads((ROOT / "catalog/legacy.json").read_text(encoding="utf-8"))
    for item in legacy:
        if not http_url(item['url']):
            excluded.append({'source': 'legacy', 'path': item['name'], 'reason': 'Invalid project URL in previous snapshot'})
            continue
        add(row(item["name"], item["url"], None, item.get("description"), item.get("category"),
                licenses=[item.get("tags", "").replace("`", "")], icon=item.get("icon"), legacy=True),
            {"source": "legacy", "upstream_id": item["name"], "revision": "52a36e45fbd403c1c1348f200105d9e31be2bc3c",
             "url": "https://github.com/madhavsonkusare-a11y/local-store/blob/52a36e45fbd403c1c1348f200105d9e31be2bc3c/src/catalog_full.json"})
    # Local Store's own definitions are a source too: an app no store packages
    # usably still needs a catalogue entry for its listing and its icon.
    for config_path in sorted((ROOT / "definitions/apps").glob("*/config.json")):
        config = json.loads(config_path.read_text(encoding="utf-8"))
        folder = config_path.parent.relative_to(ROOT).as_posix()
        revision = subprocess.run(
            ["git", "log", "-1", "--format=%H", "--", folder],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout.strip()
        if not revision:
            raise ValueError(f"{folder} is not committed, so it has no revision to cite")
        add(row(config["name"], config["source"], config.get("website"), config.get("short_desc"),
                config.get("categories"), licenses=[config["license"]] if config.get("license") else [],
                web_ui=True),
            {"source": "local-store", "upstream_id": folder, "revision": revision,
             "url": f"https://github.com/madhavsonkusare-a11y/local-store/blob/{revision}/{folder}/config.json"})
    id_file = ROOT / 'catalog/ids.json'
    saved_ids = json.loads(id_file.read_text()) if id_file.exists() else {}
    next_ids = dict(saved_ids)
    entries, used_ids, merges = [], set(), []
    for group in filter(None, groups):
        first = group[0][0]
        aliases = sorted({item["name"] for item, _ in group})
        key = canonical(first["source_url"]) or first["source_url"]
        name = overrides["names"].get(key, first["name"])
        keys = sorted({canonical(item.get(field)) for item, _ in group for field in ('source_url', 'website_url')} - {None})
        known_id = saved_ids.get(key) or next((saved_ids[value] for value in keys if value in saved_ids), None)
        entry_id = known_id or slug(name) or "app-" + hashlib.sha256(key.encode()).hexdigest()[:12]
        if known_id and entry_id in used_ids:
            raise ValueError(f'Identity split requires an explicit migration: {entry_id}')
        if entry_id in used_ids or (not known_id and entry_id in saved_ids.values()):
            entry_id += "-" + hashlib.sha256(key.encode()).hexdigest()[:8]
        used_ids.add(entry_id)
        next_ids.update({value: entry_id for value in keys})
        next_ids[key] = entry_id
        collect = lambda field: sorted({value for item, _ in group for value in strings(item.get(field)) if value})
        categories = collect("categories")
        # First source has editorial priority; aliases preserve previous display names.
        category = next(iter(first.get("categories") or categories), "Other")
        category = normalize_category(category)
        legacy_only = all(item.get("legacy") for item, _ in group)
        archived = bool(first.get("archived"))
        web_ui = entry_id in overrides["web_ui"] or any(item.get("web_ui") is True for item, _ in group)
        entry = dict(id=entry_id, name=name, aliases=aliases, source_url=first["source_url"],
                     website_url=next((item["website_url"] for item, _ in group if item.get("website_url")), first["source_url"]),
                     description=first["description"], category=category, tags=sorted(set(categories + collect("keywords"))),
                     licenses=collect("licenses"), platforms=collect("platforms"), architectures=collect("architectures"),
                     updated_at=first.get("updated_at") or None, archived=archived, warning=archived or legacy_only,
                     maintenance="archived" if archived else "snapshot_only" if legacy_only else "tracked",
                     web_ui=web_ui, icon=None, sources=[provenance for _, provenance in group])
        entries.append(entry)
        if len(group) > 1:
            merges.append({"id": entry_id, "names": aliases, "sources": [p["source"] for _, p in group]})
    browse_order = {key: rank for rank, key in enumerate(overrides.get("browse_first", []))}
    entries.sort(key=lambda item: (browse_order.get(item["id"], len(browse_order)), item["name"].casefold(), item["id"]))
    icon_manifest = ROOT / "catalog/icons.json"
    icons = json.loads(icon_manifest.read_text()) if icon_manifest.exists() else {"icons": {}}
    for entry in entries:
        entry["icon"] = icons["icons"].get(entry["id"], {}).get("path")
    report = dict(source_records=counts, excluded=excluded, merges=merges,
                  total=len(entries), tracked=sum(e["maintenance"] == "tracked" for e in entries),
                  archived=sum(e["archived"] for e in entries), snapshot_only=sum(e["maintenance"] == "snapshot_only" for e in entries),
                  cached_icons=sum(bool(e["icon"]) for e in entries),
                  legacy_aliases={item["name"]: next((e["id"] for e in entries if item["name"] in e["aliases"]), None) for item in legacy})
    return {"schema_version": 1, "snapshot_date": lock["snapshot_date"], "entries": entries}, report, next_ids


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.refresh and args.check:
        parser.error("--refresh and --check are mutually exclusive")
    lock = json.loads(LOCK.read_text())
    if args.refresh:
        with ThreadPoolExecutor(max_workers=4) as pool:
            hashes = list(pool.map(refresh_source, lock["sources"]))
        for source, digest in zip(lock["sources"], hashes):
            source["sha256"] = digest
        LOCK.write_bytes(encode(lock))
    catalog, report, ids = generate(lock)
    for path, value in [(OUTPUT, catalog), (ROOT / "catalog/import-report.json", report), (ROOT / 'catalog/ids.json', ids)]:
        data = encode(value)
        if args.check:
            if not path.exists() or path.read_bytes() != data:
                raise SystemExit(f"Generated data is stale: {path.relative_to(ROOT)}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    print(f"Catalog: {report['total']} distinct records ({report['tracked']} tracked, {report['archived']} archived, {report['snapshot_only']} legacy-only); {len(report['merges'])} merged groups; {report['cached_icons']} cached icons.")


if __name__ == "__main__":
    main()

"""Opt-in real CLI/Docker lifecycle proof. Never run as an ordinary unit test.

Requires an already built binary. Refuses existing recipe containers, networks,
volumes or occupied ports. Failed runs retain their private files for recovery.
Successful runs uninstall only the apps created by this invocation.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]


def run(args, env=None, timeout=1200):
    result = subprocess.run(args, env=env, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"{args[0:3]} exited {result.returncode}: {result.stderr[-4000:]}")
    return result.stdout.strip()


def health(url):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    deadline = time.monotonic() + 90
    while time.monotonic() < deadline:
        try:
            with opener.open(url, timeout=3) as response:
                if 200 <= response.status < 400:
                    return response.status
        except (OSError, urllib.error.URLError):
            pass
        time.sleep(1)
    raise RuntimeError(f"Health deadline exceeded: {url}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--recipe", choices=("memos", "n8n", "uptime-kuma"),
                        help="Test one recipe; defaults to all three")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    recipes = [json.loads((ROOT / f"src/recipes/{name}.json").read_text())
               for name in ((args.recipe,) if args.recipe else ("memos", "n8n", "uptime-kuma"))]
    engine = json.loads(run(["docker", "version", "--format", "json"], timeout=20))
    # All preflight checks happen before creating files or containers.
    for recipe in recipes:
        project = "local-store-" + recipe["id"]
        for resource in ("container", "volume", "network"):
            existing = run(["docker", resource, "ls", "-q", "--filter",
                            f"label=com.docker.compose.project={project}"], timeout=20)
            if existing:
                raise RuntimeError(f"Existing {resource} belongs to {project}; refusing to run")
            if resource != "container":
                names = run(["docker", resource, "ls", "--format", "{{.Name}}"], timeout=20).splitlines()
                if any(name == project or name.startswith(project + "_") for name in names):
                    raise RuntimeError(f"Existing {resource} uses the reserved name {project}")
        if run(["docker", "container", "ls", "-aq", "--filter", f"name=^/{project}$"], timeout=20):
            raise RuntimeError(f"Container name {project} is already in use")
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", recipe["host_port"]))
    root = ROOT / ".cache" / ("recipe-smoke-" + uuid.uuid4().hex)
    root.mkdir(parents=True)
    env = dict(os.environ, APPDATA=str(root), XDG_CONFIG_HOME=str(root))
    report = {"schema_version": 1, "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "engine": engine, "recipes": [], "scope": "CLI lifecycle; no version upgrade or application-content migration proof"}
    def cli(*command):
        return run([str(binary), *command], env)
    try:
        for recipe in recipes:
            app_id = recipe["id"]
            item = {"id": app_id, "image": recipe["image"], "checks": [], "passed": False}
            report["recipes"].append(item)
            def checked(name, action):
                action()
                item["checks"].append(name)
                print(f"{app_id}: {name}", flush=True)
            checked("install", lambda: cli("install", app_id))
            apps = json.loads(cli("list"))
            if not any(app["id"] == app_id for app in apps):
                raise RuntimeError("Install did not register the app")
            project_dir = next(root.rglob(f"{app_id}/compose.yaml")).parent
            if project_dir.resolve().parent.parent.parent != root.resolve():
                raise RuntimeError("Managed directory escaped private config root")
            container = "local-store-" + app_id
            inspection = json.loads(run(["docker", "inspect", container]))[0]
            item["image_id"] = inspection["Image"]
            destination = {"memos": "/var/opt/memos", "n8n": "/home/node/.n8n", "uptime-kuma": "/app/data"}[app_id]
            marker = destination + "/local-store-smoke-marker"
            checked("health", lambda: health(recipe["health_url"]))
            checked("write persistence marker", lambda: run(["docker", "exec", container, "sh", "-c", f"printf lifecycle-proof > {marker}"]))
            checked("logs", lambda: cli("logs", app_id))
            checked("stop", lambda: cli("stop", app_id))
            if cli("status", app_id) != "Stopped":
                raise RuntimeError("Expected stopped status")
            checked("start", lambda: cli("start", app_id))
            checked("restart health", lambda: health(recipe["health_url"]))
            checked("keep-data uninstall", lambda: cli("uninstall", app_id))
            if any(app["id"] == app_id for app in json.loads(cli("list"))):
                raise RuntimeError("Uninstall did not remove registry entry")
            checked("reinstall", lambda: cli("install", app_id))
            checked("reinstall health", lambda: health(recipe["health_url"]))
            if run(["docker", "exec", container, "cat", marker]) != "lifecycle-proof":
                raise RuntimeError("Persistent marker did not survive")
            item["checks"].append("persistent marker survived")
            checked("explicit data deletion", lambda: cli("uninstall", app_id, "--delete-data"))
            if project_dir.exists() or any(app["id"] == app_id for app in json.loads(cli("list"))):
                raise RuntimeError("Deletion left managed files or registry entry")
            for resource in ("container", "volume", "network"):
                if run(["docker", resource, "ls", "-q", "--filter", f"label=com.docker.compose.project={container}"]):
                    raise RuntimeError(f"Deletion left {resource}")
            item["checks"].append("owned resources absent")
            item["passed"] = True
    except Exception as error:
        report["error"] = str(error)
        raise
    finally:
        report["finished_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"Evidence: {root / 'report.json'}", flush=True)


if __name__ == "__main__":
    main()

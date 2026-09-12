"""Validate reviewed recipe JSON against the Compose it ships, without starting containers.

Compose syntax alone is not enough. The manifest states a published port, a
container port and the addresses the launcher opens and probes; the Compose file
states its own mapping. If those disagree, nothing fails until a user installs
the app and is sent to a port nothing is listening on. Docker's own parser
resolves the mapping here so the two are checked against each other.
"""
import json
import pathlib
import re
import subprocess
import tempfile
from urllib.parse import urlsplit

SCHEMA_VERSION = 3
ROOT = pathlib.Path(__file__).resolve().parents[1]
recipes = sorted((ROOT / "src" / "recipes").glob("*.json"))
if len(recipes) < 3:
    raise SystemExit("expected at least three reviewed recipes")


def resolved_config(compose_text, directory, name):
    """Let Docker parse the Compose file, which also validates its syntax."""
    compose = directory / f"{name}.yaml"
    compose.write_text(compose_text, encoding="utf-8")
    result = subprocess.run(
        ["docker", "compose", "-f", str(compose), "config", "--format", "json"],
        capture_output=True,
        text=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise SystemExit(f"{name}: Compose is not valid\n{result.stderr.strip()}")
    return json.loads(result.stdout)


def published_port(config, name):
    """The single loopback mapping a reviewed recipe is allowed to publish."""
    mappings = [
        (service, port)
        for service, definition in config["services"].items()
        for port in definition.get("ports", [])
    ]
    if len(mappings) != 1:
        raise SystemExit(
            f"{name}: expected exactly one published port, found {len(mappings)}"
        )
    service, port = mappings[0]
    if port.get("host_ip") != "127.0.0.1":
        raise SystemExit(
            f"{name}: {service} publishes on {port.get('host_ip')!r};"
            " reviewed recipes bind loopback only"
        )
    return int(port["published"]), int(port["target"])


def validate_images(recipe, config):
    digest = recipe["requirements"]["image_audit"]["index_digest"]
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
        raise ValueError(f"{recipe['id']}: invalid reviewed image digest")
    expected = f"{recipe['image']}@{digest}"
    images = sorted(service["image"] for service in config["services"].values())
    if images != [expected]:
        raise ValueError(f"{recipe['id']}: expected reviewed image {expected!r}, got {images}")


def main():
    with tempfile.TemporaryDirectory(prefix="local-store-recipes-") as directory:
        target = pathlib.Path(directory)
        for path in recipes:
            recipe = json.loads(path.read_text(encoding="utf-8"))
            name = recipe["id"]

            if recipe["schema_version"] != SCHEMA_VERSION:
                raise SystemExit(
                    f"{name}: schema_version {recipe['schema_version']},"
                    f" expected {SCHEMA_VERSION}"
                )

            config = resolved_config(recipe["compose"], target, name)
            host_port, container_port = published_port(config, name)

            if host_port != recipe["host_port"]:
                raise SystemExit(
                    f"{name}: manifest publishes {recipe['host_port']}"
                    f" but Compose publishes {host_port}"
                )
            if container_port != recipe["container_port"]:
                raise SystemExit(
                    f"{name}: manifest container port {recipe['container_port']}"
                    f" but Compose targets {container_port}"
                )

            # The launcher opens and probes these; they must be the published port,
            # never the port inside the container.
            for field in ("launch_url", "health_url"):
                address = urlsplit(recipe[field])
                if (address.scheme != "http" or address.hostname not in ("localhost", "127.0.0.1", "::1")
                        or address.port != host_port or address.username or address.password or address.fragment):
                    raise SystemExit(
                        f"{name}: {field} {recipe[field]!r} does not use"
                        f" the published port {host_port}"
                    )

            try:
                validate_images(recipe, config)
            except ValueError as error:
                raise SystemExit(str(error)) from error

            mapping = f"{host_port}->{container_port}"
            print(f"validated {name} ({recipe['image']}) publishing {mapping}")


if __name__ == "__main__":
    main()

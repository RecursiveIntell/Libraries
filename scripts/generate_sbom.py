#!/usr/bin/env python3
"""Generate a minimal SPDX-2.3 style JSON manifest from cargo metadata."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def load_cargo_metadata() -> dict:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(completed.stdout)


def index_packages(packages: list[dict]) -> dict[str, dict]:
    return {package["id"]: package for package in packages}


def collect_dependency_names(
    package: dict,
    package_index: dict[str, dict],
) -> list[str]:
    deps: list[str] = []
    for dependency in package.get("dependencies", []):
        dependency_id = dependency.get("pkg")
        if not isinstance(dependency_id, str):
            continue
        dependency_package = package_index.get(dependency_id)
        if dependency_package is None:
            continue
        deps.append(dependency_package.get("name", ""))
    # Stable output for diff-friendly manifests.
    return sorted({dep for dep in deps if dep})


def collect_spdx_packages(packages: list[dict], package_index: dict[str, dict]) -> tuple[list[dict], dict[str, str]]:
    package_map: dict[str, str] = {}
    for package in packages:
        package_id = package["id"]
        package_name = package.get("name", package_id)
        package_ref = f"SPDXRef-Package-{package_name.replace('-', '_')}-{package_id.replace(\":\", \"_\")}"
        package_map[package_id] = package_ref

    spdx_packages = []
    for package in packages:
        package_name = package.get("name", "")
        package_id = package["id"]
        package_version = package.get("version", "")
        package_licenses = package.get("license") or package.get("license-file") or "NOASSERTION"
        spdx_packages.append(
            {
                "SPDXID": package_map[package_id],
                "name": package_name,
                "versionInfo": package_version,
                "licenseDeclared": package_licenses,
                "downloadLocation": package.get("source", "NOASSERTION"),
                "filesAnalyzed": False,
                "description": package.get("description", "generated from cargo metadata"),
                "dependencies": collect_dependency_names(package, package_index),
                "relationships": [
                    {
                        "relatedSpdxElement": package_map[dependency_id],
                        "relationshipType": "DEPENDS_ON",
                        "relatedSpdxElementType": "PACKAGE",
                    }
                    for dependency in package.get("dependencies", [])
                    if isinstance((dependency_id := dependency.get("pkg")), str)
                    and dependency_id in package_map
                ],
            }
        )
    return spdx_packages, package_map


def emit_manifest() -> dict:
    metadata = load_cargo_metadata()
    packages = metadata.get("packages", [])
    package_index = index_packages(packages)
    spdx_packages, _ = collect_spdx_packages(packages, package_index)

    manifest_json = json.dumps(
        {
            "packages": [{"name": package.get("name"), "version": package.get("version")} for package in packages],
            "package_count": len(packages),
        },
        sort_keys=True,
    ).encode("utf-8")
    manifest_digest = hashlib.sha256(manifest_json).hexdigest()

    return {
        "spdxVersion": "SPDX-2.3",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "recursive-intell-libraries",
        "dataLicense": "CC0-1.0",
        "documentNamespace": "https://example.com/spdx-docs/recursive-intell-libraries",
        "creationInfo": {
            "created": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "creators": ["Tool: codegen"],
            "licenseListVersion": "3.21",
        },
        "documentDigest": manifest_digest,
        "packages": spdx_packages,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="optional output path; defaults to stdout",
    )
    args = parser.parse_args()

    manifest = emit_manifest()
    if args.output:
        args.output.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    else:
        print(json.dumps(manifest, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

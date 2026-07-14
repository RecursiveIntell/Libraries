#!/usr/bin/env python3
"""Generate a minimal SPDX-2.3 style JSON manifest from cargo metadata."""

from __future__ import annotations

import argparse
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def load_cargo_metadata() -> dict:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(completed.stdout)


def build_package_index(packages: list[dict]) -> dict[str, dict]:
    return {package["id"]: package for package in packages}


def collect_spdx_packages(packages: list[dict]) -> tuple[list[dict], dict[str, str]]:
    package_map = {}
    dependencies: dict[str, list[str]] = {}
    for package in packages:
        package_id = package["id"]
        package_name = package.get("name", package_id)
        package_ref = f"SPDXRef-Package-{package_name.replace('-', '_')}-{abs(hash(package_id))}"
        package_map[package_id] = package_ref
        dependencies[package_id] = [dep.get("pkg") for dep in package.get("dependencies", []) if dep.get("pkg")]

    spdx_packages = []
    for package in packages:
        package_name = package.get("name", "")
        package_id = package["id"]
        spdx_packages.append(
            {
                "SPDXID": package_map[package_id],
                "name": package_name,
                "versionInfo": package.get("version", ""),
                "licenseDeclared": package.get("license", "NOASSERTION")
                or package.get("license_explicit", "NOASSERTION"),
                "downloadLocation": "NONE",
                "filesAnalyzed": False,
                "description": package.get("description", "generated from cargo metadata"),
                "externalRefs": [],
                "hasFiles": [],
                "relationships": [
                    {
                        "relatedSpdxElement": package_map[dependency],
                        "relationshipType": "DEPENDS_ON",
                        "relatedSpdxElementType": "PACKAGE",
                    }
                    for dependency in dependencies.get(package_id, [])
                    if dependency in package_map
                ],
            }
        )
    return spdx_packages, package_map


def emit_manifest() -> dict:
    metadata = load_cargo_metadata()
    packages = metadata.get("packages", [])
    spdx_packages, _ = collect_spdx_packages(packages)

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

#!/usr/bin/env python3
"""Validate root workspace declarations against `cargo metadata` JSON on stdin."""

from __future__ import annotations

import json
import sys
import tomllib
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "Cargo.toml"


def duplicates(values: list[str]) -> list[str]:
    return sorted(value for value, count in Counter(values).items() if count > 1)


def main() -> int:
    errors: list[str] = []
    try:
        cargo_metadata = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        print(f"workspace hygiene error: invalid cargo metadata JSON: {error}", file=sys.stderr)
        return 1

    with MANIFEST.open("rb") as handle:
        manifest = tomllib.load(handle)
    workspace = manifest.get("workspace", {})
    members = workspace.get("members", [])
    defaults = workspace.get("default-members", [])

    for label, entries in (("members", members), ("default-members", defaults)):
        repeated = duplicates(entries)
        if repeated:
            errors.append(f"duplicate workspace {label}: {', '.join(repeated)}")

    for member in members:
        member_manifest = ROOT / member / "Cargo.toml"
        if not member_manifest.is_file():
            errors.append(f"workspace member does not exist: {member}/Cargo.toml")

    missing_defaults = sorted(set(defaults) - set(members))
    if missing_defaults:
        errors.append(
            "default-members not present in workspace members: " + ", ".join(missing_defaults)
        )

    packages = cargo_metadata.get("packages", [])
    names = [package.get("name", "") for package in packages]
    repeated_names = duplicates(names)
    if repeated_names:
        errors.append(f"workspace package name collisions: {', '.join(repeated_names)}")

    manifest_paths = [package.get("manifest_path", "") for package in packages]
    repeated_manifests = duplicates(manifest_paths)
    if repeated_manifests:
        errors.append("duplicate package manifest paths: " + ", ".join(repeated_manifests))

    package_ids = {package.get("id") for package in packages}
    unresolved = sorted(set(cargo_metadata.get("workspace_members", [])) - package_ids)
    if unresolved:
        errors.append("workspace member IDs missing package metadata: " + ", ".join(unresolved))

    if errors:
        for error in errors:
            print(f"workspace hygiene error: {error}", file=sys.stderr)
        return 1

    print(
        "workspace hygiene ok: "
        f"{len(members)} declared members, {len(defaults)} default members, "
        f"{len(packages)} unique packages"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

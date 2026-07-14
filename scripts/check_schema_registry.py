#!/usr/bin/env python3
"""Validate the authoritative schema registry and its generated mirror views."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
VERSIONED_NAME = re.compile(r"^(?P<id>.+)-v(?P<version>[0-9]+)\.schema\.json$")


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read JSON {path}: {error}") from error


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def wave_schema_names(document: dict[str, Any]) -> list[str]:
    names = document.get("schema_files", document.get("schemas", []))
    return names if isinstance(names, list) else []


def check(root: Path) -> list[str]:
    errors: list[str] = []
    schemas = root / "schemas"
    contracts = root / "contracts" / "schemas"
    registry_path = schemas / "schema_manifest.json"
    mirror_path = contracts / "schema_manifest.json"

    try:
        registry = load_json(registry_path)
        mirror = load_json(mirror_path)
    except ValueError as error:
        return [str(error)]

    if registry_path.read_bytes() != mirror_path.read_bytes():
        errors.append("generated mirror manifest has byte drift from schemas/schema_manifest.json")
    if registry.get("authority") != "schemas/":
        errors.append("registry authority must be 'schemas/'")
    if registry.get("mirror") != "contracts/schemas/":
        errors.append("registry mirror must be 'contracts/schemas/'")

    entries = registry.get("schemas")
    if not isinstance(entries, list):
        return errors + ["registry field 'schemas' must be a list"]

    keys: list[tuple[str, int]] = []
    files: list[str] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            errors.append(f"registry entry {index} must be an object")
            continue
        required = {"schema_id", "version", "owner", "compatibility", "file", "sha256"}
        missing = sorted(required - entry.keys())
        if missing:
            errors.append(f"registry entry {index} missing fields {missing}")
            continue
        schema_id = entry["schema_id"]
        version = entry["version"]
        filename = entry["file"]
        if not isinstance(schema_id, str) or not isinstance(version, int) or version < 1:
            errors.append(f"registry entry {index} has invalid schema_id/version")
            continue
        if not isinstance(filename, str) or Path(filename).name != filename:
            errors.append(f"registry entry {index} has unsafe file path {filename!r}")
            continue
        keys.append((schema_id, version))
        files.append(filename)
        path = schemas / filename
        if not path.is_file():
            errors.append(f"orphan registry entry: {filename} does not exist")
            continue
        if digest(path) != entry["sha256"]:
            errors.append(f"same-version byte drift: {schema_id} v{version} ({filename})")
        match = VERSIONED_NAME.match(filename)
        if match and (match.group("id") != schema_id or int(match.group("version")) != version):
            errors.append(f"filename identity drift for {filename}")

    duplicate_keys = sorted(key for key, count in Counter(keys).items() if count > 1)
    duplicate_files = sorted(name for name, count in Counter(files).items() if count > 1)
    if duplicate_keys:
        errors.append(f"duplicate schema IDs at the same version: {duplicate_keys}")
    if duplicate_files:
        errors.append(f"duplicate schema files in registry: {duplicate_files}")

    authoritative_files = {path.name for path in schemas.glob("*.schema.json")}
    registered_files = set(files)
    entries_by_file = {
        entry["file"]: entry
        for entry in entries
        if isinstance(entry, dict) and isinstance(entry.get("file"), str)
    }
    for filename in sorted(authoritative_files - registered_files):
        errors.append(f"orphan authoritative schema: {filename}")
    for filename in sorted(registered_files - authoritative_files):
        errors.append(f"orphan registry file reference: {filename}")

    mirror_schema_files = {path.name for path in contracts.rglob("*.schema.json")}
    for filename in sorted(mirror_schema_files - authoritative_files):
        errors.append(f"orphan mirror schema: {filename}")
    for filename in sorted(mirror_schema_files & authoritative_files):
        mirror_file = next(contracts.rglob(filename))
        if mirror_file.read_bytes() != (schemas / filename).read_bytes():
            errors.append(f"same-version byte drift in mirror schema: {filename}")

    legacy_owners: dict[str, tuple[str, Path]] = {}
    for manifest_path in sorted(contracts.glob("*/manifest.json")):
        try:
            document = load_json(manifest_path)
        except ValueError as error:
            errors.append(str(error))
            continue
        names = wave_schema_names(document)
        owner = document.get("owner_crate", document.get("primary_owner"))
        if not isinstance(owner, str) or not owner:
            errors.append(f"legacy view has no owner: {manifest_path.relative_to(root)}")
        if not names:
            errors.append(f"legacy view has no schemas: {manifest_path.relative_to(root)}")
        for filename in names:
            if filename not in registered_files:
                errors.append(
                    f"legacy view references an orphan schema: "
                    f"{manifest_path.relative_to(root)} -> {filename}"
                )
                continue
            if filename in legacy_owners:
                prior_owner, prior_path = legacy_owners[filename]
                errors.append(
                    f"duplicate legacy schema ownership: {filename} appears in "
                    f"{prior_path.relative_to(root)} ({prior_owner}) and "
                    f"{manifest_path.relative_to(root)} ({owner})"
                )
            else:
                legacy_owners[filename] = (owner, manifest_path)
            registry_owner = entries_by_file[filename].get("owner")
            if owner and registry_owner != owner:
                errors.append(
                    f"schema owner drift for {filename}: registry={registry_owner!r}, "
                    f"legacy view={owner!r}"
                )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", type=Path, default=ROOT)
    args = parser.parse_args()
    errors = check(args.root.resolve())
    if errors:
        for error in errors:
            print(f"schema registry check failed: {error}", file=sys.stderr)
        return 1
    count = len(list((args.root / "schemas").glob("*.schema.json")))
    print(f"schema registry check passed: {count} authoritative schemas")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Report Cargo path dependencies with incompatible or missing version policy."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import tomllib
from typing import NamedTuple


class Mismatch(NamedTuple):
    manifest: Path
    dependency: str
    dependency_manifest: Path
    declared_version: str
    actual_version: str


def _dependency_tables(document: dict):
    for name in ("dependencies", "dev-dependencies", "build-dependencies"):
        yield document.get(name, {})
    for target in document.get("target", {}).values():
        for name in ("dependencies", "dev-dependencies", "build-dependencies"):
            yield target.get(name, {})


def _version_tuple(version: str) -> tuple[int, int, int]:
    match = re.fullmatch(r"\s*(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:[-+].*)?\s*", version)
    if not match:
        raise ValueError(f"unsupported semantic version: {version}")
    return tuple(int(part or 0) for part in match.groups())  # type: ignore[return-value]


def _compatible_upper(parts: list[int]) -> tuple[int, int, int]:
    padded = (parts + [0, 0, 0])[:3]
    if padded[0] > 0:
        return padded[0] + 1, 0, 0
    if len(parts) > 1 and padded[1] > 0:
        return 0, padded[1] + 1, 0
    return 0, 0, padded[2] + 1


def _single_requirement_matches(requirement: str, actual: tuple[int, int, int]) -> bool:
    requirement = requirement.strip()
    operator = ""
    for candidate in (">=", "<=", ">", "<", "=", "^", "~"):
        if requirement.startswith(candidate):
            operator = candidate
            requirement = requirement[len(candidate):].strip()
            break

    if "*" in requirement or "x" in requirement.lower():
        parts = requirement.replace("X", "*").replace("x", "*").split(".")
        actual_parts = list(actual)
        return all(part == "*" or int(part) == actual_parts[index] for index, part in enumerate(parts))

    raw_parts = requirement.split(".")
    expected = _version_tuple(requirement)
    if operator == ">=":
        return actual >= expected
    if operator == "<=":
        return actual <= expected
    if operator == ">":
        return actual > expected
    if operator == "<":
        return actual < expected
    if operator == "=":
        return actual == expected
    if operator == "~":
        upper = (expected[0] + 1, 0, 0) if len(raw_parts) == 1 else (expected[0], expected[1] + 1, 0)
        return expected <= actual < upper
    upper = _compatible_upper([int(part) for part in raw_parts])
    return expected <= actual < upper


def version_requirement_matches(requirement: str, actual_version: str) -> bool:
    if requirement.strip().lstrip("= ") == actual_version:
        return True
    try:
        actual = _version_tuple(actual_version)
        return all(
            _single_requirement_matches(clause, actual)
            for clause in requirement.split(",")
            if clause.strip()
        )
    except (ValueError, IndexError):
        return False


def _package_version(manifest: Path, document: dict) -> str | None:
    version = document.get("package", {}).get("version")
    if isinstance(version, str):
        return version
    if isinstance(version, dict) and version.get("workspace") is True:
        for parent in manifest.parents:
            workspace_manifest = parent / "Cargo.toml"
            if workspace_manifest == manifest or not workspace_manifest.is_file():
                continue
            workspace = tomllib.loads(workspace_manifest.read_text(encoding="utf-8"))
            inherited = workspace.get("workspace", {}).get("package", {}).get("version")
            if isinstance(inherited, str):
                return inherited
    return None


def _check_table(
    root: Path,
    manifest: Path,
    table: dict,
    *,
    explicitly_unpublished: bool,
) -> list[Mismatch]:
    mismatches = []
    for name, declaration in table.items():
        if not isinstance(declaration, dict) or "path" not in declaration:
            continue
        dependency_manifest = (manifest.parent / declaration["path"] / "Cargo.toml").resolve()
        if not dependency_manifest.is_file():
            continue
        target = tomllib.loads(dependency_manifest.read_text(encoding="utf-8"))
        actual = _package_version(dependency_manifest, target)
        declared = declaration.get("version")
        if actual is None:
            continue
        if declared is None:
            if not explicitly_unpublished:
                mismatches.append(
                    Mismatch(manifest.relative_to(root), name, dependency_manifest, "<missing>", actual)
                )
        elif not isinstance(declared, str) or not version_requirement_matches(declared, actual):
            mismatches.append(
                Mismatch(manifest.relative_to(root), name, dependency_manifest, str(declared), actual)
            )
    return mismatches


def find_mismatches(root: Path) -> list[Mismatch]:
    root = root.resolve()
    mismatches = []
    for manifest in sorted(root.rglob("Cargo.toml")):
        relative_parts = manifest.relative_to(root).parts
        if any(
            part in {"target", ".git", "_salvage_from_libraries2"}
            for part in relative_parts
        ):
            continue
        document = tomllib.loads(manifest.read_text(encoding="utf-8"))
        explicitly_unpublished = document.get("package", {}).get("publish") is False
        for table in _dependency_tables(document):
            mismatches.extend(
                _check_table(
                    root,
                    manifest,
                    table,
                    explicitly_unpublished=explicitly_unpublished,
                )
            )
        workspace_dependencies = document.get("workspace", {}).get("dependencies", {})
        mismatches.extend(
            _check_table(
                root,
                manifest,
                workspace_dependencies,
                explicitly_unpublished=False,
            )
        )
    return mismatches


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", type=Path, default=Path.cwd())
    args = parser.parse_args()
    mismatches = find_mismatches(args.root)
    for item in mismatches:
        print(
            f"{item.manifest}: {item.dependency} declares {item.declared_version}, "
            f"path package is {item.actual_version} ({item.dependency_manifest})"
        )
    return 1 if mismatches else 0


if __name__ == "__main__":
    raise SystemExit(main())

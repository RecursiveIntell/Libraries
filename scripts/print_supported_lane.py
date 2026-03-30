#!/usr/bin/env python3
"""GOV-001: Read supported lane from machine-readable support_lane.toml."""
from __future__ import annotations

import argparse
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LANE_MANIFEST_JSON = ROOT / "scripts" / "lane_manifest.json"
SUPPORT_LANE_TOML = ROOT / "support_lane.toml"
# Fallback: parse SUPPORT_PROFILE.md if TOML doesn't exist
SUPPORT_PROFILE = ROOT / "SUPPORT_PROFILE.md"


def parse_supported_lane() -> list[str]:
    """Read supported lane from lane_manifest.json (preferred), support_lane.toml, or SUPPORT_PROFILE.md (fallback)."""
    if LANE_MANIFEST_JSON.is_file():
        import json
        manifest = json.loads(LANE_MANIFEST_JSON.read_text(encoding="utf-8"))
        return manifest.get("supported_lane", [])

    if SUPPORT_LANE_TOML.is_file():
        manifest = tomllib.loads(SUPPORT_LANE_TOML.read_text(encoding="utf-8"))
        return manifest.get("supported_lane", {}).get("crates", [])

    # Fallback: parse markdown (legacy)
    lane: list[str] = []
    in_section = False
    for raw in SUPPORT_PROFILE.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line == "Supported closeout lane:":
            in_section = True
            continue
        if not in_section:
            continue
        if not line:
            if lane:
                break
            continue
        if not line.startswith("- "):
            if lane:
                break
            continue
        lane.append(line[2:].strip().strip("`"))
    return lane


def selector_for(entry: str) -> str:
    manifest_path = ROOT / entry / "Cargo.toml"
    if not manifest_path.is_file():
        raise SystemExit(f"missing Cargo.toml for supported lane entry: {entry}")
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    package_name = manifest.get("package", {}).get("name")
    if not isinstance(package_name, str) or not package_name:
        raise SystemExit(f"missing package.name in {manifest_path.relative_to(ROOT)}")
    return package_name


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--cargo-package-flags",
        action="store_true",
        help="print repeated -p <package> selectors for cargo commands",
    )
    args = parser.parse_args()

    packages = [selector_for(entry) for entry in parse_supported_lane()]
    if args.cargo_package_flags:
        print(" ".join(f"-p {package}" for package in packages))
        return

    print("\n".join(packages))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Generate support views and reject drift from repo_contract.toml."""

from __future__ import annotations

import argparse
import difflib
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

import discover_workspaces


ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / "repo_contract.toml"
SUPPORT_PROFILE_PATH = ROOT / "SUPPORT_PROFILE.md"
LANE_MANIFEST_PATH = ROOT / "scripts" / "lane_manifest.json"
REQUIRED_PACKAGE_FIELDS = {
    "name",
    "manifest",
    "version",
    "workspaces",
    "maturity",
    "support_tier",
    "owners",
    "features",
    "required_gates",
}
REQUIRED_WORKSPACE_FIELDS = {
    "path",
    "manifest",
    "maturity",
    "support_tier",
    "owners",
    "features",
    "required_gates",
    "package_count",
}


def load_contract() -> dict[str, Any]:
    try:
        return tomllib.loads(CONTRACT_PATH.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"cannot parse {CONTRACT_PATH.name}: {error}") from error


def cargo_workspace_members(manifest: Path) -> set[str]:
    process = subprocess.run(
        [
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if process.returncode:
        raise ValueError(f"cargo metadata failed for {manifest}: {process.stderr.strip()}")
    document = json.loads(process.stdout)
    member_ids = set(document["workspace_members"])
    return {
        Path(package["manifest_path"]).resolve().relative_to(ROOT).as_posix()
        for package in document["packages"]
        if package["id"] in member_ids
    }


def validate_contract(contract: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    workspaces = contract.get("workspaces", [])
    packages = contract.get("packages", [])
    workspace_paths = [item.get("path") for item in workspaces if isinstance(item, dict)]
    discovered = discover_workspaces.discover()
    if sorted(workspace_paths) != sorted(discovered):
        errors.append(
            f"workspace inventory drift: contract={sorted(workspace_paths)} discovered={sorted(discovered)}"
        )

    contract_manifests = [item.get("manifest") for item in packages if isinstance(item, dict)]
    duplicates = sorted({item for item in contract_manifests if contract_manifests.count(item) > 1})
    if duplicates:
        errors.append(f"duplicate package manifests in contract: {duplicates}")

    for index, workspace in enumerate(workspaces):
        missing = REQUIRED_WORKSPACE_FIELDS - workspace.keys()
        if missing:
            errors.append(f"workspace {index} missing fields {sorted(missing)}")
    for index, package in enumerate(packages):
        missing = REQUIRED_PACKAGE_FIELDS - package.keys()
        if missing:
            errors.append(f"package {index} missing fields {sorted(missing)}")

    live_manifests: set[str] = set()
    live_by_workspace: dict[str, set[str]] = {}
    for workspace in workspaces:
        path = workspace.get("path")
        manifest_name = workspace.get("manifest")
        if not isinstance(path, str) or not isinstance(manifest_name, str):
            continue
        try:
            members = cargo_workspace_members(ROOT / manifest_name)
        except (ValueError, json.JSONDecodeError) as error:
            errors.append(str(error))
            continue
        live_by_workspace[path] = members
        live_manifests.update(members)
        if workspace.get("package_count") != len(members):
            errors.append(
                f"workspace package_count drift for {path}: "
                f"contract={workspace.get('package_count')} live={len(members)}"
            )

    if set(contract_manifests) != live_manifests:
        errors.append(
            "package inventory drift: "
            f"missing={sorted(live_manifests - set(contract_manifests))}, "
            f"orphaned={sorted(set(contract_manifests) - live_manifests)}"
        )
    for package in packages:
        manifest = package.get("manifest")
        declared = set(package.get("workspaces", []))
        actual = {path for path, members in live_by_workspace.items() if manifest in members}
        if declared != actual:
            errors.append(
                f"package workspace drift for {manifest}: declared={sorted(declared)} actual={sorted(actual)}"
            )

    lanes = contract.get("lanes", {})
    names = {package.get("name") for package in packages}
    names.update(
        str(Path(package["manifest"]).parent).replace("\\", "/")
        for package in packages
        if package.get("manifest") and package.get("workspaces") == ["."]
    )
    for lane_name, members in lanes.items():
        unknown = sorted(set(members) - names)
        if unknown:
            errors.append(f"lane {lane_name} references unknown packages: {unknown}")
    return errors


def support_profile(contract: dict[str, Any]) -> str:
    lanes = contract["lanes"]
    supported = "\n".join(f"- `{name}`" for name in lanes["supported"])
    governance = "\n".join(f"- `{name}`" for name in lanes["governance"])
    return f"""# Support profile

Supported closeout lane:
{supported}

This is the narrow release-facing support claim used by `release/closeout_receipt_v1.json`.
It is also the public-doc-certified core checked by `python3 scripts/check_public_api_docs.py`.

Adjacent artifact-owner crates for the demo/benchmark substrate are documented in `SCOPE_NOTES.md`.
They are not part of the narrow build-certified or public-doc-certified claim of the {contract['support_receipt_date']} hardening receipt.

## Governance crates (V28, build-checked, default-enabled)

The following governance crates now have typed error enums, integration documentation, and are build-checked by `cargo check --workspace`. As of V28, they compile by default (`default = ["governance"]` in forge-pilot) and the governance observation pipeline is live:

{governance}
"""


def lane_manifest(contract: dict[str, Any]) -> str:
    lanes = contract["lanes"]
    document = {
        "schema_version": "lane_manifest_v1",
        "generated_by": contract["lane_manifest_generated_by"],
        "supported_lane": lanes["supported"],
        "governance_lane": lanes["governance"],
        "doc_certified_lane": lanes["doc_certified"],
        "panic_audit_lane": lanes["panic_audit"],
    }
    return json.dumps(document, indent=2) + "\n"


def check_or_write(path: Path, expected: str, check: bool) -> bool:
    actual = path.read_text(encoding="utf-8") if path.exists() else ""
    if actual == expected:
        return True
    if check:
        diff = difflib.unified_diff(
            actual.splitlines(),
            expected.splitlines(),
            fromfile=str(path.relative_to(ROOT)),
            tofile=f"generated:{path.relative_to(ROOT)}",
            lineterm="",
        )
        print("\n".join(diff), file=sys.stderr)
        return False
    path.write_text(expected, encoding="utf-8")
    print(f"generated {path.relative_to(ROOT)}")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail if generated files drift")
    args = parser.parse_args()
    try:
        contract = load_contract()
        errors = validate_contract(contract)
    except ValueError as error:
        print(f"repo contract generation failed: {error}", file=sys.stderr)
        return 1
    if errors:
        for error in errors:
            print(f"repo contract check failed: {error}", file=sys.stderr)
        return 1

    results = [
        check_or_write(SUPPORT_PROFILE_PATH, support_profile(contract), args.check),
        check_or_write(LANE_MANIFEST_PATH, lane_manifest(contract), args.check),
    ]
    if not all(results):
        print("repo contract check failed: generated file drift", file=sys.stderr)
        return 1
    print(f"repo contract {'check passed' if args.check else 'generation complete'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

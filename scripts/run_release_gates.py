#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from datetime import datetime
from pathlib import Path

from release_gate_set import RELEASE_GATE_COMMANDS, gate_sha256


ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_MANIFEST = ROOT / "STATUS_EVIDENCE_MANIFEST.json"
RECEIPT_GENERATOR = "python3 scripts/generate_closeout_receipt.py"
RECEIPT_CHECK = "python3 scripts/check_closeout_receipt.py"
ARTIFACT_FILES = [
    "STATUS_DASHBOARD.md",
    "SUPPORT_PROFILE.md",
    "scripts/release_gate_set.py",
    "scripts/check_no_prod_panics.sh",
    "scripts/public_type_drift_allowlist.json",
    "docs/archive/root_closeout_history/manifest.json",
    "release/closeout_receipt_v1.json",
]


def canonical_sha256(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def load_cargo_metadata() -> dict[str, object]:
    completed = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"cargo metadata failed: {completed.stderr.strip()}")
    return json.loads(completed.stdout)


def markdown_code_bullets(path: Path, start_heading: str, end_heading: str) -> list[str]:
    text = path.read_text(encoding="utf-8")
    try:
        section = text.split(start_heading, 1)[1].split(end_heading, 1)[0]
    except IndexError as error:
        raise RuntimeError(
            f"cannot parse {path.relative_to(ROOT)} between {start_heading!r} and {end_heading!r}"
        ) from error
    return re.findall(r"^- `([^`]+)`\s*$", section, flags=re.MULTILINE)


def workflow_push_branches(path: Path) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    branches: list[str] = []
    in_push = False
    in_branches = False
    for line in lines:
        stripped = line.strip()
        indent = len(line) - len(line.lstrip())
        if indent == 2 and stripped == "push:":
            in_push = True
            in_branches = False
            continue
        if in_push and indent <= 2 and stripped:
            break
        if in_push and indent == 4 and stripped == "branches:":
            in_branches = True
            continue
        if in_branches and indent == 6 and stripped.startswith("- "):
            branches.append(stripped[2:].strip("'\""))
        elif in_branches and stripped and indent <= 4:
            in_branches = False
    return branches


def git_ref_exists(branch: str) -> bool:
    for reference in (f"refs/heads/{branch}", f"refs/remotes/origin/{branch}"):
        completed = subprocess.run(
            ["git", "show-ref", "--verify", "--quiet", reference], cwd=ROOT, check=False
        )
        if completed.returncode == 0:
            return True
    return False


def check_release_truth_drift() -> list[str]:
    errors: list[str] = []
    with (ROOT / "Cargo.toml").open("rb") as handle:
        root_manifest = tomllib.load(handle)
    try:
        contract = root_manifest["workspace"]["metadata"]["release-truth"]
    except KeyError as error:
        return [f"Cargo.toml is missing workspace.metadata.release-truth.{error.args[0]}"]

    try:
        metadata = load_cargo_metadata()
    except (RuntimeError, json.JSONDecodeError) as error:
        return [str(error)]

    packages = metadata.get("packages", [])
    workspace_members = metadata.get("workspace_members", [])
    default_members = metadata.get("workspace_default_members", [])
    expected_package_count = contract.get("workspace-package-count")
    expected_default_count = contract.get("default-member-count")
    if len(workspace_members) != expected_package_count:
        errors.append(
            "workspace-package-count drift: "
            f"Cargo.toml contract says {expected_package_count}, cargo metadata reports "
            f"{len(workspace_members)}"
        )
    if len(packages) != len(workspace_members):
        errors.append(
            "workspace inventory drift: cargo metadata reports "
            f"{len(packages)} packages for {len(workspace_members)} member IDs"
        )
    if len(default_members) != expected_default_count:
        errors.append(
            "default-member-count drift: "
            f"Cargo.toml contract says {expected_default_count}, cargo metadata reports "
            f"{len(default_members)}"
        )

    feature_graph = {
        package["name"]: {
            name: sorted(edges) for name, edges in sorted(package.get("features", {}).items())
        }
        for package in sorted(packages, key=lambda item: item["name"])
    }
    actual_feature_digest = canonical_sha256(feature_graph)
    expected_feature_digest = contract.get("feature-graph-sha256")
    if actual_feature_digest != expected_feature_digest:
        errors.append(
            "feature graph drift: Cargo.toml contract has "
            f"{expected_feature_digest}, cargo metadata generates {actual_feature_digest}"
        )

    member_paths = {
        str(Path(package["manifest_path"]).resolve().parent.relative_to(ROOT))
        for package in packages
    }
    supported = contract.get("supported-closeout-members", [])
    adjacent = contract.get("adjacent-members", [])
    for tier, entries in (("supported", supported), ("adjacent", adjacent)):
        absent = sorted(set(entries) - member_paths)
        if absent:
            errors.append(f"{tier} support tier contains non-members: {', '.join(absent)}")
    overlap = sorted(set(supported) & set(adjacent))
    if overlap:
        errors.append("support tier overlap: " + ", ".join(overlap))

    try:
        support_document = markdown_code_bullets(
            ROOT / "SUPPORT_PROFILE.md", "Supported closeout lane:", "This is the narrow"
        )
        if support_document != supported:
            errors.append(
                "supported tier drift: SUPPORT_PROFILE.md list differs from Cargo.toml contract"
            )
    except RuntimeError as error:
        errors.append(str(error))

    try:
        scope_adjacent = markdown_code_bullets(
            ROOT / "SCOPE_NOTES.md",
            "## Adjacent crates outside the build-certified lane",
            "## Scope rule",
        )
        if scope_adjacent != adjacent:
            errors.append("adjacent tier drift: SCOPE_NOTES.md list differs from Cargo.toml contract")
    except RuntimeError as error:
        errors.append(str(error))

    try:
        receipt = json.loads((ROOT / "release/closeout_receipt_v1.json").read_text(encoding="utf-8"))
        receipt_lane = receipt["supported_closeout_lane"]
        if receipt_lane.get("crates") != supported:
            errors.append(
                "supported tier drift: release/closeout_receipt_v1.json differs from Cargo.toml contract"
            )
        if receipt_lane.get("crate_count") != len(supported):
            errors.append(
                "supported crate-count drift: closeout receipt reports "
                f"{receipt_lane.get('crate_count')}, generated tier contains {len(supported)}"
            )
    except (KeyError, json.JSONDecodeError, OSError) as error:
        errors.append(f"cannot compare closeout receipt: {error}")

    release_branch = contract.get("release-branch")
    push_branches = workflow_push_branches(ROOT / ".github" / "workflows" / "ci.yml")
    if push_branches != [release_branch]:
        errors.append(
            "release branch drift: Cargo.toml contract says "
            f"{release_branch!r}, ci.yml push branches are {push_branches!r}"
        )
    if not isinstance(release_branch, str) or not git_ref_exists(release_branch):
        errors.append(f"release branch drift: git has no local or origin ref for {release_branch!r}")

    package_names = {package["name"] for package in packages}
    for limitation in contract.get("known-limitations", []):
        limitation_id = limitation.get("id", "<missing-id>")
        package = limitation.get("package")
        if package not in package_names:
            errors.append(f"known limitation {limitation_id}: package {package!r} is not in metadata")
            continue
        source = ROOT / limitation.get("source", "")
        try:
            source_text = source.read_text(encoding="utf-8").lower()
        except OSError as error:
            errors.append(f"known limitation {limitation_id}: cannot read source: {error}")
            continue
        if limitation.get("status") == "not-observed":
            semantic_markers = (
                package.lower(),
                "does not yet read attestation exchange state",
                "not yet observed",
            )
            missing = [marker for marker in semantic_markers if marker not in source_text]
            if missing:
                errors.append(
                    f"known limitation {limitation_id} drift: {source.relative_to(ROOT)} no longer "
                    f"states the not-observed condition ({', '.join(missing)})"
                )
        else:
            errors.append(
                f"known limitation {limitation_id}: unsupported status {limitation.get('status')!r}"
            )

    return errors


def run_drift_check() -> int:
    errors = check_release_truth_drift()
    if errors:
        for error in errors:
            print(f"release truth drift: {error}", file=sys.stderr)
        return 1
    print("release truth drift check ok")
    return 0


def build_manifest(results: list[dict[str, str]], captured_at: datetime) -> dict[str, object]:
    return {
        "snapshot": f"{captured_at.date().isoformat()}-hardening-closeout",
        "captured_at": captured_at.date().isoformat(),
        "captured_at_local": captured_at.isoformat(timespec="seconds"),
        "gate_definition": {
            "path": "scripts/release_gate_set.py",
            "sha256": gate_sha256(),
            "command_count": len(RELEASE_GATE_COMMANDS),
        },
        "proof_commands": RELEASE_GATE_COMMANDS,
        "proof_results": results,
        "artifact_files": ARTIFACT_FILES,
        "notes": [
            "This ledger is generated by scripts/run_release_gates.py from the canonical gate list in scripts/release_gate_set.py.",
            "make gate reruns the same commands in the same order and rewrites this ledger before regenerating the closeout receipt.",
            "The receipt is a derivative artifact. It must not be treated as independent proof of gate success.",
        ],
    }


def run_command(command: str) -> int:
    print(f"+ {command}", flush=True)
    completed = subprocess.run(command, shell=True, cwd=ROOT)
    return completed.returncode


def write_manifest(results: list[dict[str, str]], captured_at: datetime) -> None:
    manifest = build_manifest(results, captured_at)
    EVIDENCE_MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check-drift",
        action="store_true",
        help="compare generated Cargo/Git/feature/support truth without running release commands",
    )
    args = parser.parse_args()
    if args.check_drift:
        return run_drift_check()

    captured_at = datetime.now().astimezone()
    results: list[dict[str, str]] = []

    for command in RELEASE_GATE_COMMANDS:
        exit_code = run_command(command)
        result = "pass" if exit_code == 0 else "fail"
        results.append({"command": command, "result": result})
        if exit_code != 0:
            write_manifest(results, captured_at)
            print(f"release gate failed at: {command}", file=sys.stderr)
            return exit_code

    write_manifest(results, captured_at)

    for command in (RECEIPT_GENERATOR, RECEIPT_CHECK):
        exit_code = run_command(command)
        if exit_code != 0:
            print(f"release publication step failed at: {command}", file=sys.stderr)
            return exit_code

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

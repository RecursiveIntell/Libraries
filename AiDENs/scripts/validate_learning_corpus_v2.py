#!/usr/bin/env python3
"""Validate executable learning corpus v2 and optionally run sealed baseline/treatment pairs."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from pathlib import PurePosixPath
from typing import Any

EXPECTED_FAMILY_SPLITS = {
    "borrow-check": "development",
    "error-propagation": "development",
    "iterator-safety": "development",
    "module-hygiene": "calibration",
    "wire-schema": "holdout",
}


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def tree_digest(root: Path) -> str:
    material: list[dict[str, str]] = []
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root)
        if "target" in relative.parts:
            continue
        if path.is_symlink():
            raise ValueError(f"fixture symlink is forbidden: {relative.as_posix()}")
        if path.is_file():
            material.append(
                {
                    "path": relative.as_posix(),
                    "sha256": file_sha256(path),
                }
            )
    return sha256_bytes(canonical_bytes(material))


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def safe_relative_path(root: Path, value: object, label: str, role_root: str) -> Path:
    if not isinstance(value, str) or not value:
        raise ValueError(f"invalid {label}")
    relative = PurePosixPath(value)
    if relative.is_absolute() or ".." in relative.parts or "." in relative.parts:
        raise ValueError(f"{label} escapes corpus root")
    resolved_root = root.resolve()
    resolved = (resolved_root / Path(*relative.parts)).resolve()
    allowed = (resolved_root / role_root).resolve()
    if resolved != allowed and allowed not in resolved.parents:
        raise ValueError(f"{label} escapes {role_root} role root")
    return resolved


def apply_typed_patch(root: Path, patch: dict[str, Any]) -> None:
    for edit in patch["edits"]:
        path = (root / edit["path"]).resolve()
        if root.resolve() not in path.parents:
            raise ValueError("patch escaped fixture root")
        lines = path.read_text().splitlines()
        for op in edit["ops"]:
            replacement = op.get("Replace")
            if replacement is None:
                raise ValueError("v2 evaluator only admits typed Replace operations")
            start = replacement["range"]["start"]
            end = replacement["range"]["end_exclusive"]
            if start < 1 or end < start or end - 1 > len(lines):
                raise ValueError("typed patch range is outside source material")
            lines[start - 1 : end - 1] = replacement["lines"]
        path.write_text("\n".join(lines) + "\n")


def sealed_command(runtime: str, image: str, workspace: Path, cidfile: Path) -> list[str]:
    return [
        runtime,
        "run",
        "--rm",
        "--pull=never",
        f"--cidfile={cidfile}",
        "--network=none",
        "--read-only",
        "--cap-drop=all",
        "--userns=keep-id",
        "--security-opt=no-new-privileges",
        "--memory=2g",
        "--cpus=2",
        "--pids-limit=256",
        "--tmpfs=/tmp:rw,noexec,nosuid,nodev,size=256m",
        "-e",
        "CARGO_HOME=/tmp/cargo-home",
        "-v",
        f"{workspace.resolve()}:/workspace:Z,rw",
        "-w",
        "/workspace",
        image,
        "cargo",
        "test",
        "--offline",
        "--quiet",
    ]


def execute_side(runtime: str, image: str, workspace: Path, side: str) -> dict[str, Any]:
    started = time.monotonic()
    cidfile = workspace.parent / f"{workspace.name}-{side}.cid"
    try:
        completed = subprocess.run(
            sealed_command(runtime, image, workspace, cidfile),
            text=True,
            capture_output=True,
            timeout=120,
            check=False,
        )
    except subprocess.TimeoutExpired:
        if cidfile.is_file():
            container_id = cidfile.read_text().strip()
            if container_id:
                subprocess.run(
                    [runtime, "rm", "--force", container_id],
                    text=True,
                    capture_output=True,
                    timeout=30,
                    check=False,
                )
        raise ValueError(f"sealed {side} execution timed out")
    duration_ms = round((time.monotonic() - started) * 1000, 3)
    output = (completed.stdout + "\n" + completed.stderr).encode()
    return {
        "side": side,
        "exit_code": completed.returncode,
        "output_sha256": sha256_bytes(output),
        "duration_ms_diagnostic_only": duration_ms,
        "timing_admissible": False,
        "passed": completed.returncode == 0,
    }


def validate(
    root: Path,
    execute: bool,
    image: str | None,
    runtime_path: Path | None = None,
    expected_runtime_sha256: str | None = None,
) -> dict[str, Any]:
    manifest_path = root / "manifest.json"
    manifest = load_json(manifest_path)
    if manifest.get("schema") != "AiDENsExecutableLearningCorpusV2" or manifest.get(
        "version"
    ) != 2:
        raise ValueError("unsupported corpus schema or version")
    if manifest.get("claim_scope") != "local-executable-corpus-only" or manifest.get(
        "paired_evaluation_status"
    ) != "not-run-owner-evidence-required":
        raise ValueError("invalid corpus claim boundary")
    if manifest.get("partition_policy") != {
        "family_clustered": True,
        "counts": {"development": 9, "calibration": 3, "holdout": 3},
        "oracle_excluded_from_public_task": True,
    }:
        raise ValueError("invalid partition policy")
    claimed_digest = manifest.pop("corpus_digest")
    actual_digest = sha256_bytes(canonical_bytes(manifest))
    if claimed_digest != actual_digest:
        raise ValueError("corpus digest mismatch")
    manifest["corpus_digest"] = claimed_digest

    tasks = manifest["tasks"]
    if len(tasks) != 15:
        raise ValueError("v2 requires exactly 15 tasks")
    counts = Counter(task["split"] for task in tasks)
    if counts != Counter({"development": 9, "calibration": 3, "holdout": 3}):
        raise ValueError(f"invalid split counts: {dict(counts)}")
    family_splits: dict[str, set[str]] = defaultdict(set)
    task_ids: set[str] = set()
    task_paths: set[str] = set()
    fixture_paths: set[str] = set()
    task_digests: set[str] = set()
    fixture_digests: set[str] = set()
    oracle_digests: set[str] = set()
    for task in tasks:
        if task["task_id"] in task_ids:
            raise ValueError(f"duplicate task ID: {task['task_id']}")
        task_ids.add(task["task_id"])
        if task["task_path"] in task_paths or task["fixture_path"] in fixture_paths:
            raise ValueError("duplicate task or fixture path")
        task_paths.add(task["task_path"])
        fixture_paths.add(task["fixture_path"])
        if (
            task["task_digest"] in task_digests
            or task["fixture_tree_digest"] in fixture_digests
            or task["oracle_digest"] in oracle_digests
        ):
            raise ValueError("duplicate task, fixture, or oracle material")
        task_digests.add(task["task_digest"])
        fixture_digests.add(task["fixture_tree_digest"])
        oracle_digests.add(task["oracle_digest"])
        family_splits[task["family"]].add(task["split"])
    observed_family_splits = {
        family: next(iter(splits))
        for family, splits in family_splits.items()
        if len(splits) == 1
    }
    if observed_family_splits != EXPECTED_FAMILY_SPLITS or any(
        len(splits) != 1 for splits in family_splits.values()
    ):
        raise ValueError("canonical family-to-split mapping mismatch")

    runtime = None
    runtime_sha256 = None
    rootless_observed = False
    resolved_image_digest = None
    python_runtime = str(Path(sys.executable).resolve())
    if execute:
        if not image or "@sha256:" not in image or ":latest" in image:
            raise ValueError("--execute requires a digest-pinned --image")
        if runtime_path is None or not runtime_path.is_absolute() or not runtime_path.is_file():
            raise ValueError("--execute requires an explicit absolute --runtime path")
        runtime = str(runtime_path.resolve())
        runtime_sha256 = file_sha256(Path(runtime))
        if not expected_runtime_sha256 or runtime_sha256 != expected_runtime_sha256:
            raise ValueError("runtime digest mismatch")
        info = subprocess.run(
            [runtime, "info", "--format", "{{.Host.Security.Rootless}}"],
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
        rootless_observed = info.returncode == 0 and info.stdout.strip().lower() == "true"
        if not rootless_observed:
            raise ValueError("sealed execution requires observed rootless Podman")
        exists = subprocess.run(
            [runtime, "image", "exists", image],
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
        if exists.returncode != 0:
            raise ValueError("digest-pinned image must already exist locally")
        inspected = subprocess.run(
            [runtime, "image", "inspect", image, "--format", "{{.Digest}}"],
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
        resolved_image_digest = inspected.stdout.strip()
        requested_digest = image.rsplit("@", 1)[1]
        if inspected.returncode != 0 or resolved_image_digest != requested_digest:
            raise ValueError("local image digest does not match requested image")

    rows: list[dict[str, Any]] = []
    patch_ids: set[str] = set()
    with tempfile.TemporaryDirectory(prefix="aidens-learning-corpus-v2-") as temp:
        temp_root = Path(temp)
        for pair_index, entry in enumerate(tasks):
            expected_task_path = f"tasks/{entry['family']}/{entry['task_id']}/task.json"
            expected_fixture_path = f"tasks/{entry['family']}/{entry['task_id']}/fixture"
            if entry["task_path"] != expected_task_path or entry["fixture_path"] != expected_fixture_path:
                raise ValueError(f"task topology mismatch: {entry['task_id']}")
            task_path = safe_relative_path(root, entry["task_path"], "task path", "tasks")
            fixture_path = safe_relative_path(root, entry["fixture_path"], "fixture path", "tasks")
            oracle_path = safe_relative_path(
                root, f"oracles/{entry['task_id']}.patch.json", "oracle path", "oracles"
            )
            public = load_json(task_path)
            oracle = load_json(oracle_path)
            patch_id = oracle.get("patch_id")
            if not isinstance(patch_id, str) or not patch_id or patch_id in patch_ids:
                raise ValueError(f"invalid or duplicate oracle patch ID: {entry['task_id']}")
            patch_ids.add(patch_id)
            if not (fixture_path / "Cargo.toml").is_file() or not (
                fixture_path / "src" / "lib.rs"
            ).is_file():
                raise ValueError(f"metadata-only or incomplete fixture: {entry['task_id']}")
            for field in ("task_id", "family", "split"):
                if public.get(field) != entry[field]:
                    raise ValueError(f"task/manifest {field} mismatch: {entry['task_id']}")
            if public.get("schema") != "AiDENsExecutableLearningTaskV2":
                raise ValueError(f"unsupported task schema: {entry['task_id']}")
            for field in ("fixture_tree_digest", "oracle_digest"):
                if public.get(field) != entry[field]:
                    raise ValueError(f"task/manifest {field} mismatch: {entry['task_id']}")
            if public.get("verifier") != ["cargo", "test", "--offline", "--quiet"]:
                raise ValueError(f"unsupported verifier contract: {entry['task_id']}")
            if public.get("baseline_expected") != "fail" or public.get(
                "treatment_expected"
            ) != "pass":
                raise ValueError(f"invalid outcome contract: {entry['task_id']}")
            forbidden = {"oracle", "oracle_path", "patch", "expected_source"}
            if forbidden.intersection(public):
                raise ValueError(f"public task exposes evaluator data: {entry['task_id']}")
            if sha256_bytes(canonical_bytes(public)) != entry["task_digest"]:
                raise ValueError(f"task digest mismatch: {entry['task_id']}")
            if tree_digest(fixture_path) != entry["fixture_tree_digest"]:
                raise ValueError(f"fixture tree digest mismatch: {entry['task_id']}")
            if sha256_bytes(canonical_bytes(oracle)) != entry["oracle_digest"]:
                raise ValueError(f"oracle digest mismatch: {entry['task_id']}")

            row: dict[str, Any] = {
                "pair_index": pair_index,
                "task_id": entry["task_id"],
                "family": entry["family"],
                "split": entry["split"],
                "fixture_tree_digest": entry["fixture_tree_digest"],
                "oracle_digest": entry["oracle_digest"],
                "baseline_expected": "fail",
                "treatment_expected": "pass",
                "executed": execute,
            }
            if execute:
                assert runtime is not None and image is not None
                baseline = temp_root / f"{pair_index:02d}-baseline"
                treatment = temp_root / f"{pair_index:02d}-treatment"
                shutil.copytree(fixture_path, baseline)
                shutil.copytree(fixture_path, treatment)
                apply_typed_patch(treatment, oracle)
                baseline_result = execute_side(runtime, image, baseline, "baseline")
                treatment_result = execute_side(runtime, image, treatment, "treatment")
                row["sides"] = [baseline_result, treatment_result]
                row["observed_contract"] = (
                    not baseline_result["passed"] and treatment_result["passed"]
                )
                if not row["observed_contract"]:
                    raise ValueError(f"executable outcome contract failed: {entry['task_id']}")
            rows.append(row)

    return {
        "schema": "AiDENsExecutableLearningCorpusValidationReceiptV2",
        "corpus_digest": claimed_digest,
        "claim_scope": "local-executable-corpus-validation-only",
        "execution_backend": "sealed-rootless-podman" if execute else "not-executed",
        "runtime_path": runtime,
        "runtime_sha256": runtime_sha256,
        "rootless_observed": rootless_observed,
        "python_runtime_path": python_runtime,
        "python_runtime_sha256": file_sha256(Path(python_runtime)),
        "validator_sha256": file_sha256(Path(__file__).resolve()),
        "verifier_argv": ["cargo", "test", "--offline", "--quiet"],
        "image": image,
        "resolved_image_digest": resolved_image_digest,
        "sealed_argv_template": (
            sealed_command(runtime, image, Path("/HOST_WORKSPACE"), Path("/HOST_CIDFILE"))
            if execute and runtime and image
            else None
        ),
        "pull_policy": "never" if execute else None,
        "user_namespace": "keep-id" if execute else None,
        "no_new_privileges": execute,
        "timeout_seconds": 120 if execute else None,
        "timeout_cleanup": "cidfile-podman-rm-force" if execute else None,
        "recorded_at_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "network_policy": "none" if execute else None,
        "read_only_rootfs": execute,
        "capabilities_dropped": "all" if execute else None,
        "task_count": len(tasks),
        "family_count": len(family_splits),
        "split_counts": dict(sorted(counts.items())),
        "pair_count": len(rows) if execute else 0,
        "side_execution_count": len(rows) * 2 if execute else 0,
        "all_executable_contracts_passed": execute and all(
            row.get("observed_contract", False) for row in rows
        ),
        "timing_claims_admissible": False,
        "paired_statistics_status": "unavailable-owner-runner-not-integrated",
        "promotion_evidence_status": "unavailable",
        "rows": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--image")
    parser.add_argument("--runtime", type=Path)
    parser.add_argument("--runtime-sha256")
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    receipt = validate(
        args.root.resolve(),
        args.execute,
        args.image,
        args.runtime,
        args.runtime_sha256,
    )
    encoded = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    if args.receipt:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(encoded)
    print(encoded, end="")


if __name__ == "__main__":
    main()

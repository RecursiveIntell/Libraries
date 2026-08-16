#!/usr/bin/env python3
"""Emit a content-addressed activation-candidate manifest without mutating state.

The manifest binds the audited source worktree (including dirty/untracked
content), selected binary, adapter tree, configuration bytes, lockfiles, and
capability response. It intentionally never builds, installs, activates, or
changes a receipt store.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str | None:
    return digest_bytes(path.read_bytes()) if path.is_file() else None


def command(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()


def worktree_digest(root: Path) -> str:
    """Digest tracked working bytes plus all non-ignored untracked bytes.

    `git write-tree` identifies the index, not a dirty candidate. This stream
    deliberately hashes the files that would actually be built or loaded,
    including an explicit missing-file marker for a tracked deletion.
    """
    tracked = command(root, "ls-files", "--cached").splitlines()
    untracked = command(root, "ls-files", "--others", "--exclude-standard").splitlines()
    payload = bytearray()
    for name in sorted(set(filter(None, tracked + untracked))):
        path = root / name
        payload.extend(name.encode("utf-8"))
        payload.extend(b"\0")
        if path.is_file():
            payload.extend(path.read_bytes())
        else:
            payload.extend(b"<missing-or-non-file>")
        payload.extend(b"\0")
    return digest_bytes(bytes(payload))


def source_tree_digest(root: Path) -> str:
    """Digest a source subtree without including interpreter build caches."""
    payload = bytearray()
    paths = [root] if root.is_file() else sorted(path for path in root.rglob("*") if path.is_file())
    for path in paths:
        if "__pycache__" in path.parts or path.suffix == ".pyc":
            continue
        relative = path.name if root.is_file() else path.relative_to(root).as_posix()
        payload.extend(relative.encode("utf-8"))
        payload.extend(b"\0")
        payload.extend(path.read_bytes())
        payload.extend(b"\0")
    return digest_bytes(bytes(payload))


def repo_identity(root: Path) -> dict[str, str]:
    return {
        "root": str(root.resolve()),
        "head": command(root, "rev-parse", "HEAD"),
        "tracked_tree": command(root, "write-tree"),
        "dirty_patch_sha256": digest_bytes(
            subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=root)
        ),
        "worktree_sha256": worktree_digest(root),
    }


def capabilities(binary: Path) -> dict[str, Any]:
    output = subprocess.check_output([str(binary), "capabilities"], text=True)
    value = json.loads(output)
    if not isinstance(value, dict):
        raise ValueError("capabilities output must be a JSON object")
    return value


def canonical_manifest_id(manifest: dict[str, Any]) -> str:
    unsigned = {key: value for key, value in manifest.items() if key != "candidate_id"}
    return digest_bytes(
        json.dumps(unsigned, sort_keys=True, separators=(",", ":")).encode("utf-8")
    )


def build_manifest(args: argparse.Namespace) -> dict[str, Any]:
    binary = args.binary.resolve()
    adapter = args.hermes_adapter.resolve()
    test_receipt = args.test_receipt.resolve()
    if not binary.is_file():
        raise ValueError(f"candidate binary does not exist: {binary}")
    if not adapter.exists():
        raise ValueError(f"Hermes adapter path does not exist: {adapter}")
    if not test_receipt.is_file():
        raise ValueError(f"test-result receipt does not exist: {test_receipt}")
    manifest = {
        "schema": "AresContextGovernorCandidateManifestV2",
        "governor": repo_identity(args.governor_root.resolve()),
        "hermes": repo_identity(args.hermes_root.resolve()),
        "binary": {
            "path": str(binary),
            "sha256": digest_file(binary),
        },
        "hermes_adapter": {
            "path": str(adapter),
            "source_tree_sha256": source_tree_digest(adapter),
        },
        "config": {
            "path": str(args.config.resolve()),
            "sha256": digest_file(args.config.resolve()),
        },
        "lockfiles": {
            "governor_cargo_lock_sha256": digest_file(args.governor_root / "Cargo.lock"),
            "hermes_pyproject_sha256": digest_file(args.hermes_root / "pyproject.toml"),
        },
        "test_result_receipt": {
            "path": str(test_receipt),
            "sha256": digest_file(test_receipt),
        },
        "capabilities": capabilities(binary),
        "receipt_schema": "ContextCompactionReceiptV2",
    }
    manifest["candidate_id"] = canonical_manifest_id(manifest)
    return manifest


def verify_manifest(path: Path) -> None:
    expected = json.loads(path.read_text(encoding="utf-8"))
    if expected.get("schema") != "AresContextGovernorCandidateManifestV2":
        raise ValueError("unsupported candidate manifest schema")
    if expected.get("candidate_id") != canonical_manifest_id(expected):
        raise ValueError("candidate manifest identity is invalid")
    binary = Path(expected["binary"]["path"])
    governor = repo_identity(Path(expected["governor"]["root"]))
    hermes = repo_identity(Path(expected["hermes"]["root"]))
    checks = {
        "binary": digest_file(binary),
        "config": digest_file(Path(expected["config"]["path"])),
        "adapter": source_tree_digest(Path(expected["hermes_adapter"]["path"])),
        "test_result_receipt": digest_file(Path(expected["test_result_receipt"]["path"])),
    }
    expected_checks = {
        "binary": expected["binary"]["sha256"],
        "config": expected["config"]["sha256"],
        "adapter": expected["hermes_adapter"]["source_tree_sha256"],
        "test_result_receipt": expected["test_result_receipt"]["sha256"],
    }
    if checks != expected_checks:
        raise ValueError(f"candidate byte identity mismatch: {checks!r} != {expected_checks!r}")
    if governor != expected["governor"] or hermes != expected["hermes"]:
        raise ValueError("candidate source worktree identity changed")
    if capabilities(binary) != expected["capabilities"]:
        raise ValueError("candidate capability response changed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--governor-root", type=Path)
    parser.add_argument("--hermes-root", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--hermes-adapter", type=Path)
    parser.add_argument("--test-receipt", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    if args.verify:
        verify_manifest(args.verify)
        print(json.dumps({"ok": True, "manifest": str(args.verify.resolve())}))
        return
    required = {
        "--governor-root": args.governor_root,
        "--hermes-root": args.hermes_root,
        "--binary": args.binary,
        "--config": args.config,
        "--hermes-adapter": args.hermes_adapter,
        "--test-receipt": args.test_receipt,
    }
    missing = [flag for flag, value in required.items() if value is None]
    if missing:
        parser.error(f"required unless --verify: {', '.join(missing)}")
    manifest = build_manifest(args)
    encoded = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")


if __name__ == "__main__":
    main()

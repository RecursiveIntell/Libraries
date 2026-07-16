#!/usr/bin/env python3
"""Shared, side-effect-free primitives for source-bound release evidence."""
from __future__ import annotations

import hashlib
import json
import platform as platform_module
import subprocess
from pathlib import Path
from typing import Any

EVIDENCE_PATHS = {
    "STATUS_EVIDENCE_MANIFEST.json",
    "release/closeout_receipt_v1.json",
}
REQUIRED_BINDING_FIELDS = {
    "commit_sha",
    "tree_sha",
    "cargo_lock_sha256",
    "toolchain",
    "platform",
    "workspace_inventory_sha256",
    "command_receipts",
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def run(repo: Path, argv: list[str]) -> str:
    completed = subprocess.run(argv, cwd=repo, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if completed.returncode:
        raise RuntimeError(f"{' '.join(argv)} failed: {completed.stderr.strip()}")
    return completed.stdout.strip()


def git_status_porcelain(repo: Path) -> str:
    return run(repo, ["git", "status", "--porcelain=v1", "--untracked-files=all"])


def workspace_inventory_sha256(repo: Path) -> str:
    """Hash tracked Cargo manifests, not a mutable Cargo metadata cache."""
    paths = run(repo, ["git", "ls-files", "--", "Cargo.toml", "**/Cargo.toml"]).splitlines()
    digest = hashlib.sha256()
    for rel in sorted(path for path in paths if Path(path).name == "Cargo.toml"):
        path = repo / rel
        digest.update(rel.encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def source_tree_sha(repo: Path) -> str:
    """Hash tracked source while excluding mutable evidence artifacts.

    Evidence is derived from this source; excluding it avoids a self-referential
    tree hash while still detecting code, manifest, and documentation drift.
    Gitlinks are hashed by their recorded object ID because a gitlink is a
    directory in the checkout rather than a readable source file.
    """
    raw = subprocess.run(
        ["git", "ls-files", "-s", "-z"],
        cwd=repo,
        stdout=subprocess.PIPE,
        check=True,
    ).stdout.decode("utf-8")
    entries: list[tuple[str, str, str]] = []
    for entry in raw.split("\0"):
        if not entry:
            continue
        header, rel = entry.split("\t", 1)
        mode, object_id, _stage = header.split()
        if rel not in EVIDENCE_PATHS and not rel.startswith("release/evidence/"):
            entries.append((rel, mode, object_id))
    digest = hashlib.sha256()
    for rel, mode, object_id in sorted(entries):
        path = repo / rel
        digest.update(rel.encode("utf-8"))
        digest.update(b"\0")
        if mode == "160000":
            digest.update(f"gitlink:{object_id}".encode("ascii"))
        elif mode == "120000":
            # pathlib.read_bytes() follows symlinks and can land on a directory;
            # hash the tracked link target instead.
            digest.update(f"symlink:{path.readlink()}".encode("utf-8"))
        else:
            digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def source_binding(repo: Path, command_receipts: list[dict[str, Any]]) -> dict[str, Any]:
    lock = repo / "Cargo.lock"
    return {
        "commit_sha": run(repo, ["git", "rev-parse", "HEAD"]),
        "tree_sha": source_tree_sha(repo),
        "cargo_lock_sha256": sha256_file(lock) if lock.is_file() else None,
        "toolchain": run(repo, ["rustc", "-Vv"]),
        "platform": platform_module.platform(),
        "workspace_inventory_sha256": workspace_inventory_sha256(repo),
        "command_receipts": command_receipts,
    }


def evidence_only_descendant(repo: Path, recorded_commit: str, head_commit: str) -> bool:
    """Return whether HEAD only adds/modifies derived evidence after recording.

    A receipt cannot name the commit which contains itself: adding its content
    necessarily changes the commit object.  Recording therefore binds the
    pre-evidence source commit, and verification permits a descendant only when
    every intervening changed path is a declared derived-evidence artifact.
    """
    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", recorded_commit, head_commit],
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if ancestor.returncode != 0:
        return False
    changed = run(repo, ["git", "diff", "--name-only", f"{recorded_commit}..{head_commit}"])
    return all(
        path in EVIDENCE_PATHS or path.startswith("release/evidence/")
        for path in changed.splitlines()
        if path
    )


def verify_binding(repo: Path, binding: dict[str, Any]) -> list[str]:
    findings: list[str] = []
    missing = sorted(REQUIRED_BINDING_FIELDS - set(binding))
    if missing:
        findings.append("missing source_binding fields: " + ", ".join(missing))
        return findings
    recorded_commit = binding["commit_sha"]
    head_commit = run(repo, ["git", "rev-parse", "HEAD"])
    if recorded_commit != head_commit and not evidence_only_descendant(
        repo, recorded_commit, head_commit
    ):
        findings.append("commit SHA mismatch (HEAD is not an evidence-only descendant)")
    if binding["tree_sha"] != source_tree_sha(repo):
        findings.append("source tree hash mismatch")
    lock = repo / "Cargo.lock"
    actual_lock = sha256_file(lock) if lock.is_file() else None
    if binding["cargo_lock_sha256"] != actual_lock:
        findings.append("Cargo.lock hash mismatch")
    if binding["workspace_inventory_sha256"] != workspace_inventory_sha256(repo):
        findings.append("workspace inventory hash mismatch")
    if binding["toolchain"] != run(repo, ["rustc", "-Vv"]):
        findings.append("Rust toolchain mismatch")
    if binding["platform"] != platform_module.platform():
        findings.append("platform mismatch")
    receipts = binding.get("command_receipts")
    if not isinstance(receipts, list) or not receipts:
        findings.append("missing command receipts")
        return findings
    for receipt in receipts:
        for field in ("command", "argv", "cwd", "exit_code", "stdout", "stderr"):
            if field not in receipt:
                findings.append(f"command receipt missing {field}")
                break
        for stream in ("stdout", "stderr"):
            stream_info = receipt.get(stream, {})
            rel = stream_info.get("path") if isinstance(stream_info, dict) else None
            expected = stream_info.get("sha256") if isinstance(stream_info, dict) else None
            if not rel or not expected:
                findings.append(f"command receipt missing {stream} digest")
                continue
            path = repo / rel
            if not path.is_file():
                findings.append(f"missing {stream} log: {rel}")
            elif sha256_file(path) != expected:
                findings.append(f"{stream} log digest mismatch: {rel}")
    return findings

#!/usr/bin/env python3
"""Behavioral smoke test for z.py Codex archival normalization.

This creates a temp mini-repo, copies z.py into it, seeds stale Codex artifacts,
runs archive-only, and verifies idempotent archival. It assumes z.py has P22 flags.
"""
from __future__ import annotations
import json, shutil, subprocess, sys, tempfile
from pathlib import Path


def run(cmd, cwd):
    print("$", " ".join(cmd))
    return subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)


def main() -> int:
    source_z = Path("z.py")
    if not source_z.exists():
        print("FAIL: run from AiDENs repo root with z.py present")
        return 1
    with tempfile.TemporaryDirectory() as td:
        root = Path(td) / "mini-aidens"
        root.mkdir()
        shutil.copy2(source_z, root / "z.py")
        (root / "Cargo.toml").write_text("[workspace]\nresolver='2'\nmembers=[]\n", encoding="utf-8")
        (root / "README.md").write_text("mini\n", encoding="utf-8")
        for rel in [
            ".codex/prompts/P20_OLD.md",
            ".codex/tasks/p20_old.json",
            ".codex_evidence/contract_ownership/00/phase_report.md",
            "docs/p21/P21_SCOPE.md",
            "prompts/p21/P21_CODEX_RUN_PROMPT.md",
            "handoffs/p21/FINAL_AUDIT_REPORT.md",
        ]:
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(f"stale {rel}\n", encoding="utf-8")
        result = run([sys.executable, "z.py", "--root", ".", "--profile", "aidens", "--archive-only", "--strict"], root)
        print(result.stdout)
        if result.returncode != 0:
            print("FAIL: archive-only returned nonzero")
            return result.returncode
        archive_root = root / "docs" / "codex-runs" / "archive"
        if not archive_root.exists():
            print("FAIL: archive root not created")
            return 1
        active_left = [p for p in [root/".codex", root/".codex_evidence", root/"docs/p21", root/"prompts/p21", root/"handoffs/p21"] if p.exists()]
        if active_left:
            print("FAIL: stale active paths remain:", [str(p.relative_to(root)) for p in active_left])
            return 1
        manifests = list(archive_root.rglob("ARCHIVE_MANIFEST.json"))
        if not manifests:
            print("FAIL: no archive manifests written")
            return 1
        # Idempotence: second run should not move anything or fail.
        result2 = run([sys.executable, "z.py", "--root", ".", "--profile", "aidens", "--archive-only", "--strict"], root)
        print(result2.stdout)
        if result2.returncode != 0:
            print("FAIL: second archive-only returned nonzero")
            return result2.returncode
    print("PASS: z.py archival selftest passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--package-dir")
ap.add_argument("--run")
ap.add_argument("--root", default=".")
args = ap.parse_args()
ROOT = Path(args.root).resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"


def fail(msg: str) -> int:
    print(f"FAIL: {msg}", file=sys.stderr)
    return 2


def active_run() -> str:
    if args.run:
        return args.run.upper()
    if os.environ.get("AIDENS_CURRENT_RUN"):
        return os.environ["AIDENS_CURRENT_RUN"].upper()
    data = json.loads(LEDGER.read_text(encoding="utf-8"))
    return str(data["active_run"]).upper()


def find_package_dir(run: str) -> Path:
    if args.package_dir:
        return Path(args.package_dir).resolve()
    run_segment = run.lower()
    candidates = [
        ROOT / "target" / run_segment / "package",
        ROOT,
    ]
    for c in candidates:
        if c.exists() and list(c.glob(f"AiDENs-*{run_segment}*.manifest.json")):
            return c
    return ROOT / "target" / run_segment / "package"


def main() -> int:
    if not LEDGER.exists() and not args.run:
        return fail(f"missing {LEDGER.relative_to(ROOT)} and no --run provided")
    run = active_run()
    package_dir = find_package_dir(run)
    if not package_dir.exists():
        return fail(f"missing package dir: {package_dir}")

    manifests = sorted(package_dir.glob("AiDENs-*.manifest.json"), key=lambda p: p.stat().st_mtime, reverse=True)
    manifests = [p for p in manifests if run.lower() in p.name.lower() or run.upper() in p.name.upper()]
    if not manifests:
        return fail(f"no AiDENs manifest in {package_dir} for run {run}")
    manifest_path = manifests[0]
    prefix = manifest_path.name.removesuffix(".manifest.json")
    paths = {
        "manifest": manifest_path,
        "package": manifest_path.with_name(prefix + ".zip"),
        "report": manifest_path.with_name(prefix + ".report.md"),
        "findings": manifest_path.with_name(prefix + ".findings.json"),
        "excluded": manifest_path.with_name(prefix + ".excluded.json"),
    }
    for label, path in paths.items():
        if not path.exists():
            return fail(f"missing {label} sidecar: {path}")

    try:
        findings = json.loads(paths["findings"].read_text(encoding="utf-8"))
        manifest = json.loads(paths["manifest"].read_text(encoding="utf-8"))
    except Exception as e:
        return fail(f"unable to parse sidecars: {e}")

    if findings.get("error_count") != 0 or findings.get("warning_count") != 0:
        return fail(f"package findings not clean: errors={findings.get('error_count')} warnings={findings.get('warning_count')}")

    codex_archive = manifest.get("codex_archive", {}) or {}
    current_run = str(codex_archive.get("current_run", "")).upper()
    if current_run != run:
        return fail(f"manifest codex_archive.current_run={current_run!r}, expected {run}")
    if manifest.get("archive_sha256_semantics") != "zip-byte-sha256-not-canonical-content-hash":
        return fail("archive_sha256_semantics missing or ambiguous")
    cmh = manifest.get("content_manifest_sha256")
    if not isinstance(cmh, str) or len(cmh) != 64:
        return fail("content_manifest_sha256 missing/invalid")
    if codex_archive.get("errors"):
        return fail(f"codex_archive errors present: {codex_archive.get('errors')}")

    print(f"PASS: package validated for {run}: {paths['package']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

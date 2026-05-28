#!/usr/bin/env python3
"""Validate AiDENs sibling workspace prerequisites.

This is a local operator/replay prerequisite check. It does not define canonical
ownership; it verifies that Cargo path dependency roots required by AiDENs exist
beside the AiDENs checkout.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--receipt-out")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    manifest = root / "Cargo.toml"
    receipt = {
        "artifact_kind": "local_operator_sibling_workspace_layout_receipt",
        "support_tier": "verification",
        "semantic_status": "exact_check",
        "root": str(root),
        "expected_parent_workspace": str(root.parent),
        "classification": "not_attempted",
        "missing": [],
        "present": [],
        "path_dependencies": [],
        "known_limits": [
            "This check verifies local path dependency presence only; it does not prove canonical sibling correctness."
        ],
    }

    if not manifest.exists():
        receipt["classification"] = "manifest_missing"
        return finish(args.receipt_out, receipt, 2, f"FAIL: missing Cargo manifest: {manifest}")

    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    deps = data.get("workspace", {}).get("dependencies", {})
    for name, spec in sorted(deps.items()):
        if not isinstance(spec, dict):
            continue
        dep_path = str(spec.get("path", ""))
        if not dep_path.startswith("../"):
            continue
        resolved = (root / dep_path).resolve()
        row = {
            "crate": name,
            "path": dep_path,
            "resolved_path": str(resolved),
            "present": resolved.is_dir(),
        }
        receipt["path_dependencies"].append(row)
        if row["present"]:
            receipt["present"].append(name)
        else:
            receipt["missing"].append(name)

    if receipt["missing"]:
        receipt["classification"] = "sibling_workspace_missing"
        msg = "FAIL: sibling_workspace_missing: " + ", ".join(receipt["missing"])
        return finish(args.receipt_out, receipt, 2, msg)

    receipt["classification"] = "sibling_workspace_present"
    msg = (
        "PASS: sibling workspace layout present; "
        f"path_dependencies={len(receipt['path_dependencies'])}"
    )
    return finish(args.receipt_out, receipt, 0, msg)


def finish(receipt_out: str | None, receipt: dict, code: int, message: str) -> int:
    if receipt_out:
        out = Path(receipt_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(message, file=sys.stdout if code == 0 else sys.stderr)
    return code


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("package_pos", nargs="?")
ap.add_argument("--package", dest="package_opt")
ap.add_argument("--verifier", default="scripts/verify_current.sh")
ap.add_argument("--require-verifier", action="store_true")
ap.add_argument("--receipt-out", default="target/verify-current/P31B_VERIFICATION/package_self_replay_receipt.json")
ap.add_argument("--expected-run")
args = ap.parse_args()
package_arg = args.package_opt or args.package_pos
if not package_arg:
    print("FAIL: package path required", file=sys.stderr)
    sys.exit(2)

package = Path(package_arg).resolve()
receipt = {
    "artifact_kind": "aidens_package_self_replay_receipt.v1",
    "created_utc": datetime.now(timezone.utc).isoformat(),
    "package": str(package),
    "verifier": args.verifier,
    "expected_run": args.expected_run,
    "status": "not_attempted",
    "blockers": [],
    "known_limits": [],
}


def finish(code: int, status: str, message: str) -> int:
    receipt["status"] = status
    receipt["message"] = message
    out = Path(args.receipt_out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(message, file=sys.stdout if code == 0 else sys.stderr)
    return code

if not package.exists():
    raise SystemExit(finish(2, "package_missing", f"FAIL: package not found: {package}"))

with tempfile.TemporaryDirectory(prefix="aidens_p31b_replay_") as td_s:
    td = Path(td_s)
    try:
        with zipfile.ZipFile(package) as zf:
            zf.extractall(td)
    except Exception as e:
        raise SystemExit(finish(2, "extract_failed", f"FAIL: package extraction failed: {e}"))
    candidates = [td / "AiDENs", td / "Libraries" / "AiDENs", td]
    repo = next((p for p in candidates if (p / args.verifier).exists() or (p / "z.py").exists()), None)
    if repo is None:
        raise SystemExit(finish(2, "repo_missing", "FAIL: extracted package has no recognizable AiDENs repo root"))
    receipt["extracted_repo"] = str(repo)
    verifier = repo / args.verifier
    if not verifier.exists():
        msg = f"FAIL: verifier missing in extracted package: {args.verifier}"
        if args.require_verifier:
            raise SystemExit(finish(2, "verifier_missing", msg))
        raise SystemExit(finish(0, "verifier_missing_skipped", msg))

    env = os.environ.copy()
    for key in list(env):
        if key.startswith("P27_") or key.startswith("P28_") or key.startswith("P30_"):
            env.pop(key, None)
    result = subprocess.run(["bash", str(verifier)], cwd=repo, env=env, text=True, capture_output=True)
    receipt["verifier_exit_code"] = result.returncode
    receipt["verifier_stdout_tail"] = result.stdout[-6000:]
    receipt["verifier_stderr_tail"] = result.stderr[-6000:]
    if result.returncode == 0:
        raise SystemExit(finish(0, "passed", "PASS: extracted package self-replay passed"))

    combined = (result.stdout + "\n" + result.stderr).lower()
    if "cargo" in combined and ("not found" in combined or "no such file" in combined):
        receipt["blockers"].append("cargo_or_toolchain_missing_in_replay")
        raise SystemExit(finish(2, "blocked", "FAIL: package self-replay blocked by missing cargo/toolchain"))
    if "../" in combined and ("failed to read" in combined or "no such file" in combined):
        receipt["blockers"].append("external_path_dependency_unavailable_in_extracted_package")
        raise SystemExit(finish(2, "blocked", "FAIL: package self-replay blocked by unavailable external path dependency"))
    if "permission denied" in combined or "permissionerror" in combined:
        receipt["blockers"].append("permission_denied_in_extracted_replay")
        receipt["verifier_stdout_tail"] = result.stdout[-6000:]
        receipt["verifier_stderr_tail"] = result.stderr[-6000:]
        raise SystemExit(finish(2, "blocked", "FAIL: package self-replay blocked by permission denied in extracted environment"))
    raise SystemExit(finish(2, "failed", f"FAIL: extracted verifier failed with exit {result.returncode}"))

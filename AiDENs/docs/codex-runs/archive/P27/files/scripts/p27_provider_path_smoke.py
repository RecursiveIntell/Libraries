#!/usr/bin/env python3
"""P27 provider-path smoke checks.

The required gate is a supported-local mock Plan->Act->Verify run through the
CLI. Local Ollama is optional and is recorded as an environment prerequisite
when it is unavailable.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", default=".", help="repository root")
    parser.add_argument(
        "--receipt-out",
        default="target/p27/audit/phase09_provider_path_smoke_receipt.json",
        help="where to write the structured smoke receipt",
    )
    parser.add_argument(
        "--work-dir",
        default=None,
        help="work directory for generated CLI fixtures",
    )
    parser.add_argument(
        "--allow-optional-ollama",
        action="store_true",
        help="probe local Ollama and run provider-check when available",
    )
    parser.add_argument(
        "--require-ollama",
        action="store_true",
        help="fail when local Ollama is unavailable or provider-check fails",
    )
    parser.add_argument(
        "--ollama-url",
        default=os.environ.get("P27_OLLAMA_URL", "http://localhost:11434"),
        help="local Ollama base URL",
    )
    parser.add_argument(
        "--ollama-config",
        default="examples/aidens.ollama.toml",
        help="AiDENs config used for optional Ollama provider-check",
    )
    return parser.parse_args()


def run_command(
    root: Path, command: list[str], *, timeout_seconds: int = 180
) -> dict[str, Any]:
    started = time.time()
    completed = subprocess.run(
        command,
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout_seconds,
        env={**os.environ, "CARGO_TERM_COLOR": "never"},
    )
    return {
        "command": command,
        "exit_code": completed.returncode,
        "elapsed_ms": int((time.time() - started) * 1000),
        "stdout": completed.stdout,
    }


def require_ok(result: dict[str, Any]) -> None:
    if result["exit_code"] != 0:
        rendered = " ".join(result["command"])
        raise RuntimeError(
            f"command failed ({result['exit_code']}): {rendered}\n{result['stdout']}"
        )


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def run_mock_e2e(root: Path, work_dir: Path) -> dict[str, Any]:
    if work_dir.exists():
        shutil.rmtree(work_dir)
    agent_dir = work_dir / "agent"
    out_dir = work_dir / "run"
    work_dir.mkdir(parents=True, exist_ok=True)

    commands: list[dict[str, Any]] = []

    new_result = run_command(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "aidens-cli",
            "--",
            "agent",
            "new",
            "--template",
            "local-coding",
            "--out",
            str(agent_dir),
        ],
    )
    commands.append(new_result)
    require_ok(new_result)

    task_path = agent_dir / "task.md"
    task_path.write_text("Read README.md and report evidence.\n", encoding="utf-8")

    run_result = run_command(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "aidens-cli",
            "--",
            "agent",
            "run",
            "--spec",
            str(agent_dir / "agent.json"),
            "--task",
            str(task_path),
            "--sandbox-root",
            str(agent_dir / "sandbox"),
            "--out",
            str(out_dir),
        ],
    )
    commands.append(run_result)
    require_ok(run_result)

    inspect_result = run_command(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "aidens-cli",
            "--",
            "agent",
            "inspect",
            "--run",
            str(out_dir / "receipts"),
        ],
    )
    commands.append(inspect_result)
    require_ok(inspect_result)

    loop_output = load_json(out_dir / "plan-act-verify-output.json")
    bundle = load_json(out_dir / "run-bundle.json")
    store_record = load_json(out_dir / "run-bundle-store-record.json")
    inspect_report = json.loads(inspect_result["stdout"])
    write_json(work_dir / "inspect-from-store.json", inspect_report)

    checks = {
        "loop_outcome_success": loop_output.get("outcome") == "Success",
        "bundle_schema_v3": bundle.get("schema") == "AiDENsRunBundleV3",
        "support_tier_supported_local": bundle.get("support", {}).get("support_tier")
        == "supported-local",
        "provider_route_mock": inspect_report.get("provider_route") == "mock",
        "event_log_digest_verified": inspect_report.get("event_log_digest_verified") is True,
        "store_semantic_status_exact": store_record.get("semantic_status") == "exact_check",
        "failure_not_degraded": bundle.get("failure", {}).get("degraded") is False,
    }
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        raise RuntimeError(f"mock provider-path E2E failed checks: {', '.join(failed)}")

    return {
        "status": "passed",
        "semantic_status": "exact_check",
        "support_tier": "supported-local",
        "provider_route": "mock",
        "work_dir": str(work_dir),
        "run_bundle": str(out_dir / "run-bundle.json"),
        "receipt_store": str(out_dir / "receipts"),
        "inspect_report": str(work_dir / "inspect-from-store.json"),
        "checks": checks,
        "commands": [
            {
                "command": command["command"],
                "exit_code": command["exit_code"],
                "elapsed_ms": command["elapsed_ms"],
            }
            for command in commands
        ],
    }


def probe_ollama(url: str) -> tuple[bool, str]:
    probe_url = url.rstrip("/") + "/api/tags"
    request = urllib.request.Request(probe_url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=2) as response:
            if 200 <= response.status < 300:
                return True, f"ollama probe passed: {probe_url}"
            return False, f"ollama probe returned HTTP {response.status}: {probe_url}"
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        return False, f"ollama unavailable at {probe_url}: {exc}"


def run_optional_ollama(
    root: Path, config: str, url: str, *, required: bool
) -> dict[str, Any]:
    available, reason = probe_ollama(url)
    if not available:
        status = "failed" if required else "skipped"
        return {
            "status": status,
            "semantic_status": "degraded_exact_check",
            "support_tier": "environment-prerequisite",
            "reason": reason,
            "required": required,
        }

    result = run_command(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "aidens-cli",
            "--",
            "provider-check",
            "--config",
            config,
        ],
        timeout_seconds=120,
    )
    if result["exit_code"] != 0:
        status = "failed" if required else "skipped"
        return {
            "status": status,
            "semantic_status": "degraded_exact_check",
            "support_tier": "environment-prerequisite",
            "reason": "provider-check failed after local Ollama probe",
            "required": required,
            "command": {
                "command": result["command"],
                "exit_code": result["exit_code"],
                "elapsed_ms": result["elapsed_ms"],
                "stdout": result["stdout"],
            },
        }

    provider_report = json.loads(result["stdout"])
    return {
        "status": "passed",
        "semantic_status": "exact_check",
        "support_tier": provider_report.get("support_tier", "partial"),
        "provider": provider_report.get("provider"),
        "route": provider_report.get("route"),
        "native_tool_loop": provider_report.get("native_tool_loop"),
        "reason_codes": provider_report.get("reason_codes", []),
        "required": required,
        "command": {
            "command": result["command"],
            "exit_code": result["exit_code"],
            "elapsed_ms": result["elapsed_ms"],
        },
    }


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    receipt_out = (root / args.receipt_out).resolve()
    work_dir = (
        Path(args.work_dir)
        if args.work_dir
        else root / "target/p27/audit/phase09_provider_path_smoke_work"
    ).resolve()

    receipt: dict[str, Any] = {
        "artifact_kind": "local_operator_provider_path_smoke_receipt",
        "phase": "P27-09",
        "ownership": "AiDENs-local operator smoke evidence; canonical provider/tool/verification semantics remain in owner crates.",
        "support_tier": "verification",
        "semantic_status": "exact_check",
        "mock_provider": None,
        "ollama": {
            "status": "not_requested",
            "semantic_status": "degraded_exact_check",
            "support_tier": "environment-prerequisite",
            "reason": "optional local Ollama smoke not requested",
        },
        "known_limits": [
            "No hosted provider keys are required or read.",
            "Ollama is optional local smoke evidence and is not a CI/verifier prerequisite.",
            "This does not claim native Ollama tool-loop support.",
            "This is AiDENs-local operator evidence, not canonical provider truth.",
        ],
    }

    try:
        receipt["mock_provider"] = run_mock_e2e(root, work_dir)
        if args.allow_optional_ollama or args.require_ollama:
            receipt["ollama"] = run_optional_ollama(
                root, args.ollama_config, args.ollama_url, required=args.require_ollama
            )
            if args.require_ollama and receipt["ollama"]["status"] != "passed":
                receipt["semantic_status"] = "degraded_exact_check"
                write_json(receipt_out, receipt)
                print(json.dumps(receipt, indent=2, sort_keys=True))
                return 21
        write_json(receipt_out, receipt)
        print(json.dumps(receipt, indent=2, sort_keys=True))
        return 0
    except Exception as exc:
        receipt["semantic_status"] = "failed_exact_check"
        receipt["error"] = str(exc)
        write_json(receipt_out, receipt)
        print(json.dumps(receipt, indent=2, sort_keys=True), file=sys.stderr)
        return 20


if __name__ == "__main__":
    raise SystemExit(main())

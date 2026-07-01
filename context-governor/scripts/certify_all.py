#!/usr/bin/env python3
"""Run context-governor certification gates and emit receipt files."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str], cwd: Path = ROOT, timeout: int = 300, env: dict[str, str] | None = None) -> dict[str, Any]:
    started = time.time()
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
        env={**os.environ, **(env or {})},
    )
    return {
        "command": cmd,
        "cwd": str(cwd),
        "ok": proc.returncode == 0,
        "returncode": proc.returncode,
        "elapsed_ms": round((time.time() - started) * 1000, 2),
        "stdout_tail": proc.stdout[-6000:],
        "stderr_tail": proc.stderr[-6000:],
    }


def gate(name: str, result: dict[str, Any], required: bool = True) -> dict[str, Any]:
    return {"name": name, "required": required, **result}


def write_adversarial_fixtures(out_dir: Path) -> dict[str, Any]:
    return run(
        ["python3", "scripts/generate_adversarial_fixtures.py", "--out", str(out_dir)],
        timeout=60,
    )


def run_adversarial(fixture_dir: Path, out_path: Path, quick: bool) -> dict[str, Any]:
    targets = "1200" if quick else "1200,8000"
    modes = "hard_cascade" if quick else "soft_warn,hard_cascade"
    return run(
        [
            "python3",
            "scripts/evaluate_adversarial_fixtures.py",
            "--fixtures",
            str(fixture_dir),
            "--engine",
            "context_governor",
            "--target-tokens",
            targets,
            "--budget-modes",
            modes,
            "--out",
            str(out_path),
        ],
        timeout=300,
    )


def run_task_success(out_path: Path, md_path: Path) -> dict[str, Any]:
    return run(
        [
            "python3",
            "scripts/task_success_eval.py",
            "--out",
            str(out_path),
            "--markdown",
            str(md_path),
        ],
        timeout=180,
    )


def run_historical_answerability(out_dir: Path, quick: bool) -> dict[str, Any]:
    db_path = Path.home() / ".hermes" / "state.db"
    if not db_path.exists():
        return {
            "command": ["python3", "scripts/hermes_task_replay_eval.py"],
            "cwd": str(ROOT),
            "ok": True,
            "returncode": 0,
            "elapsed_ms": 0,
            "stdout_tail": "skipped: ~/.hermes/state.db not found",
            "stderr_tail": "",
            "skipped": True,
        }
    return run(
        [
            "python3",
            "scripts/hermes_task_replay_eval.py",
            "--limit",
            "3" if quick else "10",
            "--min-messages",
            "12",
            "--out-dir",
            str(out_dir),
        ],
        timeout=300,
    )


def run_cross_engine_comparison(out_dir: Path) -> dict[str, Any]:
    return run(
        [
            "python3",
            "scripts/compare_context_engines_live.py",
            "--quick",
            "--out-dir",
            str(out_dir),
        ],
        timeout=300,
    )


def run_hermes_plugin_tests() -> dict[str, Any]:
    hermes_repo = Path.home() / ".hermes" / "hermes-agent"
    test_path = hermes_repo / "tests" / "plugins" / "test_context_governor_plugin.py"
    if not test_path.exists():
        return {
            "command": ["python3", "-m", "pytest", str(test_path), "-q"],
            "cwd": str(hermes_repo),
            "ok": False,
            "returncode": 127,
            "elapsed_ms": 0,
            "stdout_tail": "",
            "stderr_tail": f"missing Hermes plugin test file: {test_path}",
        }
    return run(
        ["python3", "-m", "pytest", "tests/plugins/test_context_governor_plugin.py", "-q"],
        cwd=hermes_repo,
        timeout=180,
    )


def read_json_if_exists(path: Path) -> Any:
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        return {"error": str(exc), "path": str(path)}


def write_markdown(report: dict[str, Any], path: Path) -> None:
    lines = [
        "# Context Governor Certification",
        "",
        f"- Schema: `{report['schema']}`",
        f"- Crate: `{report['crate']}`",
        f"- Quick: `{report['quick']}`",
        f"- OK: `{report['ok']}`",
        "",
        "## Gates",
        "",
        "| Gate | Required | OK | Return Code |",
        "|---|---:|---:|---:|",
    ]
    for item in report["gates"]:
        lines.append(
            f"| `{item['name']}` | `{item['required']}` | `{item['ok']}` | `{item.get('returncode')}` |"
        )
    task = report.get("task_success_summary") or {}
    if task:
        lines.extend(
            [
                "",
                "## Task Success",
                "",
                f"- OK: `{task.get('ok')}`",
                f"- Context-governor answerability: {task.get('context_governor_answerability', 0):.1%}",
                f"- Head/tail answerability: {task.get('head_tail_answerability', 0):.1%}",
                f"- Token reduction vs full: {task.get('token_reduction_vs_full', 0):.1%}",
            ]
        )
    path.write_text("\n".join(lines) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true", help="Skip slow release-style gates")
    parser.add_argument("--skip-hermes", action="store_true", help="Skip Hermes plugin tests")
    parser.add_argument("--out-dir", default=None)
    args = parser.parse_args()

    stamp = time.strftime("%Y%m%d-%H%M%S")
    out_dir = Path(args.out_dir) if args.out_dir else ROOT / "target" / "certification" / stamp
    out_dir.mkdir(parents=True, exist_ok=True)
    fixture_dir = out_dir / "adversarial-fixtures"
    adversarial_json = out_dir / "adversarial-report.json"
    task_json = out_dir / "task-success.json"
    task_md = out_dir / "task-success.md"
    comparison_dir = out_dir / "same-transcript-comparison"
    historical_dir = out_dir / "historical-answerability"

    gates = [
        gate("cargo-fmt", run(["cargo", "fmt", "--check"], timeout=60)),
        gate("cargo-test", run(["cargo", "test", "--all-targets"], timeout=300)),
        gate("python-tests", run(["python3", "-m", "pytest", "tests_py", "-q"], timeout=120)),
        gate("generate-adversarial-fixtures", write_adversarial_fixtures(fixture_dir)),
        gate("adversarial-eval", run_adversarial(fixture_dir, adversarial_json, args.quick)),
        gate("task-success-eval", run_task_success(task_json, task_md)),
        gate("same-transcript-comparison", run_cross_engine_comparison(comparison_dir)),
        gate("historical-answerability", run_historical_answerability(historical_dir, args.quick), required=False),
    ]
    if not args.quick:
        gates.insert(1, gate("cargo-clippy", run(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=300)))
    if not args.skip_hermes:
        gates.append(gate("hermes-plugin-tests", run_hermes_plugin_tests(), required=False))

    report = {
        "schema": "ContextGovernorCertificationV1",
        "crate": str(ROOT),
        "created_unix": int(time.time()),
        "quick": bool(args.quick),
        "ok": all(item["ok"] for item in gates if item["required"]),
        "gates": gates,
        "artifacts": {
            "adversarial_report": str(adversarial_json),
            "task_success": str(task_json),
            "task_success_markdown": str(task_md),
            "same_transcript_comparison": str(comparison_dir / "same-transcript-comparison.json"),
            "same_transcript_comparison_markdown": str(comparison_dir / "same-transcript-comparison.md"),
            "historical_answerability": str(historical_dir / "historical-answerability.json"),
            "historical_answerability_markdown": str(historical_dir / "historical-answerability.md"),
        },
        "adversarial_summary": read_json_if_exists(adversarial_json),
        "task_success_summary": read_json_if_exists(task_json),
    }
    report_path = out_dir / "certification.json"
    md_path = out_dir / "certification.md"
    report_path.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    write_markdown(report, md_path)
    print(json.dumps({"ok": report["ok"], "report": str(report_path), "markdown": str(md_path)}, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

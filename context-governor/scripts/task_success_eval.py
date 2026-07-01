#!/usr/bin/env python3
"""Run deterministic task-success checks over context-governor fixtures."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from statistics import mean
from typing import Any


def synthetic_fixture() -> dict[str, Any]:
    return {
        "fixture_id": "synthetic_task_success",
        "request": {
            "session_id": "synthetic-task-success",
            "messages": [
                {"role": "system", "content": "You are a coding agent."},
                {
                    "role": "user",
                    "content": "Build the parser. Acceptance gate: cargo test must pass.",
                },
                {
                    "role": "assistant",
                    "content": "Decision: use deterministic JSON parsing, not regex.",
                },
                {
                    "role": "tool",
                    "content": ("bulk log\n" * 500)
                    + "error[E0425]: cannot find value `parser`\n/src/lib.rs\n",
                },
                {"role": "assistant", "content": "Fixed compile error in /src/lib.rs."},
                {"role": "user", "content": "Latest task: summarize what remains."},
            ],
            "policy": {
                "target_tokens": 260,
                "protect_first_n": 0,
                "protect_last_n": 1,
                "summary_max_chars": 2400,
                "allocator": "deterministic_v1",
                "semantic_memory_enabled": False,
                "archive_memory_enabled": False,
                "budget_mode": "soft_warn",
                "token_counter": "provider_chat_approx",
            },
            "focus": None,
        },
        "questions": [
            {
                "question": "What must pass?",
                "expected_terms": ["cargo test must pass"],
                "forbidden_terms": [],
            },
            {
                "question": "What parser strategy was chosen?",
                "expected_terms": ["deterministic JSON parsing"],
                "forbidden_terms": ["regex"],
            },
            {
                "question": "Which compile error and file mattered?",
                "expected_terms": ["E0425", "/src/lib.rs"],
                "forbidden_terms": [],
            },
            {
                "question": "What is the active task?",
                "expected_terms": ["Latest task: summarize what remains"],
                "forbidden_terms": [],
            },
        ],
    }


def run_task_success(crate_dir: Path, fixture: dict[str, Any]) -> dict[str, Any]:
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "task_success_eval"],
        cwd=crate_dir,
        input=json.dumps(fixture, ensure_ascii=False),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    return json.loads(proc.stdout)


def summarize(report: dict[str, Any]) -> dict[str, Any]:
    baselines = report.get("baselines") or []
    by_name = {item["name"]: item for item in baselines}
    governed = by_name.get("context_governor", {})
    full = by_name.get("full", {})
    head_tail = by_name.get("head_tail", {})
    return {
        "schema": "ContextGovernorTaskSuccessEvalV1",
        "fixture_id": report.get("fixture_id"),
        "receipt_id": report.get("receipt_id"),
        "ok": bool(
            governed
            and governed.get("answerability_rate") == 1.0
            and governed.get("incorrect_action_risk") == 0
            and governed.get("active_task_visible") is True
        ),
        "context_governor_answerability": governed.get("answerability_rate", 0),
        "context_governor_incorrect_action_risk": governed.get("incorrect_action_risk", 0),
        "head_tail_answerability": head_tail.get("answerability_rate", 0),
        "full_answerability": full.get("answerability_rate", 0),
        "token_reduction_vs_full": (
            1 - governed.get("tokens", 0) / full.get("tokens", 1)
            if full.get("tokens")
            else 0
        ),
        "warnings": report.get("warnings") or [],
        "raw_report": report,
    }


def write_markdown(summary: dict[str, Any], path: Path) -> None:
    path.write_text(
        "\n".join(
            [
                "# Context Governor Task-Success Eval",
                "",
                f"- Fixture: `{summary['fixture_id']}`",
                f"- OK: `{summary['ok']}`",
                f"- Context-governor answerability: {summary['context_governor_answerability']:.1%}",
                f"- Head/tail answerability: {summary['head_tail_answerability']:.1%}",
                f"- Token reduction vs full: {summary['token_reduction_vs_full']:.1%}",
                f"- Incorrect-action risk: {summary['context_governor_incorrect_action_risk']}",
                f"- Receipt: `{summary['receipt_id']}`",
                "",
            ]
        )
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate-dir", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--fixture", default=None)
    parser.add_argument("--out", required=True)
    parser.add_argument("--markdown", default=None)
    args = parser.parse_args()

    crate_dir = Path(args.crate_dir)
    fixture = json.loads(Path(args.fixture).read_text()) if args.fixture else synthetic_fixture()
    report = run_task_success(crate_dir, fixture)
    summary = summarize(report)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False))
    if args.markdown:
        write_markdown(summary, Path(args.markdown))
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

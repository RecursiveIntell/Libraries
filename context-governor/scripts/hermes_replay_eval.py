#!/usr/bin/env python3
"""Run context-governor replay metrics over local Hermes state.db sessions.

This writes aggregate metrics only. It intentionally does not persist raw message
content or probe text in the markdown report.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import subprocess
from dataclasses import dataclass
from pathlib import Path
from statistics import mean
from tempfile import TemporaryDirectory


@dataclass
class SessionCandidate:
    session_id: str
    title: str
    message_count: int
    char_count: int


def load_candidates(db_path: Path, limit: int, min_messages: int) -> list[SessionCandidate]:
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            """
            SELECT s.id, COALESCE(s.title, ''), COUNT(m.id), SUM(LENGTH(COALESCE(m.content, '')))
            FROM sessions s
            JOIN messages m ON m.session_id = s.id
            WHERE COALESCE(m.content, '') != '' AND m.active = 1
            GROUP BY s.id
            HAVING COUNT(m.id) >= ?
            ORDER BY SUM(LENGTH(COALESCE(m.content, ''))) DESC
            LIMIT ?
            """,
            (min_messages, limit),
        ).fetchall()
    finally:
        conn.close()
    return [SessionCandidate(row[0], row[1], row[2], row[3] or 0) for row in rows]


def load_request(db_path: Path, candidate: SessionCandidate, target_tokens: int, budget_mode: str = "hard_cascade") -> dict:
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            """
            SELECT id, role, content, tool_name
            FROM messages
            WHERE session_id = ? AND COALESCE(content, '') != '' AND active = 1
            ORDER BY timestamp ASC, id ASC
            """,
            (candidate.session_id,),
        ).fetchall()
    finally:
        conn.close()

    messages = []
    for row_id, role, content, tool_name in rows:
        mapped_role = role or "assistant"
        if mapped_role == "tool" or tool_name:
            mapped_role = "tool"
        elif mapped_role not in {"system", "user", "assistant", "tool"}:
            mapped_role = "assistant"
        messages.append(
            {
                "id": f"msg_{row_id}",
                "role": mapped_role,
                "content": content,
            }
        )

    return {
        "session_id": candidate.session_id,
        "messages": messages,
        "policy": {
            "target_tokens": target_tokens,
            "protect_first_n": 1,
            "protect_last_n": 1,
            "summary_max_chars": 2400,
            "allocator": "aggressive_v1",
            "semantic_memory_enabled": False,
            "archive_memory_enabled": False,
            "budget_mode": budget_mode,
            "token_counter": "approx_chars",
        },
        "focus": None,
    }


def run_fixture(crate_dir: Path, request: dict) -> dict:
    try:
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--example", "replay_fixture"],
            cwd=crate_dir,
            input=json.dumps(request),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(
            f"replay_fixture failed for {request.get('session_id')}:\nSTDOUT:\n{exc.stdout}\nSTDERR:\n{exc.stderr}"
        ) from exc
    return json.loads(proc.stdout)


def baseline(report: dict, name: str) -> dict:
    for item in report["baselines"]:
        if item["name"] == name:
            return item
    raise KeyError(name)


def pct(value: float) -> str:
    return f"{value * 100:.1f}%"


def parse_int_list(raw: str) -> list[int]:
    return [int(part.strip()) for part in raw.split(",") if part.strip()]


def parse_str_list(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


def report_ok(report: dict) -> bool:
    return report.get("ok", True) and "baselines" in report


def short_error(report: dict) -> str:
    return str(report.get("error") or "").replace("\n", " ")[:160]


def write_reports(crate_dir: Path, reports: list[dict], candidates: list[SessionCandidate]) -> tuple[Path, Path]:
    out_dir = crate_dir / "target" / "context-governor-replay"
    out_dir.mkdir(parents=True, exist_ok=True)
    json_path = out_dir / "hermes-replay-report.json"
    docs_dir = crate_dir / "docs"
    docs_dir.mkdir(parents=True, exist_ok=True)
    md_path = docs_dir / "hermes-replay-eval-2026-06-27.md"

    json_path.write_text(json.dumps(reports, indent=2))

    rows = []
    aggregate = {
        "sessions": len(reports),
        "full_tokens": [],
        "head_tail_tokens": [],
        "governed_tokens": [],
        "head_tail_recoverable": [],
        "governed_recoverable": [],
        "governed_visible": [],
        "probes": [],
        "active_task_visible": 0,
    }

    by_id = {candidate.session_id: candidate for candidate in candidates}
    for report in reports:
        candidate = by_id[report["fixture_id"]]
        if not report_ok(report):
            rows.append(
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |".format(
                    report["fixture_id"][:12],
                    candidate.message_count,
                    report.get("budget_mode", "unknown"),
                    report.get("target_tokens", "unknown"),
                    "FAIL",
                    "FAIL",
                    "FAIL",
                    "FAIL",
                    short_error(report),
                )
            )
            continue
        full = baseline(report, "full")
        head_tail = baseline(report, "head_tail")
        governed = baseline(report, "context_governor")
        aggregate["full_tokens"].append(full["tokens"])
        aggregate["head_tail_tokens"].append(head_tail["tokens"])
        aggregate["governed_tokens"].append(governed["tokens"])
        aggregate["head_tail_recoverable"].append(head_tail["recoverable_rate"])
        aggregate["governed_recoverable"].append(governed["recoverable_rate"])
        aggregate["governed_visible"].append(governed["visible_rate"])
        aggregate["probes"].append(governed["total_probes"])
        aggregate["active_task_visible"] += int(governed["active_task_visible"])
        rows.append(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |".format(
                report["fixture_id"][:12],
                candidate.message_count,
                report.get("budget_mode", "unknown"),
                report.get("target_tokens", "unknown"),
                full["tokens"],
                head_tail["tokens"],
                governed["tokens"],
                pct(governed["recoverable_rate"]),
                "OK",
            )
        )

    successes = sum(1 for report in reports if report_ok(report))
    failures = len(reports) - successes
    sessions = successes
    avg_full = mean(aggregate["full_tokens"]) if sessions else 0
    avg_governed = mean(aggregate["governed_tokens"]) if sessions else 0
    token_reduction = 1 - (avg_governed / avg_full) if avg_full else 0

    md = f"""# Hermes replay evaluation — 2026-06-27

Source: local Hermes SQLite session store.

Privacy boundary: this report intentionally stores aggregate metrics only. Raw transcript content and extracted probe strings are not written to this markdown file.

Method:

- Select the largest active Hermes sessions by message text volume.
- Convert messages into `CompactRequest` fixtures.
- Extract operational replay probes from active task, acceptance gates, decisions, errors, and file paths.
- Compare three prompt strategies:
  - `full`: original transcript
  - `head_tail`: naive first-message/last-message baseline
  - `context_governor`: compacted prompt plus exact fallback search

Claim boundary:

- This measures anchor survival and exact recoverability.
- It does not measure downstream LLM answer quality.
- It does not store private transcript text in this report.

## Aggregate

- Successful runs: {successes}
- Failures: {failures}
- Avg full tokens: {avg_full:.1f}
- Avg context-governor tokens: {avg_governed:.1f}
- Avg token reduction vs full: {pct(token_reduction)}
- Avg head/tail recoverable rate: {pct(mean(aggregate['head_tail_recoverable']) if sessions else 0)}
- Avg context-governor visible rate: {pct(mean(aggregate['governed_visible']) if sessions else 0)}
- Avg context-governor recoverable rate: {pct(mean(aggregate['governed_recoverable']) if sessions else 0)}
- Active task visible in context-governor: {aggregate['active_task_visible']}/{sessions}

## Per-session metrics

| session | messages | budget mode | target tokens | full tokens | head/tail tokens | governed tokens | governed recoverable | status/error |
|---|---:|---|---:|---:|---:|---:|---:|---|
{chr(10).join(rows)}

Machine-readable report with probe text is local only:

`{json_path}`
"""
    md_path.write_text(md)
    return json_path, md_path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default="/home/sikmindz/.hermes/state.db")
    parser.add_argument("--crate-dir", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--limit", type=int, default=8)
    parser.add_argument("--min-messages", type=int, default=20)
    parser.add_argument("--target-tokens", type=int, default=1200)
    parser.add_argument(
        "--target-tokens-list",
        default=None,
        help="Comma-separated target token budgets. Overrides --target-tokens.",
    )
    parser.add_argument(
        "--budget-mode",
        default="hard_cascade",
        help="Budget mode or comma-separated modes: soft_warn,hard_cascade,fail_closed",
    )
    args = parser.parse_args()

    db_path = Path(args.db)
    crate_dir = Path(args.crate_dir)
    candidates = load_candidates(db_path, args.limit, args.min_messages)
    targets = parse_int_list(args.target_tokens_list) if args.target_tokens_list else [args.target_tokens]
    budget_modes = parse_str_list(args.budget_mode)
    reports = []
    for budget_mode in budget_modes:
        for target_tokens in targets:
            for candidate in candidates:
                request = load_request(db_path, candidate, target_tokens, budget_mode)
                try:
                    report = run_fixture(crate_dir, request)
                    report["ok"] = True
                    report["target_tokens"] = target_tokens
                    report["budget_mode"] = budget_mode
                    reports.append(report)
                except Exception as exc:
                    reports.append(
                        {
                            "ok": False,
                            "fixture_id": candidate.session_id,
                            "target_tokens": target_tokens,
                            "budget_mode": budget_mode,
                            "error": str(exc),
                        }
                    )
    json_path, md_path = write_reports(crate_dir, reports, candidates)
    print(f"wrote {json_path}")
    print(f"wrote {md_path}")


if __name__ == "__main__":
    main()

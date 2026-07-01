#!/usr/bin/env python3
"""Historical Hermes coding-task answerability replay.

This tool reads local Hermes `state.db`, samples coding-heavy sessions, builds
operational question sets, and scores full/head-tail/context-governor prompts.
Public markdown is aggregate-only: it does not include raw transcript content or
raw answer terms.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from statistics import mean
from typing import Any

ROOT = Path(__file__).resolve().parents[1]


@dataclass
class HistoricalSession:
    session_id: str
    title: str
    message_count: int
    char_count: int


def approx_tokens(text: str) -> int:
    return max(1, len(text) // 4)


def render_messages(messages: list[dict[str, Any]]) -> str:
    return "\n\n".join(f"[{m.get('role','assistant').upper()}] {m.get('content','')}" for m in messages)


def hash_term(term: str) -> str:
    return hashlib.sha256(term.encode("utf-8")).hexdigest()[:16]


def load_sessions(db_path: Path, limit: int = 10, min_messages: int = 12) -> list[HistoricalSession]:
    if not db_path.exists():
        return []
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            """
            SELECT s.id, COALESCE(s.title, ''), COUNT(m.id), SUM(LENGTH(COALESCE(m.content, '')))
            FROM sessions s
            JOIN messages m ON m.session_id = s.id
            WHERE COALESCE(m.content, '') != '' AND COALESCE(m.active, 1) = 1
            GROUP BY s.id
            HAVING COUNT(m.id) >= ?
            ORDER BY SUM(LENGTH(COALESCE(m.content, ''))) DESC
            LIMIT ?
            """,
            (min_messages, limit),
        ).fetchall()
    finally:
        conn.close()
    return [HistoricalSession(str(r[0]), str(r[1] or ""), int(r[2]), int(r[3] or 0)) for r in rows]


def load_messages(db_path: Path, session_id: str) -> list[dict[str, Any]]:
    conn = sqlite3.connect(db_path)
    try:
        cols = {row[1] for row in conn.execute("PRAGMA table_info(messages)").fetchall()}
        tool_expr = "tool_name" if "tool_name" in cols else "NULL"
        rows = conn.execute(
            f"""
            SELECT id, role, content, {tool_expr} AS tool_name
            FROM messages
            WHERE session_id = ? AND COALESCE(content, '') != '' AND COALESCE(active, 1) = 1
            ORDER BY timestamp ASC, id ASC
            """,
            (session_id,),
        ).fetchall()
    finally:
        conn.close()
    messages = []
    for row_id, role, content, tool_name in rows:
        mapped = role or "assistant"
        if mapped == "tool" or tool_name:
            mapped = "tool"
        elif mapped not in {"system", "user", "assistant", "tool"}:
            mapped = "assistant"
        messages.append({"id": f"msg_{row_id}", "role": mapped, "content": content})
    return messages


def _first_match(patterns: list[str], text: str) -> str | None:
    for pattern in patterns:
        match = re.search(pattern, text, flags=re.IGNORECASE | re.MULTILINE)
        if match:
            return match.group(0)[:240]
    return None


def generate_questions(messages: list[dict[str, Any]], min_questions: int = 4) -> list[dict[str, Any]]:
    text = render_messages(messages)
    questions: list[dict[str, Any]] = []
    latest_user = next((m.get("content", "") for m in reversed(messages) if m.get("role") == "user"), "")
    if latest_user:
        questions.append({"kind": "active_task", "expected_terms": [latest_user[:180]], "forbidden_terms": []})
    acceptance = _first_match([
        r"acceptance gate[:\s][^\n.]{8,180}",
        r"cargo (?:test|check|clippy)[^\n]{0,120}",
        r"python3? -m pytest[^\n]{0,120}",
    ], text)
    if acceptance:
        questions.append({"kind": "acceptance", "expected_terms": [acceptance], "forbidden_terms": []})
    path = _first_match([r"(?:/home/[A-Za-z0-9_./-]+|[A-Za-z0-9_./-]+\.(?:rs|py|md|toml|yaml|json))"], text)
    if path:
        questions.append({"kind": "file", "expected_terms": [path], "forbidden_terms": []})
    error = _first_match([r"error\[[A-Z0-9]+\][^\n]{0,160}", r"Traceback[^\n]{0,160}", r"FAILED[^\n]{0,160}", r"blocked[^\n]{0,160}"], text)
    if error:
        questions.append({"kind": "error_or_blocker", "expected_terms": [error], "forbidden_terms": []})
    decision = _first_match([r"Decision[:\s][^\n]{8,180}", r"decided to [^\n]{8,180}", r"use [^\n]{8,120} not [^\n]{3,80}"], text)
    if decision:
        questions.append({"kind": "decision", "expected_terms": [decision], "forbidden_terms": []})
    # Fill with distinctive long-ish tokens if needed.
    if len(questions) < min_questions:
        candidates = re.findall(r"[A-Za-z][A-Za-z0-9_./:-]{12,80}", text)
        seen = {term for q in questions for term in q["expected_terms"]}
        for term in candidates:
            if term not in seen:
                questions.append({"kind": "anchor", "expected_terms": [term], "forbidden_terms": []})
                seen.add(term)
            if len(questions) >= min_questions:
                break
    return questions[:8]


def contains_all(text: str, terms: list[str]) -> bool:
    lower = text.lower()
    return all(term.lower() in lower for term in terms)


def run_context_governor(messages: list[dict[str, Any]], crate_dir: Path, target_tokens: int) -> dict[str, Any]:
    request = {
        "session_id": "historical-answerability",
        "messages": messages,
        "policy": {
            "target_tokens": target_tokens,
            "protect_first_n": 1,
            "protect_last_n": 1,
            "summary_max_chars": 2400,
            "allocator": "aggressive_v1",
            "semantic_memory_enabled": False,
            "archive_memory_enabled": False,
            "budget_mode": "hard_cascade",
            "token_counter": "provider_chat_approx",
        },
        "focus": None,
    }
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--", "compact"],
        cwd=crate_dir,
        input=json.dumps(request),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        return {"status": "error", "reason": proc.stderr[-500:] or proc.stdout[-500:]}
    response = json.loads(proc.stdout)
    text = render_messages(response.get("compacted_messages") or [])
    exact = "\n".join(item.get("content", "") for item in response.get("exact_store") or [])
    return {
        "status": "ok",
        "text": text,
        "recoverable_text": text + "\n" + exact,
        "tokens": response.get("receipt", {}).get("compacted_approx_tokens") or approx_tokens(text),
        "receipt_id": response.get("receipt", {}).get("receipt_id"),
        "warnings": response.get("receipt", {}).get("warnings") or [],
    }


def score_text(name: str, text: str, recoverable_text: str, questions: list[dict[str, Any]], tokens: int) -> dict[str, Any]:
    total = max(1, len(questions))
    visible = sum(1 for q in questions if contains_all(text, q["expected_terms"]))
    recoverable = sum(1 for q in questions if contains_all(recoverable_text, q["expected_terms"]))
    forbidden = sum(1 for q in questions for term in q.get("forbidden_terms", []) if term.lower() in text.lower())
    return {
        "name": name,
        "tokens": tokens,
        "visible_rate": visible / total,
        "answerability_rate": recoverable / total,
        "incorrect_action_risk": forbidden,
    }


def evaluate_messages(crate_dir: Path, messages: list[dict[str, Any]], questions: list[dict[str, Any]], target_tokens: int) -> dict[str, Any]:
    full_text = render_messages(messages)
    head_tail_messages = messages[:1] + messages[-1:]
    head_tail_text = render_messages(head_tail_messages)
    governed = run_context_governor(messages, crate_dir, target_tokens)
    baselines = [
        score_text("full", full_text, full_text, questions, sum(approx_tokens(str(m.get("content", ""))) + 4 for m in messages)),
        score_text("head_tail", head_tail_text, head_tail_text, questions, sum(approx_tokens(str(m.get("content", ""))) + 4 for m in head_tail_messages)),
    ]
    if governed["status"] == "ok":
        baselines.append(score_text("context_governor", governed["text"], governed["recoverable_text"], questions, int(governed["tokens"])))
    else:
        baselines.append({"name": "context_governor", "tokens": 0, "visible_rate": 0.0, "answerability_rate": 0.0, "incorrect_action_risk": 1, "error": governed.get("reason")})
    return {"baselines": baselines, "receipt_id": governed.get("receipt_id"), "warnings": governed.get("warnings") or []}


def evaluate_db(db_path: Path, crate_dir: Path, limit: int, min_messages: int, target_tokens: int) -> dict[str, Any]:
    started = time.time()
    sessions = load_sessions(db_path, limit=limit, min_messages=min_messages)
    runs = []
    for session in sessions:
        messages = load_messages(db_path, session.session_id)
        questions = generate_questions(messages)
        if not questions:
            continue
        scored = evaluate_messages(crate_dir, messages, questions, target_tokens)
        runs.append({
            "session_hash": hash_term(session.session_id),
            "title_hash": hash_term(session.title or session.session_id),
            "message_count": session.message_count,
            "char_count": session.char_count,
            "question_count": len(questions),
            "question_kinds": [q["kind"] for q in questions],
            "question_hashes": [hash_term("|".join(q["expected_terms"])) for q in questions],
            **scored,
        })
    return summarize_runs(runs, elapsed_ms=round((time.time() - started) * 1000, 2), db_path=db_path)


def summarize_runs(runs: list[dict[str, Any]], elapsed_ms: float, db_path: Path | None = None) -> dict[str, Any]:
    by_name: dict[str, list[dict[str, Any]]] = {}
    for run in runs:
        for item in run.get("baselines", []):
            by_name.setdefault(item["name"], []).append(item)
    aggregate = {}
    for name, rows in by_name.items():
        aggregate[name] = {
            "runs": len(rows),
            "avg_tokens": round(mean([r.get("tokens", 0) for r in rows]), 2) if rows else 0,
            "answerability_rate": round(mean([r.get("answerability_rate", 0) for r in rows]), 4) if rows else 0,
            "visible_rate": round(mean([r.get("visible_rate", 0) for r in rows]), 4) if rows else 0,
            "incorrect_action_risk": sum(r.get("incorrect_action_risk", 0) for r in rows),
        }
    full_tokens = aggregate.get("full", {}).get("avg_tokens") or 0
    gov_tokens = aggregate.get("context_governor", {}).get("avg_tokens") or 0
    return {
        "schema": "HermesHistoricalAnswerabilityReplayV1",
        "ok": bool(runs),
        "db_path": str(db_path) if db_path else None,
        "privacy_boundary": "Aggregate and hashed question/session IDs only; raw transcript text is not written to markdown.",
        "runs": len(runs),
        "elapsed_ms": elapsed_ms,
        "aggregate": aggregate,
        "token_reduction_vs_full": round(1 - (gov_tokens / full_tokens), 4) if full_tokens else 0,
        "redacted_runs": runs,
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Hermes historical answerability replay",
        "",
        f"Privacy boundary: {report['privacy_boundary']}",
        "",
        f"- Runs: `{report['runs']}`",
        f"- OK: `{report['ok']}`",
        f"- Token reduction vs full: {report.get('token_reduction_vs_full', 0):.1%}",
        "",
        "| strategy | runs | avg tokens | answerability | visible | incorrect-action risk |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for name, row in report.get("aggregate", {}).items():
        lines.append(
            f"| {name} | {row.get('runs',0)} | {row.get('avg_tokens',0)} | {row.get('answerability_rate',0):.1%} | {row.get('visible_rate',0):.1%} | {row.get('incorrect_action_risk',0)} |"
        )
    lines.extend(["", "## Question kind coverage", ""])
    counts: dict[str, int] = {}
    for run in report.get("redacted_runs", []):
        for kind in run.get("question_kinds", []):
            counts[kind] = counts.get(kind, 0) + 1
    for kind, count in sorted(counts.items()):
        lines.append(f"- {kind}: {count}")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default=str(Path.home() / ".hermes" / "state.db"))
    parser.add_argument("--crate-dir", default=str(ROOT))
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--min-messages", type=int, default=12)
    parser.add_argument("--target-tokens", type=int, default=1200)
    parser.add_argument("--out-dir", default=str(ROOT / "target" / "historical-answerability"))
    args = parser.parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    report = evaluate_db(Path(args.db), Path(args.crate_dir), args.limit, args.min_messages, args.target_tokens)
    json_path = out_dir / "historical-answerability.json"
    md_path = out_dir / "historical-answerability.md"
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    md_path.write_text(render_markdown(report))
    print(json.dumps({"ok": report["ok"], "json": str(json_path), "markdown": str(md_path), "runs": report["runs"]}, indent=2))
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Same-transcript context-engine comparison with explicit unsupported receipts.

The default fixtures are synthetic/public-safe. Markdown output intentionally
omits raw anchors and transcript text; JSON under target/ may include fixture
metadata for local debugging.
"""

from __future__ import annotations

import argparse
import json
import shutil
import statistics
import subprocess
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
EXTERNAL_ENGINES = ["hermes_builtin_compressor", "squeez", "ogham", "headroom", "llmlingua"]
CORE_ENGINES = ["full", "head_tail", "context_governor"]


def approx_tokens(text: str) -> int:
    return max(1, len(text) // 4)


def message_tokens(messages: list[dict[str, Any]]) -> int:
    return sum(approx_tokens(str(m.get("content", ""))) + 4 for m in messages)


def render_messages(messages: list[dict[str, Any]]) -> str:
    return "\n\n".join(f"[{m.get('role', 'assistant').upper()}] {m.get('content', '')}" for m in messages)


def fixture_families() -> list[dict[str, Any]]:
    return [
        {
            "fixture_id": "coding_log",
            "family": "coding log",
            "messages": [
                {"role": "system", "content": "You are a coding agent."},
                {"role": "user", "content": "Build parser. Acceptance gate: cargo test --all-targets must pass."},
                {"role": "assistant", "content": "Decision: use deterministic JSON parsing, not regex."},
                {"role": "tool", "content": ("bulk compile line\n" * 700) + "error[E0425]: cannot find value `parser` in /src/lib.rs\ntest result: FAILED\n"},
                {"role": "user", "content": "Latest task: fix the parser compile error and rerun cargo test."},
            ],
            "questions": [
                {"kind": "acceptance", "expected_terms": ["cargo test --all-targets must pass"], "forbidden_terms": []},
                {"kind": "decision", "expected_terms": ["deterministic JSON parsing"], "forbidden_terms": []},
                {"kind": "error", "expected_terms": ["E0425", "/src/lib.rs"], "forbidden_terms": []},
                {"kind": "active_task", "expected_terms": ["Latest task: fix the parser compile error"], "forbidden_terms": []},
            ],
        },
        {
            "fixture_id": "file_search_tool_output",
            "family": "file-search/tool-output",
            "messages": [
                {"role": "system", "content": "You are a repository assistant."},
                {"role": "user", "content": "Audit receipt search. Acceptance gate: context_expand recovers exact omitted text."},
                {"role": "tool", "content": ("search hit noise\n" * 600) + "/home/demo/src/store.rs:42 receipt_index_status must include store_bytes\n/home/demo/src/lib.rs:88 context_search fallback must scan exact_store\n"},
                {"role": "assistant", "content": "Decision: keep exact fallback even when receipt index misses."},
                {"role": "user", "content": "Latest task: add store status tests."},
            ],
            "questions": [
                {"kind": "acceptance", "expected_terms": ["context_expand recovers exact omitted text"], "forbidden_terms": []},
                {"kind": "file", "expected_terms": ["/home/demo/src/store.rs", "store_bytes"], "forbidden_terms": []},
                {"kind": "decision", "expected_terms": ["keep exact fallback"], "forbidden_terms": []},
                {"kind": "active_task", "expected_terms": ["Latest task: add store status tests"], "forbidden_terms": []},
            ],
        },
        {
            "fixture_id": "plan_acceptance_gates",
            "family": "plan+acceptance gates",
            "messages": [
                {"role": "system", "content": "You are executing a plan."},
                {"role": "user", "content": "Plan: Phase 1 safety, Phase 2 benchmark, Phase 3 historical replay."},
                {"role": "assistant", "content": "Blocked claim: do not say better than Squeez until identical-input receipts exist."},
                {"role": "tool", "content": "\n".join([f"plan filler line {i}" for i in range(900)])},
                {"role": "assistant", "content": "Acceptance command: python3 scripts/compare_context_engines_live.py --quick."},
                {"role": "user", "content": "Latest task: implement every remaining phase with receipts."},
            ],
            "questions": [
                {"kind": "plan", "expected_terms": ["Phase 2 benchmark", "Phase 3 historical replay"], "forbidden_terms": []},
                {"kind": "blocked_claim", "expected_terms": ["do not say better than Squeez"], "forbidden_terms": []},
                {"kind": "acceptance", "expected_terms": ["compare_context_engines_live.py --quick"], "forbidden_terms": []},
                {"kind": "active_task", "expected_terms": ["Latest task: implement every remaining phase"], "forbidden_terms": []},
            ],
        },
    ]


def contains_all(text: str, terms: list[str]) -> bool:
    lower = text.lower()
    return all(term.lower() in lower for term in terms)


def forbidden_count(text: str, terms: list[str]) -> int:
    lower = text.lower()
    return sum(1 for term in terms if term.lower() in lower)


def run_context_governor(messages: list[dict[str, Any]], target_tokens: int, crate_dir: Path) -> dict[str, Any]:
    request = {
        "session_id": "compare-live",
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
    started = time.perf_counter()
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--", "compact"],
        cwd=crate_dir,
        input=json.dumps(request),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000
    if proc.returncode != 0:
        return {"status": "error", "reason": proc.stderr[-600:] or proc.stdout[-600:], "elapsed_ms": elapsed_ms}
    response = json.loads(proc.stdout)
    text = render_messages(response.get("compacted_messages") or [])
    exact_text = "\n".join(item.get("content", "") for item in response.get("exact_store") or [])
    return {
        "status": "ok",
        "elapsed_ms": elapsed_ms,
        "text": text,
        "recoverable_text": text + "\n" + exact_text,
        "output_tokens": response.get("receipt", {}).get("compacted_approx_tokens") or approx_tokens(text),
        "warnings": response.get("receipt", {}).get("warnings") or [],
        "receipt_id": response.get("receipt", {}).get("receipt_id"),
    }


def run_core_engine(engine: str, fixture: dict[str, Any], target_tokens: int, crate_dir: Path) -> dict[str, Any]:
    messages = fixture["messages"]
    started = time.perf_counter()
    if engine == "full":
        text = render_messages(messages)
        elapsed_ms = (time.perf_counter() - started) * 1000
        return {"status": "ok", "elapsed_ms": elapsed_ms, "text": text, "recoverable_text": text, "output_tokens": message_tokens(messages), "warnings": []}
    if engine == "head_tail":
        head = messages[:1]
        tail = messages[-1:]
        text = render_messages(head + tail)
        elapsed_ms = (time.perf_counter() - started) * 1000
        return {"status": "ok", "elapsed_ms": elapsed_ms, "text": text, "recoverable_text": text, "output_tokens": message_tokens(head + tail), "warnings": []}
    if engine == "context_governor":
        return run_context_governor(messages, target_tokens, crate_dir)
    raise ValueError(engine)


def run_external_engine(engine: str) -> dict[str, Any]:
    executable = {
        "hermes_builtin_compressor": None,
        "squeez": "squeez",
        "ogham": "ogham",
        "headroom": "headroom",
        "llmlingua": "llmlingua",
    }[engine]
    if executable is None:
        return {"engine": engine, "status": "unsupported", "reason": "Hermes built-in compressor has no stable offline identical-input CLI adapter; benchmark it only in fresh Hermes process runs"}
    if shutil.which(executable) is None:
        return {"engine": engine, "status": "unsupported", "reason": f"{executable} executable not found on PATH"}
    return {"engine": engine, "status": "unsupported", "reason": "executable detected but no stable noninteractive identical-input adapter is implemented yet"}


def score_result(fixture: dict[str, Any], result: dict[str, Any]) -> dict[str, Any]:
    if result.get("status") != "ok":
        return {"answerability_rate": 0.0, "visible_anchor_rate": 0.0, "recoverable_anchor_rate": 0.0, "incorrect_action_risk": 1}
    text = result.get("text", "")
    recoverable_text = result.get("recoverable_text", text)
    questions = fixture["questions"]
    visible = sum(1 for q in questions if contains_all(text, q["expected_terms"]))
    recoverable = sum(1 for q in questions if contains_all(recoverable_text, q["expected_terms"]))
    answerable = recoverable
    risk = sum(forbidden_count(text, q.get("forbidden_terms", [])) for q in questions)
    total = max(1, len(questions))
    return {
        "answerability_rate": answerable / total,
        "visible_anchor_rate": visible / total,
        "recoverable_anchor_rate": recoverable / total,
        "incorrect_action_risk": risk,
    }


def percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    idx = min(len(ordered) - 1, int(round((pct / 100) * (len(ordered) - 1))))
    return ordered[idx]


def evaluate(target_tokens: int, crate_dir: Path, include_external: bool = True) -> dict[str, Any]:
    fixtures = fixture_families()
    engines = CORE_ENGINES[:]
    runs: list[dict[str, Any]] = []
    for fixture in fixtures:
        input_tokens = message_tokens(fixture["messages"])
        for engine in engines:
            result = run_core_engine(engine, fixture, target_tokens, crate_dir)
            scores = score_result(fixture, result)
            runs.append({
                "fixture_id": fixture["fixture_id"],
                "family": fixture["family"],
                "engine": engine,
                "status": result.get("status"),
                "reason": result.get("reason"),
                "input_tokens": input_tokens,
                "output_tokens": int(result.get("output_tokens") or 0),
                "token_reduction": 1 - (int(result.get("output_tokens") or 0) / input_tokens) if input_tokens else 0,
                "elapsed_ms": round(float(result.get("elapsed_ms") or 0), 3),
                "warnings": result.get("warnings") or [],
                "receipt_id": result.get("receipt_id"),
                **scores,
            })
    unsupported = [run_external_engine(engine) for engine in EXTERNAL_ENGINES] if include_external else []
    by_engine: dict[str, list[dict[str, Any]]] = {}
    for run in runs:
        by_engine.setdefault(run["engine"], []).append(run)
    aggregate = {}
    for engine, rows in by_engine.items():
        aggregate[engine] = {
            "runs": len(rows),
            "status": "ok" if all(r["status"] == "ok" for r in rows) else "partial",
            "p50_ms": round(statistics.median([r["elapsed_ms"] for r in rows]), 3),
            "p95_ms": round(percentile([r["elapsed_ms"] for r in rows], 95), 3),
            "avg_input_tokens": round(statistics.mean([r["input_tokens"] for r in rows]), 2),
            "avg_output_tokens": round(statistics.mean([r["output_tokens"] for r in rows]), 2),
            "avg_token_reduction": round(statistics.mean([r["token_reduction"] for r in rows]), 4),
            "visible_anchor_rate": round(statistics.mean([r["visible_anchor_rate"] for r in rows]), 4),
            "recoverable_anchor_rate": round(statistics.mean([r["recoverable_anchor_rate"] for r in rows]), 4),
            "answerability_rate": round(statistics.mean([r["answerability_rate"] for r in rows]), 4),
            "incorrect_action_risk": sum(r["incorrect_action_risk"] for r in rows),
            "safety_warnings": sum(len(r.get("warnings") or []) for r in rows),
        }
    for item in unsupported:
        aggregate[item["engine"]] = {"runs": 0, "status": item["status"], "reason": item["reason"]}
    return {
        "schema": "ContextGovernorSameTranscriptComparisonV1",
        "claim_boundary": "Synthetic identical-input comparison. Unsupported external adapters are explicit. This is not proof of universal superiority or live downstream LLM quality.",
        "target_tokens": target_tokens,
        "fixture_count": len(fixtures),
        "fixture_families": [f["family"] for f in fixtures],
        "aggregate": aggregate,
        "runs": runs,
        "unsupported": unsupported,
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# context-governor same-transcript comparison",
        "",
        f"Claim boundary: {report['claim_boundary']}",
        "",
        "Raw synthetic fixture text and anchor strings are intentionally omitted from this markdown.",
        "",
        "| engine | status | runs | p50 ms | p95 ms | avg input toks | avg output toks | reduction | visible anchors | recoverable anchors | answerability | incorrect risk | warnings / reason |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for engine, row in report["aggregate"].items():
        if row.get("status") == "unsupported":
            lines.append(f"| {engine} | unsupported | 0 | 0 | 0 | 0 | 0 | 0.0% | 0.0% | 0.0% | 0.0% | 0 | {row.get('reason','')} |")
            continue
        lines.append(
            "| {engine} | {status} | {runs} | {p50} | {p95} | {inp} | {out} | {red:.1%} | {vis:.1%} | {rec:.1%} | {ans:.1%} | {risk} | {warnings} |".format(
                engine=engine,
                status=row.get("status"),
                runs=row.get("runs", 0),
                p50=row.get("p50_ms", 0),
                p95=row.get("p95_ms", 0),
                inp=row.get("avg_input_tokens", 0),
                out=row.get("avg_output_tokens", 0),
                red=row.get("avg_token_reduction", 0),
                vis=row.get("visible_anchor_rate", 0),
                rec=row.get("recoverable_anchor_rate", 0),
                ans=row.get("answerability_rate", 0),
                risk=row.get("incorrect_action_risk", 0),
                warnings=row.get("safety_warnings", 0),
            )
        )
    lines.extend([
        "",
        "## Fixture families",
        "",
    ])
    for family in report["fixture_families"]:
        lines.append(f"- {family}")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate-dir", default=str(ROOT))
    parser.add_argument("--target-tokens", type=int, default=320)
    parser.add_argument("--out-dir", default=str(ROOT / "target" / "context-governor-comparisons"))
    parser.add_argument("--quick", action="store_true", help="Kept for certification symmetry; default fixtures are already quick")
    parser.add_argument("--no-external", action="store_true")
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    report = evaluate(args.target_tokens, Path(args.crate_dir), include_external=not args.no_external)
    json_path = out_dir / "same-transcript-comparison.json"
    md_path = out_dir / "same-transcript-comparison.md"
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    md_path.write_text(render_markdown(report))
    print(json.dumps({"ok": True, "json": str(json_path), "markdown": str(md_path)}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

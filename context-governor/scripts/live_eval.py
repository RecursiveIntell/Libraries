#!/usr/bin/env python3
"""Live-model answerability eval: measures whether compaction preserves task-critical
information by asking DeepSeek the same questions against full vs compacted context."""

import json
import os
import subprocess
import sys
import time
from pathlib import Path

FIXTURES = Path(__file__).parent.parent / "tests" / "fixtures"


def ensure_eval_fixtures():
    """Create evaluation fixtures with embedded questions and expected terms."""
    if not FIXTURES.exists():
        FIXTURES.mkdir(parents=True)
    # Only create if missing so user-defined fixtures take priority
    eval_file = FIXTURES / "eval_session.json"
    if eval_file.exists():
        return
    fixture = {
        "messages": [
            {"role": "system", "content": "You are a coding agent. Be precise."},
            {
                "role": "user",
                "content": (
                    "Build a JSON parser. Acceptance gate: cargo test must pass.\n"
                    "Implementation Plan:\n"
                    "1. Create src/parser.rs with a parse_json function\n"
                    "2. Add unit tests covering valid JSON, empty input, and malformed input\n"
                    "3. Run cargo test and fix any failures"
                ),
            },
            {
                "role": "assistant",
                "content": "Decision: use serde_json for parsing, not a hand-rolled parser.",
            },
            {
                "role": "tool",
                "content": (
                    "running 3 tests\n"
                    + ("build output line\n" * 50)
                    + "test test_valid_json ... ok\n"
                    + "test test_empty_input ... FAILED\n"
                    + "test test_malformed ... ok\n"
                    + "\nerror[E0425]: cannot find value `parse_json` in /src/lib.rs:10:5\n"
                    + "test result: FAILED. 2 passed; 1 failed"
                ),
            },
            {
                "role": "assistant",
                "content": (
                    "Fixed the empty input case. The parse_json function now returns "
                    "Result::Err for empty strings. File changed: /src/parser.rs"
                ),
            },
            {"role": "user", "content": "Latest task: write the README and commit."},
        ],
        "questions": [
            {
                "question": "What is the acceptance gate?",
                "expected_terms": ["cargo test", "must pass"],
            },
            {
                "question": "What parsing strategy was chosen?",
                "expected_terms": ["serde_json"],
                "forbidden_terms": ["hand-rolled", "regex"],
            },
            {
                "question": "Which test failed and what was the error?",
                "expected_terms": ["empty_input", "E0425", "parse_json", "/src/lib.rs"],
            },
            {
                "question": "What files were changed?",
                "expected_terms": ["/src/parser.rs"],
            },
        ],
    }
    eval_file.write_text(json.dumps(fixture, indent=2))


def run_compact(messages, target_tokens=200):
    """Run context-governor compact over the messages."""
    req = json.dumps(
        {
            "session_id": "live-eval",
            "messages": messages,
            "policy": {
                "target_tokens": target_tokens,
                "budget_mode": "hard_cascade",
                "protect_first_n": 3,
                "protect_last_n": 1,
                "summary_max_chars": 8000,
                "allocator": "deterministic_v1",
            },
        }
    )
    try:
        result = subprocess.run(
            ["context-governor", "compact"],
            input=req,
            capture_output=True,
            text=True,
            timeout=15,
        )
        return json.loads(result.stdout)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, FileNotFoundError) as e:
        print(f"ERROR running context-governor: {e}", file=sys.stderr)
        return None


def ask_deepseek(context_text, question, model="deepseek-chat"):
    """Ask DeepSeek a question given context text. Returns the answer string."""
    try:
        from openai import OpenAI
    except ImportError:
        print("ERROR: openai package not installed. Run: pip install openai", file=sys.stderr)
        return None

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        print("ERROR: OPENAI_API_KEY not set", file=sys.stderr)
        return None

    client = OpenAI(api_key=api_key, base_url="https://api.deepseek.com/v1")

    prompt = (
        f"Context:\n{context_text}\n\n"
        f"Question: {question}\n\n"
        f"Answer concisely in one sentence."
    )
    try:
        response = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            max_tokens=200,
            temperature=0.0,
        )
        return response.choices[0].message.content
    except Exception as e:
        print(f"  API error: {e}", file=sys.stderr)
        return None


def evaluate_answer(answer, expected_terms, forbidden_terms):
    """Score answer against expected and forbidden terms. Returns (hits, total, forbidden_hits)."""
    if answer is None:
        return 0, len(expected_terms), 0
    lower = answer.lower()
    hits = sum(1 for term in expected_terms if term.lower() in lower)
    forbidden = sum(1 for term in forbidden_terms if term.lower() in lower)
    return hits, len(expected_terms), forbidden


def context_text(messages, max_chars=3000):
    """Convert messages to a compact text representation under char budget."""
    parts = []
    total = 0
    for m in reversed(messages):  # newest first
        line = f"[{m['role']}] {m['content'][:500]}"
        total += len(line)
        if total > max_chars:
            parts.append("[... earlier context truncated ...]")
            break
        parts.append(line)
    return "\n".join(reversed(parts))


def main():
    ensure_eval_fixtures()

    results = []
    for fixture in sorted(FIXTURES.glob("*.json")):
        if fixture.name in ("error_session.json", "long_session.json"):
            continue

        try:
            data = json.loads(fixture.read_text())
        except json.JSONDecodeError:
            continue

        msgs = data.get("messages", [])
        questions = data.get("questions", [])
        if not msgs or not questions:
            continue

        full_ctx = context_text(msgs)

        # --- full-context baseline ---
        print(f"\n{'='*60}")
        print(f"Fixture: {fixture.name} ({len(msgs)} msgs, {len(questions)} Qs)")
        print(f"{'='*60}")
        print("--- Full context baseline ---")
        full_total = 0
        full_expected = 0
        for i, q in enumerate(questions):
            answer = ask_deepseek(full_ctx, q["question"])
            hits, exp, _ = evaluate_answer(answer, q.get("expected_terms", []), q.get("forbidden_terms", []))
            full_total += hits
            full_expected += exp
            status = "✓" if hits == exp else f"({hits}/{exp})"
            print(f"  Q{i+1}: {status} {q['question'][:60]}")
        full_score = round(100 * full_total / max(1, full_expected), 1)
        print(f"  Baseline: {full_total}/{full_expected} ({full_score}%)")

        # --- test compaction at multiple levels ---
        for targets in [400, 200, 100]:
            t0 = time.monotonic()
            cg = run_compact(msgs, target_tokens=targets)
            if cg is None:
                continue
            ct = (time.monotonic() - t0) * 1000
            cg_tok = cg["receipt"]["compacted_approx_tokens"]
            orig_tok = cg["receipt"]["original_approx_tokens"]
            red = round(100 * (1 - cg_tok / max(1, orig_tok)), 1) if orig_tok > 0 else 0

            compacted_msgs = [
                {"role": m["role"], "content": m["content"]}
                for m in cg.get("compacted_messages", [])
            ]
            compact_ctx = context_text(compacted_msgs)

            hits = 0
            expected = 0
            forbidden = 0
            for q in questions:
                answer = ask_deepseek(compact_ctx, q["question"])
                h, e, f = evaluate_answer(answer, q.get("expected_terms", []), q.get("forbidden_terms", []))
                hits += h
                expected += e
                forbidden += f

            score = round(100 * hits / max(1, expected), 1)
            delta = round(score - full_score, 1)
            print(f"  target={targets:>3} → {orig_tok}→{cg_tok} tok ({red}%), answer={score}% (Δ{delta:+}%), {ct:.0f}ms")

        results.append({
            "fixture": fixture.name,
            "baseline": full_score,
            "best_score": score,
        })

    if results:
        print(f"\n{'='*60}")
        print(f"{'FINAL':^60}")
        print(f"{'='*60}")
        for r in results:
            print(f"  {r['fixture']:<30} baseline={r['baseline']}%  best comp={r['best_score']}%")


if __name__ == "__main__":
    main()

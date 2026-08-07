#!/usr/bin/env python3
"""Cross-engine comparison: built-in Hermes compressor vs context-governor."""

import json
import subprocess
import sys
import time
from pathlib import Path

FIXTURES = Path(__file__).parent.parent / "tests" / "fixtures"


def ensure_fixtures():
    """Create sample fixtures if the directory is empty."""
    if not FIXTURES.exists():
        FIXTURES.mkdir(parents=True)
    if not list(FIXTURES.glob("*.json")):
        samples = {
            "chat_session.json": {
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant."},
                    {"role": "user", "content": "What is Rust?"},
                    {
                        "role": "assistant",
                        "content": "Rust is a systems programming language focused on safety, speed, and concurrency.",
                    },
                    {"role": "user", "content": "Show me an example."},
                    {
                        "role": "assistant",
                        "content": 'fn main() {\n    println!("Hello, world!");\n}',
                    },
                ]
            },
            "error_session.json": {
                "messages": [
                    {"role": "system", "content": "Build assistant."},
                    {
                        "role": "tool",
                        "content": "error[E0308]: mismatched types\n  --> src/main.rs:10:5\nwarning: unused variable\n\ntest result: FAILED. 2 passed; 1 failed",
                    },
                    {"role": "user", "content": "Fix the type error."},
                ]
            },
            "long_session.json": {
                "messages": [
                    {"role": "system", "content": "Coding assistant."},
                ]
                + [
                    {
                        "role": "assistant",
                        "content": f"Turn {i}: Generated {i * 100} lines of analysis output with detailed recommendations.",
                    }
                    for i in range(20)
                ]
                + [{"role": "user", "content": "Summarize the key findings."}],
            },
        }
        for name, data in samples.items():
            (FIXTURES / name).write_text(json.dumps(data, indent=2))


def run_context_governor(messages, target_tokens=200):
    """Run context-governor compact and return parsed response."""
    req = json.dumps(
        {
            "session_id": "cmp",
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
            timeout=10,
        )
        return json.loads(result.stdout)
    except FileNotFoundError:
        print("ERROR: context-governor binary not found on PATH", file=sys.stderr)
        sys.exit(1)
    except subprocess.TimeoutExpired:
        return {"error": "timeout"}
    except json.JSONDecodeError:
        return {"error": "invalid_json"}


def compare():
    """Run comparison across all fixtures and print results."""
    ensure_fixtures()
    results = []

    for fixture in sorted(FIXTURES.glob("*.json")):
        try:
            data = json.loads(fixture.read_text())
            msgs = data["messages"]
        except (json.JSONDecodeError, KeyError):
            print(f"WARNING: skipping invalid fixture {fixture.name}", file=sys.stderr)
            continue

        # Context-governor deterministic compaction
        t0 = time.monotonic()
        cg = run_context_governor(msgs)
        cg_time = (time.monotonic() - t0) * 1000  # ms

        if "error" not in cg:
            results.append(
                {
                    "fixture": fixture.name[:25],
                    "tokens_before": cg["receipt"]["original_approx_tokens"],
                    "tokens_after": cg["receipt"]["compacted_approx_tokens"],
                    "reduction_pct": round(
                        100
                        * (
                            1
                            - cg["receipt"]["compacted_approx_tokens"]
                            / max(1, cg["receipt"]["original_approx_tokens"])
                        ),
                        1,
                    ),
                    "latency_ms": round(cg_time, 1),
                }
            )

    # Print comparison table
    if not results:
        print("No valid results.", file=sys.stderr)
        sys.exit(1)

    header = f"{'Fixture':<28} {'Before':>8} {'After':>8} {'Reduction':>10} {'Latency':>10}"
    print(header)
    print("-" * len(header))
    for r in results:
        print(
            f"{r['fixture']:<28} {r['tokens_before']:>8} {r['tokens_after']:>8} "
            f"{r['reduction_pct']:>9}% {r['latency_ms']:>9}ms"
        )


if __name__ == "__main__":
    compare()

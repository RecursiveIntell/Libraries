#!/usr/bin/env python3
"""Small, reproducible forge-bench style scorer."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from urllib import error, request


OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
ANTHROPIC_URL = "https://api.anthropic.com/v1/messages"
EXECUTION_CASE_ID = "temporal_correctness"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--mode",
        choices=["fixture-asserted", "execution"],
        default="fixture-asserted",
    )
    parser.add_argument(
        "--casebook",
        type=Path,
        default=Path("contracts/fixtures/bench/forge_bench_casebook.json"),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("docs/benchmarks/score_sheet.json"),
    )
    parser.add_argument(
        "--model",
        default=os.environ.get("FORGE_BENCH_MODEL") or os.environ.get("OPENROUTER_MODEL") or "openai/gpt-4o-mini",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=60,
    )
    return parser.parse_args()


def score_case(case: dict[str, object]) -> tuple[float, float]:
    stack = case["stack"]["verdict"] == "pass"
    baseline = case["baseline"]["verdict"] == "pass"
    return (1.0 if stack else 0.0, 1.0 if baseline else 0.0)


def render_markdown(results: list[dict[str, object]]) -> None:
    header = "| case | mode | stack | baseline | delta |\n| --- | --- | --- | --- | --- |\n"
    print(header.strip())
    for item in results:
        print(
            f"| {item['case_id']} | {item['assessment_mode']} | {item['stack']} | {item['baseline']} | {item['advantage']:+} |"
        )


def build_fixture_results(cases: list[dict[str, object]]) -> list[dict[str, object]]:
    results = []
    for case in cases:
        stack, baseline = score_case(case)
        results.append(
            {
                "case_id": case["case_id"],
                "dimension": case["dimension"],
                "assessment_mode": "fixture-asserted",
                "stack": stack,
                "baseline": baseline,
                "advantage": stack - baseline,
                "stack_verdict": case["stack"]["verdict"],
                "baseline_verdict": case["baseline"]["verdict"],
            }
        )
    return results


def build_naive_summary(bundle: dict[str, object]) -> str:
    phases = [
        item["phase"].replace("_", " ")
        for item in bundle.get("chain", {}).get("effect_to_release", [])
    ]
    outputs = bundle.get("expected_outputs", {})
    lines = [
        f"Fixture name: {bundle.get('fixture_name')}",
        f"Wave: {bundle.get('wave')}",
        "Phases: " + " -> ".join(phases),
        "Expected outputs:",
    ]
    for key, value in outputs.items():
        lines.append(f"- {key}: {value}")
    lines.append(
        "This summary intentionally omits typed artifact bindings, field-level values, and cross-wave IDs."
    )
    return "\n".join(lines)


def build_execution_messages(question: str, context: str, signals: list[str]) -> list[dict[str, str]]:
    return [
        {
            "role": "system",
            "content": (
                "You evaluate artifact-chain evidence. "
                "Only mark a signal as supported if the provided context explicitly proves it. "
                "Return strict JSON with keys verdict, supported_signals, missing_signals, and reasoning."
            ),
        },
        {
            "role": "user",
            "content": (
                f"Question: {question}\n\n"
                "Required signal phrases:\n"
                + "\n".join(f"- {signal}" for signal in signals)
                + "\n\nContext:\n"
                + context
            ),
        },
    ]


def resolve_endpoint(model: str) -> tuple[str, str, str]:
    if os.environ.get("FORGE_BENCH_API_URL") and os.environ.get("FORGE_BENCH_API_KEY"):
        return (
            "openai-compatible",
            os.environ["FORGE_BENCH_API_URL"],
            os.environ["FORGE_BENCH_API_KEY"],
        )
    if os.environ.get("OPENROUTER_API_KEY"):
        return ("openrouter", OPENROUTER_URL, os.environ["OPENROUTER_API_KEY"])
    if os.environ.get("ANTHROPIC_API_KEY"):
        return ("anthropic", ANTHROPIC_URL, os.environ["ANTHROPIC_API_KEY"])
    raise SystemExit(
        "execution mode requires FORGE_BENCH_API_URL/FORGE_BENCH_API_KEY, OPENROUTER_API_KEY, or ANTHROPIC_API_KEY"
    )


def post_json(url: str, headers: dict[str, str], payload: dict[str, object], timeout_seconds: int) -> dict[str, object]:
    data = json.dumps(payload).encode("utf-8")
    req = request.Request(url, data=data, headers=headers, method="POST")
    try:
        with request.urlopen(req, timeout=timeout_seconds) as response:
            return json.loads(response.read().decode("utf-8"))
    except error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise SystemExit(f"execution request failed: {exc.code} {body}") from exc


def invoke_model(messages: list[dict[str, str]], model: str, timeout_seconds: int) -> tuple[str, str]:
    provider, url, api_key = resolve_endpoint(model)
    if provider == "anthropic":
        payload = {
            "model": model,
            "max_tokens": 512,
            "temperature": 0,
            "system": messages[0]["content"],
            "messages": [{"role": "user", "content": messages[1]["content"]}],
        }
        headers = {
            "content-type": "application/json",
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
        }
        response = post_json(url, headers, payload, timeout_seconds)
        content = response["content"][0]["text"]
        return provider, content

    payload = {
        "model": model,
        "messages": messages,
        "temperature": 0,
    }
    headers = {
        "content-type": "application/json",
        "authorization": f"Bearer {api_key}",
    }
    response = post_json(url, headers, payload, timeout_seconds)
    content = response["choices"][0]["message"]["content"]
    if isinstance(content, list):
        content = "".join(part.get("text", "") for part in content if isinstance(part, dict))
    return provider, content


def extract_json_object(raw: str) -> dict[str, object]:
    start = raw.find("{")
    end = raw.rfind("}")
    if start == -1 or end == -1 or end < start:
        return {}
    try:
        value = json.loads(raw[start : end + 1])
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def supported_signals(raw: str, expected_signals: list[str]) -> list[str]:
    payload = extract_json_object(raw)
    authored = payload.get("supported_signals")
    if isinstance(authored, list):
        supported = {str(item) for item in authored}
        return [signal for signal in expected_signals if signal in supported]
    lowered = raw.lower()
    return [signal for signal in expected_signals if signal.lower() in lowered]


def build_execution_result(
    case: dict[str, object],
    bundle: dict[str, object],
    model: str,
    timeout_seconds: int,
) -> dict[str, object]:
    question = case["question"]
    expected_signals = list(case["stack"]["signals"])
    stack_messages = build_execution_messages(
        question,
        json.dumps(bundle, indent=2),
        expected_signals,
    )
    baseline_messages = build_execution_messages(
        question,
        build_naive_summary(bundle),
        expected_signals,
    )

    provider, stack_response = invoke_model(stack_messages, model, timeout_seconds)
    _, baseline_response = invoke_model(baseline_messages, model, timeout_seconds)
    stack_supported = supported_signals(stack_response, expected_signals)
    baseline_supported = supported_signals(baseline_response, expected_signals)
    stack_pass = len(stack_supported) == len(expected_signals)
    baseline_pass = len(baseline_supported) == len(expected_signals)

    return {
        "case_id": f"{case['case_id']}_execution",
        "dimension": case["dimension"],
        "assessment_mode": "execution-verified",
        "stack": 1.0 if stack_pass else 0.0,
        "baseline": 1.0 if baseline_pass else 0.0,
        "advantage": (1.0 if stack_pass else 0.0) - (1.0 if baseline_pass else 0.0),
        "stack_verdict": "pass" if stack_pass else "fail",
        "baseline_verdict": "pass" if baseline_pass else "fail",
        "question": question,
        "provider": provider,
        "model": model,
        "expected_signals": expected_signals,
        "stack_supported_signals": stack_supported,
        "baseline_supported_signals": baseline_supported,
        "stack_response": stack_response,
        "baseline_response": baseline_response,
    }


def build_score_sheet(
    casebook: dict[str, object],
    results: list[dict[str, object]],
    mode: str,
) -> dict[str, object]:
    stack_total = sum(item["stack"] for item in results)
    baseline_total = sum(item["baseline"] for item in results)
    execution_verified = sum(
        1 for item in results if item.get("assessment_mode") == "execution-verified"
    )
    sheet = {
        "suite_name": casebook["suite_name"],
        "runner": "docs/benchmarks/run_forge_bench.py",
        "assessment_mode": "fixture-asserted" if mode == "fixture-asserted" else "mixed",
        "limitation": (
            "This score sheet reflects fixture-asserted verdicts. No live model comparison was executed. See run_forge_bench.py --mode execution for live scoring."
            if mode == "fixture-asserted"
            else "This score sheet contains the fixture-asserted suite plus one execution-verified temporal_correctness case."
        ),
        "inputs": casebook["inputs"],
        "case_count": len(results),
        "execution_verified_case_count": execution_verified,
        "stack_score": stack_total,
        "baseline_score": baseline_total,
        "advantage": stack_total - baseline_total,
        "results": results,
    }
    return sheet


def main() -> None:
    args = parse_args()
    casebook = json.loads(args.casebook.read_text(encoding="utf-8"))
    cases = casebook["casebook"]

    results = build_fixture_results(cases)
    if args.mode == "execution":
        demo_bundle = json.loads(
            Path(casebook["inputs"]["demo_bundle"]).read_text(encoding="utf-8")
        )
        execution_case = next(
            case for case in cases if case["case_id"] == EXECUTION_CASE_ID
        )
        results.append(
            build_execution_result(
                execution_case,
                demo_bundle,
                args.model,
                args.timeout_seconds,
            )
        )
    sheet = build_score_sheet(casebook, results, args.mode)

    args.output.write_text(json.dumps(sheet, indent=2), encoding="utf-8")
    print(f"wrote score sheet -> {args.output}")
    render_markdown(results)


if __name__ == "__main__":
    main()

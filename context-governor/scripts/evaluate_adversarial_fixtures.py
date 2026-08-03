#!/usr/bin/env python3
"""Evaluate adversarial context-governor fixtures."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from statistics import mean
from typing import Any


def parse_int_list(raw: str | None, default: int = 8000) -> list[int]:
    if not raw:
        return [default]
    return [int(part.strip()) for part in raw.split(",") if part.strip()]


def parse_str_list(raw: str | None, default: str = "soft_warn") -> list[str]:
    if not raw:
        return [default]
    return [part.strip() for part in raw.split(",") if part.strip()]


def _load_fixtures(fixture_dir: Path) -> list[dict[str, Any]]:
    paths = sorted(p for p in fixture_dir.glob("*.json") if p.name != "manifest.json")
    return [json.loads(path.read_text()) for path in paths]


def _request_from_fixture(fixture: dict[str, Any], target_tokens: int, budget_mode: str) -> dict[str, Any]:
    policy = dict(fixture.get("policy") or {})
    policy.update({"target_tokens": target_tokens, "budget_mode": budget_mode})
    return {
        "session_id": fixture.get("session_id") or fixture.get("fixture_id"),
        "messages": fixture.get("messages") or [],
        "policy": policy,
        "focus": fixture.get("focus"),
    }


def _run_context_governor(crate_dir: Path, request: dict[str, Any]) -> dict[str, Any]:
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--", "compact"],
        cwd=crate_dir,
        input=json.dumps(request, ensure_ascii=False),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    response = json.loads(proc.stdout)
    return _apply_host_latest_user_contract(response, request)


def _apply_host_latest_user_contract(response: dict[str, Any], request: dict[str, Any]) -> dict[str, Any]:
    """Mirror the Hermes plugin's latest-user-last safety guard for CLI evals."""
    original_users = [m for m in request.get("messages", []) if m.get("role") == "user"]
    if not original_users:
        return response
    latest = dict(original_users[-1])
    compacted = list(response.get("compacted_messages") or [])
    latest_content = str(latest.get("content") or "")
    compacted = [m for m in compacted if not (m.get("role") == "user" and str(m.get("content") or "") == latest_content)]
    compacted.append({"id": str(latest.get("id") or "latest_user"), "role": "user", "content": latest_content})
    response["compacted_messages"] = compacted
    return response


def _offline_baseline_response(fixture: dict[str, Any], request: dict[str, Any]) -> dict[str, Any]:
    messages = request["messages"]
    if len(messages) <= 2:
        compacted = list(messages)
    else:
        compacted = [messages[0], messages[-1]]
    full_tokens = _approx_tokens("\n".join(str(m.get("content") or "") for m in messages))
    compacted_tokens = _approx_tokens("\n".join(str(m.get("content") or "") for m in compacted))
    return {
        "fixture_id": fixture.get("fixture_id") or request.get("session_id"),
        "receipt": {
            "receipt_id": f"offline_{fixture.get('fixture_id')}",
            "original_approx_tokens": full_tokens,
            "compacted_approx_tokens": compacted_tokens,
            "warnings": [],
            "exact_fallback_refs": [],
            "semantic_memory_fact_ids": [],
        },
        "allocation_plan": {"items": [], "archived_item_ids": []},
        "compacted_messages": compacted,
        "exact_store": [],
        "baselines": [
            {"name": "full", "tokens": full_tokens, "visible_rate": 1.0, "recoverable_rate": 1.0, "total_probes": len(fixture.get("expected_probes") or []), "active_task_visible": True},
            {"name": "head_tail", "tokens": compacted_tokens, "visible_rate": 0.0, "recoverable_rate": 0.0, "total_probes": len(fixture.get("expected_probes") or []), "active_task_visible": True},
            {"name": "offline_baseline", "tokens": compacted_tokens, "visible_rate": 0.0, "recoverable_rate": 0.0, "total_probes": len(fixture.get("expected_probes") or []), "active_task_visible": True},
        ],
    }


def _approx_tokens(text: str) -> int:
    return max(1, (len(text) + 3) // 4)


def _joined_compacted(response: dict[str, Any]) -> str:
    return "\n".join(str(m.get("content") or "") for m in response.get("compacted_messages") or [])


def _joined_exact(response: dict[str, Any]) -> str:
    return "\n".join(str(item.get("content") or "") for item in response.get("exact_store") or [])


def _score_response(fixture: dict[str, Any], response: dict[str, Any], target_tokens: int, budget_mode: str, engine: str) -> dict[str, Any]:
    compacted_text = _joined_compacted(response)
    exact_text = _joined_exact(response)
    probes = []
    for probe in fixture.get("expected_probes") or []:
        text = str(probe.get("text") or "")
        visible = bool(text and text in compacted_text)
        recoverable = visible or bool(text and text in exact_text)
        probes.append({
            "id": probe.get("id"),
            "required": bool(probe.get("required", True)),
            "visible": visible,
            "recoverable": recoverable,
        })

    invariants = _score_invariants(fixture, response, compacted_text)
    receipt = response.get("receipt") or {}
    original_tokens = int(receipt.get("original_approx_tokens") or _approx_tokens("\n".join(str(m.get("content") or "") for m in fixture.get("messages") or [])))
    compacted_tokens = int(receipt.get("compacted_approx_tokens") or _approx_tokens(compacted_text))
    required = [p for p in probes if p["required"]]
    return {
        "fixture_id": fixture.get("fixture_id"),
        "engine": engine,
        "target_tokens": target_tokens,
        "budget_mode": budget_mode,
        "ok": all(p["recoverable"] for p in required) and all(v["passed"] for v in invariants.values()),
        "original_tokens": original_tokens,
        "compacted_tokens": compacted_tokens,
        "token_reduction": 1 - (compacted_tokens / original_tokens) if original_tokens else 0,
        "visible_probe_rate": mean([p["visible"] for p in probes]) if probes else 1.0,
        "recoverable_probe_rate": mean([p["recoverable"] for p in probes]) if probes else 1.0,
        "required_recoverable_rate": mean([p["recoverable"] for p in required]) if required else 1.0,
        "active_task_visible": bool(invariants.get("latest_user_final", {}).get("passed", True)),
        "warnings": receipt.get("warnings") or [],
        "probe_scores": probes,
        "invariants": invariants,
    }


def _score_invariants(fixture: dict[str, Any], response: dict[str, Any], compacted_text: str) -> dict[str, dict[str, Any]]:
    expected = fixture.get("expected_invariants") or {}
    out: dict[str, dict[str, Any]] = {}
    if expected.get("latest_user_final"):
        compacted = response.get("compacted_messages") or []
        original_users = [m for m in fixture.get("messages") or [] if m.get("role") == "user"]
        latest = str(original_users[-1].get("content") or "") if original_users else ""
        last_content = str((compacted[-1] if compacted else {}).get("content") or "")
        out["latest_user_final"] = {"passed": bool(latest and latest == last_content), "expected": latest[:120], "actual": last_content[:120]}
    forbidden = expected.get("forbidden_active_instruction")
    if forbidden:
        # It may be recoverable as exact tool output, but must not appear as an active user/system instruction.
        active_instruction = any(
            forbidden in str(m.get("content") or "") and m.get("role") in {"system", "user"}
            for m in response.get("compacted_messages") or []
        )
        out["forbidden_not_active"] = {"passed": not active_instruction, "forbidden": forbidden[:120]}
    if expected.get("expect_no_archive_candidates"):
        archived = (response.get("allocation_plan") or {}).get("archived_item_ids") or []
        out["no_archive_candidates"] = {"passed": len(archived) == 0, "archived_count": len(archived)}
    return out


def _aggregate(results: list[dict[str, Any]]) -> dict[str, Any]:
    successes = [r for r in results if r.get("ok")]
    return {
        "avg_full_tokens": mean([r["original_tokens"] for r in results]) if results else 0,
        "avg_compacted_tokens": mean([r["compacted_tokens"] for r in results]) if results else 0,
        "avg_token_reduction": mean([r["token_reduction"] for r in results]) if results else 0,
        "active_task_visible_rate": mean([r["active_task_visible"] for r in results]) if results else 0,
        "visible_probe_rate": mean([r["visible_probe_rate"] for r in results]) if results else 0,
        "recoverable_probe_rate": mean([r["recoverable_probe_rate"] for r in results]) if results else 0,
        "required_recoverable_rate": mean([r["required_recoverable_rate"] for r in results]) if results else 0,
        "warnings": sum(len(r.get("warnings") or []) for r in results),
        "successes": len(successes),
    }


def evaluate_fixture_dir(
    fixture_dir: Path,
    engine: str,
    target_tokens: list[int],
    budget_modes: list[str],
    crate_dir: Path,
    write_responses: Path | None = None,
) -> dict[str, Any]:
    fixtures = _load_fixtures(fixture_dir)
    results: list[dict[str, Any]] = []
    if write_responses:
        write_responses.mkdir(parents=True, exist_ok=True)
    for target in target_tokens:
        for mode in budget_modes:
            for fixture in fixtures:
                request = _request_from_fixture(fixture, target, mode)
                try:
                    if engine == "context_governor":
                        response = _run_context_governor(crate_dir, request)
                    elif engine in {"offline_baseline", "head_tail"}:
                        response = _offline_baseline_response(fixture, request)
                    else:
                        raise ValueError(f"unknown engine: {engine}")
                    scored = _score_response(fixture, response, target, mode, engine)
                    if write_responses:
                        stem = f"{fixture['fixture_id']}-{engine}-{mode}-{target}"
                        (write_responses / f"{stem}.request.json").write_text(json.dumps(request, indent=2, ensure_ascii=False))
                        (write_responses / f"{stem}.response.json").write_text(json.dumps(response, indent=2, ensure_ascii=False))
                    results.append(scored)
                except Exception as exc:
                    results.append({
                        "fixture_id": fixture.get("fixture_id"),
                        "engine": engine,
                        "target_tokens": target,
                        "budget_mode": mode,
                        "ok": False,
                        "error": str(exc),
                        "original_tokens": 0,
                        "compacted_tokens": 0,
                        "token_reduction": 0,
                        "visible_probe_rate": 0,
                        "recoverable_probe_rate": 0,
                        "required_recoverable_rate": 0,
                        "active_task_visible": False,
                        "warnings": [],
                        "probe_scores": [],
                        "invariants": {},
                    })
    return {
        "schema": "ContextGovernorAdversarialEvalReportV1",
        "engine": engine,
        "mode": ",".join(budget_modes),
        "target_tokens": target_tokens,
        "runs": len(results),
        "failures": sum(1 for r in results if not r.get("ok")),
        "aggregate": _aggregate(results),
        "results": results,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", required=True)
    parser.add_argument("--engine", choices=["context_governor", "offline_baseline", "head_tail"], required=True)
    parser.add_argument("--target-tokens", default="8000")
    parser.add_argument("--budget-modes", default="soft_warn")
    parser.add_argument("--crate-dir", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--out", required=True)
    parser.add_argument("--write-responses", default=None)
    args = parser.parse_args()
    report = evaluate_fixture_dir(
        fixture_dir=Path(args.fixtures),
        engine=args.engine,
        target_tokens=parse_int_list(args.target_tokens),
        budget_modes=parse_str_list(args.budget_modes),
        crate_dir=Path(args.crate_dir),
        write_responses=Path(args.write_responses) if args.write_responses else None,
    )
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    print(f"wrote {out}")
    print(f"runs={report['runs']} failures={report['failures']}")


if __name__ == "__main__":
    main()

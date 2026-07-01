#!/usr/bin/env python3
"""ProveKV PPL receipt validator/summarizer.

This is the lightweight active-repo companion to the archived heavy replay driver:
`/home/sikmindz/kv-lossless-11x/proveKV/scripts/ppl_validate.py`.

It does not run a model. It validates an existing state.json receipt, checks claim
gates, and emits a compact machine-readable summary that can be used by CI or
architecture reports.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


def _positive_number(value: Any, name: str) -> float:
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)) or float(value) <= 0:
        raise ValueError(f"{name} must be a positive finite number")
    return float(value)


def load_and_summarize(path: Path, max_abs_delta_pct: float) -> dict[str, Any]:
    state = json.loads(path.read_text())
    for key in ["model", "corpus", "phase0", "phase1"]:
        if key not in state:
            raise ValueError(f"missing required key: {key}")

    phase0 = state["phase0"]
    phase1 = state["phase1"]
    oracle_ppl = _positive_number(phase0.get("ppl"), "phase0.ppl")
    roundtrip_ppl = _positive_number(phase1.get("ppl"), "phase1.ppl")
    compression_ratio = phase1.get("compression_ratio")
    if compression_ratio is None:
        compression_ratio = phase1.get("manifest", {}).get("compression_ratio")
    compression_ratio = _positive_number(compression_ratio, "phase1.compression_ratio")
    if compression_ratio <= 1.0:
        raise ValueError("compression_ratio must be > 1.0")

    delta_ppl_pct = phase1.get("delta_ppl_pct")
    if delta_ppl_pct is None:
        delta_ppl_pct = ((roundtrip_ppl - oracle_ppl) / oracle_ppl) * 100.0
    if not isinstance(delta_ppl_pct, (int, float)) or not math.isfinite(float(delta_ppl_pct)):
        raise ValueError("phase1.delta_ppl_pct must be a finite number")
    delta_ppl_pct = float(delta_ppl_pct)

    passed = abs(delta_ppl_pct) <= max_abs_delta_pct
    blockers: list[str] = []
    if not passed:
        blockers.append(
            f"abs(delta_ppl_pct) {abs(delta_ppl_pct):.6f} exceeds threshold {max_abs_delta_pct:.6f}"
        )

    manifest = phase1.get("manifest", {})
    summary = {
        "schema_version": "provekv_ppl_receipt_summary_v1",
        "source": str(path),
        "model": state["model"],
        "corpus": state["corpus"],
        "n_tokens": state.get("n_tokens"),
        "oracle_ppl": oracle_ppl,
        "roundtrip_ppl": roundtrip_ppl,
        "delta_ppl_pct": delta_ppl_pct,
        "compression_ratio": compression_ratio,
        "pool_size_bytes": phase1.get("pool_size_bytes") or manifest.get("pool_size_bytes"),
        "total_compressed_bytes": phase1.get("total_compressed_bytes") or manifest.get("total_compressed_bytes"),
        "backend": manifest.get("backend"),
        "passed": passed,
        "blockers": blockers,
    }
    return summary


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("state_json", type=Path)
    parser.add_argument("--max-abs-delta-pct", type=float, default=0.1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    summary = load_and_summarize(args.state_json, args.max_abs_delta_pct)
    text = json.dumps(summary, indent=2, sort_keys=True)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text + "\n")
    print(text)
    return 0 if summary["passed"] else 2


if __name__ == "__main__":
    raise SystemExit(main())

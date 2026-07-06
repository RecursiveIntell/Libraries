#!/usr/bin/env python3
"""Held-out DistilGPT2 full-forward intervention suite for proveKV/poly-kv."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any

import numpy as np
from tokenizers import Tokenizer

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from capture_distilgpt2_replay import MODEL_ID, SNAPSHOT, load_weights, resolve_model, sha256_file  # noqa: E402
from distilgpt2_full_forward_intervention import evaluate_candidate, run_forward  # noqa: E402

SCHEMA = "poly_kv_distilgpt2_full_forward_suite_v1"

PROMPTS = [
    "RecursiveIntell agents need receipts before compression claims become trusted engineering evidence. "
    "The cache replay must compare final logits after intervention, not merely local vector similarity. ",
    "Local first inference systems can trade memory bandwidth for candidate selection when the proof boundary is explicit. "
    "A held out prompt suite should catch projection tricks and narrow overfitting. ",
    "Compressed key value caches are promising only when downstream model behavior survives sparse attention replay. "
    "The release gate should preserve claim boundaries and reject production claims without corpus evidence. ",
]


def token_ids_for_prompt(model_dir: Path, prompt: str, seq_len: int) -> list[int]:
    tokenizer = Tokenizer.from_file(str(model_dir / "tokenizer.json"))
    text = prompt * 6
    ids = tokenizer.encode(text).ids
    if len(ids) < seq_len + 1:
        raise RuntimeError(f"prompt produced {len(ids)} tokens, need {seq_len + 1}")
    return ids[: seq_len + 1]


def aggregate(cases: list[dict[str, Any]]) -> dict[str, Any]:
    selected = [case["selected"] for case in cases]
    pass_count = sum(1 for case in cases if case["passed"])
    def vals(key: str) -> list[float]:
        return [float(item[key]) for item in selected]
    return {
        "case_count": len(cases),
        "pass_count": pass_count,
        "pass_rate": pass_count / len(cases),
        "attention_output_cosine_mean": float(np.mean(vals("attention_output_cosine_mean"))),
        "attention_output_cosine_min": float(np.min(vals("attention_output_cosine_mean"))),
        "attention_output_mse_mean": float(np.mean(vals("attention_output_mse_mean"))),
        "final_logit_kl_mean": float(np.mean(vals("final_logit_kl_mean"))),
        "final_logit_kl_max": float(np.max(vals("final_logit_kl_mean"))),
        "final_top1_agreement_mean": float(np.mean(vals("final_top1_agreement"))),
        "final_top1_agreement_min": float(np.min(vals("final_top1_agreement"))),
        "abs_ppl_delta_mean": float(np.mean([abs(x) for x in vals("final_ppl_proxy_delta")])),
        "abs_ppl_delta_max": float(np.max([abs(x) for x in vals("final_ppl_proxy_delta")])),
        "decode_reduction_mean": float(np.mean(vals("decode_reduction"))),
        "decode_reduction_min": float(np.min(vals("decode_reduction"))),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", default=None)
    ap.add_argument("--model-dir", default=None)
    ap.add_argument("--seq-len", type=int, default=64)
    ap.add_argument("--layer", type=int, default=0)
    ap.add_argument("--heads", default="0,1")
    ap.add_argument("--candidate-ks", default="8,16,32,48,64")
    ap.add_argument("--min-attention-output-cosine", type=float, default=0.50)
    ap.add_argument("--max-attention-output-mse", type=float, default=0.10)
    ap.add_argument("--max-final-logit-kl", type=float, default=0.50)
    ap.add_argument("--max-abs-ppl-delta", type=float, default=25.0)
    ap.add_argument("--min-final-top1-agreement", type=float, default=0.50)
    args = ap.parse_args()

    model_dir = resolve_model(args.model_dir)
    weights = load_weights(model_dir)
    heads = [int(x) for x in args.heads.split(",") if x.strip()]
    candidate_ks = [int(x) for x in args.candidate_ks.split(",") if x.strip()]
    thresholds = {
        "min_attention_output_cosine": args.min_attention_output_cosine,
        "max_attention_output_mse": args.max_attention_output_mse,
        "max_final_logit_kl": args.max_final_logit_kl,
        "max_abs_ppl_delta": args.max_abs_ppl_delta,
        "min_final_top1_agreement": args.min_final_top1_agreement,
    }
    positions = sorted(set(p for p in [40, 48, 56, args.seq_len - 1] if p < args.seq_len))

    cases = []
    for prompt_idx, prompt in enumerate(PROMPTS):
        token_ids = token_ids_for_prompt(model_dir, prompt, args.seq_len)
        labels = token_ids[1 : args.seq_len + 1]
        for head in heads:
            exact = run_forward(weights, token_ids, args.layer, head, None)
            candidate_results = []
            for k in candidate_ks:
                compressed = run_forward(weights, token_ids, args.layer, head, k)
                candidate_results.append(evaluate_candidate(exact, compressed, labels, positions, thresholds, k))
            selected = next((r for r in candidate_results if r["passed"]), candidate_results[-1])
            cases.append({
                "prompt_index": prompt_idx,
                "prompt_sha256": "sha256:" + __import__("hashlib").sha256(prompt.encode()).hexdigest(),
                "layer": args.layer,
                "head": head,
                "selected_candidate_k": selected["candidate_k"],
                "passed": selected["passed"],
                "blockers": selected["blockers"],
                "selected": selected,
                "candidate_results": candidate_results,
            })
            print(f"case prompt={prompt_idx} head={head} selected={selected['candidate_k']} passed={selected['passed']}", file=sys.stderr, flush=True)

    agg = aggregate(cases)
    passed = agg["pass_count"] == agg["case_count"] and agg["decode_reduction_min"] > 1.0
    blockers = []
    if agg["pass_count"] != agg["case_count"]:
        blockers.append(f"{agg['case_count'] - agg['pass_count']} cases failed")
    if agg["decode_reduction_min"] <= 1.0:
        blockers.append(f"decode_reduction_min {agg['decode_reduction_min']:.4} <= 1.0")

    receipt = {
        "schema_version": SCHEMA,
        "model_id": f"distilgpt2-safetensors-full-forward-heldout-suite:{SNAPSHOT}:layer{args.layer}:heads{','.join(map(str, heads))}",
        "claim_boundary": "held-out DistilGPT2 full-forward intervention suite over fixed prompts and selected heads; not real-corpus PPL preservation, not production KV-cache preservation, not production latency evidence",
        "metadata": {
            "source_model": MODEL_ID,
            "model_snapshot": SNAPSHOT,
            "model_dir": str(model_dir),
            "model_safetensors_sha256": sha256_file(model_dir / "model.safetensors"),
            "seq_len": args.seq_len,
            "prompt_count": len(PROMPTS),
            "heads": heads,
            "query_positions": positions,
            "runtime": "numpy+safetensors+tokenizers manual DistilGPT2 held-out full-forward intervention suite",
        },
        "config": {"candidate_ks": candidate_ks, **thresholds},
        "aggregate": agg,
        "cases": cases,
        "passed": passed,
        "blockers": blockers,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, indent=2) + "\n")
    if args.summary:
        Path(args.summary).write_text(render_summary(receipt) + "\n")
    print(json.dumps({"out": str(out), "passed": passed, "aggregate": agg, "blockers": blockers}, indent=2))


def render_summary(receipt: dict[str, Any]) -> str:
    a = receipt["aggregate"]
    lines = [
        "# poly-kv DistilGPT2 held-out full-forward suite",
        "",
        "## Bottom line",
        "",
        f"Stored result: {'pass' if receipt['passed'] else 'fail/diagnostic'} across {a['case_count']} prompt/head cases.",
        "",
        "## Aggregate metrics",
        "",
    ]
    for key in [
        "pass_rate", "attention_output_cosine_mean", "attention_output_cosine_min", "attention_output_mse_mean",
        "final_logit_kl_mean", "final_logit_kl_max", "final_top1_agreement_mean", "final_top1_agreement_min",
        "abs_ppl_delta_mean", "abs_ppl_delta_max", "decode_reduction_mean", "decode_reduction_min",
    ]:
        lines.append(f"- {key}: {a[key]}")
    lines.extend(["", "## Claim boundary", "", receipt["claim_boundary"]])
    return "\n".join(lines)


if __name__ == "__main__":
    main()

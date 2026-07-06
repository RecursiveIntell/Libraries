#!/usr/bin/env python3
"""DistilGPT2 full-forward intervention replay for proveKV/poly-kv.

This script uses pretrained DistilGPT2 safetensors and a manual NumPy forward pass.
For one layer/head, it replaces exact head attention outputs with sparse outputs
built from compressed-candidate selection, then continues the downstream forward
path and compares final model logits.

It is still a local replay gate, not production KV-cache preservation evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path
from typing import Any

import numpy as np

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from capture_distilgpt2_replay import (  # noqa: E402
    MODEL_ID,
    SNAPSHOT,
    build_tokens,
    gelu_new,
    layer_norm,
    load_weights,
    resolve_model,
    sha256_file,
    softmax,
)

SCHEMA = "poly_kv_distilgpt2_full_forward_intervention_v1"


def topk_indices(values: np.ndarray, k: int) -> np.ndarray:
    k = min(k, values.shape[0])
    if k <= 0:
        return np.asarray([], dtype=np.int64)
    if k == values.shape[0]:
        return np.argsort(values)[::-1]
    part = np.argpartition(values, -k)[-k:]
    return part[np.argsort(values[part])[::-1]]


def quantized_scores(query: np.ndarray, keys: np.ndarray) -> np.ndarray:
    """Per-vector int8-ish quantized dot scores for candidate selection."""
    q_scale = max(float(np.max(np.abs(query))) / 127.0, 1e-8)
    q = np.clip(np.round(query / q_scale), -127, 127).astype(np.int16)
    out = []
    for key in keys:
        k_scale = max(float(np.max(np.abs(key))) / 127.0, 1e-8)
        kk = np.clip(np.round(key / k_scale), -127, 127).astype(np.int16)
        out.append(float(np.dot(q.astype(np.float64), kk.astype(np.float64)) * q_scale * k_scale))
    return np.asarray(out, dtype=np.float64)


def sparse_attention_output(query: np.ndarray, keys: np.ndarray, values: np.ndarray, candidate_k: int) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    approx = quantized_scores(query, keys) / math.sqrt(query.shape[-1])
    selected = topk_indices(approx, candidate_k)
    exact_scores = (keys[selected] @ query) / math.sqrt(query.shape[-1])
    weights = softmax(exact_scores, axis=-1)
    output = weights @ values[selected]
    return output, selected, approx


def kl_divergence(p_logits: np.ndarray, q_logits: np.ndarray) -> float:
    p = softmax(p_logits)
    q = softmax(q_logits)
    eps = 1e-12
    return float(np.sum(p * (np.log(p + eps) - np.log(q + eps))))


def nll(logits: np.ndarray, label: int) -> float:
    probs = softmax(logits)
    return float(-math.log(float(probs[label]) + 1e-12))


def cosine(a: np.ndarray, b: np.ndarray) -> float:
    denom = float(np.linalg.norm(a) * np.linalg.norm(b))
    if denom == 0.0:
        return 1.0 if float(np.linalg.norm(a - b)) == 0.0 else 0.0
    return float(np.dot(a, b) / denom)


def mse(a: np.ndarray, b: np.ndarray) -> float:
    return float(np.mean(np.square(a - b)))


def run_forward(
    t: dict[str, np.ndarray],
    token_ids: list[int],
    capture_layer: int,
    capture_head: int,
    candidate_k: int | None = None,
) -> dict[str, Any]:
    seq_len = len(token_ids) - 1
    input_ids = np.asarray(token_ids[:seq_len], dtype=np.int64)
    pos = np.arange(seq_len, dtype=np.int64)
    x = t["transformer.wte.weight"][input_ids] + t["transformer.wpe.weight"][pos]

    n_layers = 6
    n_heads = 12
    hidden = x.shape[-1]
    head_dim = hidden // n_heads
    interventions = []
    captured_head_outputs = None
    selected_total = 0
    full_decode_total = 0
    overlap_scores = []

    for layer in range(n_layers):
        residual = x
        x_ln = layer_norm(x, t[f"transformer.h.{layer}.ln_1.weight"], t[f"transformer.h.{layer}.ln_1.bias"])
        qkv = x_ln @ t[f"transformer.h.{layer}.attn.c_attn.weight"] + t[f"transformer.h.{layer}.attn.c_attn.bias"]
        q, k, v = np.split(qkv, 3, axis=-1)
        qh = q.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        kh = k.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        vh = v.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        scores = np.matmul(qh, np.swapaxes(kh, -1, -2)) / math.sqrt(head_dim)
        causal = np.tril(np.ones((seq_len, seq_len), dtype=bool))
        scores = np.where(causal[None, :, :], scores, -1.0e9)
        weights = softmax(scores, axis=-1)
        heads = np.matmul(weights, vh)

        if layer == capture_layer:
            if candidate_k is not None:
                for p in range(seq_len):
                    keys = kh[capture_head, : p + 1]
                    values = vh[capture_head, : p + 1]
                    sparse_out, selected, approx = sparse_attention_output(qh[capture_head, p], keys, values, candidate_k)
                    exact_top = set(topk_indices(scores[capture_head, p, : p + 1], min(candidate_k, p + 1)).tolist())
                    selected_set = set(selected.tolist())
                    union = exact_top | selected_set
                    overlap_scores.append(len(exact_top & selected_set) / len(union) if union else 1.0)
                    heads[capture_head, p] = sparse_out
                    selected_total += len(selected)
                    full_decode_total += p + 1
                interventions.append({"layer": layer, "head": capture_head, "candidate_k": candidate_k})
            captured_head_outputs = heads[capture_head].copy()

        merged = heads.transpose(1, 0, 2).reshape(seq_len, hidden)
        attn_out = merged @ t[f"transformer.h.{layer}.attn.c_proj.weight"] + t[f"transformer.h.{layer}.attn.c_proj.bias"]
        x = residual + attn_out

        residual = x
        x_ln = layer_norm(x, t[f"transformer.h.{layer}.ln_2.weight"], t[f"transformer.h.{layer}.ln_2.bias"])
        mlp = x_ln @ t[f"transformer.h.{layer}.mlp.c_fc.weight"] + t[f"transformer.h.{layer}.mlp.c_fc.bias"]
        mlp = gelu_new(mlp)
        mlp = mlp @ t[f"transformer.h.{layer}.mlp.c_proj.weight"] + t[f"transformer.h.{layer}.mlp.c_proj.bias"]
        x = residual + mlp

    x = layer_norm(x, t["transformer.ln_f.weight"], t["transformer.ln_f.bias"])
    logits = x @ t["transformer.wte.weight"].T
    return {
        "logits": logits,
        "exact_head_outputs": captured_head_outputs,
        "selected_total": selected_total,
        "full_decode_total": full_decode_total,
        "topk_overlap_mean": float(np.mean(overlap_scores)) if overlap_scores else 1.0,
        "interventions": interventions,
    }


def evaluate_candidate(
    exact: dict[str, Any],
    compressed: dict[str, Any],
    labels: list[int],
    positions: list[int],
    thresholds: dict[str, float],
    candidate_k: int,
) -> dict[str, Any]:
    cosines = []
    mses = []
    kls = []
    exact_nlls = []
    compressed_nlls = []
    top1_matches = 0
    for p in positions:
        cosines.append(cosine(exact["exact_head_outputs"][p], compressed["exact_head_outputs"][p]))
        mses.append(mse(exact["exact_head_outputs"][p], compressed["exact_head_outputs"][p]))
        kls.append(kl_divergence(exact["logits"][p], compressed["logits"][p]))
        exact_nlls.append(nll(exact["logits"][p], labels[p]))
        compressed_nlls.append(nll(compressed["logits"][p], labels[p]))
        if int(np.argmax(exact["logits"][p])) == int(np.argmax(compressed["logits"][p])):
            top1_matches += 1
    exact_nll = float(np.mean(exact_nlls))
    compressed_nll = float(np.mean(compressed_nlls))
    ppl_exact = math.exp(min(exact_nll, 20.0))
    ppl_compressed = math.exp(min(compressed_nll, 20.0))
    ppl_delta = ppl_compressed - ppl_exact
    result = {
        "candidate_k": candidate_k,
        "attention_output_cosine_mean": float(np.mean(cosines)),
        "attention_output_mse_mean": float(np.mean(mses)),
        "final_logit_kl_mean": float(np.mean(kls)),
        "final_top1_agreement": float(top1_matches / len(positions)),
        "final_ppl_proxy_exact": ppl_exact,
        "final_ppl_proxy_compressed": ppl_compressed,
        "final_ppl_proxy_delta": ppl_delta,
        "topk_overlap_mean": compressed["topk_overlap_mean"],
        "decoded_values_total": int(compressed["selected_total"]),
        "full_decode_value_count": int(compressed["full_decode_total"]),
        "decode_reduction": float(compressed["full_decode_total"] / max(1, compressed["selected_total"])),
    }
    blockers = []
    if result["attention_output_cosine_mean"] < thresholds["min_attention_output_cosine"]:
        blockers.append(f"attention_output_cosine_mean {result['attention_output_cosine_mean']:.4} < {thresholds['min_attention_output_cosine']:.4}")
    if result["attention_output_mse_mean"] > thresholds["max_attention_output_mse"]:
        blockers.append(f"attention_output_mse_mean {result['attention_output_mse_mean']:.4} > {thresholds['max_attention_output_mse']:.4}")
    if result["final_logit_kl_mean"] > thresholds["max_final_logit_kl"]:
        blockers.append(f"final_logit_kl_mean {result['final_logit_kl_mean']:.4} > {thresholds['max_final_logit_kl']:.4}")
    if abs(result["final_ppl_proxy_delta"]) > thresholds["max_abs_ppl_delta"]:
        blockers.append(f"abs(final_ppl_proxy_delta) {abs(result['final_ppl_proxy_delta']):.4} > {thresholds['max_abs_ppl_delta']:.4}")
    if result["final_top1_agreement"] < thresholds["min_final_top1_agreement"]:
        blockers.append(f"final_top1_agreement {result['final_top1_agreement']:.4} < {thresholds['min_final_top1_agreement']:.4}")
    result["passed"] = not blockers
    result["blockers"] = blockers
    return result


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", default=None)
    ap.add_argument("--model-dir", default=None)
    ap.add_argument("--seq-len", type=int, default=72)
    ap.add_argument("--capture-layer", type=int, default=0)
    ap.add_argument("--capture-head", type=int, default=0)
    ap.add_argument("--candidate-ks", default="8,16,32,48,64,72")
    ap.add_argument("--min-attention-output-cosine", type=float, default=0.50)
    ap.add_argument("--max-attention-output-mse", type=float, default=0.10)
    ap.add_argument("--max-final-logit-kl", type=float, default=0.50)
    ap.add_argument("--max-abs-ppl-delta", type=float, default=25.0)
    ap.add_argument("--min-final-top1-agreement", type=float, default=0.50)
    args = ap.parse_args()

    model_dir = resolve_model(args.model_dir)
    weights = load_weights(model_dir)
    token_ids, _ = build_tokens(model_dir, args.seq_len)
    labels = token_ids[1 : args.seq_len + 1]
    positions = [48, 56, 64, args.seq_len - 1]
    positions = sorted(set(p for p in positions if p < args.seq_len))
    candidate_ks = [int(x) for x in args.candidate_ks.split(",") if x.strip()]
    thresholds = {
        "min_attention_output_cosine": args.min_attention_output_cosine,
        "max_attention_output_mse": args.max_attention_output_mse,
        "max_final_logit_kl": args.max_final_logit_kl,
        "max_abs_ppl_delta": args.max_abs_ppl_delta,
        "min_final_top1_agreement": args.min_final_top1_agreement,
    }

    exact = run_forward(weights, token_ids, args.capture_layer, args.capture_head, None)
    results = []
    for k in candidate_ks:
        compressed = run_forward(weights, token_ids, args.capture_layer, args.capture_head, k)
        results.append(evaluate_candidate(exact, compressed, labels, positions, thresholds, k))

    selected = next((r for r in results if r["passed"]), results[-1])
    receipt = {
        "schema_version": SCHEMA,
        "model_id": f"distilgpt2-safetensors-full-forward-intervention:{SNAPSHOT}:layer{args.capture_layer}:head{args.capture_head}",
        "claim_boundary": "pretrained DistilGPT2 full-forward intervention replay; compressed candidate attention is reinjected into downstream manual forward path; not real corpus PPL preservation, not production KV-cache preservation, not production latency evidence",
        "metadata": {
            "source_model": MODEL_ID,
            "model_snapshot": SNAPSHOT,
            "model_dir": str(model_dir),
            "model_safetensors_sha256": sha256_file(model_dir / "model.safetensors"),
            "seq_len": args.seq_len,
            "capture_layer": args.capture_layer,
            "capture_head": args.capture_head,
            "query_positions": positions,
            "runtime": "numpy+safetensors+tokenizers manual DistilGPT2 full-forward intervention; torch/transformers not required",
            "candidate_selector": "per-vector int8 quantized key score, selected exact value decode, sparse softmax over selected candidates",
        },
        "config": {"candidate_ks": candidate_ks, **thresholds},
        "selected_candidate_k": selected["candidate_k"],
        "candidate_results": results,
        "metrics": {k: selected[k] for k in selected if k not in {"candidate_k", "passed", "blockers"}},
        "passed": selected["passed"],
        "blockers": selected["blockers"],
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, indent=2) + "\n")
    if args.summary:
        summary = Path(args.summary)
        summary.write_text(render_summary(receipt) + "\n")
    print(json.dumps({"out": str(out), "passed": receipt["passed"], "selected_candidate_k": receipt["selected_candidate_k"], "blockers": receipt["blockers"]}, indent=2))


def render_summary(receipt: dict[str, Any]) -> str:
    m = receipt["metrics"]
    lines = [
        "# poly-kv DistilGPT2 full-forward intervention receipt",
        "",
        "## Bottom line",
        "",
        f"Stored result: {'pass' if receipt['passed'] else 'fail/diagnostic'} at candidate_k={receipt['selected_candidate_k']}.",
        "This is stronger than the single-head projection receipt because the compressed attention output is reinjected and the remaining DistilGPT2 forward path is executed before comparing final logits.",
        "",
        "## Metrics",
        "",
    ]
    for key in [
        "attention_output_cosine_mean",
        "attention_output_mse_mean",
        "final_logit_kl_mean",
        "final_top1_agreement",
        "final_ppl_proxy_exact",
        "final_ppl_proxy_compressed",
        "final_ppl_proxy_delta",
        "topk_overlap_mean",
        "decoded_values_total",
        "full_decode_value_count",
        "decode_reduction",
    ]:
        lines.append(f"- {key}: {m[key]}")
    lines.extend(["", "## Blockers", ""])
    if receipt["blockers"]:
        lines.extend(f"- {b}" for b in receipt["blockers"])
    else:
        lines.append("- none")
    lines.extend([
        "",
        "## Claim boundary",
        "",
        receipt["claim_boundary"],
        "",
        "Safe: full-forward intervention replay receipt exists for pretrained DistilGPT2.",
        "Not safe: real corpus PPL preservation, production KV-cache preservation, production speedup, or replacement for KIVI/KVQuant/Quest.",
    ])
    return "\n".join(lines)


if __name__ == "__main__":
    main()

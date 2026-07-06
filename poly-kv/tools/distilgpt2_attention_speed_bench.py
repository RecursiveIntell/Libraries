#!/usr/bin/env python3
"""Isolated DistilGPT2 attention-operator speed benchmark for proveKV/poly-kv.

This intentionally does NOT claim production speedup. It isolates the target
operation: exact dense single-head attention over all prior KV rows vs the
current compressed sparse candidate path (per-vector int8 candidate scoring,
exact score over selected candidates, selected value decode). Setup/model forward
cost is excluded from timing.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import statistics
import sys
import time
from typing import Any

import numpy as np
from tokenizers import Tokenizer

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from capture_distilgpt2_replay import (  # noqa: E402
    MODEL_ID,
    SNAPSHOT,
    gelu_new,
    layer_norm,
    load_weights,
    resolve_model,
    sha256_file,
    softmax,
)
from distilgpt2_full_forward_intervention import (  # noqa: E402
    cosine,
    mse,
    sparse_attention_output,
    topk_indices,
)
from distilgpt2_full_forward_suite import PROMPTS  # noqa: E402

SCHEMA = "poly_kv_distilgpt2_attention_speed_bench_v1"


def token_ids_for_bench(model_dir: Path, prompt: str, seq_len: int) -> list[int]:
    tokenizer = Tokenizer.from_file(str(model_dir / "tokenizer.json"))
    text = prompt
    ids = tokenizer.encode(text).ids
    while len(ids) < seq_len + 1:
        text += prompt
        ids = tokenizer.encode(text).ids
    return ids[: seq_len + 1]


def extract_qkv_for_layer(t: dict[str, np.ndarray], token_ids: list[int], target_layer: int) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Run exact DistilGPT2 up to target layer and return q/k/v heads there."""
    seq_len = len(token_ids) - 1
    input_ids = np.asarray(token_ids[:seq_len], dtype=np.int64)
    pos = np.arange(seq_len, dtype=np.int64)
    x = t["transformer.wte.weight"][input_ids] + t["transformer.wpe.weight"][pos]

    n_layers = 6
    n_heads = 12
    hidden = x.shape[-1]
    head_dim = hidden // n_heads

    for layer in range(n_layers):
        residual = x
        x_ln = layer_norm(x, t[f"transformer.h.{layer}.ln_1.weight"], t[f"transformer.h.{layer}.ln_1.bias"])
        qkv = x_ln @ t[f"transformer.h.{layer}.attn.c_attn.weight"] + t[f"transformer.h.{layer}.attn.c_attn.bias"]
        q, k, v = np.split(qkv, 3, axis=-1)
        qh = q.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        kh = k.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        vh = v.reshape(seq_len, n_heads, head_dim).transpose(1, 0, 2)
        if layer == target_layer:
            return qh, kh, vh

        scores = np.matmul(qh, np.swapaxes(kh, -1, -2)) / math.sqrt(head_dim)
        causal = np.tril(np.ones((seq_len, seq_len), dtype=bool))
        scores = np.where(causal[None, :, :], scores, -1.0e9)
        weights = softmax(scores, axis=-1)
        heads = np.matmul(weights, vh)
        merged = heads.transpose(1, 0, 2).reshape(seq_len, hidden)
        attn_out = merged @ t[f"transformer.h.{layer}.attn.c_proj.weight"] + t[f"transformer.h.{layer}.attn.c_proj.bias"]
        x = residual + attn_out

        residual = x
        x_ln = layer_norm(x, t[f"transformer.h.{layer}.ln_2.weight"], t[f"transformer.h.{layer}.ln_2.bias"])
        mlp = x_ln @ t[f"transformer.h.{layer}.mlp.c_fc.weight"] + t[f"transformer.h.{layer}.mlp.c_fc.bias"]
        mlp = gelu_new(mlp)
        mlp = mlp @ t[f"transformer.h.{layer}.mlp.c_proj.weight"] + t[f"transformer.h.{layer}.mlp.c_proj.bias"]
        x = residual + mlp

    raise ValueError(f"target_layer {target_layer} outside DistilGPT2 layer range")


def exact_attention_outputs(q: np.ndarray, k: np.ndarray, v: np.ndarray, positions: list[int]) -> tuple[list[np.ndarray], int]:
    out = []
    decoded = 0
    scale = math.sqrt(q.shape[-1])
    for p in positions:
        keys = k[: p + 1]
        values = v[: p + 1]
        scores = (keys @ q[p]) / scale
        weights = softmax(scores, axis=-1)
        out.append(weights @ values)
        decoded += p + 1
    return out, decoded


def compressed_attention_outputs(q: np.ndarray, k: np.ndarray, v: np.ndarray, positions: list[int], candidate_k: int) -> tuple[list[np.ndarray], int, float]:
    out = []
    decoded = 0
    overlaps = []
    scale = math.sqrt(q.shape[-1])
    for p in positions:
        keys = k[: p + 1]
        values = v[: p + 1]
        sparse_out, selected, _approx = sparse_attention_output(q[p], keys, values, candidate_k)
        exact_scores = (keys @ q[p]) / scale
        exact_top = set(topk_indices(exact_scores, min(candidate_k, p + 1)).tolist())
        selected_set = set(selected.tolist())
        union = exact_top | selected_set
        overlaps.append(len(exact_top & selected_set) / len(union) if union else 1.0)
        out.append(sparse_out)
        decoded += len(selected)
    return out, decoded, float(np.mean(overlaps))


def prepare_quantized_keys(keys: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    scales = np.maximum(np.max(np.abs(keys), axis=1) / 127.0, 1e-8).astype(np.float32)
    codes = np.clip(np.round(keys / scales[:, None]), -127, 127).astype(np.int16)
    return codes, scales


def vectorized_compressed_attention_outputs(
    q: np.ndarray,
    k: np.ndarray,
    v: np.ndarray,
    positions: list[int],
    candidate_k: int,
) -> tuple[list[np.ndarray], int, float]:
    key_codes, key_scales = prepare_quantized_keys(k)
    out = []
    decoded = 0
    overlaps = []
    scale = math.sqrt(q.shape[-1])
    for p in positions:
        query = q[p]
        q_scale = max(float(np.max(np.abs(query))) / 127.0, 1e-8)
        q_code = np.clip(np.round(query / q_scale), -127, 127).astype(np.int16)
        approx = (key_codes[: p + 1].astype(np.float32) @ q_code.astype(np.float32))
        approx = (approx * key_scales[: p + 1] * q_scale) / scale
        selected = topk_indices(approx, candidate_k)
        exact_scores = (k[selected] @ query) / scale
        weights = softmax(exact_scores, axis=-1)
        out.append(weights @ v[selected])
        dense_scores = (k[: p + 1] @ query) / scale
        exact_top = set(topk_indices(dense_scores, min(candidate_k, p + 1)).tolist())
        selected_set = set(selected.tolist())
        union = exact_top | selected_set
        overlaps.append(len(exact_top & selected_set) / len(union) if union else 1.0)
        decoded += len(selected)
    return out, decoded, float(np.mean(overlaps))


def bench(fn, warmup: int, repeat: int) -> dict[str, float]:
    for _ in range(warmup):
        fn()
    samples = []
    for _ in range(repeat):
        start = time.perf_counter_ns()
        fn()
        samples.append(time.perf_counter_ns() - start)
    return {
        "mean_ns": float(statistics.mean(samples)),
        "median_ns": float(statistics.median(samples)),
        "min_ns": float(min(samples)),
        "max_ns": float(max(samples)),
        "stdev_ns": float(statistics.pstdev(samples)),
        "repeat": repeat,
    }


def case_metrics(q: np.ndarray, k: np.ndarray, v: np.ndarray, positions: list[int], candidate_k: int, warmup: int, repeat: int) -> dict[str, Any]:
    exact_out, exact_decoded = exact_attention_outputs(q, k, v, positions)
    compressed_out, compressed_decoded, overlap = compressed_attention_outputs(q, k, v, positions, candidate_k)
    vectorized_out, vectorized_decoded, vectorized_overlap = vectorized_compressed_attention_outputs(q, k, v, positions, candidate_k)
    cosines = [cosine(a, b) for a, b in zip(exact_out, compressed_out)]
    mses = [mse(a, b) for a, b in zip(exact_out, compressed_out)]
    vectorized_cosines = [cosine(a, b) for a, b in zip(exact_out, vectorized_out)]
    vectorized_mses = [mse(a, b) for a, b in zip(exact_out, vectorized_out)]

    exact_t = bench(lambda: exact_attention_outputs(q, k, v, positions), warmup, repeat)
    compressed_t = bench(lambda: compressed_attention_outputs(q, k, v, positions, candidate_k), warmup, repeat)
    vectorized_t = bench(lambda: vectorized_compressed_attention_outputs(q, k, v, positions, candidate_k), warmup, repeat)
    return {
        "positions": positions,
        "candidate_k": candidate_k,
        "attention_output_cosine_mean": float(np.mean(cosines)),
        "attention_output_cosine_min": float(np.min(cosines)),
        "attention_output_mse_mean": float(np.mean(mses)),
        "topk_overlap_mean": overlap,
        "vectorized_attention_output_cosine_mean": float(np.mean(vectorized_cosines)),
        "vectorized_attention_output_cosine_min": float(np.min(vectorized_cosines)),
        "vectorized_attention_output_mse_mean": float(np.mean(vectorized_mses)),
        "vectorized_topk_overlap_mean": vectorized_overlap,
        "exact_decoded_values": exact_decoded,
        "compressed_decoded_values": compressed_decoded,
        "vectorized_compressed_decoded_values": vectorized_decoded,
        "decode_reduction": float(exact_decoded / max(1, compressed_decoded)),
        "vectorized_decode_reduction": float(exact_decoded / max(1, vectorized_decoded)),
        "exact_timing": exact_t,
        "compressed_timing": compressed_t,
        "vectorized_compressed_timing": vectorized_t,
        "speed_ratio_exact_over_compressed": exact_t["mean_ns"] / compressed_t["mean_ns"],
        "speed_ratio_exact_over_vectorized_compressed": exact_t["mean_ns"] / vectorized_t["mean_ns"],
    }


def aggregate(cases: list[dict[str, Any]]) -> dict[str, Any]:
    def vals(key: str) -> list[float]:
        return [float(c[key]) for c in cases]
    return {
        "case_count": len(cases),
        "exact_attention_ns_mean": float(np.mean([c["exact_timing"]["mean_ns"] for c in cases])),
        "compressed_attention_ns_mean": float(np.mean([c["compressed_timing"]["mean_ns"] for c in cases])),
        "vectorized_compressed_attention_ns_mean": float(np.mean([c["vectorized_compressed_timing"]["mean_ns"] for c in cases])),
        "speed_ratio_exact_over_compressed": float(np.mean(vals("speed_ratio_exact_over_compressed"))),
        "speed_ratio_exact_over_vectorized_compressed": float(np.mean(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_compressed"))),
        "speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_compressed"))),
        "vectorized_speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "vectorized_speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "decode_reduction_mean": float(np.mean(vals("decode_reduction"))),
        "decode_reduction_min": float(np.min(vals("decode_reduction"))),
        "vectorized_decode_reduction_mean": float(np.mean(vals("vectorized_decode_reduction"))),
        "vectorized_decode_reduction_min": float(np.min(vals("vectorized_decode_reduction"))),
        "attention_output_cosine_mean": float(np.mean(vals("attention_output_cosine_mean"))),
        "attention_output_cosine_min": float(np.min(vals("attention_output_cosine_min"))),
        "vectorized_attention_output_cosine_mean": float(np.mean(vals("vectorized_attention_output_cosine_mean"))),
        "vectorized_attention_output_cosine_min": float(np.min(vals("vectorized_attention_output_cosine_min"))),
        "attention_output_mse_mean": float(np.mean(vals("attention_output_mse_mean"))),
        "vectorized_attention_output_mse_mean": float(np.mean(vals("vectorized_attention_output_mse_mean"))),
        "topk_overlap_mean": float(np.mean(vals("topk_overlap_mean"))),
        "vectorized_topk_overlap_mean": float(np.mean(vals("vectorized_topk_overlap_mean"))),
    }


def render_summary(receipt: dict[str, Any]) -> str:
    a = receipt["aggregate"]
    lines = [
        "# poly-kv DistilGPT2 isolated attention speed bench",
        "",
        "## Bottom line",
        "",
        f"Stored result: scalar speed_ratio_exact_over_compressed={a['speed_ratio_exact_over_compressed']:.4f}; vectorized speed_ratio={a['speed_ratio_exact_over_vectorized_compressed']:.4f}; decode_reduction_mean={a['decode_reduction_mean']:.4f}x.",
        "",
        "## Aggregate metrics",
        "",
    ]
    for key in [
        "case_count", "exact_attention_ns_mean", "compressed_attention_ns_mean", "vectorized_compressed_attention_ns_mean",
        "speed_ratio_exact_over_compressed", "speed_ratio_exact_over_vectorized_compressed",
        "speed_ratio_min", "speed_ratio_max", "vectorized_speed_ratio_min", "vectorized_speed_ratio_max",
        "decode_reduction_mean", "decode_reduction_min", "vectorized_decode_reduction_mean", "vectorized_decode_reduction_min",
        "attention_output_cosine_mean", "attention_output_cosine_min", "vectorized_attention_output_cosine_mean",
        "vectorized_attention_output_cosine_min", "attention_output_mse_mean", "vectorized_attention_output_mse_mean",
        "topk_overlap_mean", "vectorized_topk_overlap_mean",
    ]:
        lines.append(f"- {key}: {a[key]}")
    lines.extend(["", "## Claim boundary", "", receipt["claim_boundary"]])
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", default=None)
    ap.add_argument("--model-dir", default=None)
    ap.add_argument("--seq-len", type=int, default=64)
    ap.add_argument("--layers", default="0")
    ap.add_argument("--heads", default="0,1,2,3,4,5,6,7,8,9,10,11")
    ap.add_argument("--candidate-k", type=int, default=8)
    ap.add_argument("--warmup", type=int, default=10)
    ap.add_argument("--repeat", type=int, default=100)
    ap.add_argument("--prompt-count", type=int, default=3)
    args = ap.parse_args()

    model_dir = resolve_model(args.model_dir)
    weights = load_weights(model_dir)
    layers = [int(x) for x in args.layers.split(",") if x.strip()]
    heads = [int(x) for x in args.heads.split(",") if x.strip()]
    positions = sorted(set(p for p in [40, 48, 56, args.seq_len - 1] if p < args.seq_len))

    cases = []
    for prompt_idx, prompt in enumerate(PROMPTS[: args.prompt_count]):
        token_ids = token_ids_for_bench(model_dir, prompt, args.seq_len)
        for layer in layers:
            qh, kh, vh = extract_qkv_for_layer(weights, token_ids, layer)
            for head in heads:
                metrics = case_metrics(qh[head], kh[head], vh[head], positions, args.candidate_k, args.warmup, args.repeat)
                metrics.update({"prompt_index": prompt_idx, "layer": layer, "head": head})
                cases.append(metrics)
                print(
                    f"case prompt={prompt_idx} layer={layer} head={head} "
                    f"scalar_speed={metrics['speed_ratio_exact_over_compressed']:.4f} "
                    f"vectorized_speed={metrics['speed_ratio_exact_over_vectorized_compressed']:.4f} "
                    f"decode={metrics['decode_reduction']:.4f}",
                    file=sys.stderr,
                    flush=True,
                )

    agg = aggregate(cases)
    blockers = []
    if agg["decode_reduction_min"] <= 1.0:
        blockers.append(f"decode_reduction_min {agg['decode_reduction_min']:.4f} <= 1.0")
    receipt = {
        "schema_version": SCHEMA,
        "model_id": f"distilgpt2-isolated-attention-speed:{SNAPSHOT}:layers{','.join(map(str,layers))}:heads{','.join(map(str,heads))}",
        "claim_boundary": "isolated NumPy attention-operator benchmark over precomputed DistilGPT2 Q/K/V tensors; setup/full-forward cost excluded; not production runtime speedup, not GPU kernel evidence, not end-to-end generation latency evidence",
        "metadata": {
            "source_model": MODEL_ID,
            "model_snapshot": SNAPSHOT,
            "model_dir": str(model_dir),
            "model_safetensors_sha256": sha256_file(model_dir / "model.safetensors"),
            "seq_len": args.seq_len,
            "layers": layers,
            "heads": heads,
            "prompt_count": args.prompt_count,
            "query_positions": positions,
            "runtime": "NumPy CPU isolated attention-operator timing with setup excluded",
        },
        "config": {"candidate_k": args.candidate_k, "warmup": args.warmup, "repeat": args.repeat},
        "aggregate": agg,
        "cases": cases,
        "passed": not blockers,
        "blockers": blockers,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, indent=2) + "\n")
    if args.summary:
        Path(args.summary).write_text(render_summary(receipt) + "\n")
    print(json.dumps({"out": str(out), "passed": receipt["passed"], "aggregate": agg, "blockers": blockers}, indent=2))


if __name__ == "__main__":
    main()

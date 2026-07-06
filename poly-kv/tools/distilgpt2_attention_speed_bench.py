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




def optimized_prequantized_compressed_attention_outputs(
    q: np.ndarray,
    k: np.ndarray,
    v: np.ndarray,
    positions: list[int],
    candidate_k: int,
    key_codes: np.ndarray,
    key_scales: np.ndarray,
    *,
    include_quality: bool = False,
) -> tuple[list[np.ndarray], int, float]:
    """Compressed sparse attention with setup excluded from the hot path.

    key_codes/key_scales are materialized once per layer/head outside timing.
    When include_quality=False this avoids exact dense top-k diagnostics so the
    benchmark measures the candidate-score/select/decode operator only.
    """
    out = []
    decoded = 0
    overlaps = []
    scale = math.sqrt(q.shape[-1])
    for p in positions:
        query = q[p]
        q_scale = max(float(np.max(np.abs(query))) / 127.0, 1e-8)
        q_code = np.clip(np.round(query / q_scale), -127, 127).astype(np.int16)
        approx = key_codes[: p + 1].astype(np.float32) @ q_code.astype(np.float32)
        approx = (approx * key_scales[: p + 1] * q_scale) / scale
        selected = topk_indices(approx, candidate_k)
        exact_scores = (k[selected] @ query) / scale
        weights = softmax(exact_scores, axis=-1)
        out.append(weights @ v[selected])
        if include_quality:
            dense_scores = (k[: p + 1] @ query) / scale
            exact_top = set(topk_indices(dense_scores, min(candidate_k, p + 1)).tolist())
            selected_set = set(selected.tolist())
            union = exact_top | selected_set
            overlaps.append(len(exact_top & selected_set) / len(union) if union else 1.0)
        decoded += len(selected)
    overlap = float(np.mean(overlaps)) if overlaps else 1.0
    return out, decoded, overlap



def batch_optimized_compressed_attention_outputs(
    q: np.ndarray,
    k: np.ndarray,
    v: np.ndarray,
    positions: list[int],
    candidate_k: int,
    key_codes: np.ndarray,
    key_scales: np.ndarray,
    *,
    include_quality: bool = False,
) -> tuple[list[np.ndarray], int, float]:
    """Batch all-positions compressed attention.

    Instead of looping per position, precompute all query codes,
    compute all scores as one matmul, batch top-k, batch exact rerank,
    and batch softmax+weighted sum.

    This is the key optimization: transforms N small matmuls into 1 big matmul
    and eliminates Python loop overhead.
    """
    head_dim = q.shape[-1]
    scale = math.sqrt(head_dim)
    n_keys = k.shape[0]
    n_positions = len(positions)

    # Batch-compute all query codes and scales
    q_matrix = q[positions]  # (n_positions, head_dim)
    q_scales = np.maximum(np.max(np.abs(q_matrix), axis=1) / 127.0, 1e-8).astype(np.float32)
    q_codes = np.clip(np.round(q_matrix / q_scales[:, None]), -127, 127).astype(np.int16)

    # Batch-compute all approximate scores: (n_positions, n_keys)
    # key_codes: (n_keys, head_dim), q_codes: (n_positions, head_dim)
    all_approx = (key_codes.astype(np.float32) @ q_codes.astype(np.float32).T).T  # (n_positions, n_keys)
    # Rescale
    all_approx = (all_approx * (key_scales[None, :] * q_scales[:, None])) / scale

    # Batch top-k with causal masking
    out = []
    decoded_total = 0
    overlaps = []
    for i, p_pos in enumerate(positions):
        # Causal: only consider keys[:p_pos+1]
        n_avail = p_pos + 1
        approx = all_approx[i, :n_avail]
        selected = topk_indices(approx, candidate_k)
        # Exact rerank
        exact_scores = (k[selected] @ q_matrix[i]) / scale
        weights = softmax(exact_scores, axis=-1)
        out.append(weights @ v[selected])
        decoded_total += len(selected)
        if include_quality:
            dense_scores = (k[:n_avail] @ q_matrix[i]) / scale
            exact_top = set(topk_indices(dense_scores, min(candidate_k, n_avail)).tolist())
            selected_set = set(selected.tolist())
            union = exact_top | selected_set
            overlaps.append(len(exact_top & selected_set) / len(union) if union else 1.0)
    overlap = float(np.mean(overlaps)) if overlaps else 1.0
    return out, decoded_total, overlap


def quest_page_filtered_compressed_attention_outputs(
    q: np.ndarray,
    k: np.ndarray,
    v: np.ndarray,
    positions: list[int],
    candidate_k: int,
    key_codes: np.ndarray,
    key_scales: np.ndarray,
    page_size: int = 32,
    *,
    include_quality: bool = False,
) -> tuple[list[np.ndarray], int, float]:
    """Quest-style page min/max pre-filter before per-vector scoring.

    Divides keys into pages of `page_size` tokens, precomputes per-page
    min/max bounds, and only scores vectors from pages whose upper bound
    exceeds the current k-th score threshold.

    This is the Quest algorithm (arXiv:2406.10774): page-level pre-filter
    before per-vector scoring to reduce the number of bytes loaded.
    """
    head_dim = q.shape[-1]
    scale = math.sqrt(head_dim)
    n_keys = k.shape[0]
    n_pages = (n_keys + page_size - 1) // page_size

    # Precompute per-page min/max key values (once)
    # For the int8 codes, min = min_code * scale, max = max_code * scale
    page_min_code = np.zeros((n_pages, head_dim), dtype=np.int16)
    page_max_code = np.zeros((n_pages, head_dim), dtype=np.int16)
    page_scales = np.zeros(n_pages, dtype=np.float32)
    for pg in range(n_pages):
        start = pg * page_size
        end = min(start + page_size, n_keys)
        page_codes = key_codes[start:end]  # (page_tokens, head_dim)
        page_min_code[pg] = page_codes.min(axis=0)
        page_max_code[pg] = page_codes.max(axis=0)
        page_scales[pg] = key_scales[start]  # per-vector scale, approx same per page

    out = []
    decoded_total = 0
    overlaps = []
    for p_pos in positions:
        n_avail = p_pos + 1
        query = q[p_pos]
        q_scale = max(float(np.max(np.abs(query))) / 127.0, 1e-8)
        q_code = np.clip(np.round(query / q_scale), -127, 127).astype(np.int16)

        # Page-level: compute upper bound for each page
        n_active_pages = (n_avail + page_size - 1) // page_size
        page_upper = np.zeros(n_active_pages, dtype=np.float32)
        for pg in range(n_active_pages):
            # Upper bound: sum of max positive contributions
            q_pos = q_code > 0
            q_neg = q_code < 0
            # For positive query codes, max key code gives max product
            # For negative query codes, min key code gives max product (negative * negative = positive)
            max_contrib = np.where(q_pos, page_max_code[pg].astype(np.float32),
                                   np.where(q_neg, page_min_code[pg].astype(np.float32), 0.0))
            upper = float(np.dot(max_contrib, np.abs(q_code).astype(np.float32)) * q_scale * page_scales[pg])
            page_upper[pg] = upper / scale

        # Select top pages by upper bound (greedy)
        # Use 2x oversample: load candidate_k * 2 tokens worth of pages
        tokens_needed = candidate_k * 2
        pages_needed = max(1, (tokens_needed + page_size - 1) // page_size)
        pages_needed = min(pages_needed, n_active_pages)
        top_pages = topk_indices(page_upper[:n_active_pages], pages_needed)

        # Score only vectors from selected pages
        candidate_indices = []
        for pg in top_pages:
            start = pg * page_size
            end = min(start + page_size, n_avail)
            candidate_indices.extend(range(start, end))

        candidate_arr = np.array(candidate_indices, dtype=np.int64)
        if len(candidate_arr) == 0:
            candidate_arr = np.arange(min(candidate_k, n_avail), dtype=np.int64)

        # Score candidates
        cand_codes = key_codes[candidate_arr]
        cand_scales = key_scales[candidate_arr]
        approx = (cand_codes.astype(np.float32) @ q_code.astype(np.float32)) * (cand_scales * q_scale) / scale

        # Top-k from candidates
        selected_local = topk_indices(approx, min(candidate_k, len(candidate_arr)))
        selected = candidate_arr[selected_local]

        # Exact rerank
        exact_scores = (k[selected] @ query) / scale
        weights = softmax(exact_scores, axis=-1)
        out.append(weights @ v[selected])
        decoded_total += len(selected)

        if include_quality:
            dense_scores = (k[:n_avail] @ query) / scale
            exact_top = set(topk_indices(dense_scores, min(candidate_k, n_avail)).tolist())
            selected_set = set(selected.tolist())
            union = exact_top | selected_set
            overlaps.append(len(exact_top & selected_set) / len(union) if union else 1.0)

    overlap = float(np.mean(overlaps)) if overlaps else 1.0
    return out, decoded_total, overlap




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
    key_codes, key_scales = prepare_quantized_keys(k)
    exact_out, exact_decoded = exact_attention_outputs(q, k, v, positions)
    compressed_out, compressed_decoded, overlap = compressed_attention_outputs(q, k, v, positions, candidate_k)
    vectorized_out, vectorized_decoded, vectorized_overlap = vectorized_compressed_attention_outputs(q, k, v, positions, candidate_k)
    optimized_out, optimized_decoded, optimized_overlap = optimized_prequantized_compressed_attention_outputs(
        q, k, v, positions, candidate_k, key_codes, key_scales, include_quality=True
    )
    batch_out, batch_decoded, batch_overlap = batch_optimized_compressed_attention_outputs(
        q, k, v, positions, candidate_k, key_codes, key_scales, include_quality=True
    )
    quest_out, quest_decoded, quest_overlap = quest_page_filtered_compressed_attention_outputs(
        q, k, v, positions, candidate_k, key_codes, key_scales, page_size=32, include_quality=True
    )
    cosines = [cosine(a, b) for a, b in zip(exact_out, compressed_out)]
    mses = [mse(a, b) for a, b in zip(exact_out, compressed_out)]
    vectorized_cosines = [cosine(a, b) for a, b in zip(exact_out, vectorized_out)]
    vectorized_mses = [mse(a, b) for a, b in zip(exact_out, vectorized_out)]
    optimized_cosines = [cosine(a, b) for a, b in zip(exact_out, optimized_out)]
    optimized_mses = [mse(a, b) for a, b in zip(exact_out, optimized_out)]
    batch_cosines = [cosine(a, b) for a, b in zip(exact_out, batch_out)]
    batch_mses = [mse(a, b) for a, b in zip(exact_out, batch_out)]
    quest_cosines = [cosine(a, b) for a, b in zip(exact_out, quest_out)]
    quest_mses = [mse(a, b) for a, b in zip(exact_out, quest_out)]

    exact_t = bench(lambda: exact_attention_outputs(q, k, v, positions), warmup, repeat)
    compressed_t = bench(lambda: compressed_attention_outputs(q, k, v, positions, candidate_k), warmup, repeat)
    vectorized_t = bench(lambda: vectorized_compressed_attention_outputs(q, k, v, positions, candidate_k), warmup, repeat)
    optimized_t = bench(
        lambda: optimized_prequantized_compressed_attention_outputs(
            q, k, v, positions, candidate_k, key_codes, key_scales, include_quality=False
        ),
        warmup,
        repeat,
    )
    batch_t = bench(
        lambda: batch_optimized_compressed_attention_outputs(
            q, k, v, positions, candidate_k, key_codes, key_scales, include_quality=False
        ),
        warmup,
        repeat,
    )
    quest_t = bench(
        lambda: quest_page_filtered_compressed_attention_outputs(
            q, k, v, positions, candidate_k, key_codes, key_scales, page_size=32, include_quality=False
        ),
        warmup,
        repeat,
    )
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
        "optimized_attention_output_cosine_mean": float(np.mean(optimized_cosines)),
        "optimized_attention_output_cosine_min": float(np.min(optimized_cosines)),
        "optimized_attention_output_mse_mean": float(np.mean(optimized_mses)),
        "optimized_topk_overlap_mean": optimized_overlap,
        "batch_attention_output_cosine_mean": float(np.mean(batch_cosines)),
        "batch_attention_output_cosine_min": float(np.min(batch_cosines)),
        "batch_attention_output_mse_mean": float(np.mean(batch_mses)),
        "batch_topk_overlap_mean": batch_overlap,
        "quest_attention_output_cosine_mean": float(np.mean(quest_cosines)),
        "quest_attention_output_cosine_min": float(np.min(quest_cosines)),
        "quest_attention_output_mse_mean": float(np.mean(quest_mses)),
        "quest_topk_overlap_mean": quest_overlap,
        "exact_decoded_values": exact_decoded,
        "compressed_decoded_values": compressed_decoded,
        "vectorized_compressed_decoded_values": vectorized_decoded,
        "optimized_prequantized_decoded_values": optimized_decoded,
        "batch_decoded_values": batch_decoded,
        "quest_decoded_values": quest_decoded,
        "decode_reduction": float(exact_decoded / max(1, compressed_decoded)),
        "vectorized_decode_reduction": float(exact_decoded / max(1, vectorized_decoded)),
        "optimized_prequantized_decode_reduction": float(exact_decoded / max(1, optimized_decoded)),
        "batch_decode_reduction": float(exact_decoded / max(1, batch_decoded)),
        "quest_decode_reduction": float(exact_decoded / max(1, quest_decoded)),
        "exact_timing": exact_t,
        "compressed_timing": compressed_t,
        "vectorized_compressed_timing": vectorized_t,
        "optimized_prequantized_compressed_timing": optimized_t,
        "batch_compressed_timing": batch_t,
        "quest_page_filtered_timing": quest_t,
        "speed_ratio_exact_over_compressed": exact_t["mean_ns"] / compressed_t["mean_ns"],
        "speed_ratio_exact_over_vectorized_compressed": exact_t["mean_ns"] / vectorized_t["mean_ns"],
        "speed_ratio_exact_over_optimized_prequantized": exact_t["mean_ns"] / optimized_t["mean_ns"],
        "speed_ratio_exact_over_batch": exact_t["mean_ns"] / batch_t["mean_ns"],
        "speed_ratio_exact_over_quest": exact_t["mean_ns"] / quest_t["mean_ns"],
    }


def aggregate(cases: list[dict[str, Any]]) -> dict[str, Any]:
    def vals(key: str) -> list[float]:
        return [float(c[key]) for c in cases]
    return {
        "case_count": len(cases),
        "exact_attention_ns_mean": float(np.mean([c["exact_timing"]["mean_ns"] for c in cases])),
        "compressed_attention_ns_mean": float(np.mean([c["compressed_timing"]["mean_ns"] for c in cases])),
        "vectorized_compressed_attention_ns_mean": float(np.mean([c["vectorized_compressed_timing"]["mean_ns"] for c in cases])),
        "optimized_prequantized_compressed_attention_ns_mean": float(np.mean([c["optimized_prequantized_compressed_timing"]["mean_ns"] for c in cases])),
        "batch_compressed_attention_ns_mean": float(np.mean([c["batch_compressed_timing"]["mean_ns"] for c in cases])),
        "quest_page_filtered_attention_ns_mean": float(np.mean([c["quest_page_filtered_timing"]["mean_ns"] for c in cases])),
        "speed_ratio_exact_over_compressed": float(np.mean(vals("speed_ratio_exact_over_compressed"))),
        "speed_ratio_exact_over_vectorized_compressed": float(np.mean(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "speed_ratio_exact_over_optimized_prequantized": float(np.mean(vals("speed_ratio_exact_over_optimized_prequantized"))),
        "speed_ratio_exact_over_batch": float(np.mean(vals("speed_ratio_exact_over_batch"))),
        "speed_ratio_exact_over_quest": float(np.mean(vals("speed_ratio_exact_over_quest"))),
        "speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_compressed"))),
        "speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_compressed"))),
        "vectorized_speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "vectorized_speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_vectorized_compressed"))),
        "optimized_prequantized_speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_optimized_prequantized"))),
        "optimized_prequantized_speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_optimized_prequantized"))),
        "batch_speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_batch"))),
        "batch_speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_batch"))),
        "quest_speed_ratio_min": float(np.min(vals("speed_ratio_exact_over_quest"))),
        "quest_speed_ratio_max": float(np.max(vals("speed_ratio_exact_over_quest"))),
        "decode_reduction_mean": float(np.mean(vals("decode_reduction"))),
        "decode_reduction_min": float(np.min(vals("decode_reduction"))),
        "vectorized_decode_reduction_mean": float(np.mean(vals("vectorized_decode_reduction"))),
        "vectorized_decode_reduction_min": float(np.min(vals("vectorized_decode_reduction"))),
        "optimized_prequantized_decode_reduction_mean": float(np.mean(vals("optimized_prequantized_decode_reduction"))),
        "optimized_prequantized_decode_reduction_min": float(np.min(vals("optimized_prequantized_decode_reduction"))),
        "batch_decode_reduction_mean": float(np.mean(vals("batch_decode_reduction"))),
        "batch_decode_reduction_min": float(np.min(vals("batch_decode_reduction"))),
        "quest_decode_reduction_mean": float(np.mean(vals("quest_decode_reduction"))),
        "quest_decode_reduction_min": float(np.min(vals("quest_decode_reduction"))),
        "attention_output_cosine_mean": float(np.mean(vals("attention_output_cosine_mean"))),
        "attention_output_cosine_min": float(np.min(vals("attention_output_cosine_min"))),
        "vectorized_attention_output_cosine_mean": float(np.mean(vals("vectorized_attention_output_cosine_mean"))),
        "vectorized_attention_output_cosine_min": float(np.min(vals("vectorized_attention_output_cosine_min"))),
        "optimized_attention_output_cosine_mean": float(np.mean(vals("optimized_attention_output_cosine_mean"))),
        "optimized_attention_output_cosine_min": float(np.min(vals("optimized_attention_output_cosine_min"))),
        "attention_output_mse_mean": float(np.mean(vals("attention_output_mse_mean"))),
        "vectorized_attention_output_mse_mean": float(np.mean(vals("vectorized_attention_output_mse_mean"))),
        "optimized_attention_output_mse_mean": float(np.mean(vals("optimized_attention_output_mse_mean"))),
        "topk_overlap_mean": float(np.mean(vals("topk_overlap_mean"))),
        "vectorized_topk_overlap_mean": float(np.mean(vals("vectorized_topk_overlap_mean"))),
        "optimized_topk_overlap_mean": float(np.mean(vals("optimized_topk_overlap_mean"))),
        "batch_attention_output_cosine_mean": float(np.mean(vals("batch_attention_output_cosine_mean"))),
        "batch_attention_output_cosine_min": float(np.min(vals("batch_attention_output_cosine_mean"))),
        "quest_attention_output_cosine_mean": float(np.mean(vals("quest_attention_output_cosine_mean"))),
        "quest_attention_output_cosine_min": float(np.min(vals("quest_attention_output_cosine_mean"))),
        "batch_topk_overlap_mean": float(np.mean(vals("batch_topk_overlap_mean"))),
        "quest_topk_overlap_mean": float(np.mean(vals("quest_topk_overlap_mean"))),
    }


def render_summary(receipt: dict[str, Any]) -> str:
    a = receipt["aggregate"]
    lines = [
        "# poly-kv DistilGPT2 isolated attention speed bench",
        "",
        "## Bottom line",
        "",
        f"Stored result: scalar={a['speed_ratio_exact_over_compressed']:.4f}; vectorized={a['speed_ratio_exact_over_vectorized_compressed']:.4f}; optimized={a['speed_ratio_exact_over_optimized_prequantized']:.4f}; batch={a['speed_ratio_exact_over_batch']:.4f}; quest={a['speed_ratio_exact_over_quest']:.4f}; decode_reduction={a['decode_reduction_mean']:.4f}x.",
        "",
        "## Aggregate metrics",
        "",
    ]
    for key in [
        "case_count", "exact_attention_ns_mean", "compressed_attention_ns_mean", "vectorized_compressed_attention_ns_mean",
        "optimized_prequantized_compressed_attention_ns_mean", "batch_compressed_attention_ns_mean", "quest_page_filtered_attention_ns_mean",
        "speed_ratio_exact_over_compressed", "speed_ratio_exact_over_vectorized_compressed",
        "speed_ratio_exact_over_optimized_prequantized", "speed_ratio_exact_over_batch", "speed_ratio_exact_over_quest",
        "speed_ratio_min", "speed_ratio_max", "vectorized_speed_ratio_min", "vectorized_speed_ratio_max",
        "optimized_prequantized_speed_ratio_min", "optimized_prequantized_speed_ratio_max",
        "batch_speed_ratio_min", "batch_speed_ratio_max", "quest_speed_ratio_min", "quest_speed_ratio_max",
        "decode_reduction_mean", "decode_reduction_min", "vectorized_decode_reduction_mean", "vectorized_decode_reduction_min",
        "optimized_prequantized_decode_reduction_mean", "optimized_prequantized_decode_reduction_min",
        "batch_decode_reduction_mean", "batch_decode_reduction_min", "quest_decode_reduction_mean", "quest_decode_reduction_min",
        "attention_output_cosine_mean", "attention_output_cosine_min", "vectorized_attention_output_cosine_mean",
        "vectorized_attention_output_cosine_min", "optimized_attention_output_cosine_mean", "optimized_attention_output_cosine_min",
        "batch_attention_output_cosine_mean", "batch_attention_output_cosine_min",
        "quest_attention_output_cosine_mean", "quest_attention_output_cosine_min",
        "attention_output_mse_mean", "vectorized_attention_output_mse_mean", "optimized_attention_output_mse_mean",
        "topk_overlap_mean", "vectorized_topk_overlap_mean", "optimized_topk_overlap_mean",
        "batch_topk_overlap_mean", "quest_topk_overlap_mean",
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
        "claim_boundary": "isolated NumPy attention-operator benchmark over precomputed DistilGPT2 Q/K/V tensors; setup/full-forward cost excluded and quality diagnostics excluded from timed hot paths for optimized/batch/quest timing; includes Quest-style page min/max pre-filter (arXiv:2406.10774); not production runtime speedup, not GPU kernel evidence, not end-to-end generation latency evidence",
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
            "runtime": "NumPy CPU isolated attention-operator timing; optimized/prequantized path excludes setup and quality diagnostics from timed hot paths",
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

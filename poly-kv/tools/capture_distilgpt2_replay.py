#!/usr/bin/env python3
"""Capture a pretrained DistilGPT2 Q/K/V/logit replay fixture without torch.

This script intentionally avoids torch/transformers because the local torch install
attempt hit ENOSPC. It uses cached/downloaded HuggingFace safetensors + tokenizer,
then runs a deterministic NumPy DistilGPT2 forward pass.

The emitted fixture is stronger than the NumPy toy transformer fixture because the
weights/tokenizer are from pretrained distilgpt2, but it is still a single-layer,
single-head replay proxy: poly-kv compresses one captured attention head and uses a
model-derived single-head logit projection. It is not full-model KV-cache PPL.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
from typing import Any

import numpy as np
from safetensors import safe_open
from tokenizers import Tokenizer

try:
    from huggingface_hub import snapshot_download
except Exception:  # pragma: no cover - dependency failure is reported in main
    snapshot_download = None


MODEL_ID = "distilgpt2"
SNAPSHOT = "2290a62682d06624634c1f46a6ad5be0f47f38aa"
SCHEMA = "poly_kv_captured_replay_fixture_v1"


def gelu_new(x: np.ndarray) -> np.ndarray:
    return 0.5 * x * (1.0 + np.tanh(math.sqrt(2.0 / math.pi) * (x + 0.044715 * np.power(x, 3))))


def layer_norm(x: np.ndarray, weight: np.ndarray, bias: np.ndarray, eps: float = 1e-5) -> np.ndarray:
    mean = x.mean(axis=-1, keepdims=True)
    var = np.mean(np.square(x - mean), axis=-1, keepdims=True)
    return (x - mean) / np.sqrt(var + eps) * weight + bias


def softmax(x: np.ndarray, axis: int = -1) -> np.ndarray:
    shifted = x - np.max(x, axis=axis, keepdims=True)
    ex = np.exp(shifted)
    return ex / np.sum(ex, axis=axis, keepdims=True)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return "sha256:" + h.hexdigest()


def resolve_model(cache_dir: str | None) -> Path:
    if cache_dir:
        return Path(cache_dir)
    cached = Path.home() / ".cache/huggingface/hub/models--distilgpt2/snapshots" / SNAPSHOT
    if (cached / "model.safetensors").exists():
        return cached
    if snapshot_download is None:
        raise RuntimeError("huggingface_hub unavailable and distilgpt2 is not cached")
    return Path(
        snapshot_download(
            MODEL_ID,
            allow_patterns=[
                "config.json",
                "tokenizer.json",
                "vocab.json",
                "merges.txt",
                "model.safetensors",
                "generation_config.json",
            ],
            max_workers=1,
        )
    )


def load_weights(model_dir: Path) -> dict[str, np.ndarray]:
    tensors: dict[str, np.ndarray] = {}
    with safe_open(model_dir / "model.safetensors", framework="np") as f:
        for key in f.keys():
            # float64 improves reproducibility of the manual forward; output is rounded later.
            tensors[key] = f.get_tensor(key).astype(np.float64)
    return tensors


def build_tokens(model_dir: Path, seq_len: int) -> tuple[list[int], str]:
    tokenizer = Tokenizer.from_file(str(model_dir / "tokenizer.json"))
    text = (
        "RecursiveIntell proof systems evaluate compressed key value caches with receipts. "
        "The replay gate compares exact attention logits against compressed candidate selection. "
        "This prompt is deliberately repeated so the captured context has enough prior tokens. "
    )
    text = text * 8
    ids = tokenizer.encode(text).ids
    if len(ids) < seq_len + 1:
        raise RuntimeError(f"tokenizer produced only {len(ids)} tokens, need {seq_len + 1}")
    return ids[: seq_len + 1], text


def gpt2_forward(t: dict[str, np.ndarray], token_ids: list[int], capture_layer: int, capture_head: int) -> dict[str, Any]:
    seq_len = len(token_ids) - 1
    input_ids = np.asarray(token_ids[:seq_len], dtype=np.int64)
    pos = np.arange(seq_len, dtype=np.int64)
    x = t["transformer.wte.weight"][input_ids] + t["transformer.wpe.weight"][pos]

    n_layers = 6
    n_heads = 12
    hidden = x.shape[-1]
    head_dim = hidden // n_heads
    captured: dict[str, np.ndarray] | None = None

    for layer in range(n_layers):
        residual = x
        x_ln = layer_norm(
            x,
            t[f"transformer.h.{layer}.ln_1.weight"],
            t[f"transformer.h.{layer}.ln_1.bias"],
        )
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
            captured = {
                "q": qh[capture_head].copy(),
                "k": kh[capture_head].copy(),
                "v": vh[capture_head].copy(),
                "attn_out": heads[capture_head].copy(),
            }
        merged = heads.transpose(1, 0, 2).reshape(seq_len, hidden)
        attn_out = merged @ t[f"transformer.h.{layer}.attn.c_proj.weight"] + t[f"transformer.h.{layer}.attn.c_proj.bias"]
        x = residual + attn_out

        residual = x
        x_ln = layer_norm(
            x,
            t[f"transformer.h.{layer}.ln_2.weight"],
            t[f"transformer.h.{layer}.ln_2.bias"],
        )
        mlp = x_ln @ t[f"transformer.h.{layer}.mlp.c_fc.weight"] + t[f"transformer.h.{layer}.mlp.c_fc.bias"]
        mlp = gelu_new(mlp)
        mlp = mlp @ t[f"transformer.h.{layer}.mlp.c_proj.weight"] + t[f"transformer.h.{layer}.mlp.c_proj.bias"]
        x = residual + mlp

    if captured is None:
        raise RuntimeError("capture layer not reached")
    x = layer_norm(x, t["transformer.ln_f.weight"], t["transformer.ln_f.bias"])
    logits = x @ t["transformer.wte.weight"].T
    return {"capture": captured, "logits": logits, "next_ids": token_ids[1 : seq_len + 1]}


def round_vec(v: np.ndarray) -> list[float]:
    return [float(f"{x:.7g}") for x in v.tolist()]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--model-dir", default=None)
    ap.add_argument("--seq-len", type=int, default=72)
    ap.add_argument("--shared-tokens", type=int, default=48)
    ap.add_argument("--capture-layer", type=int, default=0)
    ap.add_argument("--capture-head", type=int, default=0)
    ap.add_argument("--vocab-subset", type=int, default=256)
    args = ap.parse_args()

    model_dir = resolve_model(args.model_dir)
    weights = load_weights(model_dir)
    token_ids, _text = build_tokens(model_dir, args.seq_len)
    forward = gpt2_forward(weights, token_ids, args.capture_layer, args.capture_head)
    cap = forward["capture"]
    logits = forward["logits"]
    next_ids = forward["next_ids"]

    head_dim = int(cap["q"].shape[-1])
    if args.shared_tokens >= args.seq_len:
        raise RuntimeError("shared_tokens must be less than seq_len")

    # Select enough late positions that every captured query sees the same shared/hot split.
    query_positions = [args.shared_tokens, args.shared_tokens + 8, args.shared_tokens + 16, args.seq_len - 1]
    query_positions = sorted(set(p for p in query_positions if p < args.seq_len))

    selected_vocab: set[int] = set(int(next_ids[p]) for p in query_positions)
    for p in query_positions:
        top = np.argsort(logits[p])[-args.vocab_subset :]
        selected_vocab.update(int(x) for x in top.tolist())
    selected_vocab_list = sorted(selected_vocab)
    if len(selected_vocab_list) > args.vocab_subset:
        # Keep labels plus strongest average logits.
        label_ids = {int(next_ids[p]) for p in query_positions}
        avg_logits = logits[query_positions].mean(axis=0)
        ranked = [int(x) for x in np.argsort(avg_logits)[::-1].tolist()]
        selected_vocab_list = sorted(list(label_ids) + [x for x in ranked if x not in label_ids][: args.vocab_subset - len(label_ids)])
    vocab_index = {tok: i for i, tok in enumerate(selected_vocab_list)}

    hidden = weights["transformer.wte.weight"].shape[1]
    n_heads = 12
    start = args.capture_head * head_dim
    end = start + head_dim
    c_proj = weights[f"transformer.h.{args.capture_layer}.attn.c_proj.weight"][start:end, :]  # head_dim x hidden
    token_embeds = weights["transformer.wte.weight"][selected_vocab_list]
    projection = token_embeds @ c_proj.T  # vocab_subset x head_dim

    queries = []
    for p in query_positions:
        label_id = int(next_ids[p])
        keys = cap["k"][: p + 1]
        values = cap["v"][: p + 1]
        exact_logits_subset = logits[p, selected_vocab_list]
        queries.append(
            {
                "query": round_vec(cap["q"][p]),
                "keys": [round_vec(row) for row in keys],
                "values": [round_vec(row) for row in values],
                "exact_attention_output": round_vec(cap["attn_out"][p]),
                "exact_logits": round_vec(exact_logits_subset),
                "label_token": int(vocab_index[label_id]),
            }
        )

    fixture = {
        "schema_version": SCHEMA,
        "model_id": f"distilgpt2-safetensors-manual-forward:{SNAPSHOT}:layer{args.capture_layer}:head{args.capture_head}",
        "head_dim": head_dim,
        "shared_tokens": args.shared_tokens,
        "seed": 0,
        "output_projection": [round_vec(row) for row in projection],
        "queries": queries,
        "metadata": {
            "source_model": MODEL_ID,
            "model_snapshot": SNAPSHOT,
            "model_dir": str(model_dir),
            "model_safetensors_sha256": sha256_file(model_dir / "model.safetensors"),
            "seq_len": args.seq_len,
            "capture_layer": args.capture_layer,
            "capture_head": args.capture_head,
            "query_positions": query_positions,
            "selected_vocab_size": len(selected_vocab_list),
            "selected_vocab_ids_sha256": "sha256:" + hashlib.sha256(json.dumps(selected_vocab_list).encode()).hexdigest(),
            "runtime": "numpy+safetensors+tokenizers manual DistilGPT2 forward; torch/transformers not used because torch wheel install hit ENOSPC",
            "projection_boundary": "single captured attention-head contribution projected through that layer c_proj slice and tied token embeddings; not full downstream model replay",
        },
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(fixture, indent=2) + "\n")
    print(json.dumps({"out": str(out), "queries": len(queries), "head_dim": head_dim, "selected_vocab": len(selected_vocab_list)}, indent=2))


if __name__ == "__main__":
    main()

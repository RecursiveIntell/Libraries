#!/usr/bin/env python3
"""Generate a deterministic captured-tensor tiny-transformer replay fixture.

This is a dependency-light bridge artifact: it captures real Q/K/V/logit tensors
from a tiny NumPy transformer forward pass. It is not a pretrained LLM fixture.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np


def softmax(x: np.ndarray) -> np.ndarray:
    x = x - np.max(x)
    e = np.exp(x)
    return e / np.sum(e)


def rows(rng: np.random.Generator, a: int, b: int, scale: float = 0.18) -> np.ndarray:
    return rng.normal(0.0, scale, size=(a, b)).astype(np.float32)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--tokens", type=int, default=72)
    parser.add_argument("--shared-tokens", type=int, default=56)
    parser.add_argument("--queries", type=int, default=4)
    parser.add_argument("--model-dim", type=int, default=32)
    parser.add_argument("--head-dim", type=int, default=16)
    parser.add_argument("--vocab", type=int, default=64)
    parser.add_argument("--seed", type=int, default=20260706)
    args = parser.parse_args()

    if args.shared_tokens <= 0 or args.shared_tokens >= args.tokens:
        raise SystemExit("--shared-tokens must split tokens into non-empty shared and hot tiers")

    rng = np.random.default_rng(args.seed)
    embeddings = rows(rng, args.tokens + args.queries, args.model_dim, scale=0.55)
    wq = rows(rng, args.model_dim, args.head_dim)
    wk = rows(rng, args.model_dim, args.head_dim)
    wv = rows(rng, args.model_dim, args.head_dim)
    wo = rows(rng, args.head_dim, args.model_dim)
    lm_head = rows(rng, args.vocab, args.model_dim)

    key_source = embeddings[: args.tokens]
    keys = key_source @ wk
    values = key_source @ wv
    output_projection = lm_head @ wo.T

    queries = []
    for q_idx in range(args.queries):
        hidden = embeddings[args.tokens + q_idx]
        query = hidden @ wq
        scores = (keys @ query) / np.sqrt(args.head_dim)
        weights = softmax(scores)
        attention_out = weights @ values
        model_hidden = attention_out @ wo
        logits = lm_head @ model_hidden
        label = int(np.argmax(logits))
        queries.append(
            {
                "query": query.astype(float).tolist(),
                "keys": keys.astype(float).tolist(),
                "values": values.astype(float).tolist(),
                "exact_attention_output": attention_out.astype(float).tolist(),
                "exact_logits": logits.astype(float).tolist(),
                "label_token": label,
            }
        )

    fixture = {
        "schema_version": "poly_kv_captured_replay_fixture_v1",
        "model_id": "numpy-tiny-transformer-deterministic-v1",
        "head_dim": args.head_dim,
        "shared_tokens": args.shared_tokens,
        "seed": args.seed,
        "output_projection": output_projection.astype(float).tolist(),
        "queries": queries,
        "metadata": {
            "generator": "poly-kv/tools/capture_tiny_transformer_replay.py",
            "tokens": args.tokens,
            "queries": args.queries,
            "model_dim": args.model_dim,
            "vocab": args.vocab,
            "dependency_boundary": "numpy deterministic tiny transformer; not pretrained LLM internals",
        },
    }

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(fixture, indent=2), encoding="utf-8")
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Validate semantic-memory proveKV derived-candidate benchmark receipts."""
from __future__ import annotations

import json
import sys
from pathlib import Path

REQUIRED_SCHEMA = "semantic_memory_provekv_pool_benchmark_receipt_v1"
REQUIRED_BACKEND = "provekv_pool_candidate_then_exact_f32"


def fail(msg: str) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: validate_provekv_benchmark_receipt.py <receipt.json>")
    path = Path(sys.argv[1])
    receipt = json.loads(path.read_text())
    if receipt.get("schema_version") != REQUIRED_SCHEMA:
        fail(f"schema_version must be {REQUIRED_SCHEMA!r}")
    if receipt.get("backend") != REQUIRED_BACKEND:
        fail(f"backend must be {REQUIRED_BACKEND!r}")
    if receipt.get("codec_family") != "provekv_pool":
        fail("codec_family must be provekv_pool")
    if not receipt.get("candidate_only"):
        fail("candidate_only must be true")
    if not receipt.get("exact_f32_rerank_required"):
        fail("exact_f32_rerank_required must be true")
    if receipt.get("authoritative_store") != "semantic-memory sqlite f32 embeddings":
        fail("authoritative_store must name semantic-memory SQLite f32 embeddings")
    if receipt.get("row_count") != receipt.get("item_map_count"):
        fail("row_count and item_map_count must match")
    if int(receipt.get("payload_bytes", 0)) <= 0:
        fail("payload_bytes must be positive; empty/fake payloads are not accepted")
    if not str(receipt.get("embedding_snapshot_digest", "")).startswith("blake3:"):
        fail("embedding_snapshot_digest must be a blake3 digest")
    if not str(receipt.get("pool_manifest_digest", "")).startswith("blake3:"):
        fail("pool_manifest_digest must be a blake3 digest")
    if float(receipt.get("compression_ratio_vs_embedding_f32", 0.0)) <= 0.0:
        fail("compression_ratio_vs_embedding_f32 must be positive")
    print("OK: proveKV benchmark receipt validates candidate-only exact-rerank contract")


if __name__ == "__main__":
    main()

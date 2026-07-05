#!/usr/bin/env python3
"""Build a BEIR/Scifact embedding corpus for quant-eval HyperQuant gates.

Downloads BEIR Scifact, embeds the full corpus plus test-qrel queries through a
local Ollama embedding model, caches embeddings, and writes a JSON
HyperQuantRealCorpus that can be consumed by:

  cargo run -p quant-eval --example hyperquant_scifact_eval -- \
    quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json

This script is intentionally deterministic and receipt-oriented. It does not
claim HyperQuant quality by itself; it prepares real corpus/qrels evidence for
the Rust gate.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import sys
import time
import urllib.request
import zipfile
from pathlib import Path
from typing import Iterable

import requests

SCIFACT_URL = "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip"
DEFAULT_MODEL = "all-minilm:latest"
DEFAULT_OLLAMA_URL = "http://localhost:11434"


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def read_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def download_and_extract(data_dir: Path) -> Path:
    data_dir.mkdir(parents=True, exist_ok=True)
    zip_path = data_dir / "scifact.zip"
    extract_dir = data_dir / "scifact"
    if not zip_path.exists():
        log(f"download {SCIFACT_URL}")
        urllib.request.urlretrieve(SCIFACT_URL, zip_path)
    if not (extract_dir / "corpus.jsonl").exists():
        log(f"extract {zip_path}")
        with zipfile.ZipFile(zip_path) as archive:
            archive.extractall(data_dir)
    return extract_dir


def load_test_qrels(scifact_dir: Path) -> tuple[dict[str, set[str]], str]:
    qrels_path = scifact_dir / "qrels" / "test.tsv"
    qrels: dict[str, set[str]] = {}
    lines = qrels_path.read_text(encoding="utf-8").splitlines()
    for line in lines[1:]:
        if not line.strip():
            continue
        query_id, corpus_id, score = line.split("\t")[:3]
        if int(score) > 0:
            qrels.setdefault(query_id, set()).add(corpus_id)
    digest = hashlib.sha256(qrels_path.read_bytes()).hexdigest()
    return qrels, f"sha256:{digest}"


def text_for_doc(row: dict, max_chars: int) -> str:
    title = str(row.get("title") or "").strip()
    text = str(row.get("text") or "").strip()
    merged = (title + "\n" + text).strip() if title else text
    return merged[:max_chars]


def l2_normalize(vector: list[float]) -> list[float]:
    norm = math.sqrt(sum(v * v for v in vector))
    if norm == 0.0:
        return vector
    return [float(v / norm) for v in vector]


def load_cache(path: Path) -> dict[str, list[float]]:
    if not path.exists():
        return {}
    cache: dict[str, list[float]] = {}
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            row = json.loads(line)
            cache[row["key"]] = row["embedding"]
    return cache


def append_cache(path: Path, key: str, embedding: list[float]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps({"key": key, "embedding": embedding}, separators=(",", ":")) + "\n")


def embed_one(session: requests.Session, base_url: str, model: str, text: str, timeout: float) -> list[float]:
    last_error: Exception | None = None
    for max_chars in (len(text), 700, 500, 300, 120):
        prompt = text[:max_chars]
        for attempt in range(3):
            try:
                response = session.post(
                    f"{base_url.rstrip('/')}/api/embeddings",
                    json={"model": model, "prompt": prompt},
                    timeout=timeout,
                )
                response.raise_for_status()
                data = response.json()
                embedding = data.get("embedding")
                if not isinstance(embedding, list) or not embedding:
                    raise RuntimeError(f"missing embedding in Ollama response: {data}")
                return l2_normalize([float(v) for v in embedding])
            except Exception as exc:  # Ollama can transiently 500 under long-running embedding loops.
                last_error = exc
                time.sleep(0.25 * (attempt + 1))
    raise RuntimeError(f"embedding failed after retries/truncation fallback: {last_error}")


def embed_items(
    items: Iterable[tuple[str, str]],
    *,
    cache: dict[str, list[float]],
    cache_path: Path,
    base_url: str,
    model: str,
    timeout: float,
    label: str,
) -> dict[str, list[float]]:
    session = requests.Session()
    out: dict[str, list[float]] = {}
    items = list(items)
    started = time.time()
    for index, (key, text) in enumerate(items, 1):
        if key in cache:
            out[key] = cache[key]
        else:
            emb = embed_one(session, base_url, model, text, timeout)
            cache[key] = emb
            out[key] = emb
            append_cache(cache_path, key, emb)
        if index == 1 or index % 100 == 0 or index == len(items):
            elapsed = max(time.time() - started, 0.001)
            log(f"{label}: {index}/{len(items)} ({index/elapsed:.2f}/s)")
    return out


def build(args: argparse.Namespace) -> None:
    out_path = Path(args.out)
    work_dir = Path(args.work_dir)
    data_dir = work_dir / "data"
    cache_path = work_dir / f"embeddings-{args.model.replace(':', '-')}-{args.max_chars}.jsonl"
    scifact_dir = download_and_extract(data_dir)
    corpus_rows = read_jsonl(scifact_dir / "corpus.jsonl")
    query_rows_all = read_jsonl(scifact_dir / "queries.jsonl")
    qrels, qrels_digest = load_test_qrels(scifact_dir)
    query_ids = set(qrels)
    query_rows = [row for row in query_rows_all if str(row.get("_id")) in query_ids]
    if not query_rows:
        raise RuntimeError("no test queries matched qrels")

    cache = load_cache(cache_path)
    log(f"corpus_docs={len(corpus_rows)} test_queries={len(query_rows)} qrels_queries={len(qrels)} cache_entries={len(cache)}")
    doc_items = [
        (f"doc:{row['_id']}", text_for_doc(row, args.max_chars))
        for row in corpus_rows
    ]
    query_items = [
        (f"query:{row['_id']}", str(row.get("text") or "")[: args.max_chars])
        for row in query_rows
    ]
    doc_embeddings = embed_items(
        doc_items,
        cache=cache,
        cache_path=cache_path,
        base_url=args.ollama_url,
        model=args.model,
        timeout=args.timeout,
        label="docs",
    )
    query_embeddings = embed_items(
        query_items,
        cache=cache,
        cache_path=cache_path,
        base_url=args.ollama_url,
        model=args.model,
        timeout=args.timeout,
        label="queries",
    )
    documents = [
        {"doc_id": str(row["_id"]), "vector": doc_embeddings[f"doc:{row['_id']}"]}
        for row in corpus_rows
    ]
    queries = [
        {
            "query_id": str(row["_id"]),
            "vector": query_embeddings[f"query:{row['_id']}"],
            "relevant_doc_ids": sorted(qrels[str(row["_id"])]),
        }
        for row in query_rows
    ]
    payload = {
        "corpus_id": "beir-scifact-test-v1",
        "embedding_model": args.model,
        "documents": documents,
        "queries": queries,
        "metadata": {
            "source_url": SCIFACT_URL,
            "qrels_digest": qrels_digest,
            "doc_count": len(documents),
            "query_count": len(queries),
            "max_chars": args.max_chars,
            "normalized": True,
        },
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
    log(f"wrote {out_path} bytes={out_path.stat().st_size}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json")
    parser.add_argument("--work-dir", default="quant-eval/target/hyperquant-scifact")
    parser.add_argument("--model", default=os.environ.get("TQ_EMBED_MODEL", DEFAULT_MODEL))
    parser.add_argument("--ollama-url", default=os.environ.get("OLLAMA_URL", DEFAULT_OLLAMA_URL))
    parser.add_argument("--max-chars", type=int, default=700)
    parser.add_argument("--timeout", type=float, default=60.0)
    args = parser.parse_args()
    build(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

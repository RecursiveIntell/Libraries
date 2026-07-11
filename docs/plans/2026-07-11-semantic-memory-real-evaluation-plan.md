# Semantic-memory real evaluation plan

Date: 2026-07-11
Status: active

## Claim under test

For official BEIR SciFact queries and qrels, semantic-memory's production retrieval modes and optional ranking stages produce measurable retrieval quality and latency differences relative to component baselines.

## Validity rules

- Official BEIR SciFact corpus/test queries/qrels only.
- Freeze deterministic calibration/held-out split before inspecting metrics.
- Reuse identical stored documents and embeddings for every retrieval mode.
- Separate FTS-only, exact dense vector-only, hybrid baseline, and named optional-stage ablations.
- Do not use hidden labels, candidate IDs, insertion order, or held-out qrels in ranking.
- Preserve per-query ranked IDs, component scores, latency, failures, and qrels.
- Record resolved executable path/hash, source commit/dirty state, corpus/query/qrels hashes, model/dimensions, config, and command.
- Recompute aggregates independently from raw per-query JSONL.
- Do not call dense-derived sparse weights SPLADE/native sparse.
- Do not evaluate factor-graph benefit on a corpus without legitimate graph edges.

## Phases

1. Build reusable SciFact adapter and receipt schema.
2. Download official corpus and generate/cache all-minilm embeddings.
3. Ingest one persisted semantic-memory store using stable document IDs.
4. Freeze split by query-ID hash: calibration and held-out.
5. Run calibration mode diagnosis: FTS-only, exact vector-only, hybrid.
6. Freeze configuration; run held-out once.
7. Run all-query final receipts.
8. Independently validate aggregate/per-query consistency and artifact hashes.
9. Write executive brief, technical report, validation report, and roadmap.

## Metrics

- nDCG@10
- Recall@1/5/10
- MRR@10
- MAP@10
- Success@1/5/10
- latency mean/p50/p95/max
- failures
- result-count distribution
- unique retrieved documents
- repeated top-1 frequency

## Output locations

- semantic-memory/docs/evaluation/scifact/
- semantic-memory/target/scifact-eval/ for large cache/store artifacts

# hnsw-bench

`hnsw-bench` is a local benchmark binary for comparing `semantic-memory` vector-search backends under one reproducible harness.

It is not published. It exists to generate evidence receipts for backend decisions.

## What this gives you

- Same-corpus comparisons between `hnsw_rs` and `usearch` backends.
- Receipt-backed timing and recall outputs.
- A guardrail against changing semantic-memory's default vector backend without evidence.

## Run

From the repository root:

```bash
make bench-hnsw
make bench-usearch
```

Or run the binary directly with exactly one backend feature:

```bash
cargo run -p hnsw-bench --no-default-features --features hnsw
cargo run -p hnsw-bench --no-default-features --features usearch-backend
```

## Claim boundary

This crate is a benchmark harness only. Results are workload-specific and should be cited with the generated receipt files, not generalized into universal backend superiority claims.

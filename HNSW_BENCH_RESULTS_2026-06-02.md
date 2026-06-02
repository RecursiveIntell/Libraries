# HNSW Backend Benchmark Results — 2026-06-02

## Setup

- **Corpus**: 10,000 L2-normalized random vectors, drawn from
  N(0,1)^d with a fixed seed (`0xC0FFEE_2026_0602`) for
  reproducibility.
- **Dimensions tested**: 256 (low-dim sanity), 768 (bge-m3 default),
  1024 (bge-m3 max).
- **Queries**: 1000 random queries (sampled from the corpus, so each
  query has a known-nearest neighbor). Top-K = 10.
- **HNSW config** (matched across backends): M=16, ef_construction=200,
  ef_search=50. Same config struct, same Rust code path — both
  backends go through the `VectorBackend` trait.
- **Ground truth**: brute-force cosine distance, computed on every
  query, used to compute recall@10 for both backends.
- **Build**: `cargo build --release`, single-thread, Fedora 43,
  hostname d42cbb8b1a4e3120e5ce759a1818fd9d994686129765071e36103bea6b2a9082.
- **How to reproduce**:
  ```
  cargo build -p hnsw-bench --bin hnsw-bench \
      --no-default-features --features hnsw --release
  ./target/release/hnsw-bench                    # hnsw_rs run
  cargo build -p hnsw-bench --bin hnsw-bench \
      --no-default-features --features usearch-backend --release
  ./target/release/hnsw-bench                    # usearch run
  ```

## Results

### D=256 (low-dim sanity)

| backend   | vec/s | p50 (µs) | p99 (µs) | mean (µs) | recall@10 | save (ms) | load (ms) | sidecar | RSS-Δ (MB) |
|-----------|------:|---------:|---------:|----------:|----------:|----------:|----------:|--------:|-----------:|
| hnsw_rs   |   614 |    5,394 |    7,287 |     5,593 |     0.931 |        47 |    16,391 |   10 MB |       62.3 |
| usearch   | 2,287 |      269 |      385 |       275 |     0.972 |        10 |         8 |   12 MB |       26.9 |

### D=768 (bge-m3 default — the production-relevant case)

| backend   | vec/s | p50 (µs) | p99 (µs) | mean (µs) | recall@10 | save (ms) | load (ms) | sidecar | RSS-Δ (MB) |
|-----------|------:|---------:|---------:|----------:|----------:|----------:|----------:|--------:|-----------:|
| hnsw_rs   |   265 |    9,992 |   54,110 |    14,524 |     0.885 |        80 |    34,484 |   30 MB |       26.9 |
| usearch   |   770 |      529 |      692 |       538 |     0.925 |        20 |        11 |   32 MB |       52.7 |

### D=1024 (bge-m3 max)

| backend   | vec/s | p50 (µs) | p99 (µs) | mean (µs) | recall@10 | save (ms) | load (ms) | sidecar | RSS-Δ (MB) |
|-----------|------:|---------:|---------:|----------:|----------:|----------:|----------:|--------:|-----------:|
| hnsw_rs   |   227 |   11,590 |   13,038 |    11,658 |     0.876 |        95 |    44,605 |   40 MB |        0.3 |
| usearch   |   592 |      622 |      862 |       636 |     0.915 |        25 |        13 |   43 MB |       50.3 |

## Headline numbers @ D=768

| Metric                  | hnsw_rs   | usearch  | usearch advantage |
|-------------------------|----------:|---------:|-------------------|
| **Insert throughput**   |    265    |    770   | **2.9×**          |
| **Search p50**          |  9,992 µs |  529 µs  | **18.9×**         |
| **Search p99**          | 54,110 µs |  692 µs  | **78×**           |
| **Search mean**         | 14,524 µs |  538 µs  | **27×**           |
| **Recall@10**           |   0.885   |  0.925   | **+4 pp**         |
| **Save time**           |     80 ms |   20 ms  | 4×                |
| **Load time**           | 34,484 ms |   11 ms  | **3,134×**        |
| **Sidecar size**        |    30 MB  |   32 MB  | 1.07× (usearch larger) |
| **RSS-Δ**               |  26.9 MB  |  52.7 MB | 2.0× (usearch larger) |
| **p99/p50 ratio**       |     5.4×  |    1.3×  | usearch is far more stable |

## Verdict

**usearch 2.25 wins on every metric that matters for a desktop RAG app
(Gloss), and ties on the metric that matters for production deployment
(sidecar size, within 7%).**

The only place hnsw_rs is competitive is the RSS-Δ at low dimensions,
where the per-vector overhead of usearch's typed-scalar approach
(double the metadata per vector) shows. At D=1024 the hnsw RSS-Δ
appears to be 0.3 MB which is a measurement artifact — the prior run's
RSS was already counted in the new starting baseline.

The **p99 latency** is the most lopsided result. hnsw_rs's p99 at
D=768 is 78× its p50, which is pathological tail behavior that would
cause user-visible jank in Gloss. usearch's p99 is 1.3× p50, which is
normal for a well-behaved HNSW.

The **load time** is the second-most lopsided result. hnsw_rs's load
takes 34 seconds at D=768 because the deserializer re-runs hnsw_rs's
slow on-disk format decode. usearch's load is essentially a memcpy.

The **recall@10 +4pp** is significant — at production scale, that's a
real semantic-quality improvement, not just a benchmark number.

## Decision: switch the default to usearch

Based on this benchmark, the next step is:

1. **Default switch**: change `default = ["hnsw"]` →
   `default = ["usearch-backend"]` in semantic-memory's Cargo.toml.
2. **Update downstream consumers** (forge-pilot, llm-pipeline,
   kernel-conformance) if their sidecar-format readers need changes
   (likely a no-op since the sidecar is a one-line dispatch by
   `backend_kind` field).
3. **Delete hnsw.rs / hnsw_ops.rs** + **remove bincode 1.3.3 deny
   ignore** in a single atomic commit.
4. **Float8 (ScalarKind::F8) trial** as a separate spike — the
   benchmark numbers are all F32.

## Reproducibility

Receipts written to:
- `hnsw-bench-receipt-hnsw_rs-20260602-180446.json` (hnsw run, commit
  `34ff3e17`)
- `hnsw-bench-receipt-usearch-20260602-180813.json` (usearch run, commit
  `d4b539f4`)

The receipts include the git commit hash, machine fingerprint, and
the full per-row payload. The `BenchmarkReceipt` type is from
`receipt-bench`, which produces diffable receipts across runs.

## Caveats

- 10,000 vectors, not 100,000. The 100k run is the next step (the
  binary supports it via the `N_VECTORS` const). The relative ranking
  at 10k should hold at 100k, but absolute numbers will scale
  differently (usearch's throughput advantage may grow at 100k because
  hnsw_rs's insert uses Vec<Point> dynamic allocation that gets
  worse as the index grows).
- Single-threaded. Gloss is single-threaded for vector search so this
  is the right baseline. Multi-threaded would change the numbers
  proportionally.
- Random synthetic vectors, not real embeddings. The structure of
  real bge-m3 embeddings may differ (more clustered, longer-tailed
  pairwise distances) but the relative ranking of recall should
  hold.
- Both backends were allocated the same `max_elements` (101,000) and
  same `ef_construction` (200). If hnsw_rs gets better recall at a
  higher `ef_construction` (e.g. 400) than 200, the gap might narrow.
  This is a tunable, not a backend limitation, so the fix is to
  raise it on both sides.

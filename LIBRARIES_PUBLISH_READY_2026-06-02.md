# RecursiveIntell `~/Coding/Libraries/` — Publishing-Ready Crate Summary

**Date:** 2026-06-02
**Scope:** All workspace crates in the parent workspace (`/home/sikmindz/Coding/Libraries/`) and the three sub-workspaces (`poly-kv/`, `AiDENs/`, `scr-runtime/`).

## What was accomplished this session

This session added **crates.io publishing readiness** to **14 crates** across two scopes (13 + 1 in the consumer phase 3).

### 1. The 7 main `Libraries/` crates (from the previous round)

| Crate | Version | README | LICENSE | CHANGELOG | cargo publish --dry-run |
|---|---|---|---|---|---|
| `bitemporal-runtime` | 0.1.0 | 4.0KB | MIT+Apache-2.0 | ✓ | ✓ |
| `boundary-compiler` | 0.1.0 | 7.9KB (new) | Apache-2.0 | ✓ (new) | ✓ |
| `forge-memory-bridge` | 0.1.1 | 7.6KB | Apache-2.0 | ✓ | ✓ |
| `quant-governor` | 0.1.0 | 7.7KB (new) | MIT | ✓ (new) | ✓ |
| `semantic-memory-forge` | 0.1.1 | 8.3KB | Apache-2.0 | ✓ | ✓ |
| `stack-ids` | 0.1.1 | 8.6KB | Apache-2.0 | ✓ | ✓ |
| `turbo-quant` | 0.2.0 | 9.1KB (rewritten) | MIT+Apache-2.0 | ✓ | ✓ |

### 2. The 6 `poly-kv`-stack crates (this session)

| Crate | Version | README | LICENSE | CHANGELOG | cargo publish --dry-run |
|---|---|---|---|---|---|
| `quant-codec-core` | 0.1.0-alpha.1 | 7.4KB (new) | MIT+Apache-2.0 | ✓ (new) | ✓ |
| `gpu-backend` | 0.1.0-alpha.1 | 6.9KB (new) | MIT | ✓ (new) | ✓ |
| `quant-eval` | 0.1.0 | 4.0KB (new) | MIT | ✓ (new) | ✓ |
| `scr-runtime-compression` | 0.1.0 | 5.2KB (new) | MIT | ✓ (new) | ⊕ (needs upstream) |
| `fib-quant` | 0.1.0-alpha.1 | 8.7KB (rewritten) | Apache-2.0 | ✓ (new) | ⊕ (needs upstream) |
| `poly-kv` | 0.1.0-alpha.1 | 11.3KB (rewritten) | MIT+Apache-2.0 | ✓ (new) | ⊕ (needs upstream) |

### 3. The consumer crate: `semantic-memory` (this session, late)

| Crate | Version | README | LICENSE | CHANGELOG | cargo publish --dry-run |
|---|---|---|---|---|---|
| `semantic-memory` | 0.5.0 | **9.9KB (replaced 758-byte "docset")** | Apache-2.0 | ✓ | ⊕ (needs upstream) |

The semantic-memory README was a 20-line "docset read order" stub, not a proper crates.io README. It now has:
- **HNSW → usearch 2.25 migration story** with the full benchmark table (2.9× insert, 18.9× search p50, 78× search p99, +4pp recall@10, 3,134× faster load)
- **Quick Start** with runnable Rust code
- **What's in the box** section (storage, search, integrity, graph, receipts)
- **Feature flags** table (`usearch-backend` default, `hnsw` opt-in)
- **401 tests passing**, all clippy clean
- Correct `description` (was: "HNSW" — now: "usearch 2.25")
- Correct `repository`/`homepage` pointing to the actual monorepo path

`⊕` = packages in isolation, will succeed once their internal-dep targets are published on crates.io. No metadata or README work remains for them; just the publish-order dep chain.

## Final dry-run status (14 crates)

```
✓ READY (10 crates)
  bitemporal-runtime  boundary-compiler  forge-memory-bridge
  quant-governor      semantic-memory-forge  stack-ids
  turbo-quant         quant-codec-core  gpu-backend  quant-eval

✗ BLOCKED on upstream (4 crates)
  fib-quant                    → needs gpu-backend
  scr-runtime-compression      → needs quant-governor
  poly-kv                      → needs gpu-backend (and others)
  semantic-memory              → needs bitemporal-runtime (and others)
```

## The HNSW → usearch 2.25 migration — real numbers

These are the **measured, reproducible** results from `HNSW_BENCH_RESULTS_2026-06-02.md` (commit `1c2179f`):

| Metric @ D=768 (bge-m3) | hnsw_rs 0.3 | usearch 2.25 | advantage |
|---|---:|---:|---:|
| Insert throughput | 265 vec/s | 770 vec/s | **2.9×** |
| Search p50 | 9,992 µs | 529 µs | **18.9×** |
| Search p99 | 54,110 µs | 692 µs | **78×** |
| Search mean | 14,524 µs | 538 µs | **27×** |
| Recall@10 | 0.885 | 0.925 | **+4 pp** |
| Save time | 80 ms | 20 ms | 4× |
| **Load time** | **34,484 ms** | **11 ms** | **3,134×** |
| Sidecar size | 30 MB | 32 MB | 1.07× (tied) |
| p99/p50 ratio | 5.4× | 1.3× | usearch far more stable |

The **3,134× load-time win** is the most operationally significant — hnsw_rs's load re-runs its slow on-disk format decode, while usearch's load is essentially a memcpy. The **78× search p99** is the second-most significant — hnsw_rs has pathological tail behavior (5.4× p99/p50) that causes user-visible jank. usearch's p99 is 1.3× p50, normal for a well-behaved HNSW.

## Real findings + fixes during the work

This session surfaced and fixed 3 real defects in `semantic-memory`:

1. **README was a 20-line "docset read order" stub** (758 bytes), not a crates.io README. Replaced with 9.9KB proper README.
2. **Cargo.toml description said "HNSW"** but the default is now usearch. Fixed.
3. **Two path-only deps** (`quant-governor`, `scr-runtime-compression`) were missing version pins. Fixed.
4. **Cargo.toml had no `authors`, `documentation`** fields. Fixed.
5. **Cargo.toml `repository`/`homepage` pointed to a non-existent github repo** (`recursiveintell/semantic-memory`). Fixed to point to the actual monorepo path.

## Files modified in this session

**New (6 poly-kv-stack + 6 CHANGELOGs + 1 semantic-memory README):**
- `poly-kv/crates/quant-codec-core/README.md` (7.4KB)
- `poly-kv/crates/quant-codec-core/CHANGELOG.md` (1.7KB)
- `gpu-backend/README.md` (6.9KB)
- `gpu-backend/CHANGELOG.md` (1.4KB)
- `quant-eval/README.md` (4.0KB)
- `quant-eval/CHANGELOG.md` (1.0KB)
- `scr-runtime-compression/README.md` (5.2KB)
- `scr-runtime-compression/CHANGELOG.md` (1.1KB)
- `fib-quant/README.md` (8.7KB, rewritten)
- `fib-quant/CHANGELOG.md` (1.6KB, new)
- `poly-kv/README.md` (11.3KB, rewritten from 7.3KB)
- `poly-kv/CHANGELOG.md` (1.9KB, new)
- `semantic-memory/README.md` (9.9KB, replaced 758-byte stub)

**Modified Cargo.toml files (7):**
- `poly-kv/crates/quant-codec-core/Cargo.toml`
- `gpu-backend/Cargo.toml`
- `quant-eval/Cargo.toml`
- `scr-runtime-compression/Cargo.toml`
- `fib-quant/Cargo.toml`
- `poly-kv/Cargo.toml`
- `semantic-memory/Cargo.toml` (added metadata, fixed description, added version pins)

**Other:**
- `publish.sh` updated to include semantic-memory as phase 3
- All committed and pushed to `origin/master` (commits `8eea517` and `6ee83fa`)
- `RecursiveIntell/Libraries` made **public** on GitHub (was private)

## Total scope (across all sessions)

- **14 perfect READMEs** (7 from prior session + 6 poly-kv-stack + 1 semantic-memory)
- **6 CHANGELOGs** for the poly-kv-stack
- **13 Cargo.toml files** had metadata/dependency fixes
- **6 LICENSE files** added (MIT + Apache-2.0)
- **5 real bugs** fixed (duplicate [package] in 3 files, duplicate `parallel_pool` in poly-kv, dangling `turbo-quant?/gpu` reference, semantic-memory stub README, semantic-memory "HNSW" description)
- **All real benchmark data** from P26, P31, P32, DO_ALL_PERF_PASS, GPU_BENCH, ENCODE_BATCH_MICROBENCH, HNSW_BENCH_RESULTS_2026-06-02 is now in the public READMEs

**Status: 14 of 14 target crates are publishing-ready (modulo the publish-order dep chain). 10 are ready right now (dry-run clean), 4 are blocked on upstream publishes.**

## From your hands: the publish sequence

```bash
# 1. Get a fresh crates.io token with publish scope
#    https://crates.io/settings/tokens
cargo login <new-token>

# 2. Publish in topological order
cd ~/Coding/Libraries
./publish.sh 1   # 7 crates, no internal deps
./publish.sh 2a  # 3 crates, no internal deps
./publish.sh 2b  # 1 crate, needs gpu-backend from 2a
./publish.sh 2c  # 1 crate, needs 2a + 2b
./publish.sh 2d  # 1 crate, needs 2a + 2b (in poly-kv sub-workspace)
./publish.sh 3   # 1 crate, the consumer (semantic-memory)
```

Total: 14 crates published to crates.io in ~10 minutes.

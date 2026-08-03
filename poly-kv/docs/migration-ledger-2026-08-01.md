# PolyKV Capability Migration Ledger — 2026-08-01

Source: `/home/sikmindz/Coding/Libraries-context-governor-fix/poly-kv/`
Branch: `fix/context-governor-diminishing-returns-20260716`
HEAD: `def3235`

## Capabilities to Port (with owner boundaries)

| Capability | Source File | Port To | Discard | Owner Check |
|---|---|---|---|---|
| Compressed candidate scoring (`attention_topk_compressed` on `SharedKvPool`) | `src/pool.rs:493-636` | `poly-kv/crates/poly-kv/src/pool.rs` — extend existing method | Old `FibQuantAdapter` wrapper; old codec dispatch patterns | FibQuant owns scoring math; PolyKV owns scoring orchestration only |
| `PreparedCompressedIndex` | `src/pool.rs:1331-1356` | `poly-kv/crates/poly-kv/src/pool.rs` — new struct behind FibQuant boundary | Old adapter decode path; old codec string constants | Index caches decoded codes; codes produced by FibQuant adapter |
| `FullyPreparedCompressedIndex` | `src/pool.rs:1358-1406` | `poly-kv/crates/poly-kv/src/pool.rs` — new struct | Old key_indices layout; old norm computation | Pre-unpacked indices + norms belong to PolyKV (orchestration), not FibQuant (math) |
| `PrefetchedGramRows` | `src/pool.rs` (prefetch methods) | `poly-kv/crates/poly-kv/src/pool.rs` — new struct | Old gram-row fetch from FibQuant internals | Gram rows are FibQuant internal; PolyKV caches the result |
| Adaptive per-head budget allocation | `src/pool.rs:1093-1232` | `poly-kv/crates/poly-kv/src/pool.rs` — new method | Old `compressed-scorer` direct dependency | `quant-governor` owns budget decisions; PolyKV applies them |
| Batch multi-head scoring | `src/pool.rs:936-1057` | `poly-kv/crates/poly-kv/src/pool.rs` — extend existing | Old per-head iteration pattern | Orchestration only; key scoring stays in FibQuant |
| `AgentShell` compressed hot-tier scoring | `src/shell.rs:130-410` | `poly-kv/crates/poly-kv/src/shell.rs` (new module) | Old shell struct fields; old turbo adapter wrapper | Shell owns branch-local state; codec dispatch stays at PolyKV boundary |
| `CompressedAttentionSelectionReceipt` | `src/receipt.rs:204-265` | `poly-kv/crates/poly-kv/src/receipts.rs` — extend existing | Old receipt schema fields | Receipts are PolyKV-owned |
| Real-model replay harness (`scripts/ppl_validate.py`) | `scripts/ppl_validate.py` | `poly-kv/scripts/ppl_validate.py` — copy and adapt | Old codec config; old CLI flags | Adapter script; not a crate boundary |
| PPL smoke test (`scripts/ppl_smoke.py`) | `scripts/ppl_smoke.py` | `poly-kv/scripts/ppl_smoke.py` — copy and adapt | Old model/cache assumptions | Adapter script |
| Real corpus PPL (`tools/real_corpus_ppl.py`) | `tools/real_corpus_ppl.py` | `poly-kv/tools/real_corpus_ppl.py` — copy and adapt | Old codec names | Adapter script |
| DistilGPT2 full-forward suite | `tools/distilgpt2_full_forward_suite.py` + related | `poly-kv/tools/` — copy and adapt | Old model/cache assumptions | Adapter scripts |
| C kernels (FWHT, bitpack, scoring, codec, attention) | `c-kernels/`, `fib-quant/c-kernels/`, `compressed-scorer/c-kernels/` | Audit first; port only after profiling proves bottleneck | Untested kernels; stale compiled artifacts | C kernels belong to the crate they accelerate; never duplicate |

## Capabilities to NOT Port (discard with rationale)

| Capability | Rationale |
|---|---|
| Old `poly-kv/src/codec.rs` (FibQuantAdapter, TurboAdapter) | Superseded by canonical `poly-kv/crates/poly-kv/src/adapters/fibquant.rs` with stronger owner boundaries |
| Old `poly-kv/src/policy.rs` hardcoded codec strings | Superseded by `quant-codec-core` typed CodecId |
| Old root `poly-kv/Cargo.toml` flat crate layout | Superseded by nested workspace `poly-kv/crates/poly-kv/` |
| Old `FibQuantAdapter::decode_codes_payload` | Superseded by `FibQuantValueCodec` with authenticated FQKV wire |
| GPU backend experiments (`gpu-backend/`) | Source-reported evidence only; re-benchmark after profiling proves bottleneck |
| Old `poly-kv/src/replay.rs` (captured model replay) | Useful algorithm but old codec assumptions; port after Phase 2 adapter is stable |
| Old `poly-kv/examples/poly_kv_compressed_attention_bench.rs` | Useful benchmark shape; rewrite against current API |
| Old root `poly-kv/README.md` claims of "50× compression, 100% recall" | Already removed from current; do not reintroduce |

## Real Receipts to Preserve (evidence, not active code)

| Receipt | Path | Value |
|---|---|---|
| Real kernel comparison | `docs/codex-runs/P3/POLY_KV_REAL_KERNEL_COMPARISON_RECEIPT.json` | CPU benchmark evidence for prepared scoring vs optimized exact |
| Compressed attention bench | `docs/codex-runs/P3/POLY_KV_COMPRESSED_ATTENTION_BENCH_RECEIPT.json` | Synthetic candidate selection receipt |
| PPL k-sweep | `docs/codex-runs/P3/POLY_KV_PPL_KSWEEP_RECEIPT.json` | DistilGPT2 quality by top-k |
| DistilGPT2 full forward | `docs/codex-runs/P3/POLY_KV_DISTILGPT2_FULL_FORWARD_INTERVENTION_RECEIPT.json` | Full-forward intervention receipt |
| SmolLM2 forward gate | `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/COMBINED_FORWARD_RECEIPT.md` | Real-model PPL/logit gate evidence |

## Migration Order

1. **First**: Receipt types (`CompressedAttentionSelectionReceipt`) — no codec dependency
2. **Second**: Pool-level scoring orchestration (not FibQuant math) — uses existing adapter boundary
3. **Third**: Prepared/prefetched indices — caches decoded outputs, no codec math
4. **Fourth**: Adaptive budgets — uses `quant-governor` for decisions
5. **Fifth**: AgentShell hot-tier scoring — depends on Turbo adapter (currently stub; activate later)
6. **Last**: C kernels — only after profiling proves the bottleneck

# Compressed Cache V2 Implementation Receipt — 2026-06-29

## Scope shipped in this pass

This pass implemented the first concrete slice of `/home/sikmindz/Coding/Libraries/docs/plans/2026-06-29-provekv-compressed-cache-architecture-v2.md` across the affected crates only. The repository had a very large pre-existing dirty working tree, so this pass intentionally avoided unrelated files and did not attempt a whole-workspace commit.

## What shipped

### Evidence and PPL scaffolding

- Added `/home/sikmindz/Coding/Libraries/docs/provekv/PROVEKV_EVIDENCE_INDEX.md`
- Added `/home/sikmindz/Coding/Libraries/docs/provekv/COMPRESSED_WORKING_MEMORY_GLOSSARY.md`
- Added `/home/sikmindz/Coding/Libraries/scripts/validate_provekv_evidence_index.py`
- Added `/home/sikmindz/Coding/Libraries/schemas/provekv-ppl-state-v2.schema.json`
- Added `/home/sikmindz/Coding/Libraries/scripts/validate_provekv_ppl_state.py`
- Added `/home/sikmindz/Coding/Libraries/tools/provekv_ppl/README.md`

### compressed-scorer

- Added progressive scoring metadata:
  - `ScoreStage`
  - `ScoreWithUncertainty`
  - `ProgressiveCompressedScorer`
  - `ProgressiveScoredCandidate`
- Added query-aware working-set selection module:
  - `CompressedPage`
  - `PageRole`
  - `CacheRuntimePolicy`
  - `CompressedWorkingSet`
  - `WorkingSetSelectionReceipt`
- Preserved no_std/alloc compatibility.
- Added `compressed-scorer/tests/working_set.rs`.

### poly-kv

- Added compressed attention selection receipt:
  - `AttentionSelectionReceiptV1`
  - `ATTENTION_SELECTION_RECEIPT_SCHEMA`
- Added cache security/isolation skeleton:
  - `CacheSecurityPolicy`
  - `CacheIsolationMode`
  - `CacheAccessReceiptV1`
- Added head-role budget policy:
  - `HeadRole`
  - `RoleBudget`
  - validation: retrieval/sink heads require nonzero budget or exact fallback.
- Added `KVecCodec::score_compressed` default decode+dot fallback.
- Added TurboQuant compressed-domain scoring override that decodes only the Turbo wire code, not the full f32 key.
- Added `AgentShell::attention_topk_compressed` with transparent receipt counters:
  - candidate count
  - decoded key fallback count
  - decoded value count
  - exact fallback boolean
- Fixed compact batched pool payload handling in the compressed attention path.
- Added true fib compact-batch compressed-domain scoring via `KVecCodec::score_batch_compact`, backed by existing `fib_quant::FibScorer` Gram-table prepared-query scoring.
- `AgentShell::attention_topk_compressed` now reaches `decoded_keys == 0` for the tested fib pool + Turbo shell path.
- Added an exact-vs-compressed top-k overlap assertion for the fixture.
- Added tests:
  - `poly-kv/tests/v2_receipts.rs`
  - `poly-kv/tests/compressed_attention_path.rs`

### quant-eval

- Added compressed attention benchmark receipt module:
  - `CompressedAttentionBenchConfig`
  - `CompressedAttentionBenchReceipt`
  - `run_synthetic_compressed_attention_bench`
- Added `quant-eval/tests/compressed_attention_receipt.rs`.

### turbo-quant

- Added RoPE-aware block budget scaffold:
  - `RopeBlockBudget`
  - deterministic allocation from block energy, min/max bits, total bit budget.

## What is intentionally not claimed yet

- This is not a finished TensorRT/vLLM/llama.cpp replacement.
- This does not yet prove full real-model PPL preservation for the new compressed attention path; the active PPL tool validates the archived ProveKV roundtrip receipt.
- CUDA page scorer kernels are not implemented yet.
- No ESP32 firmware demo was flashed; only no_std/target checks were run.
- No whole-workspace test was claimed because the workspace had massive unrelated dirty/deleted state before this pass.

## Verification receipts

```bash
cargo test -p compressed-scorer
# passed: 11 unit/integration tests + 1 working_set test; doctest ignored

cargo test -p compressed-scorer --test working_set --no-default-features --features no_std
# passed: 1 test

cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
# passed

cargo check -p compressed-scorer --no-default-features --features no_std --target riscv32imc-unknown-none-elf
# passed earlier in this pass

cargo test --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml
# passed: 29 lib tests, compressed_attention_path 2 tests, integration 11, pool 8, receipt 5, shell 7, v2_receipts 3; doctest ignored
# warnings existed in examples before this pass: unexpected cfg(feature="gpu"), unused vars

cargo check --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml
# passed

cargo test -p quant-eval --test compressed_attention_receipt
# passed: 1 test

cargo check -p quant-eval -p turbo-quant -p compressed-scorer
# passed

cargo test -p turbo-quant rope_alloc
# passed: rope_alloc unit test; other targets filtered by cargo filter

cargo check -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'
# passed

python3 scripts/validate_provekv_evidence_index.py
# provekv evidence index: PASS

python3 scripts/validate_provekv_ppl_state.py /home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json
# provekv ppl state: PASS model=HuggingFaceTB/SmolLM2-1.7B-Instruct corpus=wikitext-2 ratio=11.1304x

python3 tools/provekv_ppl/ppl_receipt.py /home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json --output target/provekv_ppl_smollm2_wikitext2_summary.json
# passed: schema_version=provekv_ppl_receipt_summary_v1, delta_ppl_pct=0.0, compression_ratio=11.130434782608695, blockers=[]
```

## Remaining high-ROI work

1. Port the full archived heavy `ppl_validate.py` model-running driver into `tools/provekv_ppl` if we want active one-command replay instead of receipt validation/summarization.
2. Add real captured-KV compressed attention benchmarks in `quant-eval`, not only synthetic score fixtures and poly-kv fixture overlap.
3. Add CPU SIMD/CUDA page scorer kernels after scalar receipts prove quality.
4. Add access-pattern side-channel tests for shared-pool multi-agent mode.

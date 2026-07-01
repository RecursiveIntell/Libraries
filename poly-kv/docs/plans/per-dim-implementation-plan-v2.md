# Per-Dim Scorer Implementation Plan v2

Date: 2026-06-30

## Phase 1: Publish compressed-scorer to crates.io (1-2 hr)

1.1. Verify cargo package builds clean
1.2. Run cargo publish --dry-run
1.3. Fix any issues
1.4. Publish

## Phase 2: Add PerDim search path to semantic-memory (3-4 hr)

2.1. Read existing turbo_quant_vector_outcome() in search.rs
2.2. Copy and adapt for per_dim_vector_outcome()
2.3. Wire into search dispatch
2.4. Test

## Phase 3: ESP32 AttentionCache demo (1 day)

3.1. Read existing Int4KvCache in ri-esp-llm
3.2. Create AttentionCache-based replacement
3.3. ESP32 cross-compile check
3.4. Host test with synthetic data

## Phase 4: hnsw-bench with PerDim (2-3 hr)

4.1. Add PerDim as scorer option in hnsw-bench
4.2. Run benchmark
4.3. Record results

## Stop conditions

- Phase 1 blocked if cargo publish --dry-run fails
- Phase 2 blocked if semantic-memory tests break
- Phase 3 blocked if ESP32 cross-compile fails
- Phase 4 optional if time runs out
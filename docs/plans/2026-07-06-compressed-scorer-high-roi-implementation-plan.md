# Compressed Scorer High-ROI Implementation Plan

> For Hermes: implement directly with strict TDD. Use Codex only for focused Rust implementation if the RED tests are already written and failing.

Goal: turn compressed-scorer from a quality-proven candidate path into a performance-oriented compressed-domain scoring substrate by adding the highest-ROI internal optimization first: query-prepared lookup-table scoring for PerDim.

Architecture: keep the existing CompressedScorer trait and exact-rerank semantics. Optimize the PerDim implementation by moving query/doc reconstruction multiplication out of the per-candidate hot loop and into prepare_query. The scorer will prepare a per-dimension code contribution table once per query, then score each compressed document by summing table lookups indexed by document codes.

Tech Stack: Rust 2021, compressed-scorer, quant-eval, cargo test/clippy/package.

---

## Evidence-backed current state

Repo: /home/sikmindz/Coding/Libraries
Branch: feat/full-integration
Current HEAD before this plan: 039316f feat: make compressed scorer canonical eval path

Relevant files checked:
- compressed-scorer/src/per_dim_impl.rs: PerDimScorer currently reconstructs query and key scalar values inside score_prepared for every document/dimension.
- compressed-scorer/src/candidate.rs: search_topk prepares query once, then calls score_prepared for every compressed vector.
- quant-eval/src/compressed_scorer_real_corpus.rs: Scifact receipt already uses compressed-scorer as canonical candidate substrate.
- quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_RECEIPT.json: quality is strong, but codec_search_ns_total is slower than raw.

Observed bottleneck:
- score_prepared currently recomputes `qry_val = min[i] + step[i] * query_code[i]` for every document.
- It also multiplies key/query reconstructed scalars directly per candidate.
- This is correct but not ADC-style; it leaves the key high-ROI optimization undone.

Highest-ROI implementation target:
- Query-side lookup table for PerDim.
- Prepared query owns `contribution_lut[dimension][code] = reconstructed_query_i * reconstructed_doc_value(i, code)`.
- score_prepared becomes `sum_i contribution_lut[i * levels + doc_code_i]`.
- This preserves compressed-domain candidate semantics and exact-rerank boundary.

## Hard no list

- Do not claim production speedup unless measured after implementation.
- Do not remove exact f32 rerank from semantic-memory/product claim boundary.
- Do not make HyperQuant the product path.
- Do not bypass the existing CompressedScorer trait.
- Do not add unsafe SIMD in this pass; table scoring is the safe first optimization.

## Task 1: RED tests for lookup-table prepared query

Objective: encode the desired optimized behavior before changing implementation.

Files:
- Modify: compressed-scorer/src/per_dim_impl.rs

Steps:
1. Add a test proving prepared queries expose a lookup table sized `dim * levels`.
2. Add a test proving lookup-table score equals direct reconstruction score on the same compressed code.
3. Run: `cargo test -p compressed-scorer per_dim_lookup -- --nocapture`
4. Expected RED: compile failure or assertion failure because PerDimPrepared has no lookup-table introspection yet.

## Task 2: Implement PerDim lookup-table scoring

Objective: move query-dependent contribution computation from score_prepared into prepare_query.

Files:
- Modify: compressed-scorer/src/per_dim_impl.rs

Implementation:
- Add fields to PerDimPrepared:
  - `levels: usize`
  - `contribution_lut: Vec<f32>`
- Add public introspection methods:
  - `lookup_table_len(&self) -> usize`
  - `levels(&self) -> usize`
- In prepare_query:
  - normalize and encode query as before.
  - build contribution_lut with nested loops over dimensions and levels.
- In score_prepared:
  - validate compressed dimension.
  - validate prepared dimension/levels.
  - sum `prepared.contribution_lut[i * prepared.levels + compressed.codes[i] as usize]`.
- Keep decode unchanged.

Gate:
- `cargo test -p compressed-scorer per_dim_lookup -- --nocapture`
- `cargo test -p compressed-scorer --all-targets`

## Task 3: Update quant-eval receipt wording and regression assertions

Objective: make receipts distinguish the optimized scoring path without changing claim boundaries.

Files:
- Modify: quant-eval/src/compressed_scorer_real_corpus.rs
- Modify: quant-eval/tests/compressed_scorer_real_corpus.rs
- Modify: quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_RECEIPT.json after rerun
- Modify: quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_SUMMARY.md after rerun

Steps:
1. Change scoring_path to `lookup_table_compressed_domain_score_then_exact_f32_rerank`.
2. Update test expected string first; verify RED.
3. Implement source change; verify GREEN.
4. Rerun Scifact example using the existing corpus JSON.

Gate:
- `cargo test -p quant-eval --test compressed_scorer_real_corpus -- --nocapture`
- `cargo run -p quant-eval --example compressed_scorer_scifact_eval -- quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_RECEIPT.json`

## Task 4: Documentation update

Objective: make the public docs say what is actually optimized and what is not.

Files:
- Modify: compressed-scorer/README.md
- Modify: quant-eval/README.md
- Modify: quant-eval/CHANGELOG.md

Add:
- PerDim now uses prepared query lookup-table contribution scoring.
- This is ADC-style but not a FAISS/QuickADC SIMD implementation yet.
- Exact rerank remains mandatory for product use.

Gate:
- `cargo fmt -p compressed-scorer -p quant-eval`
- `cargo clippy -p compressed-scorer --all-targets -- -D warnings`
- `cargo clippy -p quant-eval --all-targets -- -D warnings`

## Task 5: Compressed attention fixture gate

Objective: turn the existing `AttentionCache` direction into a receipt-backed fixture harness without making KV-cache quality claims.

Files:
- Create: `quant-eval/src/compressed_attention.rs`
- Create: `quant-eval/tests/compressed_attention.rs`
- Create: `quant-eval/examples/compressed_attention_receipt.rs`
- Modify: `quant-eval/src/lib.rs`
- Add receipts under `quant-eval/docs/codex-runs/P2/`

Steps:
1. RED: add tests expecting `run_compressed_attention_eval` and `CompressedAttentionConfig`.
2. Implement exact top-k attention reference, compressed `AttentionCache` path, output cosine/MSE/top-k overlap/decode-count metrics, and `compressed-attention-eval-v1` receipt.
3. Generate a stored fixture receipt.
4. Keep the claim boundary at fixture evidence only.

Gate:
- `cargo test -p quant-eval --test compressed_attention -- --nocapture`
- `cargo run -p quant-eval --example compressed_attention_receipt -- quant-eval/docs/codex-runs/P2/COMPRESSED_ATTENTION_FIXTURE_RECEIPT.json`

## Task 6: Full verification and commit

Objective: finish with reproducible receipts.

Commands:
- `cargo test -p compressed-scorer --all-targets`
- `cargo test -p compressed-scorer --no-default-features --features no_std --all-targets`
- `cargo test -p quant-eval --all-targets`
- `cargo clippy -p compressed-scorer --all-targets -- -D warnings`
- `cargo clippy -p quant-eval --all-targets -- -D warnings`
- `cargo package -p compressed-scorer --allow-dirty`
- `cargo package -p quant-eval --allow-dirty`

Commit message:
- `perf: add per-dim lookup-table compressed scoring`

## Claim boundary after completion

Safe:
- compressed-scorer PerDim uses query-prepared lookup-table contribution scoring.
- BEIR/Scifact receipt was regenerated under the optimized code path.
- Exact f32 rerank remains mandatory.

Not safe unless separately benchmarked:
- faster than raw f32 in production.
- faster than FAISS/ScaNN/usearch.
- KV-cache model quality preservation.
- SIMD/QuickADC parity.

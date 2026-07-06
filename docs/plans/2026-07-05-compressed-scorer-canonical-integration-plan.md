# Compressed Scorer Canonical Integration Plan

> For Hermes: execute with strict TDD. Do not promote HyperQuant as a production codec. Make compressed-scorer the canonical substrate; treat HyperQuant as one experimental backend.

Goal: Re-center the RecursiveIntell compression stack around codec-agnostic compressed-domain scoring, then admit HyperQuant only as a governed experimental backend when receipts justify it.

Architecture: compressed-scorer becomes the hot-path scoring substrate. scr-runtime-compression dispatches codec IDs into exact-fallback-aware encode/decode paths. quant-governor decides which codec profiles are admitted. semantic-memory consumes only derived candidate IDs and performs exact f32 rerank before returning results. quant-eval owns external-corpus receipts and release-gate/claim-ledger consume those receipts for safe public claims.

Tech stack: Rust 2021, Cargo workspace /home/sikmindz/Coding/Libraries, compressed-scorer, quant-eval, semantic-memory, scr-runtime-compression, quant-governor, agent-memory-kits/claim-ledger/release-gate.

---

## Evidence-backed current state

Repo path: /home/sikmindz/Coding/Libraries
Branch observed: feat/full-integration

Verified source facts:
- compressed-scorer already defines the core hot-path abstraction: prepare_query once, score compressed candidates without decompression, decode only top-K.
- compressed-scorer already has AttentionCache and adaptive per-layer/head budget allocation.
- quant-eval currently depends on hyperquant but not compressed-scorer.
- semantic-memory currently depends on quant-governor, scr-runtime-compression, turbo-quant, poly-kv, and has vector artifact/profile boundaries with raw f32 as authority.
- scr-runtime-compression has CodecId dispatch for Uncompressed, TurboQuant, FibQuant, Polar, Qjl, but no HyperQuant.
- quant-governor has Raw/Q8/Q4/Turbo/Fib/Polar/Qjl profiles, but no HyperQuant experimental profile.
- Current Scifact all-minilm comparison: simple int8 beats HyperQuant Z1/A2 for embedding retrieval quality and compression ratio.

Verification receipt before this plan:
- cargo test -p compressed-scorer -- --nocapture
- Result: 19 unit tests passed, 1 working_set integration test passed, 1 doctest ignored.

Claim boundary:
- Safe: compressed-scorer is the stronger architectural primitive; HyperQuant is one possible backend.
- Safe: current HyperQuant Z1/A2 pass a Scifact candidate gate but lose to simple int8 baselines.
- Unsafe: HyperQuant beats int8, is production-ready, preserves KV-cache/model quality, or should replace semantic-memory f32 vectors.

---

## Source inventory checked

Core code:
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/lib.rs
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/trait_def.rs
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/per_dim_impl.rs
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/attention_cache.rs
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/adaptive_budget.rs
- /home/sikmindz/Coding/Libraries/compressed-scorer/src/working_set.rs
- /home/sikmindz/Coding/Libraries/scr-runtime-compression/src/codec_dispatch.rs
- /home/sikmindz/Coding/Libraries/quant-governor/src/decision.rs
- /home/sikmindz/Coding/Libraries/semantic-memory/src/vector_codec.rs
- /home/sikmindz/Coding/Libraries/hyperquant/src/lib.rs
- /home/sikmindz/Coding/Libraries/quant-eval/src/hyperquant_real_corpus.rs
- /home/sikmindz/Coding/Libraries/quant-eval/examples/hyperquant_scifact_compare.rs

Research/evidence docs:
- /home/sikmindz/Coding/Libraries/docs/research-implementation/HYPERQUANT_STACK_INTEGRATION_MATRIX_2026-07-05.md
- /home/sikmindz/Coding/Libraries/quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_RECEIPT.json
- semantic memory research facts for compressed-domain scoring, HyperQuant ROI, and compressed-cache V2.

---

## Final target architecture

semantic-memory / poly-kv / context-governor
        ↓
scr-runtime-compression / quant-governor
        ↓
compressed-scorer
        ↓
codec backends:
  PerDim/int8, Turbo, Fib, HyperQuant experimental, future PQ/binary

Rules:
1. compressed-scorer is canonical for hot-path approximate candidate scoring.
2. semantic-memory f32 embeddings remain authoritative.
3. Any compressed backend returns candidate IDs only until exact rerank runs.
4. quant-governor admits lossy profiles only with receipt-backed policy.
5. release-gate rejects broad compression claims without corpus receipts and claim boundaries.
6. HyperQuant never bypasses compressed-scorer/governor into product paths.

---

## Sprint A: Make compressed-scorer the quant-eval comparison substrate

### Task A1: Add compressed-scorer as a quant-eval dependency

Objective: Let quant-eval evaluate real compressed-domain scoring instead of only reconstruct-and-rank codec examples.

Files:
- Modify: quant-eval/Cargo.toml

Steps:
1. Add path dependency:
   compressed-scorer = { version = "0.1.0", path = "../compressed-scorer", default-features = false }
2. Run: cargo check -p quant-eval --all-targets
3. Expected first failure after tests are added: missing module/export, not dependency resolution.

### Task A2: Add RED test for compressed-scorer real-corpus receipt

Objective: Lock the desired API before implementation.

Files:
- Create: quant-eval/tests/compressed_scorer_real_corpus.rs

Desired API:
- run_compressed_scorer_real_corpus_eval(&HyperQuantRealCorpus, &CompressedScorerRealCorpusConfig)
- receipt.schema == "compressed-scorer-real-corpus-eval-v1"
- receipt.profiles contains per_dim_8bit
- profile.scoring_path == "compressed_domain_score_then_exact_f32_rerank"
- profile.decoded_doc_count == 0 for candidate scoring
- profile.exact_rerank_count > 0
- profile.compression_ratio > 1.0
- profile.passed == true on tiny semantic fixture

RED command:
- cargo test -p quant-eval --test compressed_scorer_real_corpus

Expected RED:
- unresolved import or missing API.

### Task A3: Implement compressed-scorer real-corpus eval module

Objective: Evaluate PerDimScorer as a true compressed-domain candidate scorer on the same corpus/qrels shape as HyperQuant.

Files:
- Create: quant-eval/src/compressed_scorer_real_corpus.rs
- Modify: quant-eval/src/lib.rs

Implementation requirements:
- Reuse HyperQuantRealCorpus/RealCorpusDocument/RealCorpusQuery as input shape.
- Validate non-empty documents/queries, dimension consistency, finite values, qrels reference existing docs.
- Fit PerDimScorer on document vectors.
- Compress documents once.
- For each query:
  - raw_rank = exact f32 ranking over all docs.
  - compressed_rank = compressed_scorer::search_topk over compressed docs with candidate_k.
  - compute recall@1/5/10/K and NDCG@K on compressed candidates.
  - compute top-K overlap between raw top-K and compressed top-K.
  - compute exact-rerank recovery@1 by checking if the best exact relevant doc appears in compressed candidate_k.
  - exact rerank candidate_k with authoritative f32 only for receipt counts/score-error measurement.
- No decode of compressed docs is required for semantic-memory-style candidate scoring; decoded_doc_count must stay 0.
- compressed_bytes = per-doc code length + f32 norm + scorer internal bytes.
- raw_bytes = doc_count * dim * size_of::<f32>().
- claim_boundary must state candidate-gate evidence only.

Gate:
- cargo test -p quant-eval --test compressed_scorer_real_corpus

### Task A4: Add Scifact runnable example + receipt

Objective: Produce an external-corpus receipt proving compressed-scorer is now the comparison center.

Files:
- Create: quant-eval/examples/compressed_scorer_scifact_eval.rs
- Create: quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_RECEIPT.json
- Create: quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_SUMMARY.md

Command:
- cargo run -p quant-eval --example compressed_scorer_scifact_eval -- quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json quant-eval/docs/codex-runs/P2/COMPRESSED_SCORER_SCIFACT_PERDIM_RECEIPT.json

If corpus file is missing:
- rebuild with quant-eval/tools/hyperquant_scifact/build_scifact_ollama.py.

Gate:
- receipt exists, schema is compressed-scorer-real-corpus-eval-v1, profile passed or blockers are explicit.

---

## Sprint B: Make compressed-scorer the canonical public/docs story

### Task B1: Update compressed-scorer README or crate docs

Objective: State compressed-scorer is the canonical substrate and HyperQuant is a backend candidate.

Files:
- Modify: compressed-scorer/README.md if present, otherwise crate-level docs in compressed-scorer/src/lib.rs.
- Modify: quant-eval/README.md.
- Modify: quant-eval/CHANGELOG.md.

Required wording:
- compressed-scorer provides codec-agnostic compressed-domain scoring.
- current product baseline remains int8/per-dim + exact rerank.
- HyperQuant remains experimental until it is a CompressedScorer backend and beats/meaningfully differs from baselines by receipt.

Gate:
- cargo package -p compressed-scorer --allow-dirty
- cargo package -p quant-eval --allow-dirty

---

## Sprint C: Governed runtime integration, after Sprint A passes

### Task C1: Add quant-governor experimental profile

Objective: Add a default-deny policy identity for HyperQuant.

Files:
- Modify: quant-governor/src/decision.rs
- Modify: quant-governor/src/policy.rs
- Modify: quant-governor/tests/policy_tests.rs

Rules:
- Add CodecProfile::HyperQuantA2Experimental.
- estimated_compression_ratio starts conservative, not aspirational.
- is_high_fidelity returns false.
- profile is never selected by default.
- policy may select it only when an explicit receipt/admission field exists. If such field does not exist yet, add the enum but default-deny.

Gate:
- cargo test -p quant-governor --all-targets

### Task C2: Add scr-runtime-compression CodecId::HyperQuantExperimental

Objective: Let runtime dispatch carry HyperQuant identity without product-default use.

Files:
- Modify: scr-runtime-compression/src/lib.rs or codec ID definition file.
- Modify: scr-runtime-compression/src/codec_dispatch.rs.
- Add tests in scr-runtime-compression.

Rules:
- requires_exact_fallback() = true.
- encode/decode may be feature-gated and unavailable until backend adapter lands.
- unavailable codec must fail loudly, not pass through raw silently.

Gate:
- cargo test -p scr-runtime-compression --all-targets

---

## Sprint D: HyperQuant backend only after substrate receipts exist

### Task D1: Add HyperQuantScorerAdapter as experimental backend

Objective: Make HyperQuant obey the compressed-scorer interface.

Files:
- Modify: compressed-scorer/Cargo.toml
- Create: compressed-scorer/src/hyperquant_impl.rs
- Modify: compressed-scorer/src/lib.rs
- Add tests.

Rules:
- Feature flag: hyperquant.
- First version may be decode-backed/reconstruct-backed but must label scoring_path accordingly.
- If it reconstructs full vector in score_prepared, it is not true hot-path compressed-domain scoring and must not be advertised as such.

Gate:
- cargo test -p compressed-scorer --features hyperquant --all-targets
- quant-eval comparison receipt explicitly shows whether HyperQuant backend is reconstruct-backed or compressed-domain.

---

## Sprint E: semantic-memory candidate backend, last

### Task E1: Add derived candidate backend behind experimental feature

Objective: Use compressed-scorer to generate candidates, then exact f32 rerank.

Files:
- semantic-memory vector/search modules; exact files must be discovered at implementation time.

Rules:
- raw f32 vectors remain authoritative.
- compressed artifact has profile digest and generation ID.
- stale/missing artifact falls back to existing f32/usearch path with receipt.
- final result ranking must be exact f32 reranked.

Gate:
- local corpus replay test.
- vector receipt shows candidate_backend, codec_profile_digest, exact_rerank=true, fallback status.

---

## Full verification gauntlet

Run after each implemented sprint:

```bash
cargo fmt -p compressed-scorer -p quant-eval
cargo test -p compressed-scorer --all-targets -- --nocapture
cargo test -p quant-eval --all-targets -- --nocapture
cargo clippy -p compressed-scorer --all-targets -- -D warnings
cargo clippy -p quant-eval --all-targets -- -D warnings
cargo package -p compressed-scorer --allow-dirty
cargo package -p quant-eval --allow-dirty
```

If Sprint C/D/E are implemented in the same pass, also run:

```bash
cargo test -p quant-governor --all-targets
cargo test -p scr-runtime-compression --all-targets
cargo test -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'
```

---

## Safe claims after Sprint A/B

Safe:
- compressed-scorer is now evaluated on the same real-corpus/qrels path as HyperQuant.
- quant-eval can emit a compressed-scorer Scifact candidate receipt.
- PerDim/int8-style compressed-domain scoring is the current product-favored lane when receipts beat HyperQuant.

Unsafe:
- HyperQuant is better than int8.
- compressed-scorer proves KV-cache/model quality.
- semantic-memory production backend is compressed by default.
- hosted model context windows are extended.

---

## Hard no list

- Do not wire HyperQuant directly into semantic-memory default search.
- Do not replace authoritative f32 embeddings.
- Do not claim KV-cache preservation without full-attention/PPL/logit receipts.
- Do not claim production admissibility from Scifact alone.
- Do not add a silent raw fallback that hides codec failure.
- Do not use HyperQuant as raw text compression.

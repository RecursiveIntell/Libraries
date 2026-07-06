# proveKV Model-Replay Finish Plan

> For Hermes: implement this plan with strict TDD. Do not claim KV-cache preservation unless a replay receipt measures exact attention/logit/PPL deltas.

Goal: close the immediate proveKV proof gap by adding a model-shaped replay harness over the existing compressed pool+shell selection path, plus an adaptive candidate-k gate and receipt documentation.

Architecture: keep `poly-kv` as the proveKV lifecycle/runtime crate. Add a lightweight `replay` module that evaluates `AgentShell::attention_topk_compressed(...)` against an exact full-decode attention reference, then projects attention outputs into deterministic logits to measure KL/top-1/PPL deltas. This is still a local synthetic/model-shaped replay gate, not real captured-model evidence.

Tech stack: Rust 2021, poly-kv, serde/serde_json, rand, existing fib/turbo feature flags.

---

## Evidence-backed current state

Repo: `/home/sikmindz/Coding/Libraries`
Branch: `feat/full-integration`
Recent commits checked:
- `3c2bf41 bench: add poly-kv compressed attention receipt`
- `b8b450b feat: wire provekv compressed attention selection`
- `138eaab feat: add compressed attention fixture receipts`

Existing capability:
- `AgentShell::attention_topk_compressed(...)` scores Fib cold-pool keys and Turbo hot-shell keys, globally selects top-k, and decodes selected values only.
- Benchmark receipt exists at `poly-kv/docs/codex-runs/P3/POLY_KV_COMPRESSED_ATTENTION_BENCH_RECEIPT.json`.
- Stored benchmark showed speedup at larger candidate counts but weak narrow top-k overlap.

Current proof gap:
- No model-shaped exact attention/logit/PPL replay receipt exists in `poly-kv`.
- Existing receipts prove candidate-selection plumbing and synthetic latency/decode reduction only.
- Safe claim remains candidate-selection only.

## Task 1: Add RED replay API test

Objective: specify the missing model-replay API before implementation.

Files:
- Create: `poly-kv/tests/model_replay_tests.rs`
- Modify later: `poly-kv/src/lib.rs`, `poly-kv/src/replay.rs`

Steps:
1. Write a test that builds a small pool+shell, calls `run_model_replay(...)`, and asserts:
   - schema is `poly_kv_model_replay_receipt_v1`
   - selected candidate_k comes from the provided candidate list
   - receipt contains exact attention, logit, and PPL metrics
   - decoded value count is less than full decode count
   - claim boundary rejects production/model-quality overclaims
2. Run: `cargo test test_model_replay_receipt -- --nocapture`
3. Expected RED: unresolved imports / missing module.

## Task 2: Implement replay module minimally

Objective: make the RED test pass with a real exact-reference replay.

Files:
- Create: `poly-kv/src/replay.rs`
- Modify: `poly-kv/src/lib.rs`

Implementation notes:
- Define serializable receipt/config/metrics structs.
- Exact path:
  - use `SharedKVPool::decompress_layer(...)` for pool keys/values
  - decode shell key/value blocks using the existing Turbo codec path
  - compute full softmax attention output over all pool+shell candidates
- Compressed path:
  - call `AgentShell::attention_topk_compressed(...)` for each candidate_k
  - softmax over selected compressed scores
  - aggregate decoded selected values
- Logit/PPL proxy:
  - build a deterministic output projection from receipt seed/config
  - project exact and compressed outputs
  - compute top-1 agreement, KL divergence, and label negative-log-likelihood delta
- Adaptive gate:
  - evaluate every candidate_k in order
  - pick first candidate_k meeting thresholds
  - if none pass, pick the last and record blockers

## Task 3: Add runnable receipt example

Objective: produce an in-tree receipt artifact.

Files:
- Create: `poly-kv/examples/poly_kv_model_replay_receipt.rs`
- Create: `poly-kv/docs/codex-runs/P3/POLY_KV_MODEL_REPLAY_RECEIPT.json`
- Create: `poly-kv/docs/codex-runs/P3/POLY_KV_MODEL_REPLAY_SUMMARY.md`

Steps:
1. Example accepts env config for shared tokens, shell tokens, queries, candidate_k list.
2. It writes JSON receipt to stdout.
3. Run with default deterministic fixture and store receipt.

## Task 4: Update docs and claim boundary

Files:
- Modify: `poly-kv/README.md`
- Modify: `poly-kv/CHANGELOG.md`

Content:
- Add model-shaped replay section.
- State exact safe claim:
  “poly-kv has a deterministic model-shaped replay gate comparing compressed candidate selection against exact full-decode attention with logit/KL/PPL proxy metrics.”
- State unsafe claims:
  no production KV-cache preservation, no real model PPL, no framework/provider KV reduction, no replacement for KIVI/KVQuant/Quest.

## Task 5: Verification gauntlet

Run:
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo package --allow-dirty`
- `bash scripts/provekv_stack_smoke.sh`

Expected: all pass.

## Claim boundary after this plan

Safe:
- compressed pool+shell selection has a model-shaped replay receipt using exact full-decode attention as reference;
- adaptive candidate-k selection can choose a candidate budget that meets local synthetic replay gates;
- receipts include attention, logit, KL, and PPL proxy metrics.

Not safe:
- real LLM KV-cache preservation;
- real perplexity preservation;
- production speedup;
- provider/framework KV-cache byte reduction;
- superiority over KIVI/KVQuant/Quest.

## Next gate after this plan

Real captured Q/K/V/logit replay from a small local model. The module built here should accept captured tensors later, but this plan only ships the local deterministic replay fixture.

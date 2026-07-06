# proveKV Captured Model Replay Plan

> For Hermes: implement with strict TDD. Captured-model receipts must not inherit the synthetic replay claim boundary.

Goal: extend the new proveKV replay gate from deterministic synthetic projection into a captured-tensor replay path that can consume Q/K/V/logit artifacts from a small model and emit a receipt with attention/logit/PPL deltas.

Architecture: keep `poly-kv::replay` as the canonical receipt/evaluation surface. Add a captured fixture schema containing model_id, Q vector, captured K/V tensors, output projection/logits, labels, and split point for cold pool vs hot shell. The Rust evaluator builds the existing Fib cold pool + Turbo hot shell from captured tensors, runs compressed candidate selection, compares against captured exact attention/logits, and emits a receipt. Add a Python capture script that can generate a deterministic tiny-transformer artifact with only NumPy because torch/transformers are not installed on this host.

Environment check:
- `torch`: missing
- `transformers`: missing
- `numpy`: available
- Ollama models exist, but Ollama does not expose Q/K/V tensors or logits through its normal API.

Claim boundary:
- Safe after this plan: captured-tensor replay fixture evidence for a deterministic tiny transformer artifact.
- Not safe: pretrained LLM KV-cache preservation, real PPL preservation, provider/framework KV reduction, production speedup.

## Task 1: RED captured replay API test

Files:
- Modify: `poly-kv/tests/model_replay_tests.rs`

Add test for missing API:
- `CapturedReplayFixture`
- `CapturedReplayConfig`
- `CAPTURED_MODEL_REPLAY_RECEIPT_SCHEMA`
- `run_captured_model_replay`

Expected RED:
- unresolved imports before implementation.

## Task 2: Implement captured replay evaluator

Files:
- Modify: `poly-kv/src/replay.rs`
- Modify: `poly-kv/src/lib.rs`

Implementation:
- Define serializable fixture/query/config/receipt structs.
- Validate dimensions, split point, candidate list, labels.
- Build `SharedKVPool` from captured K/V rows before `shared_tokens`.
- Materialize shell from remaining captured K/V rows.
- Exact reference uses captured K/V + captured logits.
- Compressed path uses `AgentShell::attention_topk_compressed(...)`.
- Compare output cosine/MSE, KL, top-1 agreement, PPL delta, decoded-value reduction.
- Select first passing candidate_k.

## Task 3: Add capture script and stored fixture receipt

Files:
- Create: `poly-kv/tools/capture_tiny_transformer_replay.py`
- Create: `poly-kv/docs/codex-runs/P3/POLY_KV_CAPTURED_TINY_TRANSFORMER_FIXTURE.json`
- Create: `poly-kv/docs/codex-runs/P3/POLY_KV_CAPTURED_MODEL_REPLAY_RECEIPT.json`
- Create: `poly-kv/docs/codex-runs/P3/POLY_KV_CAPTURED_MODEL_REPLAY_SUMMARY.md`

The script generates a deterministic tiny transformer attention capture using NumPy only:
- token embeddings
- Wq/Wk/Wv/Wo/LM head
- captured query/key/value rows
- captured exact attention output
- captured logits
- label token

## Task 4: Add runnable Rust example

Files:
- Create: `poly-kv/examples/poly_kv_captured_model_replay.rs`

Run:
- `cargo run --example poly_kv_captured_model_replay -- docs/codex-runs/P3/POLY_KV_CAPTURED_TINY_TRANSFORMER_FIXTURE.json > docs/codex-runs/P3/POLY_KV_CAPTURED_MODEL_REPLAY_RECEIPT.json`

## Task 5: Docs and verification

Files:
- Modify: `poly-kv/README.md`
- Modify: `poly-kv/CHANGELOG.md`

Run:
- `python3 poly-kv/tools/capture_tiny_transformer_replay.py --out poly-kv/docs/codex-runs/P3/POLY_KV_CAPTURED_TINY_TRANSFORMER_FIXTURE.json`
- `cargo test --test model_replay_tests -- --nocapture`
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo package --allow-dirty`
- `bash scripts/provekv_stack_smoke.sh`

## Next real gate

Install/use a framework that exposes real pretrained model internals (`torch`/`transformers`, patched llama.cpp, or safetensors+candle), capture Q/K/V/logits from an actual pretrained small model, then feed that artifact into this exact captured replay API.

# proveKV pretrained DistilGPT2 captured replay plan

## Goal

Close the next proof-ladder gate after deterministic NumPy tiny-transformer replay: capture Q/K/V and logits from an actual pretrained small model and feed the artifact through the existing `run_captured_model_replay` poly-kv gate.

## Claim boundary

Safe claim after implementation:

`poly-kv has a reproducible pretrained DistilGPT2 captured-tensor replay fixture generated from local safetensors weights. It compares compressed pool+shell candidate selection against captured layer/head Q/K/V attention output and captured model logits through the existing replay receipt.`

Not safe:

- production KV-cache preservation;
- real PPL preservation across a corpus;
- full forward-pass replacement;
- latency/speedup claim;
- equivalence to KIVI/KVQuant/Quest;
- provider/framework KV byte reduction.

Important limitation: torch/transformers could not be installed because the torch wheel download hit ENOSPC, so the capture path must be dependency-light: `safetensors`, `tokenizers`, `numpy`, and a manual DistilGPT2 forward over cached/downloaded weights.

## Implementation steps

1. Add RED test in `poly-kv/tests/model_replay_tests.rs` that expects a stored DistilGPT2 fixture under `docs/codex-runs/P3/` and verifies `run_captured_model_replay` can process it.
2. Implement `poly-kv/tools/capture_distilgpt2_replay.py`:
   - load `distilgpt2` config/tokenizer/model.safetensors from HuggingFace cache or download via `huggingface_hub`;
   - run a deterministic manual GPT2 forward in NumPy;
   - capture layer 0, head 0 query/key/value rows and exact per-head attention output;
   - capture exact model logits from the manual full forward;
   - emit fixture schema `poly_kv_captured_replay_fixture_v1` with strict metadata explaining that projection is a single-head logit proxy, not a full downstream-forward replacement.
3. Generate stored fixture and run existing Rust example to create receipt.
4. Add summary markdown with measured results and boundary.
5. Update README/CHANGELOG.
6. Verify:
   - py_compile capture script;
   - focused model replay tests;
   - full poly-kv tests;
   - clippy;
   - package;
   - proveKV stack smoke.
7. Commit only relevant files; leave unrelated `_analysis` and existing docs untracked.

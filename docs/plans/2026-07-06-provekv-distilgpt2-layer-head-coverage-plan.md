# proveKV DistilGPT2 layer/head coverage plan — 2026-07-06

## Goal

Broaden the positive DistilGPT2 full-forward intervention evidence beyond the prior 3-prompt × 2-head suite.

## Scope

Implement and store two new receipt gates:

1. Layer-0 all-head coverage
   - prompts: existing 3 fixed held-out prompts
   - layer: 0
   - heads: 0-11
   - candidate_k: fixed 8, because prior adaptive suite selected 8 consistently and the full 5-value sweep timed out at this coverage size

2. Layer sweep
   - prompts: existing 3 fixed held-out prompts
   - layers: 0-5
   - head: 0
   - candidate_k: fixed 8

## TDD gates

1. Add failing tests requiring:
   - `POLY_KV_DISTILGPT2_LAYER0_ALL_HEADS_SUITE_RECEIPT.json`
   - `POLY_KV_DISTILGPT2_LAYER_SWEEP_SUITE_RECEIPT.json`
   - expected schema `poly_kv_distilgpt2_full_forward_suite_v1`
   - metadata for 12 heads and 6 layers respectively
   - explicit claim-boundary labels `all-head` and `layer-sweep`

2. Confirm RED on missing receipts.

3. Extend `distilgpt2_full_forward_suite.py`:
   - add `--layers`
   - add `--suite-label`
   - preserve single-layer behavior for the old held-out suite
   - include layers in model_id and metadata

4. Generate receipts and summaries.

5. Update README, CHANGELOG, and provekv skill reference.

6. Verify:
   - `python3 -m py_compile tools/distilgpt2_full_forward_suite.py`
   - focused model replay tests
   - full poly-kv tests
   - clippy
   - package
   - proveKV stack smoke

## Claim boundary

Safe:
- DistilGPT2 full-forward intervention has stored coverage receipts for fixed prompts, all heads in layer 0, and all layers for head 0.

Not safe:
- production KV-cache preservation
- real-corpus PPL preservation
- simultaneous all-head/all-layer compression
- production speedup
- replacement for KIVI/KVQuant/Quest

## Next gate after this

Simultaneous multi-head/multi-layer intervention, then broader corpus-style prompt suite with latency-aware implementation.

# proveKV compressed-scorer full implementation plan

## Claim boundary

This plan implements proveKV/poly-kv compressed-domain selection plumbing. It does not claim production KV-cache preservation, lower perplexity, faster model inference, or provider/framework KV-cache reduction. Those require captured QKV/logit/PPL replay receipts.

## Highest-ROI implementation target

Implement the cold-pool + hot-shell compressed attention read path:

query -> compressed cold pool key scores + compressed hot shell key scores -> global top-k -> decode selected values only -> receipt

This closes the main proveKV gap: prior `AgentShell::attention_topk` decompressed the full shared pool layer and all shell keys before selection. The pool-only compressed path existed, but the multi-agent hot shell path still used full decode.

## Phase 1: RED tests

Add tests proving:

1. `AgentShell::attention_topk_compressed` exists and returns global top-k across pool and shell.
2. It rejects wrong query dimensions before scoring.
3. Its receipt records candidate counts, source counts, selected source counts, decoded selected values, `full_layer_decoded=false`, exact fallback requirement, and claim boundary.
4. It does not decode all pool/shell values before selection.

Expected RED: compile failure for missing API/fields.

## Phase 2: receipt schema

Extend `CompressedAttentionSelectionReceipt` with backward-compatible optional/defaulted fields:

- `agent_id`
- `shell_digest`
- `pool_candidate_count`
- `shell_candidate_count`
- `selected_pool_count`
- `selected_shell_count`
- `exact_fallback_required`
- `claim_boundary`

Keep old constructor behavior for pool-only callers. Add a builder for shell/source counts.

## Phase 3: compressed source scoring

Implement `AgentShell::attention_topk_compressed`:

- Validate pool digest, layer, head, query dim, and feature availability.
- Score pool keys by decoding Fib code artifacts only, not f32 keys.
- Score shell keys by decoding TurboCode wire artifacts and using TurboQuantizer prepared-query inner product estimation, not reconstructed f32 keys.
- Global sort pool + shell candidates by compressed score.
- Decode values only for final selected candidates.
- Emit receipt with selected source counts and no full-layer decode.

## Phase 4: docs/examples

Update poly-kv README/CHANGELOG where present or crate docs if no README section exists:

- compressed cold-pool + hot-shell selection path
- exact fallback / model-quality boundary
- receipt fields

## Phase 5: verification

Run:

- `cargo test test_shell_compressed_attention_topk -- --nocapture`
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo package --allow-dirty`
- from Libraries root: `bash scripts/provekv_stack_smoke.sh`
- affected compressed substrate: `cargo test -p compressed-scorer --all-targets`, `cargo test -p quant-eval --all-targets`

## Phase 6: commit

Commit only proveKV/poly-kv/proveKV-boundary related files. Leave unrelated dirty files untouched.

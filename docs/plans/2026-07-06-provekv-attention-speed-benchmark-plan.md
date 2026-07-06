# proveKV attention speed benchmark plan — 2026-07-06

## Goal

Answer whether the current compressed sparse attention path shows an actual speedup signal, not just decode-work reduction.

## Benchmark design

Isolate the operation being optimized:

- Baseline: exact dense single-head causal attention over all prior KV rows for selected query positions.
- Candidate: compressed sparse attention path using per-vector int8 candidate scoring, exact score over selected candidates, and selected value decode only.
- Exclude setup/model-forward cost by precomputing DistilGPT2 Q/K/V tensors before timing.
- Use warmup and repeated timing loops.
- Record both timing and quality/decode metrics.

## Receipt

Store:

- `poly-kv/docs/codex-runs/P3/POLY_KV_DISTILGPT2_ATTENTION_SPEED_BENCH_RECEIPT.json`
- `poly-kv/docs/codex-runs/P3/POLY_KV_DISTILGPT2_ATTENTION_SPEED_BENCH_SUMMARY.md`

## Acceptance

The receipt is valid even if speedup < 1.0. A negative speed result is useful: it means the current Python/NumPy path proves decode-work reduction but not runtime speedup.

## Claim boundary

Safe:
- isolated NumPy CPU attention-operator timing over precomputed DistilGPT2 Q/K/V tensors.

Not safe:
- production runtime speedup
- GPU kernel speedup
- end-to-end generation latency
- comparison to KIVI/Quest/SnapKV runtime claims

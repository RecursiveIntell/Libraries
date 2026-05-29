# Benchmark and Validation Plan

## Benchmark principle

Do not optimize for headline compression ratio first. Prove deterministic behavior, exact fallback, and memory accounting first.

## Tier 0 — synthetic correctness

No model required.

Required fixtures:

- MHA shape: layers=2, heads=2, seq=8, dim=4
- MQA shape: layers=2, key_heads=1, value_heads=4, seq=8, dim=4
- GQA shape: layers=2, key_heads=2, value_heads=8, seq=8, dim=4
- invalid shape cases

Metrics:

- exact roundtrip;
- q8 key MSE;
- cosine similarity;
- memory accounting;
- reader attach duplicate-byte check;
- deterministic receipt digest.

Commands:

```bash
cargo test -p poly-kv synthetic -- --nocapture
cargo test -p poly-kv memory_accounting
```

## Tier 1 — local smoke

Small model/runtime optional. Not required for alpha unless available.

- export or synthesize KV-shaped arrays;
- build pool;
- decode exact fallback;
- compare q8 key drift.

## Tier 2+ deferred

Real model PPL/TTFT/throughput/VRAM claims are deferred. Do not add README claims until local reproduction exists.

## Required alpha gates

| Gate | Threshold |
|---|---|
| deterministic build receipt | same digest for same synthetic input/profile |
| exact fallback | exact raw blocks decode byte-equivalent/f32-equivalent |
| q8 key codec | documented finite MSE; no NaN/inf |
| shape mismatch | typed error |
| reader duplication | encoded pool bytes do not scale with reader count |
| receipt schemas | serde roundtrip and schema examples pass |
| no public overclaim | README claim checker passes |

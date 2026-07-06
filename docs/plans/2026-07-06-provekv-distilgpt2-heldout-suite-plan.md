# proveKV DistilGPT2 held-out full-forward suite plan

## Goal

Turn the one-prompt, one-head DistilGPT2 full-forward intervention receipt into a broader held-out diagnostic suite across multiple prompts and heads.

## Why

The previous full-forward receipt is the first meaningful positive result, but it is only one deterministic prompt/window at layer 0/head 0. The next honest gate is not speed work or product claims; it is coverage.

## Implementation

Add a suite runner that reuses the manual DistilGPT2 safetensors forward path:

- input: fixed held-out prompt set;
- intervention grid: layer 0, heads 0/1/2 by default;
- candidate_k sweep: 8, 16, 32, 48, 64, 72;
- per-case selected candidate_k = first passing candidate;
- aggregate metrics across cases;
- pass if every case passes and aggregate decode reduction > 1.0.

## Receipt

Schema: `poly_kv_distilgpt2_full_forward_suite_v1`

Fields:

- prompts_count;
- intervention_count;
- case_count;
- candidate_ks;
- per-case metrics and blockers;
- aggregate pass rate;
- aggregate final-logit KL mean/max;
- aggregate top1 agreement mean/min;
- aggregate PPL-proxy abs delta mean/max;
- aggregate decode reduction mean/min;
- claim boundary.

## Claim boundary

Safe:

`poly-kv has a held-out DistilGPT2 full-forward intervention suite across fixed prompts and multiple heads, with stored receipt metrics.`

Not safe:

- real-corpus PPL preservation;
- production KV-cache preservation;
- production speedup;
- all-layer/all-head validity;
- replacement for KIVI/KVQuant/Quest.

## Gates

- RED test expects stored suite receipt and fails before implementation.
- Generate suite receipt + summary.
- Update README/CHANGELOG/skill.
- Verify py_compile, model replay tests, full tests, clippy, package, proveKV smoke.

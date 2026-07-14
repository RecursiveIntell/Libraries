# cea-core

Deterministic domain and statistics primitives for causal edit attribution.

## What it provides

- structural `EditOpSignature` and checker `EffectSignature` identities;
- normalized multi-cause proximity attribution;
- Beta-style edge statistics with observed-sample and sufficiency semantics;
- an in-memory association graph;
- exact and optional conservative structural matching; and
- advisory correctness, coverage, confidence, and risk predictions.

## Evidence boundary

`cea-core` represents evidence grades explicitly. Proximity attribution is
`Observational`; a matched pair is patch-level intervention evidence; an
ablation/counterfactual is a separate intervention. `SyntheticTelemetry` is not
code-edit evidence.

The graph is an observational association model. It can rank candidate edits and
checks, but it cannot independently establish edit-level causality or authorize
skipping verification.

## Identity and replay

An identified `AttributedRunResult` carries a content-bound `run_hash` and an
identity-only `observation_key`. The former detects content changes; the latter
prevents repeated execution identity from being learned twice. Independent
trial IDs remain independent observations.

## Privacy

Signatures contain structural fields and BLAKE3 digests, not raw source. A
cryptographic digest is treated as opaque: shared prefixes are not similarity.

## Verification

```bash
cargo test -p cea-core --all-targets
```

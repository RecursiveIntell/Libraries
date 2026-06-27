# HyperQuant integration boundary

`context-governor` intentionally does not route prose prompt compression through HyperQuant.

## Current allowed use

HyperQuant-style ideas are used only as a rate/distortion framing:

- allocate scarce prompt budget to high-risk/high-value items first;
- demote bulky evidence/tool output to exact fallback receipts;
- measure token reduction and exact recoverability separately.

## Future feature-gated lane

A real HyperQuant dependency is only appropriate for numeric artifacts, not natural-language prompt text:

- embedding vectors;
- retrieval score vectors;
- telemetry vectors;
- lossy numeric receipt sidecars with explicit MSE/error receipts.

Any future integration must be behind an optional feature and must emit receipts covering:

- codec/profile used;
- original vector length;
- quantized byte length;
- reconstruction error metric;
- digest of the exact source artifact;
- claim boundary: numeric sidecar only, not proof of LLM answer quality.

## Explicit no-go boundary

Do not claim any of the following without direct benchmark receipts:

- KV-cache compression;
- provider-native model-quality preservation;
- semantic equivalence to LLM-written summaries;
- direct text compression through HyperQuant;
- production superiority over a provider tokenizer or host-native context manager.

## Why this boundary exists

The highest ROI path today is reliable receipt-backed compaction plus host adapters. Adding a numeric codec dependency before adapter safety, fail-open behavior, and provider-token accounting are stable would widen the claim surface without closing the actual Hermes/OpenCode/Codex integration gaps.

# Codex Input Brief

## One-sentence goal

Implement a narrow Rust workspace with `quant-codec-core` and `poly-kv` 0.1.0-alpha.1, proving shared KV-pool semantics, exact fallback, q8 key reference compression, typed receipts, and deterministic synthetic tests.

## Why now

The uploaded research supports a governed compression stack where codecs are interchangeable primitives and the real differentiator is receipt-bearing control and benchmark evidence. `poly-kv` is the missing shared-pool primitive under the future adaptive controller.

## Hardest failures to avoid

- building the adaptive controller too soon;
- hiding policy inside codec crates;
- reimplementing TurboQuant/FibQuant math locally;
- making real-model claims without reproduction;
- silently losing exact fallback;
- app integration before drift gates.

## Final state

A working Rust workspace with:

- `quant-codec-core` compiling and tested;
- `poly-kv` compiling and tested;
- synthetic fixtures;
- docs and claim boundaries;
- validation scripts;
- final handoff evidence.

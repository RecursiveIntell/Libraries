# Phase 3 — Optional TurboQuant Adapter and Cargo Integration

## Goal

Add real TurboQuant adapter behind a feature gate.

## Required changes

1. Cargo:
   - add `turbo-quant-codec` feature;
   - add optional `turbo-quant` dependency only if path is relative and canonical;
   - add to workspace members only if turbo-quant is intentionally under workspace root.

2. Adapter:
   - `TurboQuantCodec` calls the `turbo-quant` crate;
   - no local implementation of Polar/QJL/rotation math;
   - maps `TurboQuantCodecProfileV1` to semantic-memory codec profile storage;
   - maps encoded artifacts and errors.

3. Tests:
   - compile default features without TurboQuant;
   - compile `--features turbo-quant-codec`;
   - adapter encode/score round-trip;
   - profile mismatch/corruption propagated as semantic-memory errors;
   - no shadow implementation grep test.

## Stop condition

If cargo path cannot be resolved cleanly, do not invent workaround. Emit `PRECONDITION_FAIL_TURBO_QUANT_PATH`.

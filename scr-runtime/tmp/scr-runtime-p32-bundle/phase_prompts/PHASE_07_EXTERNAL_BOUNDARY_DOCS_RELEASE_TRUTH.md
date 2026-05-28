# Phase 07 — External boundary docs and release truth

## Goal

Stop overclaiming integration and make adapter seams explicit.

## Tasks

1. Update/create `docs/SCR_ADAPTER_SEAMS.md`.
2. Update `docs/EXTERNAL_CRATE_BOUNDARY_MAP.md`.
3. If external crates are not directly compiled, state:
   ```text
   SCR-P0A currently exposes opaque adapter seams and does not claim direct integration.
   ```
4. Add compile-time feature names only if actually implemented:
   - `standalone-reference`
   - `external-adapters`
5. Update README release honesty:
   - reference kernel
   - deterministic evaluator
   - no LLM/network
   - external truth remains adapter-supplied
6. Update `docs/SourceTruthAmbiguityRecord.md`.

## Acceptance gate

- No docs imply full stack integration unless Cargo proves it.
- External boundary scan passes.

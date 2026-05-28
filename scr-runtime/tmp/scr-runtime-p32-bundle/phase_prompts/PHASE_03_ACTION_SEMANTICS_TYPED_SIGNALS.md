# Phase 03 — Proposed-action semantics and typed signal discipline

## Goal

Make SCR evaluate the proposed action and requested effect materially.

## Tasks

1. Remove evaluator dependence on `evidence_ref.ref_kind == "signal"` or `ref_value` token scanning.
2. Add deterministic action/effect risk model:
   - read/analyze/verify + no mutation = low materiality
   - prepare patch / repair packet = moderate materiality
   - apply patch / mutate / release / generated release artifact = high materiality
   - quarantine/block release = protective but material
3. Map proposed action/effect into axes/pressures/candidates.
4. Add typed `control_signals` where explicit signals are needed.
5. Legacy fixtures may keep signal-like concepts only through `scr-audit-adapter`, which must convert fixture signal fields into typed `ControlSignalV1`.
6. Add tests proving:
   - same signals + different action/effect can produce different result,
   - high-risk action without authority/evidence/rollback cannot approve,
   - low-risk advisory can proceed with weaker basis,
   - release action is stricter than analyze.

## Acceptance gate

- Static grep proves no opaque-ref signal scanning.
- Unit tests prove action/effect semantics materially affect decision.
- Fixture adapter is the only place where legacy signal fixtures are translated.

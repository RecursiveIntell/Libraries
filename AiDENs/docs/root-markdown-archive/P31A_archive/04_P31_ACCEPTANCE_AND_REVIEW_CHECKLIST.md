# P31 Acceptance and Review Checklist

## Hard pass/fail gates

- [ ] A single selected crate/module owns the P31 implementation.
- [ ] Targeted `cargo test` passes for that crate/module.
- [ ] `cargo fmt` passes for touched Rust files.
- [ ] `BoundaryCompilerProfileV1` exists.
- [ ] `ParseReceiptV1` exists.
- [ ] `RepairReceiptV1` exists.
- [ ] `TreatmentIntegrityReceiptV1` exists.
- [ ] `BoundaryDecisionV1` exists.
- [ ] `BoundaryCompileResultV1` exists.
- [ ] Strict JSON parse path detects duplicate keys before conversion to ordinary `serde_json::Value`.
- [ ] `{"a":1,"a":2}` is not silently accepted as `{"a":2}`.
- [ ] Malformed JSON returns Reject with parse receipt.
- [ ] Accepted JSON returns canonical digest.
- [ ] Unknown fields can be rejected/quarantined by profile policy.
- [ ] Number/string/null coercion is rejected by default.
- [ ] Resource ceilings are enforced at least for bytes and nesting depth.
- [ ] NoRepair cannot return RepairedAccept.
- [ ] RepairReceiptV1 is emitted only when a real repair happens.
- [ ] Treatment-critical missing/touched paths produce TreatmentIntegrityReceiptV1.
- [ ] P31 report exists at `docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md`.
- [ ] The report confirms no v11B graph/region/subtraction scope was added.

## Review questions

1. Did Codex select an existing crate when one was clearly available and buildable?
2. Did Codex avoid root-workspace breakage from unrelated missing path dependencies?
3. Are receipts data-bearing, or are they just empty status wrappers?
4. Does the parse receipt exist for both accepted and rejected inputs?
5. Does canonicalization have an honest profile name?
6. Are schema validation limitations explicitly documented?
7. Are all errors explicit enough for a future conformance harness?
8. Does the implementation make P32 easier, or did it hard-code itself into a corner?

## Immediate reject conditions

Reject the pass if any of these occur:

- duplicate keys are accepted through last-write-wins behavior;
- a repair receipt is emitted without real repair;
- tests only assert happy-path parsing;
- no targeted cargo test was run;
- graph compiler or region runtime code was added;
- Codex rewrote unrelated architecture/spec documents instead of implementing code;
- user-visible completion is claimed without the P31 report.

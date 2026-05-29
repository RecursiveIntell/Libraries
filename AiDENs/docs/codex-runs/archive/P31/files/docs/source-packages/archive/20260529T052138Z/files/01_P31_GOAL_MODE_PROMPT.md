/goal Complete P31 — v11A Boundary Compiler Microkernel without stopping until a targeted Rust crate/module implements strict JSON boundary compilation with v11A receipts, required fixture tests pass, and docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md is written.

Read first if present:
- CANONICAL_STACK_SPEC_V11A_CONSTITUTIONAL_ARTIFACT_RUNTIME_CORE.md
- V11_PLUS_ARTIFACT_FAMILY_INDEX.md
- V11_PLUS_CONFORMANCE_AND_RELEASE_BAR.md
- json_research.md
- control.md
- Coding-research-next-codex-context-20260511.report.md

Hard scope:
- Implement only v11A boundary compiler microkernel.
- Do not implement v11B graph compiler, region runtime, recursive inference, causal attribution, or lawful subtraction.
- Do not perform whole-repo cleanup except what is strictly necessary for the chosen crate/module.

Required end state:
- One selected crate/module builds with targeted cargo test.
- Types exist: BoundaryCompilerProfileV1, ParseReceiptV1, RepairReceiptV1, TreatmentIntegrityReceiptV1, BoundaryDecisionV1, BoundaryCompileResultV1.
- compile_json_boundary or equivalent exists.
- Strict duplicate-key detection is implemented before JSON Value semantics can erase duplicates.
- Every accepted/rejected/quarantined result has a parse receipt.
- NoRepair never emits fake RepairedAccept.
- Treatment-critical missing/touched paths emit TreatmentIntegrityReceiptV1.
- Accepted input receives stable canonical digest.
- Required tests pass:
  1. valid_minimal_json_is_accepted_and_gets_canonical_digest
  2. malformed_json_is_rejected_with_parse_receipt
  3. duplicate_key_is_rejected_or_quarantined
  4. duplicate_key_is_not_silently_last_write_wins
  5. unknown_field_policy_rejects_surprise_structure
  6. string_number_coercion_is_rejected_by_default
  7. resource_ceiling_rejects_large_input
  8. resource_ceiling_rejects_deep_input
  9. treatment_critical_missing_path_requires_integrity_receipt
  10. no_repair_policy_never_emits_fake_repair_accept
  11. canonical_digest_is_stable_for_equivalent_object_ordering
  12. accepted_and_rejected_results_both_have_receipts
- Write docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md with files changed, commands run, results, blockers, and P32 follow-up.

Validation loop:
- Prefer cargo fmt/test on the selected crate only.
- If root workspace is broken due to unrelated path dependencies, switch to --manifest-path for the chosen crate and document it.
- Keep a short progress log in the P31 report.

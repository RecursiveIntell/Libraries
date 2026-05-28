# Next Codex Task Matrix - P20

Do not treat this as a feature promise. It is a phase queue with explicit labels.

| Priority | Phase | Label | Task | Required proof |
|---|---:|---|---|---|
| P0 | 02 | supported | Documentation honesty | `docs/p20/DOCS_CODE_TRUTH_REPORT.md` and patched active docs |
| P0 | 03 | supported | Contract ownership inventory | `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md` and `.json` |
| P0 | 04 | supported | P20 scanner/verify gate integration | `scripts/p20_verify.sh` emits scanner reports and fails on configured hard findings |
| P0 | 05 | supported | Provider capability matrix | `docs/p20/PROVIDER_CAPABILITY_MATRIX.md` and tests |
| P0 | 06 | partial/proved | Runner vertical slice | fixture test proving config -> provider -> tool -> result -> receipts |
| P0 | 07 | partial/proved | Canonical adapter proof | delegation tests for claimed memory/kernel/verification/repair surfaces |
| P0 | 08 | partial/proved | Agency/influence governance | eval pass and runner receipts |
| P0 | 09 | partial/proved | Reference interpreter closeout | `crates/aidens-testkit/tests/phase_09_reference_hostile_tests.rs` |
| P0 | 10 | deferred | Final audit bundle | `scripts/p20_verify.sh` and `scripts/p20_generate_audit_bundle.sh` pass |

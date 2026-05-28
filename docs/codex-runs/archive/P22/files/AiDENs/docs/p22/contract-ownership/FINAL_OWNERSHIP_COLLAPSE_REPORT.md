# Final Ownership Collapse Report

SOURCE BASIS: 2026-04-28

## Result

The AiDENs Contract Ownership Collapse run is complete at the proof level required by the v2 phase pack.

`crates/aidens-contracts` no longer owns canonical stack truth semantics for the duplicate P0 type families, digest/identity law, canonical schema generation, tool runtime truth, repair/control truth, runtime-view truth, kernel/region truth, federation, mechanism, or verification artifacts.

## Final Proof Checklist

| Requirement | Result | Evidence |
|---|---|---|
| Generated duplicate-type gate passes | PASS | `docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv`, `.codex_evidence/contract_ownership/07/phase_verify_final_initial.txt` |
| Digest-law gate passes | PASS | `.codex_evidence/contract_ownership/final/assert_no_local_canonical_digest_law.txt` |
| Schema-scope gate passes | PASS | `.codex_evidence/contract_ownership/final/assert_schema_generation_scope.txt` |
| Tool-runtime delegation gate passes | PASS | `.codex_evidence/contract_ownership/final/assert_tool_runtime_delegation.txt` |
| Stale-doc/source-basis gate passes | PASS | `.codex_evidence/contract_ownership/final/assert_docs_source_basis_current.txt` |
| No `aidens-contracts` split occurred | PASS | `.codex_evidence/contract_ownership/final/assert_no_crate_split.txt` |
| No compatibility ledger rows exist | PASS | `docs/contract-ownership/COMPATIBILITY_LEDGER.md` |
| Wrapper/backpointer gate passes | PASS | `.codex_evidence/contract_ownership/final/assert_wrapper_backpointers.txt` |
| Existing AiDENs build/test gates pass | PASS | `.codex_evidence/contract_ownership/07/cargo_check_workspace.txt`, `.codex_evidence/contract_ownership/07/cargo_test_workspace.txt` |
| `aidens-contracts` owns no canonical stack truth semantics | PASS with quarantined residual P1/P2 ambiguity | `docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md`, `docs/contract-ownership/FINAL_UNRESOLVED_RISKS.md` |

## P0 Duplicate Collapse

The following names are no longer locally defined in `aidens-contracts`; where surfaced, they are explicit canonical re-exports:

| Type | Canonical owner |
|---|---|
| `AttestationEnvelopeV1` | `attestation-exchange` |
| `SharedDispositionV1` | `federated-settlement` |
| `SettlementCaseV1` | `federated-settlement` |
| `TheoryRefuterSuiteV1` | `mechanism-runtime` |
| `TheoryVersionV1` | `mechanism-runtime` |
| `HypothesisLibraryV1` | `mechanism-runtime` |

Final duplicate findings file:

```text
docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv
```

The file contains only the CSV header after final generation.

## Digest and Identity

Canonical identity and digest semantics are owned by `stack-ids`.

AiDENs retains only non-authoritative display helpers such as `DisplayDigestV1` and display digest functions. These are not artifact identity law.

## Schema Ownership

Canonical artifact-family schema generation belongs to owner crates and `contract-schema-gen`.

AiDENs generated schemas are limited to AiDENs-local display/report/operator/product/schema-governance DTO families. Final generated schema count is 58.

## Tool, Repair, Runtime, Kernel, and Subtraction Wrappers

Tool runtime truth is grounded in `llm-tool-runtime`. AiDENs tool DTOs are display/report wrappers with canonical backpointers.

Repair/control truth is grounded in `verification-control` or quarantined. Display reports can carry `StackBoundaryRepairRecordId` and `StackControlReceiptId`, but empty ID vectors are not proof of owner record creation.

Runtime view/widening/degradation reports carry canonical backpointers to `knowledge-runtime`, `semantic-memory`, and `forge-memory-bridge`.

Kernel/region and support/subtraction DTOs are display/report wrappers with typed stack IDs or canonical backpointers. Remaining concrete owner-record production decisions are quarantined.

## Final Proof Files

- `docs/contract-ownership/FINAL_TYPE_OWNERSHIP_INVENTORY.csv`
- `docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md`
- `docs/contract-ownership/FINAL_GATE_OUTPUTS.md`
- `docs/contract-ownership/FINAL_AUDITOR_HANDOFF.md`
- `docs/contract-ownership/FINAL_UNRESOLVED_RISKS.md`
- `docs/contract-ownership/DEPENDENCY_SOURCE_OF_TRUTH.md`

## Build and Test

```text
cargo check --workspace: passed
cargo test --workspace: passed
```

No final build/test step was skipped.

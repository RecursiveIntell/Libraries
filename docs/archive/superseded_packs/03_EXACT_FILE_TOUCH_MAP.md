# 03. Exact File Touch Map

Every file that must be created or modified, grouped by crate/directory, with the issue(s) that require the change.

## forge-pilot/

| File | Action | Issues |
|------|--------|--------|
| `Cargo.toml` | Modify: `default = ["governance"]` | GOV-003 |
| `src/governance_gate.rs` | Rewrite: accept store/runtime, query governance state, return real observation | GOV-001, GOV-002 |
| `src/observe.rs` | Modify: pass store/runtime to `observe_governance()` | GOV-001 |
| `src/loop_runner.rs` | Modify: call `gate_execution()`, populate `governance_receipt`, wire Blocked/AdvisoryOnly | GOV-002 |
| `src/error.rs` | Modify: add `///` doc comment to undocumented public function | DOC-001 |
| `src/types.rs` | Modify: promote String fields to typed enums | TYPE-002 |
| `src/receipts.rs` | Modify: add execution-context backpointers; typed repair fields | EXEC-001, TYPE-002 |
| `tests/governance_observation_tests.rs` | Create: governance observation integration test | GOV-001 |
| `tests/governance_gating_tests.rs` | Create: governance gating behavior test | GOV-002 |

## forge-memory-bridge/

| File | Action | Issues |
|------|--------|--------|
| `src/transform.rs` | Modify: replace `unwrap_or_else(EpisodeId::generate)` at lines 499 and 644 with error returns | ID-001 |
| `src/legacy.rs` | Modify: add quarantine handling for missing episode identity | ID-001 |
| `src/error.rs` | Modify: add `MissingEpisodeIdentity` variant to `BridgeError` | ID-001 |
| `tests/episode_identity_regression.rs` | Create: regression test for legacy import without episode_id | ID-001 |

## knowledge-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/obs/trace.rs` | Modify: add `valid_as_of`, `recorded_as_of`, `temporal_mode` to `RuntimeQueryProvenanceV1` | TMP-001 |
| `src/runtime/core.rs` | Modify: populate temporal coordinates in provenance construction | TMP-001 |
| `src/views.rs` | Modify: replace String fields with typed enums from owner crates | TYPE-003 |
| `Cargo.toml` | Modify: add deps for typed enums if needed | TYPE-003 |
| `tests/cross_crate_proof.rs` | Modify: add temporal provenance fidelity assertions | TMP-001 |

## semantic-memory/

| File | Action | Issues |
|------|--------|--------|
| `src/embedder.rs` | Modify: `#[cfg(test)]` on deprecated `new()` or remove entirely | SAFE-001 |
| `src/lib.rs` | Modify: wrap HNSW operations in `spawn_blocking` | CONC-001 |
| `src/hnsw.rs` | Modify: adjust for `spawn_blocking` wrapping | CONC-001 |
| `benches/search_bench.rs` | Create: criterion benchmarks for search | TEST-003 |

## semantic-memory-forge/

| File | Action | Issues |
|------|--------|--------|
| `src/v9.rs` | Modify: add backpointer fields to `ExecutionContextV1` | EXEC-001 |
| `src/envelope.rs` | Modify: add `///` doc comment to undocumented public function | DOC-001 |

## effect-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `EffectValidationResult`; add doc comment | GATE-001, DOC-001 |
| `src/effect.rs` | Modify: add timestamp validation in builders; refactor builder API | TYPE-001, API-001 |
| `src/v25.rs` | Review: serde(flatten) implications | API-002 |
| `tests/proptest_builders.rs` | Create: property-based tests for builders | TEST-002 |

## assurance-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `AssuranceValidationResult` | GATE-001 |
| `src/assurance.rs` | Modify: add timestamp validation for critical fields | TYPE-001 |
| `src/certification.rs` | Modify: add timestamp validation for critical fields | TYPE-001 |
| `tests/proptest_builders.rs` | Create: property-based tests for builders | TEST-002 |

## mechanism-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `MechanismValidationResult` | GATE-001 |

## continuity-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `ContinuityValidationResult` | GATE-001 |
| `src/incident.rs` | Modify: add timestamp validation for critical fields | TYPE-001 |
| `src/slo.rs` | Modify: add timestamp validation for critical fields | TYPE-001 |

## authority-delegation/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `AuthorityValidationResult` | GATE-001 |

## attestation-exchange/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `AttestationValidationResult` | GATE-001 |

## constitutional-memory/

| File | Action | Issues |
|------|--------|--------|
| `src/error.rs` | Modify: rename `ValidationResult` to `ConstitutionalValidationResult` | GATE-001 |

## llm-tool-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/contracts.rs` | Modify: document execution-context backpointer contract | EXEC-001 |
| `tests/core_tests.rs` | Expand: add dispatch, receipt, error classification tests | TEST-001 |
| `tests/dispatch_tests.rs` | Create: tool dispatch routing tests | TEST-001 |
| `tests/receipt_roundtrip_tests.rs` | Create: receipt serialization round-trip tests | TEST-001 |

## contract-schema-gen/

| File | Action | Issues |
|------|--------|--------|
| `src/lib.rs` | Modify: add temporal provenance schema; update execution context schema | TMP-001, EXEC-001, TYPE-002 |

## profile-runtime/

| File | Action | Issues |
|------|--------|--------|
| `src/compose.rs` | Modify: add `///` doc comment to undocumented public function | DOC-001 |

## verification-policy/

| File | Action | Issues |
|------|--------|--------|
| `src/permit.rs` | Modify: add `///` doc comment to undocumented public function | DOC-001 |

## kernel-conformance/

| File | Action | Issues |
|------|--------|--------|
| `tests/semantic_oracle_fixtures.rs` | Create: golden fixtures for temporal/import/widening semantics | SEM-001 |

## scripts/

| File | Action | Issues |
|------|--------|--------|
| `release_gate_set.py` | Modify: add `cargo check --workspace`; consume lane manifest | GOV-004 |
| `check_no_prod_panics.sh` | Modify: update for embedder.rs fix; consume lane manifest | SAFE-001, SURF-002 |
| `prod_panic_allowlist.json` | Modify: remove stale embedder.rs entry | SAFE-001 |
| `check_public_type_drift.py` | Modify: update after ValidationResult renames; consume lane manifest | GATE-001, SURF-002 |
| `public_type_drift_allowlist.json` | Modify: remove resolved entries | GATE-001 |
| `check_public_api_docs.py` | Modify: consume lane manifest | DOC-001, SURF-002 |
| `check_doc_truth.sh` | Modify: align to current pack narrative | DOC-002 |
| `check_repo_surface.sh` | Modify: scope to shipped pack | SURF-001 |
| `check_commit_permit_paths.py` | Modify: make existence-aware; consume scope manifest | CHECK-001 |
| `check_mirror_discipline.sh` | Modify: restore or remove sync script expectation | OPS-001 |
| `check_hotspot_budgets.sh` | Modify: scope to shipped pack | OPS-002 |
| `check_pack_truth.sh` | Modify: match actual artifact set | PACK-001 |
| `print_supported_lane.py` | Modify: consume lane manifest | SURF-002 |
| `lane_manifest.json` | Create: single source of truth for lane membership | SURF-002, GOV-004 |
| `manifest/scope_manifest.json` | Create: scope/path manifest for checkers | CHECK-001 |

## Root / docs /

| File | Action | Issues |
|------|--------|--------|
| `Makefile` | Modify: add `cargo check --workspace` target | GOV-004 |
| `SUPPORT_PROFILE.md` | Modify: align to actual gate proof surface | GOV-004 |
| `README.md` | Modify: align release claims | GOV-004 |
| `PACK_README.md` | Modify: align release claims | GOV-004, DOC-002 |
| `RELEASE_CHECKLIST.md` | Modify: align release claims | GOV-004, DOC-002 |
| `CONFORMANCE_GATES.md` | Modify: align to current gate set | DOC-002 |
| `MASTER_ISSUE_MATRIX.md` | Replace: with this pack's tensor | DOC-002 |
| `AGENTS.md` | Create: front-door agent instructions | SURF-001 |
| `PACK_MANIFEST.json` | Modify: match actual artifact set | PACK-001 |
| `release/closeout_receipt_v1.json` | Create: release receipt | SURF-001 |
| `docs/README.md` | Create: docs directory README | SURF-001 |
| `docs/closeout_v21_v24/governance_surface_decision_table.md` | Create: governance surface decision table | DOC-001 |
| `docs/module_budget_exceptions.md` | Create: module budget exceptions | OPS-002 |
| `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh` | Create or remove expectation | OPS-001 |

## Summary Counts

- Files to **modify**: ~55
- Files to **create**: ~18
- Crates touched: 18 of 30
- Scripts touched: 13

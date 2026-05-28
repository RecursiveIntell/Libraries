# P29 Support Traceability

Status: Phase 20 converged; final labels remain pending Phase 21 command bar, strict package, and extracted package self-replay.

| Support surface | Required P29 evidence | Current state |
|---|---|---|
| Package repair | Phase 00-03 reports, verifier scripts, manifest validator, final package sidecars | verifier and manifest evidence present; final package sidecars pending |
| v11A local release candidate | Phase 12-15 reports, receipts, operator contracts, proof/degradation checks | implemented for declared supported-local coding-agent path; final package gate pending |
| v11B executable seed | Phase 17-19 reports and seed tests | implemented as advisory executable seed; no completion claim |
| v11C reserved only | Support profile and forbidden-claim assertions | reserved-only posture asserted; final package gate pending |

No support label in this file is final until `P29_STATUS_EVIDENCE_MANIFEST.json` records passing final gates. The allowed final labels are exactly `p29-package-repaired`, `p29-supported-local-plus`, `v11A-local-release-candidate`, `v11B-executable-seed`, and `v11C-reserved-only`.

## P29 v11A Supported-Local Trace

| Required surface | Evidence carrier | Current state |
|---|---|---|
| Artifact envelope | `coding-agent-report.json#/v11a_evidence/artifact_envelope` | implemented for `run-coding-agent` |
| Execution context | `coding-agent-report.json#/v11a_evidence/execution_context` | implemented for local-tools-only route |
| Operator contract | `coding-agent-report.json#/v11a_evidence/operator_contract` | implemented for `aidens.runner.turn` |
| Input/output manifest | `coding-agent-report.json#/v11a_evidence/input_manifest` and `#/output_manifest` | implemented |
| Invocation receipt | `coding-agent-report.json#/v11a_evidence/operator_invocation_receipt` | implemented when tool receipt refs exist |
| Proof/debt state | `coding-agent-report.json#/v11a_evidence/proof_profile` and `#/proof_debt_ledger` | implemented |
| Semantic/view disclosure | `coding-agent-report.json#/v11a_evidence/semantic_state` and `#/view_disclosure` | implemented |
| Completion gate | `coding-agent-report.json#/v11a_evidence/completion_gate` | blocks/degrades when receipts or proof state are missing |

## Module Ownership Boundaries

| Module surface | Owner boundary | P29 containment evidence |
|---|---|---|
| `crates/aidens-cli/src/lib.rs` | CLI facade and command assembly only; agent/package internals remain split modules. | `scripts/assert_p29_cli_megafile_containment.py` |
| `crates/aidens-contracts/src/lib.rs` | Re-export facade only; artifact, boundary, execution, operator, proof, and semantic DTOs remain split modules. | `scripts/assert_p29_contracts_megafile_containment.py` |
| Canonical identity/digest/proof/memory/runtime truth | Delegated to sibling owner crates named in `SOURCE_BASIS.md`. | P29 verifier and known limitations register |

The v11A surface above is a local release-candidate path for the declared supported-local coding-agent lane only. It is not a cloud, autonomous, full v11B, or v11C claim.

## P29 v11B Executable Seed Trace

| Required seed | Evidence carrier | Current state |
|---|---|---|
| Right-graph misuse tests | `p29_right_graph_misuse_is_blocked_for_storage_or_unbounded_regions` | implemented |
| Region contract seed | `RegionContractV1` / `CompiledRegionGraphV1` | implemented as reserved/advisory seed |
| Boundary message/receipt seed | `RegionBoundaryMessageV1` / `RegionBoundaryReceiptV1` | implemented as advisory seed; cannot admit runtime payload |
| Residual/syndrome/convergence seed | `KernelResidualReportV1`, `KernelSyndromeReportV1`, `ConvergenceReportV1` | implemented with stop-rule evidence |
| Lawful subtraction seed | `SupportCoreV1`, `RemovalFrontierV1`, `SubtractionPlanV1` | implemented as dry-run append-only reduction seed |

All v11B surfaces remain `AdvisoryOnly` or `ReservedDraft` unless a future canonical owner activates them. P29 does not claim full v11B runtime completion.

## Final Label Readiness

| Final label | Phase 20 status | Remaining blocker |
|---|---|---|
| `p29-package-repaired` | candidate evidence present | final strict package and extracted replay |
| `p29-supported-local-plus` | candidate evidence present for the declared local operator/coding-agent path | final command bar and package replay |
| `v11A-local-release-candidate` | candidate evidence present only for `run-coding-agent` supported-local path | final command bar and package replay |
| `v11B-executable-seed` | seed evidence present | final package inclusion; no completion claim |
| `v11C-reserved-only` | reserved-only evidence present | final forbidden-claim scan |

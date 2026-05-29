# Artifact and Schema Registry

P20 status: historical P00-P19 design registry, not the active contract
ownership source. The current Phase 03 ownership inventory is
`docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md` and
`docs/p20/CONTRACT_OWNERSHIP_INVENTORY.json`. If a row below names a type that
Phase 03 removed or renamed, the Phase 03 inventory is authoritative for the
current code.

This registry names the artifact families Codex must create or harden across the super pass. Every wire-visible family must have:

- owner crate;
- versioned Rust type;
- generated JSON Schema;
- golden fixture;
- serde round-trip test;
- compatibility policy;
- receipt/provenance linkage when meaning-bearing.

## Core artifact families

| Family | First pass | Owner | Purpose | Schema policy |
|---|---|---|---|---|
| `AidensRunContextV1` | P05 | `aidens-contracts` | app run/attempt/config identity that converts to stack TraceCtx/AttemptId; not canonical execution provenance | transitive backward compatible within major |
| `PoisonReceiptRecordV1` | P05 | `aidens-receipts` / `aidens-contracts` | durable record for unreadable receipt transaction lines | raw line digest and reason code required |
| `ExecutionLineageGraphV1` | P05 | `aidens-receipts` / `aidens-contracts` | queryable receipt parent/child graph | edges must reference immutable receipt ids |
| `ProviderBackendMatrixV1` | P02 | `aidens-contracts` / `aidens-provider-kit` | executable provider backend matrix | provider kind cannot imply native capability |
| `ProviderReadinessReceiptV1` | P02 | `aidens-contracts` / `aidens-provider-kit` | configured/executable provider truth | unavailable backends must carry reason codes |
| `ProviderRouteReceiptV2` | P02 | `aidens-provider-kit` | executable provider route truth | no native flag without backend proof |
| `ProviderCertificationFixtureV1` | P02 | `aidens-contracts` / `aidens-provider-kit` | provider readiness/route certification cases | fixture expectations must keep native flag false without executable tool loop |
| `TurnExecutionPlanV1` | P03 | `aidens-contracts` / `aidens-runner` | per-turn provider/tool/budget plan | mode must distinguish native, parser fallback, no-tools, provider-unavailable, and budget-exhausted |
| `TurnReceiptV1` | P03 | `aidens-contracts` / `aidens-runner` | final turn state and evidence links | degraded/blocked stop states must be explicit |
| `ToolCallRequestV1` | P03 | `aidens-contracts` / `aidens-runner` | provider requested tool call | parser fallback requests must be degraded |
| `ToolCallResultV1` | P03 | `aidens-contracts` / `aidens-runner` | tool result returned to provider | links to invocation receipt and input/output digests |
| `StopRuleReceiptV1` | P03 | `aidens-contracts` / `aidens-runner` | final-output, blocked, retry, recursive, and deadline stop evidence | every non-final stop reason is receipt-bearing |
| `BudgetExhaustionReceiptV1` | P03 | `aidens-contracts` / `aidens-budget-kit` / `aidens-runner` | tool-call/retry/deadline budget stop evidence | exhaustion must block/degrade rather than truncate |
| `CapabilityGateDecisionV1` | P04 | `aidens-contracts` / `aidens-tool-kit` | per-tool declared/registered/executable/exposed/hidden/blocked gate evidence | blocked side-effect decisions must carry approval request or reason code |
| `ToolExposurePlanV2` | P04 | `aidens-tool-kit` | lifecycle-aware tool exposure | strict schema; no unknown lifecycle values |
| `ApprovalRequestV1` | P04 | `aidens-contracts` / `aidens-permit-kit` / `aidens-cli` | operator approval request for side-effect tool exposure or invocation | scope must name risk, tool, sandbox root, and optional run/attempt family |
| `ApprovalDecisionV1` | P04 | `aidens-contracts` / `aidens-permit-kit` / `aidens-cli` | approval or denial outcome | approved decisions must embed scoped permit grant; denials carry reason codes |
| `PermitGrantV1` | P04 | `aidens-contracts` / `aidens-permit-kit` | explicit scoped permit for risk-bearing tool use | no profile-default grants; scope is risk/tool/sandbox/time/run-attempt optional |
| `PermitUseReceiptV1` | P04 | `aidens-contracts` / `aidens-permit-kit` / `aidens-tool-kit` | evidence that a scoped permit matched or was denied/revoked | invocation/exposure can reference use receipt id |
| `ToolInvocationReportV1` | P03/P05 | `aidens-tool-kit` | tool attempt display/report surface | canonical runtime receipt content stays library-owned |
| `BoundaryCompileRequestV1` | P06 | `aidens-contracts` / `aidens-boundary-kit` | boundary compiler input, schema, repair policy, and treatment-critical field declaration | compiler inputs must preserve repair and treatment-integrity policy |
| `BoundaryCompileOutcomeV1` | P06 | `aidens-boundary-kit` | syntax/shape/repair/canonicalization output | strict dialect + repair receipt |
| `SchemaValidationReceiptV1` | P06 | `aidens-contracts` / `aidens-boundary-kit` / `aidens-tool-kit` | schema validation evidence before meaning-bearing use or tool dispatch | invalid input must block before invocation and include error paths |
| `JsonRepairReceiptV2` | P06 | `aidens-contracts` / `aidens-boundary-kit` | degraded repair provenance for fences, substring extraction, and treatment-critical integrity warnings | repair may not silently alter treatment-critical fields |
| `CanonicalDigestV1` | P06 | `aidens-contracts` | SHA-256 digest over deterministic canonical JSON or explicit text canonicalization | no non-cryptographic digest for evidence-bearing content |
| `DuplicateKeyFindingV1` | P06 | `aidens-contracts` / `aidens-boundary-kit` | duplicate JSON object key finding before serde value collapse | duplicate keys are blocked, not last-write-wins |
| `ArtifactFamilyRegistryV1` | P07 | `aidens-contracts` | type-owned artifact family registry for schema generation and compatibility checks | new wire-visible families must be registered with owner, version, schema path, fixture, and policy |
| `GeneratedSchemaManifestV1` | P07 | `aidens-contracts` / `aidens-cli` | deterministic manifest for generated Rust-owned JSON schemas | manifest entries are generated from the registry and schema digests |
| `SchemaCompatibilityReportV1` | P07 | `aidens-contracts` / `aidens-cli` | schema check output for missing, unregistered, or drifted schemas | unregistered families or same-version schema drift fail the gate |
| `MigrationPlanV1` | P07 | `aidens-contracts` | expand/backfill/flip-read/contract migration law for artifact families | interpretation changes require a new major version |
| `BackfillReceiptV1` | P07 | `aidens-contracts` | receipt that historical fixtures/artifacts stayed readable through a migration path | backfill evidence is append-only and fixture-counted |
| `ReferenceCaseV1` | P08 | `aidens-contracts` / `aidens-testkit` | typed reference case with input, expected output, and semantic coverage metadata | expected semantics are major-version immutable |
| `ReferenceInterpreterReportV1` | P08 | `aidens-contracts` / `aidens-testkit` | reference self-check or production conformance report | report additions are compatible; finding meaning is immutable |
| `DifferentialConformanceFindingV1` | P08 | `aidens-contracts` / `aidens-testkit` | human-readable production/reference mismatch finding | mismatch semantics are immutable |
| `GoldenFixtureManifestV1` | P08 | `aidens-contracts` / `aidens-testkit` | fixture and coverage manifest for reference cases | coverage fields are append-only unless a major version changes semantics |
| `RepoReadReceiptV1` | P10 | `aidens-contracts` / `aidens-tool-kit` / `aidens-receipts` | sandboxed repo file read evidence | path, sandbox root, byte count, and content digest are immutable |
| `RepoListReceiptV1` | P10 | `aidens-contracts` / `aidens-tool-kit` / `aidens-receipts` | sandboxed repo directory listing evidence | listing digest and sandbox path semantics are immutable |
| `PatchProposalV1` | P10 | `aidens-contracts` / `aidens-tool-kit` / `aidens-receipts` | non-mutating patch proposal artifact | `mutates_files=false` is semantic law for proposals |
| `PatchApplyReceiptV1` | P10 | `aidens-contracts` / `aidens-tool-kit` / `aidens-receipts` | approved patch application evidence | touched paths, before/after digests, and permit ids are immutable |
| `CommandRunReportV1` | P10 | `aidens-contracts` / `aidens-tool-kit` | allowlisted local check command report | command argv, stdout/stderr digests, timeout, and exit code are report fields |
| `CodexPacketV1` | P10 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | resumable handoff packet for another agent | current pass, next pass, source map, commands, receipts, blockers, and notes required |
| `SandboxCapabilityTruthV1` | P10 | `aidens-contracts` / `aidens-security-kit` / `aidens-tool-kit` | sandbox root, denied prefixes, env/network/process policy truth | file-write/shell/network must not become default-granted through compatibility drift |
| `JobV1` | P11 | `aidens-contracts` / `aidens-queue-kit` | durable queue job identity | job identity is namespace plus idempotency key plus payload digest, never timestamp alone |
| `QueueLeaseV1` | P11 | `aidens-contracts` / `aidens-queue-kit` | lease owner and expiry evidence for daemon execution | owner, expiry, stolen-from lease, and attempt family are immutable |
| `ScheduleOccurrenceV1` | P11 | `aidens-contracts` / `aidens-schedule-kit` | one-shot schedule occurrence input | occurrence id requires schedule id, occurrence key, and payload digest before recurrence exists |
| `WakeSignalV1` | P11 | `aidens-contracts` / `aidens-wake-kit` | external/internal wake input to queue | source, signal key, and payload digest define idempotency |
| `DaemonNamespaceV1` | P11 | `aidens-contracts` / `aidens-daemon-kit` | daemon-owned queue namespace | namespace owner and idempotency scope are immutable |
| `SafeModeReceiptV1` | P11 | `aidens-contracts` / `aidens-queue-kit` / `aidens-daemon-kit` / `aidens-receipts` | safe-mode transition or risky enqueue block evidence | safe mode blocks new risky jobs while preserving inspection and drain |
| `DuplicateSuppressionReceiptV1` | P11 | `aidens-contracts` / `aidens-queue-kit` / `aidens-receipts` | duplicate logical job suppression evidence | existing job id and idempotency key are immutable |
| `QueueHopReceiptV1` | P11 | `aidens-contracts` / `aidens-queue-kit` / `aidens-receipts` | enqueue/lease/cancel/execute/drain/poison transition evidence | every queue state transition records from/to state and reason |
| `RuntimeViewRequestV1` | P13 | `aidens-contracts` / `aidens-runner` / `aidens-memory-kit` | requested semantic/temporal/entity/causal/control/execution runtime view | view mode, query, and retrieval policy must be explicit |
| `RetrievalPolicyV1` | P13 | `aidens-contracts` / `aidens-memory-kit` / `aidens-cli` | time scope, identity expansion, widening, and fallback policy for retrieval | time-scoped queries cannot silently fall back to timeless retrieval |
| `QueryWideningReceiptV1` | P13 | `aidens-contracts` / `aidens-memory-kit` / `aidens-governance-kit` | alias/scope/time/index widening evidence | alias expansion requires receipt evidence and policy allowance |
| `DegradationEventV1` | P13 | `aidens-contracts` / `aidens-memory-kit` / `aidens-governance-kit` | explicit retrieval degradation or fallback event | fallback and control/execution disclosure must be visible before results |
| `ProjectionDigestV1` | P13 | `aidens-contracts` / `aidens-memory-kit` | deterministic digest for a rebuilt projection under a policy | projection digest must rebuild from authoritative memory/evidence, not a runtime shadow store |
| `ViewDisclosureReceiptV1` | P13 | `aidens-contracts` / `aidens-runner` / `aidens-memory-kit` | disclosure receipt linking request, policy, widening, degradation, projection, and matched claims | execution/control views must remain separated from domain truth unless relation artifacts exist |
| `ReleaseReadinessReportV1` | P14 | `aidens-contracts` / `aidens-cli` | release gate report over docs, examples, public surfaces, and install smoke | false public claims about scaffold surfaces block release |
| `OperatorStatusReportV1` | P14 | `aidens-contracts` / `aidens-cli` | operator-facing status report with doctor truth, blocked modes, and degraded modes | degraded and blocked modes must remain explicit |
| `ExampleAppManifestV1` | P14 | `aidens-contracts` / `aidens-cli` | typed manifest for example configs and profile coverage | unsupported advanced features must be listed, not implied available |
| `InstallSmokeReceiptV1` | P14 | `aidens-contracts` / `aidens-cli` | receipt for new-user install/product smoke steps | failed or skipped operator steps block or degrade release readiness |
| `CompiledRegionGraphV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | bounded right-graph compiled from memory/projection evidence | storage graph cannot execute directly; regions must stay bounded |
| `RegionContractV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | typed region boundary | graph kind, region nodes, boundary nodes, and factor ids required |
| `SyndromeV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | contradiction/high-residual/boundary syndrome display with canonical repair backpointers | canonical repair record ids or explicit recompute reason required |
| `ResidualV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | per-iteration residual evidence | residual threshold and stop-rule evidence required |
| `OracleSliceRequestV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | exact-on-small oracle slice and approximate comparison | agreement or bounded disagreement required |
| `KernelRunDisplayReportV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` | kernel run display report linking graph, convergence, residuals, syndromes, and oracle requests | convergence display cannot be true without explicit canonical stop-rule evidence |
| `ConvergenceReportV1` | P15 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-receipts` | bounded message-passing convergence/degradation report | non-convergence and oscillation must be degraded with terminal stop state |
| `SubtractionPlanV1` | P16 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-memory-kit` | dry-run reduction plan linked to support core, removal frontier, and invariant budget | destructive deletion is false; blocked plans cannot compact |
| `SupportCoreV1` | P16 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-repair-kit` | accepted-claim support core | accepted claim support must be explicit before any reduction |
| `RemovalFrontierV1` | P16 | `aidens-contracts` / `aidens-kernel-kit` | candidate removal boundary with blocked/removable ids | accepted-claim support is blocked unless superseded or quarantined |
| `InvariantBudgetV1` | P16 | `aidens-contracts` / `aidens-memory-kit` | declared replay/as-of/support/receipt/legal retention budget | receipt compaction requires retention policy and approval when audit retention applies |
| `CompactionReceiptV1` | P16 | `aidens-contracts` / `aidens-memory-kit` / `aidens-receipts` | append-only compaction evidence | compaction must link plan, frontier, budget, and history report; destructive deletion is false |
| `HistoryPreservationReportV1` | P16 | `aidens-contracts` / `aidens-memory-kit` / `aidens-receipts` | before/after digest report for declared history budget | as-of query and support invariants must remain preserved or report is degraded |
| `AttestationEnvelopeV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-receipts` | signed external artifact admission | trust-root and subject digest required |
| `TrustRootV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-receipts` | trusted producer key/policy root with revocation state | revocation must downgrade affected admitted artifacts |
| `AdmissionDecisionV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-memory-kit` / `aidens-receipts` | explicit external artifact disposition | no external artifact import without this decision |
| `RemoteOracleReceiptV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-memory-kit` / `aidens-receipts` | advisory import evidence for remote oracle artifacts | remote oracle outputs remain advisory unless locally promoted through governance |
| `TreatyV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-receipts` | federation boundary and settlement policy record | treaties cannot create central truth or remote overwrite rights |
| `SettlementCaseV1` | P17 | `aidens-contracts` / `aidens-delegation-kit` / `aidens-memory-kit` / `aidens-receipts` | remote contradiction/dispute case | local authority must be preserved and direct overwrite prevented |
| `SharedDispositionV1` | P17 | `aidens-contracts` / `aidens-governance-kit` / `aidens-receipts` | federated settlement disposition | shared disposition must disclose local and remote outcomes |
| `MechanismBundleV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` | candidate mechanism with variables, assumptions, causal claims, program, parameters, digests, and replay recipe | mechanism identity and replay semantics are immutable; raw weights alone are insufficient |
| `SimulatorContractV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` | simulator input/output schema, backend, determinism, seed policy, replay command, and digest | replay boundary changes require a new major version |
| `FitRunReportV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` | fit report over mechanism, theory, simulator, dataset, parameters, output digest, score, and replay command | score is not verification; replay linkage is immutable |
| `InvarianceReportV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-governance-kit` | perturbation/environment invariance evidence and causal-identification disclosure | observational equivalence cannot imply causal identification |
| `TheoryRefuterSuiteV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-governance-kit` | falsifiable refuter suite linking cases, fit runs, invariance reports, outcomes, and replay handles | promotion bar requires refuter evidence; no-refutation-found is not final truth |
| `TheoryVersionV1` | P18 | `aidens-contracts` / `aidens-kernel-kit` / `aidens-memory-kit` | versioned theory state with fit/refuter/invariance/governance/supersession links | state, supersession, and promotion basis are immutable |
| `HypothesisLibraryV1` | P18 | `aidens-contracts` / `aidens-memory-kit` | hypothesis/theory/mechanism index with explicit equivalence/alias decisions | observationally equivalent mechanisms remain distinct unless an admitted alias decision exists |
| `CompletionAuditReportV1` | P19 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | final release-bar report over gates, readiness, traceability, package manifest, limitations, and regression debt | completion state and release-bar basis are immutable; deferred horizon surfaces must be disclosed |
| `ReleaseArtifactManifestV1` | P19 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | release package content manifest for schemas, fixtures, examples, scripts, docs, CI, handoffs, and source-basis files | required package paths and content digests are immutable evidence |
| `CrossPassTraceabilityMatrixV1` | P19 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | requirement to pass to crate to tests to artifacts to docs to evidence mapping | unwaived uncovered requirements block completion |
| `KnownLimitationsRegisterV1` | P19 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | current register of partial, deferred, and blocked surfaces | release cannot ship without a current limitations register; incomplete surfaces cannot be labeled healthy |
| `RegressionDebtLedgerV1` | P19 | `aidens-contracts` / `aidens-cli` / `aidens-receipts` | guarded, accepted, deferred, or blocking regression debt with detection paths | blocking debt blocks completion; guarded debt must name tests/scripts that catch regressions |

## Minimum CoreArtifactHeaderV1

```json
{
  "artifact_family": "string",
  "artifact_version": "integer",
  "artifact_id": "string",
  "logical_id": "string|null",
  "content_digest": "sha256-or-blake3",
  "created_record_time": "RFC3339 timestamp",
  "valid_time": { "start": "RFC3339|null", "end": "RFC3339|null", "bounds": "[)" },
  "producer": { "crate": "string", "version": "string", "build_digest": "string|null" },
  "schema_digest": "sha256-or-blake3",
  "parent_artifact_ids": ["string"],
  "receipt_ids": ["string"]
}
```

## Compatibility law

- Shape-compatible additions require defaults and must pass old-reader/new-writer fixtures.
- Interpretation changes require new major version.
- Historical artifacts must remain readable or have an explicit migration/backfill receipt.
- Every generated schema must be meta-validated before release.

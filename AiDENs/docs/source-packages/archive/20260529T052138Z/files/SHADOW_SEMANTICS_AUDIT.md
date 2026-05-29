# Shadow Semantics Audit

Severity:

- **P0**: duplicate truth/evidence/memory/governance/kernel semantics that can corrupt architecture.
- **P1**: duplicate receipts/budget/execution semantics that can cause drift.
- **P2**: app-level duplicate vocabulary that can become display/facade code.
- **P3**: harmless naming overlap.

| Finding | Severity | Source reference | Local surface | Canonical owner | Risk | Required action | Acceptance gate |
|---|---|---|---|---|---|---|---|
| `SHADOW-P0-001` | `resolved` | current contracts source | `aidens-contracts::ArtifactId` | stack-ids::ArtifactId and sibling ID types | Split identity space even though stack-ids says no competing ID newtypes. | Collapsed to canonical stack IDs. | assert_no_shadow_truth; canonical_id_roundtrip |
| `SHADOW-P0-002` | `resolved` | current contracts source | local `ExecutionContextV1` | semantic-memory-forge::ExecutionContextV1 | Local execution context loses canonical TraceCtx/AttemptId/TrialId/queue/degradation semantics. | Removed local canonical execution context; adapters use Forge execution context. | contract_owner_proof; golden_vertical_slice |
| `SHADOW-P0-003` | `resolved` | current contracts source | local evidence/claim/projection/episode DTOs | semantic-memory-forge + forge-memory-bridge + semantic-memory | Creates parallel truth/evidence/projection model. | Removed local truth DTOs; memory adapter uses Forge/bridge/semantic-memory/runtime. | contract_owner_proof; import_atomicity |
| `SHADOW-P0-004` | `resolved` | current memory-kit source | local append-only memory store | semantic-memory::MemoryStore + forge-memory-bridge import path | Can become shadow database. | Replaced with `CanonicalMemoryAdapter` over semantic-memory and knowledge-runtime. | bitemporal_asof_query; release_truth_audit |
| `SHADOW-P0-005` | `resolved` | current contracts/governance source | local verification plan DTO | verification-control/check plans + verification-policy | Local promotion precondition may diverge from canonical verification law. | Removed local plan DTO; governance adapter builds canonical CheckPlan/ControlReceipt. | promotion_denies_without_verification |
| `SHADOW-P0-006` | `resolved` | current contracts/repair source | local repair record DTO | verification-control::BoundaryRepairRecord + Forge retraction/contradiction artifacts | Repair can mutate/supersede truth without canonical repair lineage. | Removed local repair DTO; repair adapter emits canonical records. | repair_record_backpointer |
| `SHADOW-P0-007` | `resolved` | current kernel/contracts source | local kernel receipt DTO | recursive-kernel-core::KernelRun + kernel-execution::ExecutionReport + kernel-oracles | Local kernel receipt can claim convergence/oracle state without canonical execution. | Removed local kernel receipt naming; kernel surfaces use canonical stop reason/report imports and display reports only. | kernel_exact_small_slice; loopy_nonconvergence_degrades |
| `SHADOW-P0-008` | `resolved` | current contracts source | app-surface schema registry metadata | contract-schema-gen + canonical type owners | Could make aidens-contracts look like the schema constitution. | Registry is limited to AiDENs app/display artifact families and labels ownership as `aidens-orchestration`, not canonical stack truth. | release_truth_audit |
| `SHADOW-P1-009` | `resolved` | ~/Coding/Libraries/AiDENs/crates/aidens-receipts/src/lib.rs | old durable receipt envelope wrappers removed | llm-tool-runtime ToolReceipt + ForgeToolReceiptV2 + verification-control ControlReceipt | Receipt envelope may become independent execution evidence ledger. | `aidens-receipts` is now only `CanonicalReceiptLog` plus `ToolReceiptSink`; removed local envelope/store/outbox fixtures and schemas. | adapter_delegation_proof; budget_exhaustion_receipt |
| `SHADOW-P1-010` | `resolved` | current queue/daemon source | lifecycle-only queue log, daemon namespace, duplicate suppression, schedule/wake inputs | AiDENs app lifecycle; Libraries2 job-queue deferred while it depends on forbidden Libraries2 stack-ids | Queue could have become execution/domain truth before golden slice and governance were lawful. | Phase 7 keeps queue/daemon as app lifecycle only, namespaces shared queue roots by owner/root/name, suppresses duplicate schedule/wake storms, and preserves completed jobs across restart without re-enqueue. | daemon_namespace_isolated; schedule_no_duplicate_storm; restart_does_not_reenqueue_completed_jobs |

## Mandatory interpretation

A P0 finding must be resolved or isolated before feature expansion. A P1 finding must be resolved before the golden vertical slice can be trusted.

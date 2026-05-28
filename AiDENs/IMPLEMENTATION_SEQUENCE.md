# Implementation Sequence

The sequence is intentionally conservative. The main failure mode is not lack of
features; it is semantic duplication.

Phases 7 and 8 are blocked until phase 3 (`golden_vertical_slice`) and phase 6
(governance/promotion) pass.

## Phase 00 — Source truth and layout

**Objective:** Establish exact source roots, owner maps, scripts, and stop conditions before code changes.

**Allowed:** No code rewrite except adding handoff docs/scripts/tests skeletons.

**Forbidden:** Must not implement new features.

**Prompt:** `CODEX_PROMPTS/PHASE_00_SOURCE_TRUTH_AND_LAYOUT.md`

## Phase 01 — Contract collapse

**Objective:** Reduce aidens-contracts to app-only/re-export and eliminate P0 duplicate canonical types.

**Allowed:** Edit aidens-contracts, manifests, tests.

**Forbidden:** Must not preserve local truth types as public canonical APIs.

**Prompt:** `CODEX_PROMPTS/PHASE_01_CONTRACT_COLLAPSE.md`

## Phase 02 — Canonical adapter spine

**Objective:** Create/reshape memory/receipt/kernel/governance/tool/provider adapters over canonical crates.

**Allowed:** Edit adapter crates and testkit.

**Forbidden:** Must not implement daemon/scheduler/kernel expansion.

**Prompt:** `CODEX_PROMPTS/PHASE_02_CANONICAL_ADAPTER_SPINE.md`

## Phase 03 — Golden vertical slice

**Objective:** Prove one operator-to-receipt-to-forge-to-bridge-to-memory-to-runtime-to-CLI flow.

**Allowed:** Edit runner/CLI/adapters/testkit.

**Forbidden:** Must not fake the vertical slice with only local mocks.

**Prompt:** `CODEX_PROMPTS/PHASE_03_GOLDEN_VERTICAL_SLICE.md`

## Phase 04 — Failure honesty

**Objective:** Make malformed tools, denied tools, provider failure, budget exhaustion, and fallback produce canonical receipts/degradation.

**Allowed:** Edit runner/provider/tool/receipts/governance adapters.

**Forbidden:** Must not silently swallow failures.

**Prompt:** `CODEX_PROMPTS/PHASE_04_FAILURE_HONESTY.md`

## Phase 05 — Memory/runtime hardening

**Objective:** Replace local memory authority with semantic-memory + knowledge-runtime as-of/widening behavior.

**Allowed:** Edit memory adapter/CLI/tests.

**Forbidden:** Must not add an AiDENs-local memory store as production truth.

**Prompt:** `CODEX_PROMPTS/PHASE_05_MEMORY_RUNTIME_HARDENING.md`

## Phase 06 — Governance/promotion

**Objective:** Enforce verification plans, approval, promotion/refutation/rollback via canonical verification crates.

**Allowed:** Edit governance/delegation/permit/runner/testkit.

**Forbidden:** Must not allow model-only promotion.

**Prompt:** `CODEX_PROMPTS/PHASE_06_GOVERNANCE_PROMOTION.md`

## Phase 07 — Daemon/queue/schedule/wake

**Objective:** Only after phases 03 and 06 pass, reintroduce queue/daemon as app lifecycle only.

**Allowed:** Edit daemon/queue/schedule/wake or adapter to supplemental Libraries2 job-queue if Libraries has no queue owner.

**Forbidden:** Forbidden before golden vertical slice and governance gates.

**Prompt:** `CODEX_PROMPTS/PHASE_07_DAEMON_QUEUE_SCHEDULE_WAKE.md`

## Phase 08 — Kernel/oracle integration

**Objective:** Expose kernel/oracle as adapter over canonical compiler/execution/oracle/conformance crates.

**Allowed:** Edit kernel adapter/testkit only after prior gates.

**Forbidden:** Must not define local kernel receipts or convergence truth.

**Prompt:** `CODEX_PROMPTS/PHASE_08_KERNEL_ORACLE_INTEGRATION.md`

## Phase 09 — Release audit

**Objective:** Run full gates, docs/cargo parity, no-compat ledger, and package readiness.

**Allowed:** Edit docs/tests/scripts only unless gate fixes required.

**Forbidden:** Must not claim finished without commands passing.

**Prompt:** `CODEX_PROMPTS/PHASE_09_RELEASE_AUDIT.md`

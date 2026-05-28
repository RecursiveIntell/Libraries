# P26 Phase Plan

## Phase 00 — Preflight, replay triage, and scope lock

- Read P25 final handoff, P25 final audit, support profile, status, and current package sidecars.
- Triage the P25 extracted package self-replay failure.
- Inventory existing commands, tests, examples, and support labels.
- Emit `handoffs/p26/PHASE_00_REPORT.md`.

Gate: no capability implementation until replay failure is classified as fixed, reproducible external/environment issue, or explicitly quarantined with operator approval.

## Phase 01 — AgentSpecV1 contract and schema

- Define `AgentSpecV1` in the correct AiDENs-local contract layer.
- Include agent identity, support label, profile, memory policy, tool policy, permit policy, verification policy, evidence policy, budget/turn policy, and output policy.
- Generate/check schema.
- Add fixtures and validation tests.
- STOP for operator gate after phase report.

## Phase 02 — PlanActVerifyLoopV1 core

- Implement bounded local loop from `AgentSpecV1`.
- Produce plan, action, verification, and finalization receipts.
- Enforce budget/turn/deadline limits.
- Abstain on unsupported tool/provider/memory conditions.

## Phase 03 — Evidence model and RunBundleV3

- Add `AiDENsRunBundleV3` or compatible extension.
- Capture agent spec digest, trace/attempt/trial IDs, tool receipts, permit receipts, memory evidence, verification results, support labels, replay recipe.
- Maintain backward compatibility or explicit migration from V2.
- STOP for operator gate.

## Phase 04 — Memory-grounded agent lane

- Integrate memory seam fixture as an agent grounding input.
- Use canonical sibling crates for export/import/query evidence.
- Add view/widening/degradation disclosure where supported.
- Prove no AiDENs-local memory truth store is introduced.

## Phase 05 — CodingAgentV1 generalization

- Generalize supported-local coding agent against sandbox roots.
- Required: repo read/list/search, patch proposal, permit-gated patch apply, permit-gated run-checks, inspect/replay.
- Add tests for blocked writes without permit and abstention on ambiguity.
- STOP for operator gate.

## Phase 06 — Repair and abstention behavior

- Add explicit local display artifacts for abstention and repair plans.
- Fail closed on ambiguous JSON, duplicate keys, invalid patches, missing permits, failed checks, unsupported provider/tool paths.
- Do not promote display artifacts to canonical repair truth.

## Phase 07 — CLI and examples

- Add or normalize commands such as `agent validate`, `agent run`, `agent inspect`, `agent doctor`, `agent new` if safe.
- Add examples: basic local agent, memory-grounded agent, coding agent.
- Update operator quickstart.
- STOP for operator gate.

## Phase 08 — Verifier and support claims

- Add P26 verifier checks for AgentSpecV1 schema, loop evidence, memory grounding, coding-agent v1, abstention behavior, support labels, replay.
- Regenerate support profile and status.
- Keep cloud/autonomy/V10 claims deferred.

## Phase 09 — Integration and hostile audit

- Run cargo fmt/check/test/clippy/doc.
- Run P26 verifier.
- Run package validation and package self-replay.
- Produce final audit draft and known limitations.
- STOP for operator gate before final.

## Phase 10 — Final package, handoff, and audit-ready closeout

- Emit final evidence manifest, final auditor handoff, updated docs/codex-runs state.
- Package with strict validation.
- No success claim if package replay failure remains unclassified.

# P26 Codex Run Prompt — Advanced Local Agent Spine

You are executing P26 for AiDENs.

## Objective

Implement the next capability pass for AiDENs: a reusable advanced supported-local agent spine over the existing canonical libraries and AiDENs kits.

This is not a packaging-only pass. It must materially further AiDENs by making advanced local agents easier and safer to create.

## Required outcomes

1. Add `AgentSpecV1` with schema, fixtures, validation, support-tier disclosure, memory policy, tool capability policy, permit policy, verification policy, and evidence policy.
2. Add a bounded `PlanActVerifyLoopV1` that can execute from `AgentSpecV1` using existing local tools, provider/mock routes, memory grounding, permits, and receipts.
3. Add memory-grounded agent support using the canonical memory seam. AiDENs must not create local memory truth.
4. Generalize the local coding-agent lane beyond a single fixture: read/list/search/propose/apply/check/inspect against sandbox roots with permit-gated writes.
5. Add or extend `AiDENsRunBundleV3` to capture run identity, trace/attempt/trial IDs, agent spec digest, memory grounding evidence, tool receipts, permit receipts, verification receipts, abstention/repair records, support labels, and replay instructions.
6. Add explicit abstention and repair-display behavior. Ambiguity, blocked authority, failed verification, and invalid structured outputs must not be faked into success.
7. Add CLI/operator flow for creating, validating, running, inspecting, and doctoring supported-local agents.
8. Fix or precisely quarantine package self-replay failure from P25.
9. Keep V10+ design-only and create a V10-ready boundary map without implementing V10 runtime geometry.

## Hard constraints

- AiDENs is consumer-only with respect to canonical truth.
- Do not invent canonical memory, verification, repair, governance, schema, or execution semantics inside AiDENs.
- Use sibling crates as source of truth wherever possible.
- Do not build cloud provider execution.
- Do not build broad autonomous daemon behavior.
- Do not expand `z.py` except to fix blocker-level replay/root Markdown defects required for P26 validation.
- Do not silently widen schemas, leniently parse hostile JSON, or create compatibility shims.
- Every phase must emit changed files, commands run, validation results, invariant status, and unresolved risks.

## Phase gates

Use every-other-phase human gates. At the end of phases 01, 03, 05, 07, and 09, STOP. Emit the phase report and wait for the operator’s pasted gate injection before continuing.

Starting phase 02, 04, 06, 08, or 10 without the corresponding pasted gate is a failed run.

## Final output required

- `P26_STATUS_EVIDENCE_MANIFEST.json`
- `STATUS.md` and `SUPPORT_PROFILE.md` updated truthfully
- `docs/p26/P26_FINAL_AUDIT_REPORT.md`
- `docs/p26/P26_KNOWN_LIMITATIONS.md`
- `handoffs/p26/FINAL_AUDITOR_HANDOFF.md`
- updated verifier scripts
- examples for AgentSpecV1 and advanced local coding agent
- full validation evidence under `target/p26/audit/`

# P26 Master Packet — Advanced Local Agent Spine

## Mission

Turn AiDENs from a supported-local demo/operator layer into a reusable advanced local agent framework over the canonical libraries.

P26 must create the first real **advanced-agent spine**:

1. `AgentSpecV1`: declarative agent configuration and support-tier contract.
2. `PlanActVerifyLoopV1`: bounded local execution loop with receipts.
3. `MemoryGroundingV1`: canonical memory seam use as an agent grounding input.
4. `CodingAgentV1`: sandboxed local coding agent over repo tools and permits.
5. `AiDENsRunBundleV3`: richer replayable evidence package, backward-compatible or explicitly migrated from V2.
6. `AbstentionReceiptV1` / `RepairPlanDisplayV1`: local display evidence for blocked/ambiguous/failed paths.
7. CLI and examples that make new agents fast to create without widening support claims.

## Success state

After P26, an operator should be able to create a new supported-local agent from an example spec, run it against a sandbox repo or fixture, inspect every tool/action/permit/check receipt, replay its run bundle, and see explicit abstention/repair output when the agent cannot act safely.

## Architectural posture

AiDENs remains a directing/wiring layer. The pass may add AiDENs-local display/support artifacts, but it must not mint canonical truth. All canonically meaningful truth, memory, execution context, repair/governance semantics, and schema law remain delegated to sibling crates.

## P26 is not

- full autonomy,
- cloud runtime,
- V10 runtime geometry,
- federation,
- proof-governed inference,
- a general z.py pass,
- a giant refactor pass.

# P26 Advanced Local Agent Runtime Spec

## PlanActVerifyLoopV1

The loop is bounded and evidence-bearing.

```text
load AgentSpecV1
validate support/provider/tool/memory/permit/evidence policies
create run identity and execution context links
plan next action
validate action authority
execute permitted local action
emit receipt
verify result
continue until success, abstention, repair, or budget stop
emit AiDENsRunBundleV3
```

## Required receipts

- `PlanReceipt`
- `ToolRouteReceipt`
- `ToolCallReceipt`
- `PermitUseReceipt`
- `VerificationReceipt`
- `MemoryGroundingReceipt`
- `AbstentionReceipt`
- `RepairPlanDisplayReceipt`
- `FinalizationReceipt`

## Stop conditions

- success verified;
- unsupported capability;
- missing permit;
- invalid structured output;
- failed verification;
- budget/deadline exhausted;
- ambiguous ownership;
- package/replay validation failure.

## No-go behavior

The loop must not:

- silently retry forever;
- change support tier automatically;
- invent memory truth;
- apply patches without permit;
- hide provider fallback;
- report fake completion.

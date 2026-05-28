# PHASE 06 — Tool, Repair, Runtime-View Wrapper Collapse

## Objective

Collapse remaining high-risk local DTOs into canonical wrappers or quarantine.

## Required actions

### Tool runtime

Inspect local tool surfaces:

- `ToolSchemaV1`
- `ToolDescriptorV1`
- `ToolProviderSchemaV1`
- `ToolExposurePlanV1`
- `ToolCallRequestV1`
- `ToolCallResultV1`
- `ToolInvocationReportV1`

Convert canonical semantics to `llm-tool-runtime` types. AiDENs may retain display/report wrappers only.

### Repair/verification

Inspect:

- `BoundaryRepairReportV1`
- `JsonRepairReportV2`
- `SchemaValidationReportV1`

Ensure repair truth is canonical or quarantined. Display reports must include canonical repair record references.

### Runtime view/widening/degradation

Inspect:

- `RuntimeViewRequestV1`
- `RetrievalPolicyV1`
- `QueryWideningReportV1`
- `DegradationEventV1`
- `ProjectionDigestV1`
- `ViewDisclosureReportV1`

Ensure local types are display/report wrappers with canonical backpointers, not independent policy semantics.

### Kernel/region/subtraction

Inspect region/syndrome/residual/subtraction DTOs. Keep only if explicitly report/display; otherwise quarantine.

## Required gates

```bash
bash scripts/assert_tool_runtime_delegation.sh
bash scripts/assert_wrapper_backpointers.sh
```

## Acceptance

- Tool invocation truth is grounded in `llm-tool-runtime`.
- Repair truth is grounded in verification crates or quarantined.
- Runtime reports have canonical backpointers.
- No standalone local truth semantics remain in wrappers.

## Stop

Stop after this phase and wait for `GUARDRAIL_06_TO_07`.

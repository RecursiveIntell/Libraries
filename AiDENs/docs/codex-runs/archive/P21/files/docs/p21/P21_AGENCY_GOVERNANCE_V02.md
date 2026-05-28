# P21 Agency Governance v0.2

## Goal

Upgrade AiDENs agency policy from a v0.1 heuristic gate into a stronger, eval-backed runtime guard without pretending it is a complete manipulation theorem.

## Required expansion

Expand agency eval cases to cover at least 20 cases across:

- high-impact financial advice;
- legal/medical caution;
- employment/life decisions;
- emotional dependence;
- repeated nudging;
- memory-personalized influence;
- fake urgency;
- single-path recommendation;
- low-quality alternative set;
- tool-output persuasion risk;
- subagent influence aggregation;
- exit/retention manipulation;
- sycophancy/overvalidation;
- vulnerability inference;
- reversibility disclosure;
- external influence source.

## Runner rule

If `agency.enabled = true`, final output and tool-output incorporation must go through agency evaluation and emit receipts/reports.

## Required receipts

Depending on context, high-impact outputs should emit:

- `AgencyPolicyDecisionV1`;
- `InfluenceReceiptV1`;
- `AdviceReceiptV1`;
- `HighImpactRecommendationReceiptV1`;
- `MemoryInfluenceTraceV1`;
- `NudgeCounterV1`;
- `RepeatedSteeringReceiptV1`;
- `ToolOutputPersuasionRiskV1`.

## Acceptance

Agency eval tests must fail if high-impact or personalized outputs bypass agency receipts.

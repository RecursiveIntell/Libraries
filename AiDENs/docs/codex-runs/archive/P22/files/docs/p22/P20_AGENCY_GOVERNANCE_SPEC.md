# P20 Agency and Influence Governance Spec

## Why this is in P20

AiDENs/Recall are stateful, memory-rich, tool-using systems. Once advice is personalized, repeated, scheduled, or tool-mediated, influence is infrastructure. P20 must add a real policy surface, not a prompt-only safety note.

## Required v0.1 scope

Implement an AiDENs boundary policy layer that classifies, gates, and receipts:

- high-impact recommendations;
- consequential advice with a single-path recommendation;
- repeated nudges or repeated steering;
- memory-personalized advice;
- vulnerability/sensitive-signal use;
- tool-output urgency or persuasion risk;
- delegated/subagent influence aggregation;
- scheduled follow-up/nudge attempts.

## Required artifacts

Minimum public types or equivalents:

```text
InfluenceClassV1
AgencyPolicyInputV1
AgencyPolicyDecisionV1
AgencyPolicyOutcomeV1
AdviceEnvelopeV1
DecisionSupportEnvelopeV1
RecommendationTraceV1
InfluenceReceiptV1
AdviceReceiptV1
HighImpactRecommendationReceiptV1
MemoryInfluenceTraceV1
PersonalizationUsePolicyV1
PersonalizationFeatureUseV1
PersuasionBudgetV1
NudgeCounterV1
RepeatedSteeringReceiptV1
DecisionDomainV1
HighImpactGateV1
AlternativeSetV1
TradeoffMatrixV1
ExternalInfluenceSourceV1
ToolOutputPersuasionRiskV1
DelegatedInfluencePolicyV1
InfluenceAggregationReceiptV1
SensitiveSignalRetentionPolicyV1
EphemeralContextReceiptV1
RedactedInfluenceReceiptV1
AgencyIncidentRecordV1
```

## Required gates

- Before answer generation.
- Before memory-personalized response generation.
- Before high-impact recommendation.
- Before repeated nudge/scheduled follow-up.
- Before tool execution if tool output can shape advice.
- Before subagent/delegated influence aggregation.

## Required policy outcomes

- `allow`
- `allow_with_disclosure`
- `require_alternatives`
- `require_user_confirmation`
- `defer_to_professional_or_external_source`
- `block`
- `quarantine`

## Required evals

Use `evals/p20_agency_eval_cases.jsonl`. P20 fails if cases are ignored.

## Prohibited patterns

- Policy only in system prompts.
- Repeated paraphrased nudges that bypass counters.
- Memory personalization without a memory influence trace.
- Decorative alternatives where only one option is made viable.
- Tool-origin urgency accepted without classification.
- Emotional dependence hooks or exit-resistance behavior.

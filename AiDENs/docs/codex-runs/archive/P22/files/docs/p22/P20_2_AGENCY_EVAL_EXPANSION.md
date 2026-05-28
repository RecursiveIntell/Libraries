# P20.2 Agency Eval Expansion

## Required eval classes

- high-impact single-path recommendation;
- sensitive memory personalization;
- repeated nudge / paraphrase budget bypass;
- tool-origin urgency or scarcity;
- delegated/subagent aggregate pressure;
- relational exit resistance / guilt hook;
- user-requested manipulation;
- decorative alternatives;
- sycophantic overvalidation.

## Eval file

`evals/p20_agency_eval_cases.jsonl` is required and must be loaded by tests.

## Required per-case fields

```json
{
  "id": "case-id",
  "risk_surface": "high_impact_advice",
  "input": {},
  "expected_policy": "require_alternatives",
  "required_receipts": ["InfluenceReceiptV1", "AgencyPolicyDecisionV1"],
  "forbidden_behavior": ["single_path_recommendation"]
}
```

## Receipt assertions

Tests must assert both:

- policy outcome label;
- required receipt schema names;
- blocked/forbidden behavior codes.

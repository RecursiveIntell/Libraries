# Phase 1 — Deterministic Core Types

Implement deterministic core types.

Create:

```text
crates/scr-kernel/
crates/scr-reference/
```

Minimum types:

```text
ScoreBps
WeightBps
ExternalArtifactRef
ControlEvaluationInputV1
ProposedAction
RequestedEffect
Domain
ScoreAxesV1
DerivedPressuresV1
ControlAction
RejectedActionV1
ReasonCode
HardRuleResultV1
AuthorityBasisV1
EvidenceBasisV1
ControlDecisionReceiptV1
ScrError
```

## Hard requirements

- Scores are integer/fixed-point only.
- `ScoreBps` validates 0..=10000.
- `WeightBps` validates 0..=10000.
- Public constructors reject invalid values.
- No durable score artifact contains `f32` or `f64`.
- No public decision path returns `bool`.
- External refs are boundary references, not canonical ownership claims.
- Rust types are source of truth for generated schemas.

## Tests

Add tests for:

- score bound validation
- serialization round-trip
- invalid score rejection
- receipt required shape
- no naked decision bool API by static assertion script

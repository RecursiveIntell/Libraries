# P20 Control-Plane and Execution Evidence Spec

## Control-plane law

Routing and control decisions are first-class auditable artifacts. P20 must ensure the following decisions are recorded where implemented:

- turn arbitration;
- tool exposure;
- provider selection/fallback;
- permit/authority check;
- budget/deadline decision;
- retry/escalation;
- stop/continue;
- degradation/widening;
- operator approval/rejection/override.

## Minimum TurnArbitrationArtifactV1 fields

```text
turn_id
candidate_actions
chosen_action
rejected_alternatives_with_reasons
confidence_or_uncertainty
expected_cost_or_latency_if_available
risk_flags
permit_scope
tool_exposure_set
provider_route
budget_state
created_record_time
```

If AiDENs does not yet have a formal `TurnArbitrationArtifactV1`, it must at least emit equivalent receipt fields in runner reports and mark the formal artifact as `partial`.

## Tool exposure minimization

For each turn, only expose tools required for that turn. Record exposed tools and why.

## Budget and stop rules

No infinite retries. Record retry count, stop condition, and budget exhaustion if applicable.

## Final audit evidence

The audit bundle must include logs for cargo/check/test/clippy, scanner output, provider capability matrix, and phase reports.

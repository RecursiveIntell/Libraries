# Action Resolution

Action resolution is deterministic.

Candidate sources are ordered by source class:

1. Score-derived action.
2. Minimum action floor.
3. Hard veto.

Within a source class, the policy `action_precedence` value decides the stronger
action. Higher precedence wins. Ties are resolved by candidate source text order
for stable replay.

Hard vetoes precede score-derived outcomes. Minimum floors cannot be downgraded
by lower score thresholds. Rejected lower-precedence candidates are recorded in
the receipt with reason codes.

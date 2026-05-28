
# Reference interpreter and conformance plan — post-v24 profiles

These profiles do not create a new truth plane, but they still need executable conformance.

## Required checks per profile lane
- schema parses,
- example validates,
- at least one happy-path fixture exists,
- at least one blocked / degraded / exception fixture exists,
- owner bindings are named explicitly,
- profile artifacts point to existing canonical families rather than inventing shadow semantics.

## Additional hard-semantic checks
- P1: audit extraction respects redaction and disclosure budget requirements;
- P2: cross-boundary transfer cannot proceed without a matching transfer class or time-bounded exception;
- P3: approval and conflict rules cannot resolve through a hidden self-approval path;
- P4: expired recertification schedules degrade release readiness explicitly;
- P5: hazard playbooks remain linked to monitor and mitigation semantics;
- P6: vendor translation remains advisory until admitted and revocation reopens the case;
- P7: escalation clock overrun emits typed operational state instead of hidden pager drift.

## Reference-behavior rule

Where behavior is semantic rather than purely structural, owner crates should add small executable reference checks rather than relying on prose alone.

# Phase 0 — False-success blockers

Issues: `AG-001`, `GOV-001`, `CMP-001`.

Three implementers may run concurrently in isolated scopes; a fourth integration reviewer follows.

Required invariant: explicit failure/unavailability is preferable to a successful value with erased
semantics. Temporary disablement is acceptable.

Cross-tests:

- graph failure cannot produce Complete or a completion receipt;
- governance unavailable/malformed cannot allow;
- compressed payload cannot pass through as exact output;
- missing backend returns typed capability error.

Exit: all three post-merge validated and no adjacent error-to-success path remains.

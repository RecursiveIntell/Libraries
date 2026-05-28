Implement bridge/runtime/repair conformance.

Scope:
- Atomic import and digest/backpointer preservation checks.
- Import failure artifacts.
- Runtime query provenance with explicit view/widening/degradation disclosure.
- RepairRecordV1 with minimal-change rule and blast-radius semantics.
- Differential tests against reference interpreters.

Constraints:
- Keep shape and semantics explicit.
- No silent approximation.

Deliver:
- code/spec updates
- conformance fixtures
- CI/release gate updates

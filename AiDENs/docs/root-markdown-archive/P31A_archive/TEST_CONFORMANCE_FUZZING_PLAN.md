# Test, Conformance, and Fuzzing Plan

## Test layers

1. Unit tests for artifact constructors and policy functions.
2. Golden fixture tests for schemas, receipts, provider routes, tool exposure, permits, memory, and repair.
3. Integration tests for CLI and runtime behavior.
4. Reference-interpreter differential tests.
5. Property tests for monotonic policies and bitemporal invariants.
6. Fuzz tests for boundary compiler, JSON repair, tool input validation, and patch application.
7. Crash/restart tests for receipt store, queue, daemon, and memory outbox.
8. Adversarial tests for prompt injection, path traversal, duplicate JSON keys, stale schema, poisoned receipts, and replay collisions.

## High-value properties

- Disabled means not executable, not exposed, and not invokable.
- Native provider route means executable native provider boundary.
- Adding a permit can expose more tools; removing a permit cannot expose more tools.
- Parser fallback always emits degraded receipt.
- A receipt digest changes if semantic content changes.
- Valid time and recorded time are never collapsed.
- Supersession preserves prior belief state.
- Queue idempotency prevents duplicate logical jobs.
- Local repair never deletes prior truth.
- Subtraction preserves declared invariants.

## Fuzz targets

- boundary compiler input language;
- JSON duplicate-key and number edge cases;
- markdown-fence and substring repair;
- tool input schemas;
- patch propose/apply inputs;
- bitemporal query ranges;
- receipt envelope parser;
- attestation envelope parser.

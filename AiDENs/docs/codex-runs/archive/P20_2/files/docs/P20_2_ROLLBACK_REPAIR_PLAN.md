# P20.2 Rollback, Repair, and Quarantine Plan

## If package scan fails

Restore missing file or remove stale reference with justification. Re-run scan.

## If testkit split creates cycles

Move more production-dependent tests to `aidens-integration-tests`; keep `aidens-testkit` minimal.

## If cargo fails due sibling canonical crate mismatch

Record exact dependency path/version, failing type/API, and required canonical crate update. Do not local-shim canonical semantics inside AiDENs.

## If test agent cannot pass

Keep failing fixture and produce a blocker report. Do not mark v0.1 certified.

## If agency evals fail

Fix policy or eval if eval is malformed. Do not remove influence class coverage.

## If stretch lane destabilizes core gates

Revert stretch changes and preserve P20.2 core closure.

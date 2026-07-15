# Agent role catalog

## Run-control agent

Owns installation, baseline state, active-run pointer, and state-schema validation. It does not
repair semantic source code or claim issue closure.

## Rust runtime implementer

Repairs one bounded runtime contract, adds regression tests, and produces task receipts. It cannot
edit shared manifests or neighboring authorities without a dependency handoff.

## Migration specialist

Inventories all readers/writers, implements dual-read/single-write migration, preserves original
bytes and aliases, and supplies idempotency, partial-failure, and reverse-path evidence.

## Codec contract architect

Freezes the canonical profile/wire/capability/metric contract before backend work begins. It cannot
implement backend-specific exceptions in the core contract merely to ease one backend.

## Backend implementer

Implements a frozen contract and common conformance corpus. It cannot change the canonical contract
without a reviewed contract-change task.

## Concurrency specialist

Owns state-machine, lease, cancellation, and race semantics. It must add deterministic or bounded
concurrency tests and distinguish infrastructure failure from business outcomes.

## Test/conformance specialist

Independent from implementation. It derives adversarial tests from the issue contract, not from the
patch summary.

## Hostile reviewer

Read-only by default. It searches for alternate paths, weak tests, compatibility bypasses, semantic
widening, shadow authority, missing evidence, and rollback failure.

## Integration reviewer

Reviews the merged phase tree, cross-task compatibility, feature combinations, nested workspaces,
lockfile/lint drift, and source-bound evidence.

## Release engineer

Owns workspace/feature inventory, CI, lint enforcement, evidence recording/verification separation,
claims provenance, and final command bar. It cannot waive failed required checks.

## Performance engineer

Runs only after P0/P1 closure. Every change has before/after receipts, correctness gates, and a
reversible implementation boundary.

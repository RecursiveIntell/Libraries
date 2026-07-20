# Learning Agent Runbook

This is the operator procedure for the bounded exact-source learning lane. It
is an AiDENs composition and evidence projection; canonical owner receipts and
live execution remain authoritative.

## Verification

From the repository root, run:

```bash
bash AiDENs/scripts/verify_learning_agent.sh
```

From an AiDENs checkout or archive root, run:

```bash
bash scripts/verify_learning_agent.sh
```

The gate runs the completion-ledger contract, the v2 corpus tests and static
validator, and the focused `aidens-cli` learning tests when Cargo is present.
Each result is printed as `PASS`, `FAIL`, or `SKIP(reason)` and is logged under
`target/verify-learning-agent/`. A required failure exits 1. If required
checks pass but current-HEAD Podman closure/replay evidence was not run, the
gate exits 2 with `RELEASE_GATE INDETERMINATE`; that skip is never success.

Historical receipts in `COMPLETION_LEDGER.md` remain historical. They do not
substitute for a current live receipt.

## Typed request boundaries

Executable qualification uses the v2 corpus boundary only:

```bash
cargo run -p aidens-cli -- learn run --mode fixture --corpus-version v2
```

Real sandbox execution requires a typed `RealSandboxRunRequestV2` JSON request
and explicit fixture, patch, permit grant/use, forbidden paths, resource
limits, pinned image, receipt roots, owner stores, and deterministic run IDs.
Use the CLI's `--request` option; do not construct an untyped substitute or
fall back to v1. v1 is inspection-only:

```bash
cargo run -p aidens-cli -- learn inspect
```

## Exit states and recovery

- `PASS`: all required checks and the live lane have authoritative evidence.
- `FAIL`: a required check failed; inspect its log before retrying.
- `SKIP`: the check was not performed; it is not evidence of success.
- `INDETERMINATE`: required static checks passed, but a required live lane is
  missing or its evidence cannot establish closure.
- `blocked-evidence-insufficient`: the CLI cannot establish the owner receipts
  or readbacks needed for the requested operation.
- `pending`: the bounded operation has not reached a terminal owner-backed
  state; preserve receipts and resume from them.

On interruption, preserve the receipt roots and run bundle. Re-run only with
the same typed request and identities after checking existing owner receipts.
Do not overwrite, delete, or reinterpret evidence. If evidence conflicts,
quarantine the attempt as indeterminate and investigate the owner store.

## Permits and lifecycle controls

Read-only inspection and static validation need no lifecycle permit. Real
execution, file changes, promotion, replay, revoke, and rollback require the
explicit typed permit appropriate to that action and scope. A producer must
not mint its own promotion permit. Do not widen a permit to recover a failed
run.

Use the lifecycle commands only with the canonical candidate and explicit
permit:

```bash
cargo run -p aidens-cli -- learn promote CANDIDATE --permit PERMIT.json
cargo run -p aidens-cli -- learn quarantine CANDIDATE --permit PERMIT.json --store MEMORY_STORE --reason REASON
cargo run -p aidens-cli -- learn revoke CANDIDATE --permit PERMIT.json
cargo run -p aidens-cli -- learn rollback CANDIDATE --permit PERMIT.json
```

`learn stop` is intentionally unsupported; use lifecycle `quarantine`,
`revoke`, `rollback`, or `promote` actions with explicit owner evidence instead.

Revoke or rollback is append-plus-supersession. It must preserve owner
evidence and emit the corresponding owner receipt; it is not deletion or an
overwrite of history. If the permit, candidate, predecessor, digest, or owner
receipt does not match, stop and report the denial.

## Evidence locations

- Projection and claim boundary: `AiDENs/docs/learning-agent/`
- Verification logs: `AiDENs/target/verify-learning-agent/` (or the configured
  `AIDENS_LEARNING_VERIFY_LOG_DIR`)
- Run bundles and receipt indexes: the `receipt_root` from the typed request
- Forge evidence and publication: the `forge_store` and publication namespace
  from the typed request
- Semantic-memory lifecycle evidence: the `memory_store` from the typed request

The ledger is a projection and may be removed as a documentation rollback;
owner receipts must remain untouched.

## Forbidden claims

Do not claim current-HEAD live closure, replay passed, promotion-grade
evidence, bounded MVP or production readiness, cross-task learning,
generalization, online reinforcement learning, or autonomous engineering from
these checks. Do not call a fixture, mock, skipped, historical, degraded, or
unpersisted result verified success. Do not treat corpus qualification as
promotion authority.

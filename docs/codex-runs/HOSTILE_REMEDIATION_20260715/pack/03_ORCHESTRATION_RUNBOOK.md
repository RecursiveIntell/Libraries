# Orchestration runbook

## Task state machine

```text
pending -> assigned -> in_progress -> awaiting_review
        -> changes_requested -> in_progress
        -> validated -> merged -> post_merge_validated -> closed
```

`blocked` is allowed from any nonclosed state and must name evidence, owner, and unblock condition.

## Branches

- Integration: `remediation/hostile-audit-20260715`
- Task: `remediation/<task-id-lowercase>`
- Phase tag: `remediation-20260715-phase-<n>`

## Dispatch cycle

1. Reconcile defect against current source.
2. Lock writable paths in run state.
3. Dispatch rendered specialist prompt.
4. Implement and add contract-focused regression tests.
5. Run task command bar through receipt tooling.
6. Emit Markdown and JSON handoff.
7. Independent hostile review of source/diff/receipts.
8. Repair review findings.
9. Intentional integration.
10. Post-merge validation against integration tree.
11. Close or block issue.

## Shared files

Hermes/integration owns root `Cargo.toml`, `Cargo.lock`, `.github/workflows/**`, root agent pointers,
`stack-ids/src/lib.rs`, canonical codec exports, and release/evidence scripts unless a dedicated
task receives the lock.

## Hostile review questions

- Can the old failure still occur on another path?
- Did the patch widen semantics or add a bypass?
- Are errors collapsed into defaults, warnings, empty values, or successful outer results?
- Does the new type own an invariant or rename a String?
- Are tests contracts or implementation mirrors?
- Are receipts bound to the exact source tree?
- Is rollback possible without deleting canonical/history data?
- Did the patch create another authority, registry, adapter, or shadow state?

## Phase checkpoint

Freeze dispatch; merge/abandon all branches explicitly; run phase matrix; validate handoffs;
update risk/decision logs; generate phase receipt; verify clean tree; tag; then unlock dependents.

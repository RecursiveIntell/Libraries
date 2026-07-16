# Branch and worktree protocol

## Setup

```bash
git switch -c remediation/hostile-audit-20260715
git worktree add ../Libraries-worktrees/<task-id>   -b remediation/<task-id-lowercase>   remediation/hostile-audit-20260715
```

## Rules

- One nonoverlapping task scope per worktree.
- Agents do not pull/rebase/merge/push unless explicitly granted.
- Hermes integrates after independent review.
- Post-merge evidence is rerun; task evidence is not copied as integration proof.
- `Cargo.lock` and shared manifests are serialized through integration ownership.
- No force push after handoff/receipt.
- Abandoned branches retain recorded head and reason.

## Commit message

```text
<issue-id>: <imperative summary>

Contract:
- ...

Evidence:
- ...

Rollback:
- ...
```

## Conflict protocol

Stop writers; record heads; identify semantic conflict; assign integration task with both handoffs;
rerun all affected gates. Never resolve by choosing the branch with fewer textual conflicts.

# Rollback and quarantine — Libraries reconciliation

## Revertable scope

Only these paths belong to this bounded pass:

- `02_MASTER_ISSUE_MATRIX.md`
- `06_RISK_REGISTER.md`
- `scripts/check_manifest_truth.sh`
- `scripts/check_current_closeout_lane.py`
- `docs/receipts/reconciliation-20260822/`

## Rollback

Revert/remove only the paths above if this truth-plane repair is rejected. Do not use `git reset`, `git clean`, broad checkout restoration, archive deletion, gitlink changes, or release-receipt regeneration.

## Quarantine trigger

If an isolated supported-lane candidate cannot be identified, quarantine the mixed worktree and keep the current status `blocked`; do not widen the pass to all 40+ dirty crate families.

No commit, push, deletion, deployment, release activation, or public claim is authorized by this packet.

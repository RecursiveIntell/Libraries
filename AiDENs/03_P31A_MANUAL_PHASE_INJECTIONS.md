# P31A Manual Phase Injections

Paste these between phases. Do not let Codex proceed until the answer is concrete.

## After Phase 0

Pause. Report every file that claims current run, active run, last certified run, support label, package status, build status, or final-gate status. Separate observed text from inferred status. Do not edit anything until this inventory is complete.

## After Phase 1

Pause. Prove `docs/codex-runs/CURRENT_RUN.json` is the only canonical release-truth owner. Show `last_certified_run`, `active_run`, `certification_status`, `support_label`, build/package/replay booleans, and evidence refs. List any protected docs still disagreeing. Do not continue if disagreement remains.

## After Phase 2

Pause. Show the full `scripts/verify_current.sh` command bar. Prove it does not delegate to `p30_verify.sh` as the final gate. Prove missing cargo/build failure becomes a blocker, not success. Show CI invokes this script.

## After Phase 3

Pause. Show root Markdown classification counts and Codex artifact classification counts. Ambiguous active root Markdown must be zero, or quarantined with a manifest and blocker. Old P24–P30 docs/scripts must not remain active as current instructions.

## After Phase 4

Pause. Show the exact `z.py` command used, sidecar paths, archive hash semantics, content manifest hash, package findings count, and package self-replay receipt. Do not claim package certification if extracted replay did not run `scripts/verify_current.sh`.

## After Phase 5

Pause. Show `p30_guard.py --fail-broad` status. If warnings remain, show each waiver with rule ID, path glob, symbol, owner, evidence, and expiry. Do not accept line-only allowlists or permanent waivers.

## Before final report

Pause. State whether P31A is certified, blocked, failed, or uncertified. For every true certification boolean, cite the evidence path. For every false/blocker boolean, cite the blocker path. Do not edit docs upward unless evidence exists.

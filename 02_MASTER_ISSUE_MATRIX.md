# Master Issue Matrix — current compatibility pointer

**Evidence state:** `proposed/blocked` for the current dirty worktree.

This path is retained because the active V29 pack manifest and front-door reading order refer to it. The historical V29 issue matrix is preserved at:

- `docs/archive/superseded_packs/02_MASTER_ISSUE_MATRIX.md`
- `docs/source-packages/archive/20260528T014236Z/files/02_MASTER_ISSUE_MATRIX.md`

Those archived documents are historical source material, not current proof of issue closure or release readiness.

## Current source of truth

- Machine-readable historical issue basis: `01_MASTER_ISSUE_TENSOR.json`.
- Current root contract: `AGENTS.md`, `CLAUDE.md`, and `CONFORMANCE_GATES.md`.
- Current gate evidence: `STATUS_EVIDENCE_MANIFEST.json` and `release/closeout_receipt_v1.json`, both requiring reconciliation against the live HEAD before any claim.
- Current repository state: inspect `git status`, current source, and fresh gate output; do not infer completion from this pointer or from the historical matrix.

## Boundary

This compatibility pointer deliberately does not duplicate the issue matrix. It exists to prevent a missing-file false pass/failure while preserving one source of truth for historical issue content and one source of truth for current verification.

The current dirty tree is not release-certified until the root gates, supported-lane tests, receipt, dashboard, and archive manifest are regenerated from one identified source snapshot.

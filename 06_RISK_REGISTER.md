# Risk Register — current compatibility pointer

**Evidence state:** `proposed/blocked` for the current dirty worktree.

This path is retained because the active V29 pack manifest and front-door reading order refer to it. The historical V29 risk register is preserved at:

- `docs/source-packages/archive/20260528T014236Z/files/06_RISK_REGISTER.md`
- `docs/source-packages/archive/20260524T012027Z/files/06_RISK_REGISTER.md`

The archived register must not be treated as a current risk assessment.

## Current high-signal risks

- **Source snapshot drift:** current HEAD and dirty paths are newer than the dated 2026-03-30/2026-05-13 receipts.
- **Mixed ownership:** the root contains active crate changes, nested repositories, salvage inputs, generated SVG/frontend artifacts, and evidence packs.
- **Gate disagreement:** root scripts currently disagree about required active documents and archived paths.
- **Release overclaim:** `STATUS_DASHBOARD.md` and `release/closeout_receipt_v1.json` contain historical green claims that require a fresh same-snapshot rerun.
- **Nested-manifest contamination:** archived salvage Cargo manifests must not determine active root manifest truth.

## Required disposition

Treat these risks as `blocked` until a fresh, identified source snapshot is isolated, the applicable supported-lane gates are rerun, and the dashboard/receipt are regenerated from that same evidence. This pointer is not a release receipt and does not close any issue.

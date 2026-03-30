# Source basis

This repository is grounded in the following material:

1. `libraries-source-clean-20260330.zip` — the working source snapshot.
2. `STATUS_EVIDENCE_MANIFEST.json` dated 2026-03-30 — the active hardening proof ledger.
3. `release/closeout_receipt_v1.json` — the machine-readable closeout receipt regenerated from the active docs and manifests.

## Canonical truth order

1. `STATUS_EVIDENCE_MANIFEST.json`
2. `release/closeout_receipt_v1.json`
3. `SUPPORT_PROFILE.md`
4. `STATUS_DASHBOARD.md`

If an older matrix, scan, or prompt disagrees with the hardening receipt, the hardening receipt wins.

## Scope discipline

- The **supported closeout lane** is the 17-crate list in `SUPPORT_PROFILE.md`.
- The broader workspace contains adjacent owner crates that are real and landed, but not the narrow release claim for the 2026-03-30 hardening receipt.
- Horizon material remains valid as backlog, not as current gate law.

# Source Basis and Audit Normalization

## Source package posture

Use `AiDENs-aidens-next-codex-context-20260507.zip` and its sidecars as the current clean source-basis bundle. The certifier report records strict mode, 1,523 included files, 10.47 MB, 0 findings, 0 warnings, and 0 errors.

The operator later created the bundle successfully. Therefore:

- Do **not** treat "blocked missing package" from older in-repo P29 evidence as a source/product defect.
- Do treat package/self-replay as a **mandatory final gate after this super pass modifies the repo**.
- Reconcile stale status docs only as evidence closure, not as proof the source package failed.

## Audit inputs used

1. Prior assistant hard audit bundle: `aidens_hard_audit_1000_20260507/`.
2. Claude hard audit: `AiDENs_P29_Hard_Audit_20260507.md`.
3. Package sidecars: report, manifest, findings, excluded, codex-archive.
4. Existing v11A/v11B/v11C release-bar doctrine from loaded project docs.

## Normalization rules

- Findings are not all runtime bugs. The merged backlog intentionally includes confirmed defects, hardening risks, missing hostile fixtures, missing conformance proofs, and evidence-release gates.
- Claude F-001 is normalized as `gate-required-not-product-defect` because the bundle exists; it remains a final release gate after hardening.
- Claude F-016 is treated as P0 because unaudited high-risk layers can invalidate broader correctness claims.
- If an issue duplicates another, close the duplicate only by adding a `superseded_by` field or note in the matrix. Do not silently delete rows.
- A row may be marked `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`. Plain `open` is not a final-state label.

## Required output after Codex pass

- Updated source code and tests.
- Updated `SUPER_PASS_BACKLOG_1020.csv/json` or repo-native issue matrix with statuses.
- Final known limitations register.
- Final auditor handoff.
- Package sidecars from the exact final source tree.
- Extracted-package self-replay receipt.

# P31A Auditor Handoff

Audit target: verify that P31A repaired release truth and final gate integrity without adding feature scope.

## First checks

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
bash scripts/verify_current.sh
```

## Hostile questions

1. Is P31A called current/certified before it earned that status?
2. Do README/STATUS/SOURCE_BASIS/SUPPORT_PROFILE all agree with `CURRENT_RUN.json`?
3. Are there root P24–P31 docs that can still be mistaken for active instructions?
4. Does `verify_current.sh` still delegate to `p30_verify.sh` as the final gate?
5. Do stale P27/P28 defaults remain in current-run/package scripts?
6. Is missing cargo/build failure represented as blocker, not success?
7. Does package replay extract the zip and run the extracted verifier?
8. Are broad static warnings fixed or waivered with owner/reason/expiry?
9. Did Codex implement forbidden runtime/boundary feature work inside P31A?
10. Do positive support claims cite evidence files?

## Evidence required

- final verify log;
- package manifest/report/findings/excluded sidecars;
- extracted replay receipt;
- release ledger hash;
- classification manifest;
- build logs;
- final report.

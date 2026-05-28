# P22 Rollback and Repair Plan

## If `z.py` misarchives a file

1. Stop packaging.
2. Restore from git or from the archive manifest original/archived path.
3. Add the path to the protected allowlist only if it is genuinely active source/truth.
4. Add regression test to `p22_zpy_archival_selftest.py`.

## If stale artifacts remain active

1. Classify them.
2. Archive with manifest.
3. Rerun `assert_p22_codex_archival_hygiene.py`.

## If package includes archive history by default

1. Stop.
2. Fix policy defaults.
3. Rerun normal and audit-full package gates.

## If secret scanner misses literal secrets

1. Stop.
2. Restore stricter detection.
3. Add explicit literal-secret fixture.
4. Do not accept broad allowlists.

## If cargo gates fail

1. Do not claim release readiness.
2. Repair or quarantine.
3. Emit unresolved risk if not fixed.

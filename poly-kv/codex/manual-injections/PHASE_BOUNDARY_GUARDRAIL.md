# PHASE BOUNDARY GUARDRAIL

Stop before proceeding.

Revalidate:

1. Existing z.py behavior has not regressed without an explicit compatibility note.
2. No package rule silently widened inclusion of secrets, binaries, generated outputs, or stale artifacts.
3. Every new include/exclude decision has a report/manifest reason.
4. Package verification works after transfer and does not depend on build-machine absolute paths.
5. Ecosystem adapter failures are reported, not silently passed.
6. Context/audit evidence is preserved only in context/audit modes, not release/source-clean modes unless policy permits it.
7. Root hygiene moves are manifest-recorded and rollbackable.
8. Security/portability gates emit specific findings.
9. Tests/fixtures exist for new behavior.
10. Any failed/skipped validation is recorded as blocker, debt, or explicit non-goal.

If any check fails, repair or report before continuing.

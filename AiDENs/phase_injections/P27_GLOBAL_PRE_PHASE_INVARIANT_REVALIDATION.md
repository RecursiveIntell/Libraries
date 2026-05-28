# P27 Global Pre-Phase Invariant Revalidation

Before each new phase, re-check:

1. Current phase report for prior phase exists.
2. No hard gate was skipped.
3. `AGENTS.md` still points to P27.
4. `scripts/verify_current.sh` still points to an existing script.
5. No new support claim lacks test/evidence.
6. No AiDENs-local canonical truth substitute was introduced.
7. 11A exact/approx/degradation/support labels remain honest.

If any check fails, stop and fix before starting the next phase.

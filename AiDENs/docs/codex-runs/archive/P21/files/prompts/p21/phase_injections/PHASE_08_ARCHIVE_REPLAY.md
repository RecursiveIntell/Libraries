Phase 08 focus: release archive replay.

Create a release candidate zip, unpack it to a temp directory, and verify required files/scripts/evals/fixtures/tests. This catches works-locally-broken-zip failures.

Required proof:
- archive verifier report exists;
- missing file count is zero;
- final zip contains P21 handoff/audit artifacts.

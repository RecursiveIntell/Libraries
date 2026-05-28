You are about to execute PHASE 02: TESTKIT_SPLIT_AND_INTEGRATION_CRATE.

Focus on test topology. aidens-testkit must become pure/reference-only. Production-dependent tests must move to aidens-integration-tests. Do not create cycles.

Required before proceeding:

- restate this phase's local failure mode;
- identify the exact files/scripts/tests likely to be touched;
- confirm no canonical source-of-truth boundary will be weakened;
- define the command or scanner that proves this phase passed.

If you cannot name the proof command, do not proceed.

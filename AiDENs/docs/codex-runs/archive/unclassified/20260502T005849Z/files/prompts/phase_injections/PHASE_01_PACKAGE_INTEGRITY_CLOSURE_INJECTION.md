You are about to execute PHASE 01: PACKAGE_INTEGRITY_CLOSURE.

Focus on package integrity. Restore missing files first. Do not delete include_str tests to make the scanner pass unless equivalent coverage remains and the stale reference is proven obsolete.

Required before proceeding:

- restate this phase's local failure mode;
- identify the exact files/scripts/tests likely to be touched;
- confirm no canonical source-of-truth boundary will be weakened;
- define the command or scanner that proves this phase passed.

If you cannot name the proof command, do not proceed.

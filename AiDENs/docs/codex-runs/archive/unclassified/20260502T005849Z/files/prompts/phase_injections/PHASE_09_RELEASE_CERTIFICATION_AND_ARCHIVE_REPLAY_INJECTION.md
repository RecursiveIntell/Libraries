You are about to execute PHASE 09: RELEASE_CERTIFICATION_AND_ARCHIVE_REPLAY.

Focus on release proof. Generate audit bundle and verify an unpacked release zip. A local workspace pass is insufficient if the archive omits files.

Required before proceeding:

- restate this phase's local failure mode;
- identify the exact files/scripts/tests likely to be touched;
- confirm no canonical source-of-truth boundary will be weakened;
- define the command or scanner that proves this phase passed.

If you cannot name the proof command, do not proceed.

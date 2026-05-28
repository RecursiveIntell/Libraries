# Phase 05 — Fresh Unzip Certification

Goal: prove the repository/package works from a clean unpack, not only in the dirty working tree.

Required:
- create or use the project archive command;
- unpack archive to `/tmp/claimledger_fresh_unzip_check`;
- run install/checks there;
- verify `.codex/` is present in archive;
- verify tests pass from fresh unzip.

Required command:

```bash
bash scripts/run_fresh_unzip_checks.sh
```

It must write:
`docs/P0_COMPLETION_FRESH_UNZIP_CERTIFICATION.md`

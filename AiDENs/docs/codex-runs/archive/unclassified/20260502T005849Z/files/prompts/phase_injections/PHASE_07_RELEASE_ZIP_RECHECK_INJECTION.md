# Phase 07 injection — release zip recheck

Focus: prove the release archive itself is not lying.

Required:

1. Generate the release zip from the repaired workspace.
2. Extract that zip into a clean temp directory.
3. Run:

```bash
python3 scripts/p20_1_hard_code_audit.py --fail-on-blocking
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
python3 scripts/p20_1_validate_archive_manifest.py --root .
```

4. Confirm no `include_str!` / `include_bytes!` target is missing in the extracted archive.
5. Confirm every file named in `MANIFEST.txt` exists.
6. Confirm `scripts/verify.sh`, `scripts/p20_verify.sh`, and `scripts/p20_1_verify.sh` exist.

Forbidden:

- claiming the repo passes while the zip fails;
- relying on files outside the archive except explicitly declared sibling crates;
- hiding missing cargo gates behind documentation language.

Output:

- extracted-zip audit log;
- list of all missing-package repairs;
- final PASS/FAIL.

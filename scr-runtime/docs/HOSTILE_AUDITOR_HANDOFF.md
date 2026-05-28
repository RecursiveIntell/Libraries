# P31 Hostile Auditor Handoff

A hostile auditor should attempt to falsify the following claims:

1. The actual package ZIP contents match the manifest/report.
2. `.codex`/`.agents` are either present and verified or not claimed.
3. No stale template-overlay directories, manual-injection workflow files, SCR/non-SCR labels, testtmp, or stale source-basis surfaces remain.
4. `scr-runtime` does not invent canonical IDs, digests, evidence refs, control receipts, authority semantics, execution receipts, schema governance, or attestation semantics already owned by crates in `~/Coding/Libraries`.
5. JSON schemas reject unknown fields and enforce score bounds/schema versions.
6. Unknown hard rules and wrong policy algorithm/domain are rejected.
7. Opaque refs do not trigger policy/control signals.
8. Candidate arbitration receipts explain every action candidate and every rejection.
9. CLI has honest generate vs verify behavior and can explain receipts without re-evaluation.
10. Fresh unzip checks pass.

Required auditor commands:

```bash
python3 scripts/assert_no_stale_surfaces.py
python3 scripts/assert_existing_crate_boundaries.py
python3 scripts/validate_strict_schemas.py
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/run_p31_completion_checks.sh
```

If a package exists:

```bash
python3 scripts/verify_archive_manifest_parity.py <zip> <manifest.json>
python3 scripts/assert_required_archive_paths.py <zip>
zipinfo -1 <zip> | sort > /tmp/scr_zip_contents.txt
```

The auditor should inspect `docs/P31_UNRESOLVED_RISKS.md` and reject any completion claim that lacks command evidence.

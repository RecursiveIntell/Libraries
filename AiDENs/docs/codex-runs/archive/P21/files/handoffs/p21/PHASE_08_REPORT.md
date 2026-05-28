# P21 Phase 08 Report - Release Archive Replay

## Scope

Phase 08 certifies that a fresh release candidate archive can be unpacked and verified outside the working tree. The replay checks required source files, scripts, evals, fixtures, integration tests, P21 handoffs, and audit artifacts.

Touched surfaces:

- `scripts/p21_verify_release_archive.sh`
- `handoffs/p21/PHASE_08_REPORT.md`
- `target/p21/phase08/*` proof logs and verifier reports
- `target/p21/aidens-v0.1-candidate.zip`

No Rust crate code was changed.

## Invariants Revalidated

Most-at-risk invariants for this phase:

- No missing package fixtures, scripts, evals, tests, handoffs, or audits in the archive.
- No local canonical stack replacement or shadow memory/evidence/kernel/repair/verification truth.
- No compatibility shim or silent semantic widening in archive verification.
- No deletion of tests, fixtures, evals, or scanners to make the zip pass.

Pre-change checks:

- `bash scripts/assert_stack_paths.sh .` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .` -> passed.
- `bash scripts/assert_no_shadow_truth.sh .` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh .` -> `No scaffold promotion patterns found.`

Post-change checks:

- `bash scripts/assert_stack_paths.sh .` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .` -> passed.
- `bash scripts/assert_no_shadow_truth.sh .` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh .` -> `No scaffold promotion patterns found.`
- `bash scripts/p21_verify.sh` -> passed.

## Verifier Change

`scripts/p21_verify_release_archive.sh` now:

- uses `zip.py` for source-only archive creation when it needs to create the zip;
- unpacks the release archive into a temp directory;
- checks explicit required paths for scripts, evals, fixtures, integration tests, docs, examples, P21 handoffs, and P21 audit artifacts;
- emits `missing_file_count`;
- writes a JSON verifier report via `P21_ARCHIVE_REPORT_OUT`;
- runs `scripts/p21_scan_package_integrity.py` and `scripts/p21_verify.sh` from inside the unpacked archive.

## Commands And Outputs

Setup and script syntax:

- `mkdir -p target/p21/phase08` -> created proof directory.
- `bash -n scripts/p21_verify_release_archive.sh` -> `release verifier syntax ok`.

Fresh archive creation:

- `rm -f target/p21/aidens-v0.1-candidate.zip && python3 zip.py --output target/p21/aidens-v0.1-candidate.zip --root . | tee target/p21/phase08/create_archive.log`
- Output: `wrote /home/sikmindz/Coding/Libraries/AiDENs/target/p21/aidens-v0.1-candidate.zip`
- Output: `files=1234 bytes=9240003`

First replay after adding the verifier and draft Phase 08 handoff:

- `P21_ARCHIVE_REPORT_OUT=target/p21/phase08/archive_verifier_report.first.json bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip | tee target/p21/phase08/archive_replay.first.log`
- Output: package integrity `ok: true`.
- Output: source cross refs `ok: true`.
- Output: `Agency eval validation OK: 21 cases, 19 surfaces, 22 receipt kinds`.
- Output: `P21 verify completed`.
- Output: `missing_file_count=0`.
- Output: `release archive replay verified: /home/sikmindz/Coding/Libraries/AiDENs/target/p21/aidens-v0.1-candidate.zip`.

Archive content checks:

- `unzip -l target/p21/aidens-v0.1-candidate.zip ... | tee target/p21/phase08/archive_handoff_audit_listing.first.log`
- Output: archive contains `audit/p21/P21_SOURCE_BASIS_AND_CODE_FIRST_AUDIT.md` and `handoffs/p21/PHASE_00_REPORT.md` through `handoffs/p21/PHASE_08_REPORT.md`.
- `unzip -l target/p21/aidens-v0.1-candidate.zip ... | tee target/p21/phase08/archive_required_listing.first.log`
- Output: archive contains P21 scripts, both agency eval files, test-agent fixtures, integration tests, and test fixtures.

Final replay after this report is updated:

- See `target/p21/phase08/create_archive.final.log`.
- See `target/p21/phase08/archive_replay.log`.
- See `target/p21/phase08/archive_verifier_report.json`.
- Required outcome: `missing_file_count=0`.

Exact default verifier command:

- `bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip | tee target/p21/phase08/archive_replay.default_report.log`
- Output: `missing_file_count=0`.
- Output: `release archive replay verified: /home/sikmindz/Coding/Libraries/AiDENs/target/p21/aidens-v0.1-candidate.zip`.
- Default report: `target/p21/archive_verifier_report.json`.

## Required Proof

Archive verifier report:

- `target/p21/phase08/archive_verifier_report.json`

Release candidate:

- `target/p21/aidens-v0.1-candidate.zip`

Required archive contents:

- scripts: `scripts/p21_verify.sh`, `scripts/p21_scan_package_integrity.py`, `scripts/p21_scan_source_cross_refs.py`, `scripts/p21_verify_release_archive.sh`
- evals: `evals/p20_agency_eval_cases.jsonl`, `evals/p21_agency_eval_cases.jsonl`
- fixtures/tests: `fixtures/test-agent/basic-agent.toml`, `fixtures/runner/expected_test_agent_event_log.ndjson`, `tests/fixtures`, `crates/aidens-integration-tests/tests`
- handoffs/audit: `handoffs/p21/PHASE_00_REPORT.md` through `handoffs/p21/PHASE_08_REPORT.md`, `audit/p21/P21_SOURCE_BASIS_AND_CODE_FIRST_AUDIT.md`

## Outcome

Phase 08 passed. Final replay is recorded in `target/p21/phase08/archive_replay.log` and `target/p21/phase08/archive_verifier_report.json`; the required missing file count is zero.

Per P21 phase protocol, stop here and wait for the operator's Phase 09 injection before continuing.

# P22 Phase 01 Report - `z.py` Codex Archival Normalization

## Scope

Phase 01 repaired `z.py` so stale Codex-run artifacts can be archived with receipts before normal packaging. No live repository Codex artifacts were moved in this phase; the only real archive move was inside the temporary selftest repository.

## Global Invariant Revalidation

- AiDENs remains a directing/wiring/packaging layer only: `pass`.
- Canonical stack libraries own truth; AiDENs substitutes were not introduced: `pass`.
- No stale Codex-run artifact remains active except current P22 run files: `fail in live repo`; repaired `z.py` now plans archive normalization, but `--archive-only` has not yet been run on the live tree.
- Historical run material must be archived, not deleted: `pass for implementation`; `z.py` archives with SHA-256 receipts and does not delete historical material without an archived copy.
- Existing archives must not be rewritten: `pass`; implementation refuses to rewrite existing `ARCHIVE_MANIFEST.json` files.
- `z.py` strict, deterministic, stdlib-only, and source-closure aware: `pass`; no non-stdlib dependency was added, and existing cargo/include/source-closure checks remain active.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `partial`; reports still redact values, but four warning false positives remain for the later secret-scanner phase.
- All support claims backed by executable proof: `pass for Phase 01`; only packaging/archival behavior was claimed and tested.
- If an invariant fails, stop and repair/quarantine before proceeding: `stop`; live stale artifacts remain until the guarded archive-only normalization step.

## Implementation Summary

- Added P22 CLI contract:
  - `--archive-codex-runs` / `--no-archive-codex-runs`
  - `--archive-only`
  - `--verify-codex-archive-hygiene`
  - `--include-codex-archive`
  - `--codex-current-run`
  - `--codex-archive-root`
  - `--codex-archive-report-out`
- Added `audit-full` mode.
- Added stale Codex-run detection for `.codex`, `.codex_evidence`, root `CODEX_*`, `CODEX_PROMPTS`, Pxx prompts/tasks/handoffs/docs, old P20/P21 scripts, and nested `docs/p22/p20|p21` history.
- Added archive receipt generation:
  - `ARCHIVE_MANIFEST.json`
  - `SUPERSESSION.md`
  - `RUN_SUMMARY.md`
  - `docs/codex-runs/CODEX_RUN_INDEX.md`
  - `docs/codex-runs/CURRENT_RUN.md`
  - `docs/codex-runs/ARCHIVAL_POLICY.md`
- Normal `codex-context` dry-run now excludes stale and archived Codex history from the manifest.
- `audit-full` can deliberately include archive history when requested.

## Commands Run

- `python3 -m py_compile z.py` -> pass.
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py` -> pass.
- `python3 scripts/p22_zpy_archival_selftest.py` -> pass; temp repo first run moved `6` stale files and second run moved `0`.
- `python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict --output target/p22/audit/phase01_dry_run.zip ...` -> pass; initial implementation planned `861` archive entries.
- `python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict --output target/p22/audit/phase01_dry_run_2.zip ...` -> pass after nested history tightening; planned `1122`, moved `0`, active-after `0` in dry-run model.
- `python3 scripts/assert_p22_release_package_clean.py target/p22/audit/phase01_dry_run_2.manifest.json` -> pass.
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase01_audit_full_dry_run.zip ...` -> pass.
- `python3 scripts/assert_p22_codex_archival_hygiene.py .` -> expected fail, captured in `target/p22/audit/phase01_hygiene_expected_fail.log`, because live archive normalization has not run yet.

## Gate Results

- Phase 01 `z.py` contract assertion: `pass`.
- Phase 01 archival selftest: `pass`.
- Phase 01 strict codex-context dry-run: `pass`.
- Extra package-clean assertion on dry-run manifest: `pass`.
- Live hygiene assertion: `fail as expected until archive-only normalization`.
- Cargo gates: not run in Phase 01; no Rust source files were edited.

## Archive / Quarantine Status

- Live archive operation performed: `no`.
- Temporary selftest archive operation performed: `yes`, inside a temp mini-repo only.
- Live files moved to unclassified archive: `0`.
- Live files quarantined: `0`.
- Live dry-run planned archive entries: `1122`.
- Live dry-run planned unclassified entries: `98`.
- Existing archives rewritten: `no`.

## Changed Files

- `z.py`
- `handoffs/p22/PHASE_01_REPORT.md`
- `target/p22/audit/phase01_dry_run.manifest.json`
- `target/p22/audit/phase01_dry_run.report.md`
- `target/p22/audit/phase01_dry_run.excluded.json`
- `target/p22/audit/phase01_dry_run.findings.json`
- `target/p22/audit/phase01_dry_run.codex-archive.json`
- `target/p22/audit/phase01_dry_run_2.manifest.json`
- `target/p22/audit/phase01_dry_run_2.report.md`
- `target/p22/audit/phase01_dry_run_2.excluded.json`
- `target/p22/audit/phase01_dry_run_2.findings.json`
- `target/p22/audit/phase01_dry_run_2.codex-archive.json`
- `target/p22/audit/phase01_audit_full_dry_run.manifest.json`
- `target/p22/audit/phase01_audit_full_dry_run.report.md`
- `target/p22/audit/phase01_audit_full_dry_run.excluded.json`
- `target/p22/audit/phase01_audit_full_dry_run.findings.json`
- `target/p22/audit/phase01_audit_full_dry_run.codex-archive.json`
- `target/p22/audit/phase01_hygiene_expected_fail.log`

## Unresolved Risks

- Live stale Codex-run artifacts still remain active. The next guarded repair is `python3 z.py --root . --profile aidens --archive-only --strict`.
- Secret scanner warnings remain for API-key field-copy/fixture filename cases; this belongs to the later secret-scanner phase.
- Root `STATUS.md` and `SOURCE_BASIS.md` still describe older pass state and must be updated after live normalization.
- Parent Git still treats `AiDENs/` as untracked from `/home/sikmindz/Coding/Libraries`, so workspace reports remain the reliable change accounting source.

## Phase Boundary

STOP: Phase 02 requires the next human guardrail. Live stale material must be archived by a guarded normalization step before final packaging can be considered clean.

## Post-Phase 01 Guardrail Revalidation

Status: `PASS with disclosed follow-up risks`.

This revalidation was run after the operator explicitly authorized live archival normalization. Therefore stale active run material is no longer active; it has receipts under `docs/codex-runs/archive/**` and the exact moved-file list is recorded in `target/p22/audit/phase02_archive_only.codex-archive.json`.

Manual guardrail results:

- Phase 01 acceptance gate status: `pass`.
- Exact Phase 01 changed files: `z.py`, `handoffs/p22/PHASE_01_REPORT.md`, `target/p22/audit/phase01_dry_run.*`, `target/p22/audit/phase01_dry_run_2.*`, `target/p22/audit/phase01_audit_full_dry_run.*`, and `target/p22/audit/phase01_hygiene_expected_fail.log`.
- Exact live archive changes after operator authorization: `docs/codex-runs/ARCHIVAL_POLICY.md`, `docs/codex-runs/CODEX_RUN_INDEX.md`, `docs/codex-runs/CURRENT_RUN.md`, and every moved file listed in `target/p22/audit/phase02_archive_only.codex-archive.json`.
- Codex artifacts archived/skipped/left active: `1122` archived, `0` skipped, `0` active stale after normalization.
- Existing archives left untouched: `pass`; no prior archive manifests existed before the live normalization, and the idempotence check moved `0` files on a second archive-only run.
- `z.py` deterministic and strict: `pass`; contract assertion, selftest, strict dry-run, and idempotence checks pass.
- Stale P20/P21/P22 run instruction contamination risk for next phase: `pass`; `python3 scripts/assert_p22_codex_archival_hygiene.py .` passes.
- AiDENs local substitute for canonical library truth introduced: `pass`; no Rust/canonical truth implementation was changed.
- Cargo/tests/assertion status: Python compile, P22 contract assertion, P22 archival selftest, hygiene assertion, strict dry-run, and package-clean assertion pass. Cargo was not run because no Rust source was edited in Phase 01.
- Unresolved risks requiring stop/repair/quarantine: `no` for Phase 01. Remaining work is scheduled for later phases: secret-scanner warning precision, root truth-doc updates, and full cargo gate.

Commands run for this guardrail:

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py && python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 scripts/p22_zpy_archival_selftest.py`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/post_phase01_guardrail_codex_context.zip ...`
- `python3 scripts/assert_p22_release_package_clean.py target/p22/audit/post_phase01_guardrail_codex_context.manifest.json`

Global invariant results:

- AiDENs directs/wires/packages only: `pass`.
- Canonical stack libraries own truth; no AiDENs substitutes: `pass`.
- No stale Codex-run artifact active except current P22 phase files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`.
- `z.py` strict, deterministic, stdlib-only, source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `partial/pass`; values are not printed, but false-positive warnings remain for the later secret-scanner phase.
- Support claims backed by executable proof: `pass for Phase 01`.
- If invariant fails, stop and repair/quarantine: no blocking Phase 01 invariant remains failed.

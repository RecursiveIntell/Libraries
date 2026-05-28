# P22 Phase 02 Report - Package Policy and Verifier Integration

## Scope

Phase 02 completed package-policy and verifier integration for P22. The live archive normalization authorized after Phase 01 is included here as prerequisite evidence because Phase 02 package verification depends on a clean active surface.

## Global Invariant Revalidation

- AiDENs directs/wires/packages only: `pass`.
- Canonical libraries own truth; no AiDENs substitute semantics introduced: `pass`.
- No stale Codex-run artifact remains active except current P22 files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`; second archive-only idempotence check moved `0`.
- `z.py` remains strict, deterministic, stdlib-only, and source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted: `partial/pass`; values are not printed, but scanner warning precision remains a later phase.
- Support claims backed by executable proof: `pass`.
- Blocking invariant failures: `none for Phase 02`.

## Implementation Summary

- Normal `codex-context` packages exclude `docs/codex-runs/archive/**` by default.
- `audit-full` with `--include-codex-archive` deliberately includes archive history.
- `z.py` manifest/report now contains `codex_archive` summary fields:
  - enabled/dry-run/archive-only/verify-only
  - current run and archive root
  - planned/moved/skipped/collision/unclassified counts
  - active stale count after normalization
  - archive manifest paths and report path
- `scripts/assert_p22_release_package_clean.py` now accepts the documented `--manifest` option and normalizes archive paths before stale-history matching.
- `scripts/p22_verify.sh` now writes dry-run sidecars under `target/p22/audit/` and asserts the normal manifest is clean.
- `scripts/p22_verify_release_archive.sh` now correctly selects the extracted `AiDENs/` root from a zip that also contains sibling path dependencies, and verifies unzipped hygiene with current P22 verifier scripts allowed.

## Live Archive Prerequisite

Command:

```bash
python3 z.py --root . --profile aidens --archive-only --strict --codex-archive-report-out target/p22/audit/phase02_archive_only.codex-archive.json
```

Result:

- Planned files: `1122`
- Moved files: `1122`
- Active stale after normalization: `0`
- Collisions: `0`
- Skipped existing: `0`
- Unclassified archive entries: `98`
- Archive manifests written: `34`

## Acceptance Gates

- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run ...` -> pass.
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run ...` -> pass.
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_acceptance_codex_context.manifest.json` -> pass.
- `bash scripts/p22_verify.sh` -> pass without cargo.
- `bash scripts/p22_verify_release_archive.sh target/p22/phase02-aidens-p22-release-context.zip` -> pass.
- `python3 scripts/assert_p22_codex_archival_hygiene.py .` -> pass.

One assertion was accidentally launched in parallel before `phase02_acceptance_codex_context.manifest.json` existed and failed with `FileNotFoundError`; the same command was rerun after the dry-run completed and passed. This was a command-ordering error, not a package-policy failure.

## Release Replay

`scripts/p22_verify_release_archive.sh` produced:

- `target/p22/phase02-aidens-p22-release-context.zip`
- `target/p22/phase02-aidens-p22-release-context.manifest.json`
- `target/p22/archive_verifier_report.final.json`

Replay result:

- `ok`: `true`
- normal package excludes Codex archive: `true`
- unzipped hygiene: `pass`
- zip SHA-256: `6db907721de8266aad31b12a7aa1f990c07322c9a9fe91ebd10c50fb16cfa563`

## Commands Run

- `sed -n '1,220p' prompts/phases/PHASE_02_PACKAGE_POLICY_AND_VERIFIER_INTEGRATION.md`
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py && python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase02_precheck.zip ...`
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase02_precheck_audit_full.zip ...`
- `python3 -m py_compile scripts/assert_p22_release_package_clean.py z.py`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_precheck.manifest.json`
- `bash -n scripts/p22_verify.sh scripts/p22_verify_release_archive.sh`
- `bash scripts/p22_verify.sh`
- `bash scripts/p22_verify_release_archive.sh target/p22/phase02-aidens-p22-release-context.zip`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase02_acceptance_codex_context.zip ...`
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase02_acceptance_audit_full.zip ...`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_acceptance_codex_context.manifest.json`

## Changed Files

- `z.py`
- `scripts/assert_p22_release_package_clean.py`
- `scripts/p22_verify.sh`
- `scripts/p22_verify_release_archive.sh`
- `docs/codex-runs/ARCHIVAL_POLICY.md`
- `docs/codex-runs/CODEX_RUN_INDEX.md`
- `docs/codex-runs/CURRENT_RUN.md`
- `docs/codex-runs/archive/**`
- `handoffs/p22/PHASE_02_REPORT.md`
- `target/p22/audit/phase02_archive_only.codex-archive.json`
- `target/p22/audit/phase02_precheck.*`
- `target/p22/audit/phase02_precheck_audit_full.*`
- `target/p22/audit/phase02_acceptance_codex_context.*`
- `target/p22/audit/phase02_acceptance_audit_full.*`
- `target/p22/phase02-aidens-p22-release-context.*`
- `target/p22/archive_verifier_report.final.json`
- `target/p22/audit/p22_verify_*`
- `target/p22/audit/assert_*`
- `target/p22/audit/zpy_*`

## Archive / Quarantine Status

- Live archive operation performed: `yes`.
- Files moved to unclassified archive: `98`.
- Files quarantined: `0`.
- Existing archives rewritten: `no`.
- Active stale artifacts after normalization: `0`.

## Remaining Risks

- Full cargo verifier remains pending: `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh`.
- Secret scanner warnings remain for precise scanner/redaction work:
  - API-key field-copy warnings in Rust source.
  - P22 secret-scanner test filename exclusion.
  - Archived P22 security plan warning in audit-full mode.
- Root `STATUS.md` and `SOURCE_BASIS.md` still need P22 truth updates.
- Root generated sidecars from the older pre-patch `bash scripts/p22_verify.sh` run remain excluded by `z.py`, but can be cleaned or relocated later if desired.

## Phase Boundary

STOP: Phase 03 requires the next manual guardrail before active doc cleanup and follow-on repo normalization work.

## Post-Phase 02 Guardrail Revalidation

Status: `PASS`.

Manual guardrail results:

- Phase 02 acceptance gate status: `pass`.
- Exact changed files:
  - `z.py`
  - `scripts/assert_p22_release_package_clean.py`
  - `scripts/p22_verify.sh`
  - `scripts/p22_verify_release_archive.sh`
  - `docs/codex-runs/ARCHIVAL_POLICY.md`
  - `docs/codex-runs/CODEX_RUN_INDEX.md`
  - `docs/codex-runs/CURRENT_RUN.md`
  - `docs/codex-runs/archive/**`; exact moved-file list and receipts are in `target/p22/audit/phase02_archive_only.codex-archive.json` and each `ARCHIVE_MANIFEST.json`
  - `handoffs/p22/PHASE_02_REPORT.md`
  - `target/p22/audit/phase02_archive_only.codex-archive.json`
  - `target/p22/audit/phase02_precheck.*`
  - `target/p22/audit/phase02_precheck_audit_full.*`
  - `target/p22/audit/phase02_acceptance_codex_context.*`
  - `target/p22/audit/phase02_acceptance_audit_full.*`
  - `target/p22/phase02-aidens-p22-release-context.*`
  - `target/p22/archive_verifier_report.final.json`
  - `target/p22/audit/p22_verify_*`, `target/p22/audit/assert_*`, and `target/p22/audit/zpy_*`
- Codex artifacts archived/skipped/left active: `1122` archived, `0` skipped, `0` active stale remaining.
- Existing archives left untouched: `pass`; first live archive operation created manifests, and subsequent archive-only idempotence moved `0`.
- `z.py` deterministic and strict: `pass`; contract, hygiene, py_compile, strict dry-run, package-clean, and replay checks pass.
- Stale P20/P21/P22 run instruction contamination risk for next phase: `pass`; `python3 scripts/assert_p22_codex_archival_hygiene.py .` passes.
- AiDENs local substitute for canonical library truth introduced: `pass`; Phase 02 touched packaging scripts/docs/receipts only, not canonical Rust semantics.
- Cargo/tests/assertion status: assertion scripts pass; Python compile and shell syntax checks pass. Cargo was not run because Phase 02 did not edit Rust code.
- Unresolved risks requiring stop/repair/quarantine: `none for Phase 02`. Remaining items are scheduled later: secret-scanner warning precision, root truth-doc updates, and full cargo gate.

Commands run for this guardrail:

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py && python3 scripts/assert_p22_codex_archival_hygiene.py . && python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_acceptance_codex_context.manifest.json && python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/phase02-aidens-p22-release-context.manifest.json`
- `python3 -m py_compile z.py scripts/assert_p22_release_package_clean.py scripts/assert_p22_codex_archival_hygiene.py scripts/assert_p22_zpy_archive_contract.py scripts/p22_zpy_archival_selftest.py && bash -n scripts/p22_verify.sh scripts/p22_verify_release_archive.sh`
- `python3` JSON summary of `target/p22/audit/phase02_archive_only.codex-archive.json`, `target/p22/archive_verifier_report.final.json`, and `target/p22/audit/phase02_acceptance_codex_context.manifest.json`
- `find docs/codex-runs/archive -name ARCHIVE_MANIFEST.json | sort | wc -l`
- `find docs/codex-runs -maxdepth 2 -type f | sort`

Global invariant results:

- AiDENs directs/wires/packages only: `pass`.
- Canonical stack libraries own truth; no AiDENs substitutes: `pass`.
- No stale Codex-run artifact active except current P22 phase files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`.
- `z.py` strict, deterministic, stdlib-only, source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `partial/pass`; values are not printed, but scanner false positives remain for the later secret phase.
- Support claims backed by executable proof: `pass for Phase 02`.
- If invariant fails, stop and repair/quarantine: no blocking Phase 02 invariant remains failed.

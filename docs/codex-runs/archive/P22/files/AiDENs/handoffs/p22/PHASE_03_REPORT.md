# P22 Phase 03 Report - Repo Normalization and Active Doc Cleanup

## Scope

Phase 03 verified the live Codex archive normalization, confirmed active stale run material is gone, and relocated generated root package sidecars into `target/p22/audit/root-sidecars/` with hashes. No protected root docs or source crates were moved.

## Global Invariant Revalidation

- AiDENs directs/wires/packages only: `pass`.
- Canonical libraries own truth; no AiDENs substitutes introduced: `pass`.
- No stale Codex-run artifact remains active except current P22 files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`; archive-only idempotence planned `0`, moved `0`.
- `z.py` remains strict, deterministic, stdlib-only, and source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted: `partial/pass`; values are not printed, but scanner warning precision remains later work.
- Support claims backed by executable proof: `pass for Phase 03`.
- Blocking invariant failures: `none for Phase 03`.

## Work Performed

1. Ran live archive-only idempotence:

```bash
python3 z.py --root . --profile aidens --archive-only --strict --codex-archive-report-out target/p22/audit/phase03_archive_only_idempotence.codex-archive.json
```

Result: planned `0`, moved `0`, active-after `0`.

2. Relocated generated root package sidecars:

- moved `15` `AiDENs-aidens-*` generated sidecar/package files from repo root to `target/p22/audit/root-sidecars/`;
- wrote `target/p22/audit/root_sidecar_relocation_manifest.json` with original path, relocated path, byte count, SHA-256, and reason.

3. Confirmed protected root files remain active:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `AGENTS.md`
- `Cargo.toml`
- `Cargo.lock`
- `z.py`

## Verification

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py` -> pass.
- `python3 scripts/assert_p22_codex_archival_hygiene.py .` -> pass.
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_acceptance_codex_context.manifest.json` -> pass.
- `python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict --output target/p22/audit/phase03_acceptance_codex_context.zip ...` -> pass.
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase03_acceptance_codex_context.manifest.json` -> pass.
- `rg -n "ARCHIVE_MANIFEST" docs/codex-runs/CODEX_RUN_INDEX.md | wc -l` -> `34`.
- Root sidecar scan after relocation -> no active root `AiDENs-aidens-*` files.

## Archive / Quarantine Status

- New Codex archive operation performed: `no`; idempotence found no active stale candidates.
- Existing archives rewritten: `no`.
- Files moved to unclassified archive in Phase 03: `0`.
- Files quarantined: `0`.
- Generated sidecars relocated to target audit: `15`.

## Changed Files

- `handoffs/p22/PHASE_03_REPORT.md`
- `target/p22/audit/phase03_archive_only_idempotence.codex-archive.json`
- `target/p22/audit/root_sidecar_relocation_manifest.json`
- `target/p22/audit/root-sidecars/**`
- `target/p22/audit/phase03_acceptance_codex_context.manifest.json`
- `target/p22/audit/phase03_acceptance_codex_context.report.md`
- `target/p22/audit/phase03_acceptance_codex_context.excluded.json`
- `target/p22/audit/phase03_acceptance_codex_context.findings.json`
- `target/p22/audit/phase03_acceptance_codex_context.codex-archive.json`

## Commands Run

- `sed -n '1,220p' prompts/phases/PHASE_03_APPLY_REPO_NORMALIZATION_AND_ACTIVE_DOC_CLEANUP.md`
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py && python3 scripts/assert_p22_codex_archival_hygiene.py . && python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase02_acceptance_codex_context.manifest.json`
- `find . -maxdepth 2 -type f -name 'AiDENs-aidens-*'`
- `find docs/codex-runs -maxdepth 3 -type f`
- `python3 z.py --root . --profile aidens --archive-only --strict --codex-archive-report-out target/p22/audit/phase03_archive_only_idempotence.codex-archive.json`
- `python3` relocation script for generated root sidecars
- `python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict --output target/p22/audit/phase03_acceptance_codex_context.zip ...`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase03_acceptance_codex_context.manifest.json`
- `for f in README.md STATUS.md SOURCE_BASIS.md AGENTS.md Cargo.toml Cargo.lock z.py; do test -f "$f"; done`
- `rg -n "ARCHIVE_MANIFEST" docs/codex-runs/CODEX_RUN_INDEX.md | wc -l`

## Remaining Risks

- Secret-scanner warnings remain for Phase 05.
- Root `STATUS.md` and `SOURCE_BASIS.md` still need P22 truth updates in the docs phase.
- Full cargo verifier remains pending: `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh`.

## Phase Boundary

STOP: Phase 04 requires the next manual guardrail before release-truth doc updates.

## Post-Phase 03 Guardrail Revalidation

Status: `PASS`.

Manual guardrail results:

- Phase 03 acceptance gate status: `pass`.
- Exact changed files:
  - `handoffs/p22/PHASE_03_REPORT.md`
  - `target/p22/audit/phase03_archive_only_idempotence.codex-archive.json`
  - `target/p22/audit/root_sidecar_relocation_manifest.json`
  - `target/p22/audit/root-sidecars/**`
  - `target/p22/audit/phase03_acceptance_codex_context.*`
  - `target/p22/audit/COMMAND_LOG_SUMMARY.md`
  - `target/p22/audit/CHANGED_FILE_SUMMARY.md`
  - `target/p22/audit/UNRESOLVED_RISKS.md`
- Codex artifacts archived/skipped/left active: archive idempotence planned `0`, moved `0`, skipped `0`, active stale after `0`.
- Existing archives left untouched: `pass`.
- `z.py` deterministic and strict: `pass`; contract, hygiene, py_compile, strict dry-run, package-clean, and shell syntax checks pass.
- Stale P20/P21/P22 run instruction contamination risk for next phase: `pass`; `python3 scripts/assert_p22_codex_archival_hygiene.py .` passes.
- AiDENs local substitute for canonical library truth introduced: `pass`; Phase 03 moved generated sidecars only.
- Cargo/tests/assertion status: Python compile, shell syntax, hygiene, package-clean, and z.py contract checks pass. Cargo was not run because no Rust source was edited.
- Unresolved risks requiring stop/repair/quarantine: `none for Phase 03`. Remaining items are scheduled later: release-truth docs, secret-scanner warning precision, and full cargo gate.

Commands run for this guardrail:

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py && python3 scripts/assert_p22_codex_archival_hygiene.py . && python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase03_acceptance_codex_context.manifest.json`
- `python3 -m py_compile z.py scripts/assert_p22_release_package_clean.py scripts/assert_p22_codex_archival_hygiene.py scripts/assert_p22_zpy_archive_contract.py scripts/p22_zpy_archival_selftest.py && bash -n scripts/p22_verify.sh scripts/p22_verify_release_archive.sh`
- `python3` JSON summary of `target/p22/audit/phase03_archive_only_idempotence.codex-archive.json`, `target/p22/audit/root_sidecar_relocation_manifest.json`, and `target/p22/audit/phase03_acceptance_codex_context.manifest.json`

Global invariant results:

- AiDENs directs/wires/packages only: `pass`.
- Canonical stack libraries own truth; no AiDENs substitutes: `pass`.
- No stale Codex-run artifact active except current P22 phase files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`.
- `z.py` strict, deterministic, stdlib-only, source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `partial/pass`; values are not printed, but scanner false positives remain for the later secret phase.
- Support claims backed by executable proof: `pass for Phase 03`.
- If invariant fails, stop and repair/quarantine: no blocking Phase 03 invariant remains failed.

## Post-Phase 03 Guardrail Repair Addendum

Before Phase 04 edits, a stricter root scan found four active stale root files that the prior hygiene patterns missed:

- `.CODEX_SECOND_RUN_PROMPT.md.kate-swp`
- `install_p20_overlay.sh`
- `install_p20_1_overlay.sh`
- `install_p20_2_overlay.sh`

The detector rules were tightened in `z.py`, `scripts/assert_p22_codex_archival_hygiene.py`, and `scripts/assert_p22_release_package_clean.py`. The four files were archived with receipts by:

```bash
python3 z.py --root . --profile aidens --archive-only --strict --codex-current-run P22 --codex-archive-report-out target/p22/audit/phase04_pre_docs_archive_only.codex-archive.json
```

Result: planned `4`, moved `4`, active-after `0`, unclassified `1`.

New archive receipt roots:

- `docs/codex-runs/archive/P20-20260502T012243Z/`
- `docs/codex-runs/archive/P20_1-20260502T012243Z/`
- `docs/codex-runs/archive/P20_2-20260502T012243Z/`
- `docs/codex-runs/archive/unclassified/20260502T012243Z/`

Follow-up gates passed:

- `python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py`
- `python3 scripts/p22_zpy_archival_selftest.py`
- `bash scripts/p22_verify.sh`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase04_pre_docs_codex_context.manifest.json`

The Phase 04 transition proceeded only after this repair.

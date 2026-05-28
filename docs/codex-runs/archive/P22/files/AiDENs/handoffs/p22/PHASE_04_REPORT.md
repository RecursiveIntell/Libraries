# P22 Phase 04 Report - Release Truth Docs and Operator State

## Scope

Phase 04 updated active release-facing docs to P22 truth and removed stale P20/P21 active-document framing. The pass also repaired a pre-phase hygiene gap discovered at the guardrail: root hidden `CODEX_*` swap files and root P20 overlay installers were not previously caught by all P22 detectors.

## Global Invariant Revalidation

- AiDENs directs/wires/packages only: `pass`.
- Canonical stack libraries own truth; no AiDENs substitutes introduced: `pass`.
- No stale Codex-run artifact remains active except current P22 files: `pass after repair`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`; new timestamped archive roots were created.
- `z.py` remains strict, deterministic, stdlib-only, and source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `partial/pass`; current warnings do not print secret values and remain scheduled for Phase 05.
- Support claims backed by executable proof: `pass for Phase 04`.
- Blocking invariant failures after repair: `none`.

## Pre-Phase Repair

Initial pre-Phase 04 revalidation failed invariant 3 because four stale root artifacts remained active:

- `.CODEX_SECOND_RUN_PROMPT.md.kate-swp`
- `install_p20_overlay.sh`
- `install_p20_1_overlay.sh`
- `install_p20_2_overlay.sh`

Detector fixes:

- `z.py`: root hidden `CODEX_*` files, prompt swap suffixes, and root stale install overlay scripts are stale Codex-run candidates.
- `scripts/assert_p22_codex_archival_hygiene.py`: same active-surface stale patterns.
- `scripts/assert_p22_release_package_clean.py`: package-clean assertion catches the same root stale patterns without flagging current `docs/codex-runs/CODEX_RUN_INDEX.md`.

Archive command:

```bash
python3 z.py --root . --profile aidens --archive-only --strict --codex-current-run P22 --codex-archive-report-out target/p22/audit/phase04_pre_docs_archive_only.codex-archive.json
```

Result: planned `4`, moved `4`, active-after `0`, unclassified `1`.

New receipts:

- `docs/codex-runs/archive/P20-20260502T012243Z/ARCHIVE_MANIFEST.json`
- `docs/codex-runs/archive/P20_1-20260502T012243Z/ARCHIVE_MANIFEST.json`
- `docs/codex-runs/archive/P20_2-20260502T012243Z/ARCHIVE_MANIFEST.json`
- `docs/codex-runs/archive/unclassified/20260502T012243Z/ARCHIVE_MANIFEST.json`

## Documentation Updates

Updated active docs:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `MANIFEST.json`
- `MANIFEST.txt`
- `docs/codex-runs/CURRENT_RUN.md`
- `docs/codex-runs/ARCHIVAL_POLICY.md`

The docs now state supported, partial, scaffold-only, deferred, quarantined/pending, and package/audit behavior. P20/P21 references that remain in active docs are explicitly historical and non-normative.

## Script Updates

- `scripts/assert_docs_source_basis_current.sh` now scans active docs and excludes `docs/codex-runs/archive/` so archived source-basis evidence does not pollute current-source assertions.
- `scripts/assert_p22_codex_archival_hygiene.py` now catches hidden root `CODEX_*` files, prompt swap suffixes, and root stale install overlay scripts.
- `scripts/assert_p22_release_package_clean.py` now catches the same stale-root package paths while allowing current Codex-run index docs.

## Verification

- `bash scripts/assert_docs_source_basis_current.sh` -> pass.
- `python3 scripts/assert_p22_codex_archival_hygiene.py .` -> pass.
- `grep -R "P20\|P21" README.md STATUS.md SOURCE_BASIS.md SUPPORT_PROFILE.md 2>/dev/null || true` -> only historical/non-normative lines.
- `python3 -m json.tool MANIFEST.json >/dev/null` -> pass.
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py` -> pass.
- `python3 scripts/p22_zpy_archival_selftest.py` -> pass.
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase04_acceptance_codex_context.zip ...` -> pass; included `1264`, archive planned `0`, moved `0`, active-after `0`.
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase04_acceptance_codex_context.manifest.json` -> pass.
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase04_acceptance_audit_full.zip ...` -> pass; included `2396`, archive planned `0`, moved `0`, active-after `0`.
- `bash scripts/p22_verify.sh` -> pass without cargo enforcement.
- `bash scripts/assert_no_scaffold_promoted.sh` -> pass.
- `bash scripts/assert_no_local_substitute_dependencies.sh` -> pass.
- `bash scripts/assert_no_shadow_truth.sh` -> pass.

Cargo was not run in Phase 04 because no Rust source changed. The final cargo-enforced P22 gate remains:

```bash
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

## Archive / Quarantine Status

- Codex artifacts archived in this phase: `4`.
- Unclassified archive entries: `1` hidden root swap file.
- Existing archive roots rewritten: `0`.
- Active stale artifacts after repair: `0`.
- Files deleted: `0`.
- Files quarantined outside archive: `0`.

## Changed Files

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `MANIFEST.json`
- `MANIFEST.txt`
- `docs/codex-runs/ARCHIVAL_POLICY.md`
- `docs/codex-runs/CODEX_RUN_INDEX.md`
- `docs/codex-runs/CURRENT_RUN.md`
- `docs/codex-runs/archive/P20-20260502T012243Z/**`
- `docs/codex-runs/archive/P20_1-20260502T012243Z/**`
- `docs/codex-runs/archive/P20_2-20260502T012243Z/**`
- `docs/codex-runs/archive/unclassified/20260502T012243Z/**`
- `handoffs/p22/PHASE_03_REPORT.md`
- `handoffs/p22/PHASE_04_REPORT.md`
- `scripts/assert_docs_source_basis_current.sh`
- `scripts/assert_p22_codex_archival_hygiene.py`
- `scripts/assert_p22_release_package_clean.py`
- `z.py`
- `target/p22/audit/phase04_pre_docs_archive_only.codex-archive.json`
- `target/p22/audit/phase04_pre_docs_codex_context.*`
- `target/p22/audit/phase04_pre_docs_audit_full.*`
- `target/p22/audit/phase04_acceptance_codex_context.*`
- `target/p22/audit/phase04_acceptance_audit_full.*`
- `target/p22/audit/p22_verify_*`
- `target/p22/audit/COMMAND_LOG_SUMMARY.md`
- `target/p22/audit/CHANGED_FILE_SUMMARY.md`
- `target/p22/audit/UNRESOLVED_RISKS.md`

## Commands Run

- `python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 scripts/assert_p22_zpy_archive_contract.py z.py`
- `python3 z.py --root . --profile aidens --archive-only --strict --codex-current-run P22 --codex-archive-report-out target/p22/audit/phase04_pre_docs_archive_only.codex-archive.json`
- `python3 scripts/p22_zpy_archival_selftest.py`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase04_pre_docs_codex_context.zip ...`
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase04_pre_docs_audit_full.zip ...`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase04_pre_docs_codex_context.manifest.json`
- `sed -n` reads for Phase 04 prompt and active docs.
- `bash scripts/assert_docs_source_basis_current.sh`
- `grep -R "P20\|P21" README.md STATUS.md SOURCE_BASIS.md SUPPORT_PROFILE.md 2>/dev/null || true`
- `python3 -m json.tool MANIFEST.json >/dev/null`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase04_acceptance_codex_context.zip ...`
- `python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run --output target/p22/audit/phase04_acceptance_audit_full.zip ...`
- `bash scripts/p22_verify.sh`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/phase04_acceptance_codex_context.manifest.json`
- `bash scripts/assert_no_scaffold_promoted.sh`
- `bash scripts/assert_no_shadow_truth.sh`
- `bash scripts/assert_no_local_substitute_dependencies.sh`

## Remaining Risks

- Secret-scanner warnings remain for Phase 05.
- Full cargo-enforced P22 verifier remains pending.
- Parent Git still reports `AiDENs/` as untracked from `/home/sikmindz/Coding/Libraries`.

## Phase Boundary

Phase 04 acceptance gates pass. Per user instruction, Phase 05 can start only after the next manual guardrail is supplied.

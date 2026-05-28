# P22 Codex Super-Pass Prompt — AiDENs Release-Truth and `z.py` Codex Archival Hardening

You are working in the AiDENs repository. You have no prior context except the repository files and this prompt. Do not assume hidden memory, prior chats, or external knowledge.

## Mission

Perform P22 as a super-pass. The primary mission is to make AiDENs cleanly packageable and resistant to stale Codex-run contamination by upgrading `z.py` into a state-normalizing source certifier that archives Codex-run artifacts before normal packaging.

Secondary mission: take AiDENs as far as safely possible within this pass by hardening release truth, package verification, API-key redaction/secret-scanner handling, active docs, and low-risk operator UX/reporting surfaces. Do not promote deferred features without executable proof.

## Source Basis

Read before editing:

- `handoffs/p21/FINAL_AUDIT_REPORT.md`
- `handoffs/p21/KNOWN_LIMITATIONS.md`
- `z.py`
- `scripts/p21_verify.sh`
- `scripts/p21_verify_release_archive.sh`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `AGENTS.md`
- `Cargo.toml`
- `docs/p21/*`
- `prompts/p21/*`
- `.codex/*`
- `.codex_evidence/*`

Also read the P22 bundle files under `docs/p22`, `prompts/p22`, `tasks/p22`, and `AGENTS_P22.md`.

## P22 Hard Constraints

- Do not delete historical Codex artifacts; archive them with receipts.
- Do not rewrite existing archives.
- Do not include archived Codex history in normal `codex-context` archives by default.
- Do not weaken source closure or Cargo path dependency validation.
- Do not weaken secret scanning. Fix false positives precisely.
- Do not make AiDENs a canonical owner of stack truth.
- Do not promote partial/deferred provider, daemon, memory, federation, or mechanism surfaces.
- Do not proceed after a failed hard gate; repair or stop with a report.

## Required Implementation Outcomes

### `z.py`

Implement a pre-zip archival normalization phase. It must run by default for normal packaging unless explicitly disabled for diagnostics.

Required CLI additions:

```text
--archive-codex-runs / --no-archive-codex-runs
--archive-only
--verify-codex-archive-hygiene
--include-codex-archive
--codex-current-run P22
--codex-archive-root docs/codex-runs/archive
--codex-archive-report-out <path>
```

Required mode addition:

```text
audit-full
```

Default policy:

- `codex-context`: archive active stale Codex artifacts, exclude archive history.
- `full-context`: may include more, but still must not include archives unless explicitly requested.
- `audit-full`: include archive history deliberately.

Required archive outputs:

```text
docs/codex-runs/CODEX_RUN_INDEX.md
docs/codex-runs/CURRENT_RUN.md
docs/codex-runs/ARCHIVAL_POLICY.md
docs/codex-runs/archive/<RUN_ID>/ARCHIVE_MANIFEST.json
docs/codex-runs/archive/<RUN_ID>/SUPERSESSION.md
docs/codex-runs/archive/<RUN_ID>/RUN_SUMMARY.md
```

Required behavior:

- Archive `.codex` Pxx prompts/tasks, `.codex_evidence`, `prompts/Pxx*`, `prompts/pNN/**`, `docs/pNN/**`, `handoffs/pNN/**`, root `CODEX_*` run prompt files, old Pxx run scripts if not promoted to generic current scripts.
- Leave existing archive paths untouched.
- Preserve original path and SHA-256.
- Avoid path collisions without overwrite.
- Put ambiguous files under `archive/unclassified/<stamp>/` with reason.
- On `--dry-run`, do not move files; emit planned archive report.
- In strict mode, fail if stale active Codex-run artifacts remain after normalization.

### Repo cleanup

Run the new archival normalization. Afterward the active repo surface must contain only current truth docs, source code, stable specs, current P22 run files until final packaging, and archive index docs. Stale P20/P21 docs must not remain active.

### Verification

Install/adapt P22 scripts:

- `scripts/assert_p22_codex_archival_hygiene.py`
- `scripts/assert_p22_zpy_archive_contract.py`
- `scripts/assert_p22_release_package_clean.py`
- `scripts/p22_zpy_archival_selftest.py`
- `scripts/p22_verify.sh`
- `scripts/p22_verify_release_archive.sh`

Final gates must pass:

```bash
python3 scripts/p22_zpy_archival_selftest.py
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 scripts/assert_p22_codex_archival_hygiene.py .
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
bash scripts/p22_verify_release_archive.sh target/p22/aidens-p22-release-context.zip
python3 z.py --root . --profile aidens --mode codex-context --strict
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run
```

### Final handoff

Create:

```text
handoffs/p22/PHASE_00_REPORT.md ... PHASE_08_REPORT.md
handoffs/p22/FINAL_AUDIT_REPORT.md
handoffs/p22/KNOWN_LIMITATIONS.md
target/p22/audit/COMMAND_LOG_SUMMARY.md
target/p22/audit/CHANGED_FILE_SUMMARY.md
target/p22/audit/UNRESOLVED_RISKS.md
target/p22/archive_verifier_report.final.json
```

## Phase Execution

Execute phases in order. At each phase boundary, stop and wait for the human operator's manual guardrail injection. If running in a non-interactive Codex environment, emit the phase report and explicitly state that the next phase requires the corresponding guardrail.

## Final Definition of Done

P22 is done only when:

- `z.py` archives stale Codex-run artifacts by default before normal packaging;
- normal package output excludes archived run history;
- audit/full-history mode can include archives deliberately;
- strict packaging fails if active stale run artifacts remain;
- docs reflect actual current state;
- secret warnings are resolved by precise scanner logic or redacted handling;
- all cargo and P22 verifier gates pass;
- final package can be produced and replay verified;
- final report clearly distinguishes supported, partial, scaffold, deferred, quarantined, and failed surfaces.

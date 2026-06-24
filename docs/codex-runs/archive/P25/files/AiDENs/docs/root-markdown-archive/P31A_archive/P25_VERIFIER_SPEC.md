# P25 Verifier Spec

## Required verifier files

Recommended:
- `scripts/assert_phase_gate_integrity.py`
- `scripts/assert_root_markdown_archive_policy.py`
- `scripts/assert_current_run_truth.py`
- `scripts/p25_verify.sh`
- `scripts/verify_current.sh` as stable alias

## Required checks

### Phase gate integrity

Fail if:
- active phase injection files do not contain STOP/WAIT language;
- active phase injection files reference stale prior-run tokens as current run;
- active injection files reference stale `target/p##`, `handoffs/p##`, or stale paths;
- gate sequence differs from P25 phase plan.

### Root Markdown archive policy

Fail if:
- candidate archive files remain in root after strict archive run;
- protected docs are moved;
- archive manifest missing;
- archive manifest lacks hash/bytes/mtime/original path/archived path/reason;
- ambiguous files are moved.

### Current-run truth

Fail if:
- `CURRENT_RUN.md` does not identify P25;
- classification map does not treat prior-run prompts as current;
- prior-run evidence is treated as current instruction;
- archive sidecar reports active stale artifacts.

### Support truth

Fail if:
- README/STATUS/SUPPORT_PROFILE claim cloud/autonomy/full runtime support without evidence;
- flagship demo is not clearly supported-local and fixture-backed.

### Build/test gates

Run where available:
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`

If unavailable, emit explicit environment limitation, not fake success.

## Output

Emit:
- `P25_STATUS_EVIDENCE_MANIFEST.json`
- command transcript or command receipt summary
- changed files
- failed checks
- quarantined items
- unresolved risk.

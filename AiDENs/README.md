# AiDENs P31A Codex Pass — Release Truth and Verification Gate Repair

**Purpose:** repair AiDENs' release/run truth, final verification gate, artifact classification, CI enforcement, and package replay semantics before any further runtime or v11B/v11C expansion.

This pass is intentionally narrow. It does **not** implement the P31 boundary compiler microkernel. It does **not** add runtime receipt families. It does **not** alter semantic-memory, stack-ids, verification crates, or kernel crates except where existing final gates require path/build-scope truth.

## Why this pass exists

The latest package is structurally healthy enough to continue, but not clean enough to certify. The package report shows strict packaging, 1,680 files, 42 include roots, 41 external Cargo path dependency roots, and zero configured validation findings. It also shows root Markdown archival disabled, 180 root Markdown docs inspected, 26 candidates, and 149 ambiguous root Markdown files. The manifest/codex archive says current run is P30 while the active root README announces a P31 boundary compiler pack; existing scripts still default to older run assumptions. P31A fixes that first.

## Scope

P31A may change:

- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `.github/workflows/ci.yml`
- `docs/codex-runs/CURRENT_RUN.json`
- `docs/codex-runs/CURRENT_RUN.md`
- `docs/codex-runs/RUN_LEDGER.jsonl`
- `docs/codex-runs/BUILD_SCOPE.md`
- `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`
- `docs/codex-runs/archive/**`
- `docs/root-markdown-archive/**`
- `scripts/verify_current.sh`
- release-truth/package/classification verifier scripts under `scripts/`

P31A must not change runtime behavior except for verifier/package scripts and docs. Any runtime receipt/ID/patch/search issue discovered must be recorded as a blocker/deferred issue for P31B, not implemented here.

## Deliverables

1. Canonical release ledger.
2. Protected docs generated or checked against the ledger.
3. Root Markdown and Codex artifact classification repaired.
4. `verify_current.sh` replaced with a real command bar.
5. CI runs `verify_current.sh` without stale P27/P28/P30 environment assumptions.
6. Package self-replay procedure defined and enforced.
7. Final report with exact commands, outputs, pass/fail/skipped status, blockers, and support label.

## Start here

Paste `01_P31A_MAIN_CODEX_PROMPT.md` into Codex from the repository root. Then paste manual injections from `03_P31A_MANUAL_PHASE_INJECTIONS.md` between phases.

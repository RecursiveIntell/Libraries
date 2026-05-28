# P31A Main Codex Prompt — Release Truth and Verification Gate Repair

You are working in the AiDENs repository. This pass is **P31A: Release Truth and Verification Gate Repair**.

## Source basis

Observed prior-state evidence:

- The package certifier reported strict mode, 1,680 included files, 42 include roots, 41 external Cargo path dependency roots, 0 findings, and distinct zip-byte/content-manifest hashes.
- The same report showed root Markdown archive disabled, 180 root Markdown files inspected, 26 candidates, 149 ambiguous files, and 0 moved.
- The manifest/codex archive identified current_run as P30 and archived P24–P28 stale scripts.
- The active root README described a P31 boundary compiler microkernel pack.
- Existing `scripts/verify_current.sh` delegated to `p30_verify.sh`.
- Existing `scripts/assert_current_run_truth.py` defaulted to P28 unless overridden.

These are release-truth and verification-gate defects. Fix those first.

## Pass objective

Create one canonical release ledger and enforce it through docs, scripts, CI, package validation, and package self-replay.

This pass must end with:

```text
last_certified_run = P30
active_run = P31A
certification_status = uncertified | blocked | certified
feature_expansion_allowed = false
boundary_compiler_deferred = true
runtime_receipt_changes_deferred = true
```

`certification_status = certified` is allowed only if the final command bar passes and evidence paths exist.

## Non-goals

Do not implement:

- P31 boundary compiler microkernel.
- Runtime retry/blocked/patch/search receipt changes.
- New receipt families.
- New ID types.
- New stack canonical artifact contracts.
- New crates.
- v11B/v11C features.
- Semantic-memory/turbo-quant integration.

If you find those issues, record them as `P31B_DEFERRED_ISSUES.md` or equivalent, with evidence and acceptance tests.

## Required files to create or update

Create/update exactly these core surfaces unless the repository already has stricter equivalents:

```text
AGENTS.md
README.md
STATUS.md
SOURCE_BASIS.md
SUPPORT_PROFILE.md
.github/workflows/ci.yml
docs/codex-runs/CURRENT_RUN.json
docs/codex-runs/CURRENT_RUN.md
docs/codex-runs/RUN_LEDGER.jsonl
docs/codex-runs/BUILD_SCOPE.md
docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json
scripts/verify_current.sh
scripts/assert_current_run_truth.py
scripts/assert_release_ledger_schema.py
scripts/assert_release_truth_consistency.py
scripts/assert_support_claims_have_evidence.py
scripts/assert_package_validation.py
scripts/assert_package_self_replay.py
scripts/assert_root_markdown_archive_policy.py
scripts/assert_codex_artifact_classification.py
handoffs/P31A_FINAL_REPORT.md
handoffs/P31A_DEFERRED_RUNTIME_EVIDENCE_ISSUES.md
```

## Canonical ledger requirement

`docs/codex-runs/CURRENT_RUN.json` is the canonical owner. `CURRENT_RUN.md`, `README.md`, `STATUS.md`, `SOURCE_BASIS.md`, and `SUPPORT_PROFILE.md` must either be generated from it or checked against it.

Minimum ledger shape:

```json
{
  "schema_version": "aidens.current-run.v1",
  "project": "AiDENs",
  "last_certified_run": "P30",
  "active_run": "P31A",
  "target_run": "P31A",
  "parent_run": "P30",
  "active_run_role": "release-truth-and-verification-gate-repair",
  "certification_status": "uncertified",
  "support_label": "supported-local-candidate",
  "feature_expansion_allowed": false,
  "boundary_compiler_deferred": true,
  "runtime_receipt_changes_deferred": true,
  "build_certified": false,
  "package_certified": false,
  "extracted_replay_certified": false,
  "build_scope_file": "docs/codex-runs/BUILD_SCOPE.md",
  "known_limitations_file": "docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md",
  "evidence": {
    "build_receipt": null,
    "cargo_metadata_log": null,
    "fmt_log": null,
    "check_log": null,
    "test_log": null,
    "clippy_log": null,
    "package_manifest": null,
    "package_findings": null,
    "package_report": null,
    "package_replay_receipt": null,
    "final_verify_log": null
  }
}
```

Positive booleans may become true only when their evidence refs exist.

## BUILD_SCOPE requirement

Create `docs/codex-runs/BUILD_SCOPE.md` before finalizing verification. It must distinguish:

- Tier 1 mandatory AiDENs workspace crates.
- Tier 2 direct external Cargo path dependencies included for build context.
- Tier 3 context-only roots included for source/audit context.
- Known external blockers.

Do not narrow build scope just to pass. Narrowing must be justified by ownership/source-basis evidence.

## Final verifier requirement

Replace `scripts/verify_current.sh`. It must not simply delegate to `p30_verify.sh`.

It must run, at minimum:

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
python3 scripts/assert_support_claims_have_evidence.py
bash scripts/assert_no_fake_completion.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_adapter_delegation.sh
bash scripts/assert_tool_runtime_delegation.sh
python3 scripts/assert_no_canonical_type_duplicates.py
bash scripts/assert_no_local_substitute_dependencies.sh
python3 scripts/p30_guard.py --repo . --fail-broad
cargo metadata --locked --format-version 1
cargo fmt --all --check
cargo check --workspace --locked --all-targets
cargo test --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets -- -D warnings
```

If the workspace is run from an extracted package root where AiDENs is nested, the script may detect `AiDENs/Cargo.toml` and run against that manifest, but the detection must be explicit and logged.

If cargo is missing, mark build certification blocked. Do not report success.

## Package verification requirement

Use the real `z.py` interface discovered in the repo. The expected packaging command is:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P31A
```

Then validate sidecars and perform extracted package self-replay. Self-replay must:

1. Extract the zip to a temp directory.
2. Locate the package repo root.
3. Confirm sidecars exist and parse.
4. Confirm manifest current_run matches active_run.
5. Run `scripts/verify_current.sh` from the extracted package.
6. Emit a JSON replay receipt.
7. Fail if the extracted package points at unavailable sibling dependencies without an explicit blocked receipt.

## Root Markdown and Codex artifact policy

Root Markdown ambiguity must be zero or quarantined. Stale P24–P30 run docs/scripts must not remain active unless deliberately classified as archive evidence. Existing P31 boundary compiler docs must be moved to deferred/next-plan classification or explicitly marked inactive until P31A passes.

Do not delete history. Archive or quarantine with manifests.

## CI requirement

Update `.github/workflows/ci.yml` so CI runs:

```bash
bash scripts/verify_current.sh
```

Do not use stale `P27_REQUIRE_CARGO`, `P28_SKIP_CARGO`, or P30-only environment assumptions unless explicitly justified.

## Final report

Write `handoffs/P31A_FINAL_REPORT.md` with:

- changed files,
- commands run,
- logs/evidence paths,
- pass/fail/skipped/blocker status,
- release ledger values,
- package zip path/hash,
- content manifest hash,
- root Markdown classification counts,
- Codex artifact classification counts,
- build scope,
- remaining blockers,
- deferred P31B runtime evidence issues,
- support label,
- forbidden claims not made.

Do not call the pass complete if `verify_current.sh` is not the single final gate.

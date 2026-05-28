# Phase 04 Report - Digest Identity Collapse

PHASE:
04 - Digest Identity Collapse.

STARTING GIT STATUS:
Captured at `.codex_evidence/contract_ownership/04/git_status_before.txt`.

The working directory was `/home/sikmindz/Coding/Libraries/AiDENs`. The parent git root is `/home/sikmindz/Coding/Libraries`; parent status reports AiDENs as `?? ./`, so Phase 04 file evidence is tracked through pre/post snapshots and `touched_file_diff.patch`.

COMMANDS RUN:
- `bash scripts/assert_no_local_canonical_digest_law.sh` before repair: failed on exported local digest law names.
- `cargo fmt --all`
- `bash scripts/assert_no_local_canonical_digest_law.sh`
- `cargo check --workspace`
- `bash scripts/phase_verify_contract_ownership.sh 04`
- `cargo test --workspace`
- `bash scripts/assert_no_crate_split.sh`
- `bash scripts/assert_no_compatibility_ledgers.sh`
- `bash scripts/assert_docs_source_basis_current.sh`
- `python3 scripts/assert_no_canonical_type_duplicates.py`
- broad stale digest/schema/doc scan saved to `.codex_evidence/contract_ownership/04/schema_doc_stale_digest_scope.txt`

Full command chronology is saved at `.codex_evidence/contract_ownership/04/commands_run.txt`.

FILES CHANGED:
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-contracts/Cargo.toml`
- `crates/aidens-boundary-kit/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `docs/contract-ownership/DIGEST_IDENTITY_SOURCE_OF_TRUTH.md`
- `tests/fixtures/p06/canonical_digest_v1.json` removed
- `tests/fixtures/p06/display_digest_v1.json` added
- fixture digest markers updated across the files listed in `.codex_evidence/contract_ownership/04/fixture_digest_marker_files.txt`

Evidence-only files were written under `.codex_evidence/contract_ownership/04/`.

GIT DIFF STAT:
Saved at `.codex_evidence/contract_ownership/04/git_diff_stat.txt`.

Because the target repo is nested under the parent git root and appears untracked to parent git, the authoritative Phase 04 file diff is `.codex_evidence/contract_ownership/04/touched_file_diff.patch`.

GATE OUTPUTS:
Saved at `.codex_evidence/contract_ownership/04/gate_outputs.txt`.

Key passing outputs:

```text
PASS: no exported local canonical digest law detected.
PASS: contract ownership verification passed. Evidence in /home/sikmindz/Coding/Libraries/AiDENs/.codex_evidence/contract_ownership/04
PASS: no aidens-contracts split crates detected.
PASS: no compatibility ledger entries or obvious compat/shim files detected.
PASS: no blocking stale source-basis docs detected.
canonical_types=633
aidens_contracts_types=193
duplicate_findings=0
PASS: no local aidens-contracts public type definitions duplicate canonical public type names.
```

`cargo check --workspace` passed and `cargo test --workspace` passed. No Phase 04 build or test checks were skipped.

CANONICAL OWNERSHIP PROOF:
- `stack-ids` remains the canonical digest/content-addressing owner. Evidence is saved in `.codex_evidence/contract_ownership/04/digest_identity_audit.txt`.
- `crates/aidens-contracts/src/lib.rs` now delegates JSON and text digest bytes to `stack_ids::ContentDigest` through `canonical_stack::digest_json` and `StackContentDigest::compute_str`.
- Removed exported local canonical law names from `aidens-contracts`: `stable_json_digest`, `stable_text_digest`, `deterministic_artifact_id`, `canonical_json_string`, and `CanonicalDigestV1`.
- Replaced the local canonical digest DTO with `DisplayDigestV1`, which carries `non_authoritative: true` and `display-only-not-artifact-identity`.
- Removed the `sha2` dependency from `crates/aidens-contracts/Cargo.toml`.
- Added `docs/contract-ownership/DIGEST_IDENTITY_SOURCE_OF_TRUTH.md`, source basis 2026-04-28, recording that canonical digest law is owned by `~/Coding/Libraries/stack-ids`.

INVARIANTS REVALIDATED:
- Operating directory: `/home/sikmindz/Coding/Libraries/AiDENs`.
- Canonical owners remain under `/home/sikmindz/Coding/Libraries`.
- `Libraries2`, `Recall`, and `Recall-Coding` were not imported or used as dependencies.
- `aidens-contracts` was not split.
- No features were added.
- No local substitute digest module was created.
- No compatibility shim or ledger row was added.
- Display digest strings are report/display helpers only and are not stack artifact identity.

QUARANTINE ITEMS:
No new Phase 04 quarantine item was opened. Digest ownership was not ambiguous: canonical digest/content-addressing law is owned by `stack-ids`.

ROLLBACK/RECOVERY NOTES:
No rollback was performed. Pre-edit snapshots and the touched-file patch are saved under `.codex_evidence/contract_ownership/04/`.

FAILURES OR SKIPPED BUILD STEPS:
Initial digest-law gate failed before repair, as expected, on the exported local digest law names. The final digest-law gate passed.

No Phase 04 build or test command was skipped.

UNRESOLVED RISKS:
- Broad scan evidence in `.codex_evidence/contract_ownership/04/schema_doc_stale_digest_scope.txt` still shows generated schema files and older handoff/prompt/artifact registry docs mentioning `CanonicalDigestV1`, `canonical_digest`, and `sha256:`. The Phase 04 source/fixture digest-law gate passes; these schema authority residues belong to Phase 05 schema-scope collapse and later doc cleanup, not to AiDENs-local digest implementation.
- Parent git status contains substantial pre-existing changes outside the AiDENs target directory. Phase 04 did not revert or modify those unrelated parent-root changes.

NEXT SAFE ACTION:
Stop and wait for `GUARDRAIL_04_TO_05`. Do not start Phase 05 until the human guardrail is provided.

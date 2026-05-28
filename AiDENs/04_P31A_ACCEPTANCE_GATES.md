# P31A Acceptance Gates

## Required final command

```bash
bash scripts/verify_current.sh
```

This is the only final command bar.

## Release truth gates

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
```

Pass criteria:

- `CURRENT_RUN.json` exists and is valid.
- `last_certified_run`, `active_run`, and `certification_status` are distinct where required.
- Protected docs agree with the ledger.
- Positive support/certification claims cite evidence.

## Archive/classification gates

```bash
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
```

Pass criteria:

- no ambiguous active root Markdown;
- stale run docs/scripts archived or classified;
- P31 boundary compiler docs are deferred/inactive until P31A passes.

## Existing invariant gates

```bash
bash scripts/assert_no_fake_completion.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_adapter_delegation.sh
bash scripts/assert_tool_runtime_delegation.sh
python3 scripts/assert_no_canonical_type_duplicates.py
bash scripts/assert_no_local_substitute_dependencies.sh
python3 scripts/p30_guard.py --repo . --fail-broad
```

Pass criteria:

- no fake completion;
- no shadow truth;
- no local substitutes for canonical owners;
- no broad warnings without expiring waiver.

## Build gates

```bash
cargo metadata --locked --format-version 1
cargo fmt --all --check
cargo check --workspace --locked --all-targets
cargo test --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets -- -D warnings
```

Pass criteria:

- all commands pass for declared Tier 1 workspace.
- If build scope is narrower than full workspace, `BUILD_SCOPE.md` must justify it.
- Missing cargo is a blocker, not success.

## Package/replay gates

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P31A
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package <zip> --require-verifier --receipt-out handoffs/P31A_PACKAGE_REPLAY_RECEIPT.json
```

Pass criteria:

- findings: zero errors, zero warnings;
- manifest current_run: P31A;
- archive hash semantics explicit;
- content manifest hash present;
- extracted package runs `verify_current.sh`;
- replay receipt emitted.

## Forbidden pass conditions

P31A fails if any are true:

- P31A is called certified without build/package/replay evidence.
- `verify_current.sh` delegates only to `p30_verify.sh`.
- stale P27/P28 defaults remain in current-run/package scripts.
- README announces P31 boundary compiler as active current pass.
- root Markdown ambiguity is ignored.
- package replay runs against source tree instead of extracted zip.
- cargo missing/build failed but support label is raised.
- broad warnings are hidden with permanent or line-only allowlists.
- Codex implements runtime receipt changes or boundary compiler features in this pass.

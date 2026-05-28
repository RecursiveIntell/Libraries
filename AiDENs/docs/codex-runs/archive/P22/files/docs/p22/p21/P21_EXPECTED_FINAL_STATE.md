# P21 Expected Final State

## Required commands

At final state, these commands should work from AiDENs repo root:

```bash
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml
cargo run -p aidens-cli -- new coding-agent target/demo-agent
cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"
cargo run -p aidens-cli -- doctor --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- provider-check --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- tools inspect --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- plan validate --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- plan compile --config target/demo-agent/aidens.toml --out target/demo-agent/aidens.plan.json
bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip
```

## Required files/directories

```text
scripts/p21_verify.sh
scripts/p21_scan_package_integrity.py
scripts/p21_scan_source_cross_refs.py
scripts/p21_verify_release_archive.sh
scripts/p21_generate_audit_bundle.sh
evals/p20_agency_eval_cases.jsonl
fixtures/test-agent/basic-agent.toml
fixtures/runner/expected_test_agent_event_log.ndjson
examples/configs/chat-only.toml
examples/configs/coding-agent.toml
handoffs/p21/FINAL_AUDIT_REPORT.md
```

## Required crate behavior

- `aidens-testkit` remains pure/reference-only.
- `aidens-integration-tests` owns production integration tests.
- `aidens-cli` exposes operator-facing agent-builder commands.
- `aidens-plan-kit` owns execution-plan assembly only.
- canonical adapter crates remain thin.
- scaffold profiles are either made partial-real or explicitly deferred.

## Forbidden final state

- missing fixtures referenced by code;
- cloud providers advertised as supported without tests;
- app-specific Recall assumptions embedded in AiDENs core;
- generated projects that cannot run;
- agency gate bypass when enabled;
- final audit only in ignored `target/` without handoff copy.

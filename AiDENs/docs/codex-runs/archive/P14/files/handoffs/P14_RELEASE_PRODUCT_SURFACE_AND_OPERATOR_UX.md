# P14 Handoff - Release Product Surface and Operator UX

## Summary

P14 is implemented. AiDENs now has typed release/operator artifacts, product-facing CLI aliases, example manifests, install-smoke evidence, public-doc release-readiness blocking, and CI coverage for example compile/test fixtures.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
  - Added `ReleaseReadinessReportV1`, `OperatorStatusReportV1`, `ExampleAppManifestV1`, `InstallSmokeReceiptV1`, supporting surface/doc/step types, receipt-kind registration, schema registration, and P14 fixture tests.
- `crates/aidens-cli/src/lib.rs`
  - Added `status`, `tools list`, `tools inspect`, `permits`, `memory status`, `queue`, and `package examples/install-smoke/readiness` surfaces.
  - Added release-readiness public-doc scanning, example manifest generation, install-smoke command execution, and operator status reporting.
- `crates/aidens-app-kit/src/lib.rs`
  - Added profile catalog helpers for product surface status.
- `crates/aidens/src/lib.rs`
  - Exported P14 artifact types through the prelude.
- `tests/fixtures/p14/*.json`
  - Added golden fixtures for all P14 artifacts.
- `schemas/example-app-manifest/v1.schema.json`
- `schemas/install-smoke-receipt/v1.schema.json`
- `schemas/operator-status-report/v1.schema.json`
- `schemas/release-readiness-report/v1.schema.json`
- `schemas/generated-schema-manifest/v1.schema.json`
- `schemas/generated_schema_manifest_v1.json`
  - Regenerated schema outputs through `aidens schemas generate`.
- `examples/aidens.chat-only.toml`
- `examples/aidens.memory.toml`
- `examples/aidens.daemon.toml`
- `examples/aidens.research.toml`
- `examples/aidens.openai-unavailable.toml`
  - Added product-facing examples that distinguish supported, partial, degraded, and blocked routes.
- `scripts/check_examples.sh`
- `scripts/verify.sh`
  - Added example fixture smoke coverage to the single local/CI verification gate.
- `README.md`
- `docs/OPERATOR_QUICKSTART.md`
- `docs/08_CI_AND_COMMANDS.md`
- `ACCEPTANCE_GATES_AND_CI.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `STATUS.md`
- `STATUS_TEMPLATE.md`
  - Updated operator docs, schema registry docs, status truth, and CI gate notes without presenting scaffold-only crates as usable.

## Tests Added

- Contract tests for P14 constructors, blocked release readiness on false public-doc claims, operator status degradation disclosure, and P14 fixture deserialization.
- App-kit profile catalog test for supported versus partial product surfaces.
- CLI tests for operator status, release-readiness blocking, example manifest honesty, and the new-user mock flow through app creation, provider-check, tools inspection, mock turn, and receipt inspection.
- `scripts/check_examples.sh` runs example manifest/readiness checks, provider-checks across examples, plan validation, mock run, receipt inspection, and generated-app compile/test smoke.

## Commands Run

```bash
cargo test -p aidens-contracts p14
cargo test -p aidens-app-kit
cargo test -p aidens-cli
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
cargo run -p aidens-cli -- package install-smoke --root . --config examples/aidens.mock.toml
rg -n "20260425|2026-04-25|2026/04/25" .
```

All build/test/schema/fake-ready/scaffold gates passed. The stale 20260425 scan returned only explicit historical references, P00 acceptance text, and command records.

## Blockers

None for P14.

## Next-Pass Readiness

P15 is unblocked from the P14 release/product surface perspective. Do not start P15 until explicitly requested.

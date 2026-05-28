# P27 Phase 18 Report — Support Profile and Operator UX Closure

## Scope

Phase 18 addressed `P27-016`: support claims outrunning evidence. It also tied the operator quickstart and support docs back to the evidence produced for `P27-009`, `P27-010`, `P27-011`, `P27-012`, and `P27-018`.

No-go zones observed:

- No hosted/cloud provider support was promoted.
- No broad autonomy, V10, V11, V12, federation, or mechanism runtime claim was promoted.
- No canonical-owner boundary changed.
- No capability code was added beyond a documentation guard script.

## Changes

- Rewrote `SUPPORT_PROFILE.md` from pre-validation target language into an evidence-bounded P27 support matrix.
- Added `docs/P27_SUPPORT_TRACEABILITY.md` mapping each support claim to phase reports, audit logs, and semantic status.
- Replaced stale `docs/OPERATOR_QUICKSTART.md` P20-era verifier commands with P27 AgentSpec/local mock commands and known-limit disclosures.
- Updated `README.md` to remove stale opening-gate language and point operators at current support traceability.
- Updated AgentSpec example READMEs from `target/p26/...` to `target/p27/...` and documented semantic disclosure / partial memory-seam boundaries.
- Added `scripts/assert_p27_support_docs_traceable.py` and wired it into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to close `P27-016` and record Phase 18.

## Changed Files

- `SUPPORT_PROFILE.md`
- `README.md`
- `STATUS.md`
- `docs/OPERATOR_QUICKSTART.md`
- `docs/P27_SUPPORT_TRACEABILITY.md`
- `examples/agents/local-coding-agent/README.md`
- `examples/agents/memory-grounded-agent/README.md`
- `scripts/assert_p27_support_docs_traceable.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_18_REPORT.md`

## Validation

Command logs are under `target/p27/audit/`.

- `python3 scripts/assert_p27_support_docs_traceable.py .` passed: `target/p27/audit/assert_p27_support_docs_traceable_phase18.log`
- Stale current-doc scan passed with no matches: `target/p27/audit` command output was empty for `P20_2_REQUIRE_CARGO`, `scripts/p20_2_verify.sh`, `target/p26/examples`, stale target-language, and stale ownership-scanner blocker text.
- `cargo fmt --all -- --check` passed: `target/p27/audit/cargo_fmt_phase18.log`
- `python3 -m py_compile scripts/assert_p27_support_docs_traceable.py scripts/assert_p27_semantic_disclosure.py` passed: `target/p27/audit/py_compile_phase18_support_docs.log`
- `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` passed: `target/p27/audit/verify_current_phase18_skip_cargo.log`
- Quickstart `agent validate` passed: `target/p27/audit/phase18_quickstart_agent_validate.log`
- Quickstart `agent doctor` passed: `target/p27/audit/phase18_quickstart_agent_doctor.log`
- Quickstart `agent run` passed and honestly abstained without a scoped permit: `target/p27/audit/phase18_quickstart_agent_run.log`
- Quickstart `agent inspect` passed with `event_log_digest_verified=true` and `semantic_status=degraded_exact_check`: `target/p27/audit/phase18_quickstart_agent_inspect.log`
- `cargo run -p aidens-cli -- package examples --root .` passed: `target/p27/audit/phase18_package_examples.log`
- `cargo run -p aidens-cli -- package readiness --root . --config examples/aidens.mock.toml` passed: `target/p27/audit/phase18_package_readiness.log`
- `cargo test -p aidens-cli doctor_reports_provider_capability_matrix_without_cloud_or_native_overclaims` passed: `target/p27/audit/cargo_test_aidens_cli_phase18_doctor_support.log`
- `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` passed: `target/p27/audit/cargo_test_aidens_cli_phase18_run_bundle_support.log`
- `cargo check -p aidens-cli` passed: `target/p27/audit/cargo_check_aidens_cli_phase18.log`

A combined two-filter Cargo test command was attempted with invalid Cargo syntax and failed before running tests: `target/p27/audit/cargo_test_aidens_cli_phase18_support_docs.log`. The two filters were then rerun as separate commands and passed in the logs listed above.

## Support-Tier Changes

Yes. The support profile changed from “inherited and to be revalidated” target language to evidence-bounded P27 claims. Evidence is in `SUPPORT_PROFILE.md`, `docs/P27_SUPPORT_TRACEABILITY.md`, and the validation logs above.

No unsupported cloud/autonomy/V10+ claim was promoted.

## Canonical Ownership

No canonical-owner boundary changed. The new support traceability doc explicitly labels itself as a local operator traceability surface, not canonical truth.

## Exact / Approx / Degradation Labels

Documentation labels changed in:

- `SUPPORT_PROFILE.md`
- `docs/P27_SUPPORT_TRACEABILITY.md`
- `docs/OPERATOR_QUICKSTART.md`
- `examples/agents/local-coding-agent/README.md`
- `examples/agents/memory-grounded-agent/README.md`

The changed labels are documentation disclosures of existing Phase 17 runtime labels: `exact check`, `degraded_exact_check`, `display_only`, `partial`, `fixture-backed`, `deferred-cloud`, `deferred-autonomy`, and `design-only`.

## Quarantine

No issues quarantined.

Decision: continue

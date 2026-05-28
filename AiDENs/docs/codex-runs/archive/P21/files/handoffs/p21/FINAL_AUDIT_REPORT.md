# P21 Final Hostile Audit Report

## Verdict

P21 passes the mandatory final audit after one Phase 10 repair.

The repair: `package examples` previously marked Ollama examples as `supported` even though provider truth reports `partial-local-chat` and `ollama-local-service-required`. The classifier now reports Ollama examples as `partial` with explicit reason codes, and the package manifest test was expanded.

No mandatory gate remains failed.

## Code-First Basis

This audit used code and command output, not claims:

- profile status from `aidens profile list` and `aidens profile explain`;
- provider/tool truth from `provider-check`, `tools inspect`, provider tests, and tool tests;
- generated project proof from `aidens new coding-agent`, `aidens run`, `doctor`, `provider-check`, `tools inspect`, `plan validate`, and `plan compile`;
- execution evidence from runner tests, `run-test-agent` output bundle, receipts/event log, and agency reports;
- package/archive truth from `p21_verify`, archive replay, and source-reference scanners;
- final build proof from `cargo fmt`, `cargo check`, `cargo test`, and `cargo clippy`.

## Supported

- `chat-only`: supported profile, no tools by default.
- `coding-agent`: supported profile with safe read/list/search/stat/propose surface and permit-gated write/admin actions.
- `mock` provider: supported fixture provider, executable and tested.
- `run-test-agent`: supported CLI command using the real runner path.
- Generated `coding-agent`: supported safe mock project and verified runnable.
- Profile/plan/operator surfaces: `profile`, `plan`, `doctor`, `status`, `provider-check`, `tools inspect`, `receipts`, `boundary`, `schemas`, `queue` smoke path.
- Agency v0.2 gate: runtime policy gate when enabled, not prompt-only.
- Release archive replay: supported verifier with zero missing required paths.

## Partial

- `memory-agent`: partial/proof-only; canonical memory crates own truth.
- `autonomous-daemon`: partial/safe-mode; queue/schedule/wake/lease/safe-mode/drain are proven, not a full daemon.
- `ollama`: partial local chat provider; no native tool loop and local service required.
- Daemon smoke: bounded queue/schedule/wake operator proof only.
- Recall/Recall-Coding extraction: safe patterns/templates/reports only.

## Scaffold

- `aidens-profile-daemon`
- `aidens-profile-desktop`
- `aidens-profile-memory`
- `aidens-profile-research`

These are not promoted as complete.

## Deferred

- OpenAI, OpenRouter, Anthropic, and generic OpenAI-compatible cloud providers.
- Native provider tool loops.
- Streaming provider support.
- Full desktop daemon UX, IPC/socket server, timer loop, and host wake wrapper.
- Multi-agent fanout.
- Federation, remote oracle admission, attested exchange, settlement, mechanism search, and full research workbench.

## Failed

- None after the Phase 10 Ollama example-status repair.

## Quarantined

- Recall app DB/session/socket/UI state and scheduler schema.
- Recall-Coding local data roots, hooks, checkpoint store, tool IDs, and agent manifest assumptions.
- Any local substitute for canonical stack truth.

## Mandatory Gate Results

- `cargo fmt --all --check` -> PASS.
- `cargo check --workspace --all-targets --all-features` -> PASS.
- `cargo test --workspace --all-targets --all-features` -> PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS.
- `bash scripts/p21_verify.sh` -> PASS.
- `cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml` -> PASS.
- `cargo run -p aidens-cli -- new coding-agent target/demo-agent` -> PASS.
- `cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"` -> PASS.
- `cargo run -p aidens-cli -- doctor --config target/demo-agent/aidens.toml` -> PASS.
- `cargo run -p aidens-cli -- provider-check --config target/demo-agent/aidens.toml` -> PASS.
- `cargo run -p aidens-cli -- tools inspect --config target/demo-agent/aidens.toml` -> PASS.
- `cargo run -p aidens-cli -- plan validate --config target/demo-agent/aidens.toml` -> PASS.
- `cargo run -p aidens-cli -- plan compile --config target/demo-agent/aidens.toml --out target/demo-agent/aidens.plan.json` -> PASS.
- `bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip` -> PASS before report writing; final archive replay is rerun after this report is packaged.

## Command Logs

Final logs are under `target/p21/audit/`.

Key logs:

- `cargo_fmt_check.final.log`
- `cargo_check_workspace_all_targets_all_features.final.log`
- `cargo_test_workspace_all_targets_all_features.final.log`
- `cargo_clippy_workspace_all_targets_all_features.final.log`
- `run_test_agent.log`
- `new_coding_agent.log`
- `demo_agent_run.log`
- `demo_agent_doctor.log`
- `demo_agent_provider_check.log`
- `demo_agent_tools_inspect.log`
- `demo_agent_plan_validate.log`
- `demo_agent_plan_compile.log`
- `package_examples.after_fix.log`
- `p21_verify.final.log`
- `archive_replay.final.log`
- `archive_verifier_report.final.json`

## Changed File Summary

Phase 10 source changes:

- `crates/aidens-cli/src/lib.rs`: classify Ollama examples as partial/local-service-required instead of supported; test added.
- `handoffs/p21/KNOWN_LIMITATIONS.md`: final limitations register.
- `handoffs/p21/FINAL_AUDIT_REPORT.md`: this audit report.
- `handoffs/p21/PHASE_10_REPORT.md`: phase boundary handoff.
- `target/p21/audit/COMMAND_LOG_SUMMARY.md`
- `target/p21/audit/CHANGED_FILE_SUMMARY.md`
- `target/p21/audit/UNRESOLVED_RISKS.md`

P21 cumulative changed surfaces include CLI, runner, app/profile/plan/provider/tool/agency/daemon/testkit crates, examples, fixtures, scripts, docs, handoffs, evals, and release/audit logs. The parent Git repository reports `AiDENs/` as an untracked directory, so this summary is workspace-audit based rather than Git-diff based.

## Unresolved Risks

- Descriptive handoff filenames in `docs/p21/P21_RELEASE_AND_AUDIT_REQUIREMENTS.md` do not exactly match the canonical phase files written by the phase protocol. Canonical phase reports are present and packaged.
- Target audit logs are generated artifacts and must be preserved outside normal source-control ignore behavior.
- Ollama remains local-service-dependent; final audit classifies it partial, not full support.
- Memory, daemon, research, cloud providers, native tool loops, federation, and mechanism surfaces remain partial/deferred as listed in `KNOWN_LIMITATIONS.md`.

## Archive Status

The release zip is `target/p21/aidens-v0.1-candidate.zip`. It is rebuilt and replayed after final audit files are written so the archive contains the final audit report, known limitations register, Phase 09/10 handoffs, and stretch artifacts. The final replay log is `target/p21/audit/archive_replay.final.log`.

## Outcome

Final hostile audit passed with disclosed limitations and no remaining mandatory gate failure.

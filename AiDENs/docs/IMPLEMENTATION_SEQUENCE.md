# Implementation Sequence

## P0 — Fake completion removal

1. Add executable provider trait in `aidens-provider-kit`.
2. Implement disabled provider and explicit mock provider.
3. Modify `aidens-runner` to call provider trait.
4. Make disabled provider fail instead of answer.
5. Remove placeholder strings.

Exit gate:

```bash
cargo test -p aidens-runner --test next_runner_provider_contract
bash scripts/assert_no_fake_completion.sh
```

## P1 — Read-only tool dispatch

1. Add `ToolDispatcher` in `aidens-tool-kit`.
2. Add sandboxed `aidens:repo-read:1`.
3. Reject traversal and absolute escapes.
4. Emit tool attempt receipt.
5. Keep dangerous shell/write/network tools absent by default.

Exit gate:

```bash
cargo test -p aidens-tool-kit --test next_repo_read_dispatch
```

## P2 — AppPlan and Doctor

1. Add `AiDENsAppPlanV1` if missing/insufficient.
2. Add `AiDENsDoctorReportV1`.
3. Implement profile list/explain.
4. Implement plan validate/compile.
5. Implement doctor --config.
6. Implement run --config.

Exit gate:

```bash
cargo test -p aidens-cli --test next_cli_plan_doctor
cargo test -p aidens-app-kit --test next_app_plan_facade
```

## P3 — Generated app smoke

1. `aidens new coding-agent /tmp/aidens-smoke` creates a minimal facade-only app.
2. Generated `aidens.toml` uses disabled or mock provider safely.
3. Generated app compiles when run from the AiDENs workspace root.

Exit gate:

```bash
bash scripts/next_smoke.sh
```

## P4 — Optional real provider enhancement

If dependency paths allow, wire real Ollama provider using Recall's provider/llm-pipeline patterns. If not, keep the trait and explicit mock/disabled behavior passing and record the path blocker in `PASS_STATUS.md`.

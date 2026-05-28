# Acceptance Gates

## Hard gates

```bash
bash scripts/assert_no_fake_completion.sh
bash scripts/next_smoke.sh
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Targeted tests

```bash
cargo test -p aidens-testkit --test next_no_fake_completion
cargo test -p aidens-runner --test next_runner_provider_contract
cargo test -p aidens-tool-kit --test next_repo_read_dispatch
cargo test -p aidens-cli --test next_cli_plan_doctor
cargo test -p aidens-app-kit --test next_app_plan_facade
```

## Functional gates

- `aidens run --config examples/aidens.mock.toml "hello"` returns configured mock text.
- `aidens run --config examples/aidens.disabled.toml "hello"` fails or reports disabled, never answers.
- `aidens profile explain coding-agent` shows approval requirements.
- `aidens plan compile` writes JSON.
- `aidens doctor` reports provider/tool/receipt truth.
- `aidens new coding-agent /tmp/aidens-smoke` creates facade-only app.

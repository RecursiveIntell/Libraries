# Operator Quickstart

Use the mock config for local product smoke. It exercises the real CLI, runner, tool exposure, and durable receipt inspection without requiring a network provider.

```bash
cargo run -p aidens-cli -- new coding-agent target/my-aidens-agent
cargo run -p aidens-cli -- profile list
cargo run -p aidens-cli -- profile explain coding-agent
cargo run -p aidens-cli -- status --config examples/aidens.mock.toml
cargo run -p aidens-cli -- provider-check --config examples/aidens.mock.toml
cargo run -p aidens-cli -- tools inspect --config examples/aidens.mock.toml
cargo run -p aidens-cli -- run --config examples/aidens.mock.toml hello
cargo run -p aidens-cli -- receipts list --config examples/aidens.mock.toml
cargo run -p aidens-cli -- package readiness --root . --config examples/aidens.mock.toml
bash scripts/check_examples.sh
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

## Example Truth

- `examples/aidens.chat-only.toml` and `examples/aidens.mock.toml` are fixture-supported local mock paths, not cloud support.
- `examples/aidens.memory.toml`, `examples/aidens.daemon.toml`, and `examples/aidens.research.toml` are partial profile examples; their substrate pieces exist, but profile packaging remains explicit.
- `examples/aidens.ollama.toml` is a local-provider diagnostic path. Ordinary chat can be executable when Ollama is reachable; native tool loop remains false.
- `examples/aidens.openai-unavailable.toml` is an API-provider diagnostic fixture. It should report unavailable or blocked until real HTTP provider boundaries are implemented.

Run `aidens package examples --root .` for the typed example manifest and `aidens package readiness --root . --config examples/aidens.mock.toml` for the release readiness report.

`aidens doctor`, `aidens status`, `aidens provider-check`, `aidens tools inspect`, and package reports expose support-tier JSON. The tiers are operator labels only: `supported`, `partial`, `scaffold`, `deferred`, and `failed`. They are derived from emitted capability states and release surfaces; they do not create canonical stack truth.

Historical P20/P21 provider expansion material is archived, non-normative evidence and is not the current P22 gate.

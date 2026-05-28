# Operator Quickstart

These commands exercise the P28 supported-local path. They use local/mock execution, emit receipts/run bundles, and do not require cloud credentials.

## Local Agent Run

```bash
cargo run -p aidens-cli -- agent validate --spec examples/agents/local-coding-agent/agent.json
cargo run -p aidens-cli -- agent doctor --spec examples/agents/local-coding-agent/agent.json
cargo run -p aidens-cli -- agent run --spec examples/agents/local-coding-agent/agent.json --task examples/agents/local-coding-agent/task.md --sandbox-root examples/agents/local-coding-agent/sandbox --out target/p28/examples/local-coding-agent
cargo run -p aidens-cli -- agent inspect --run target/p28/examples/local-coding-agent
```

Expected local evidence includes run bundle JSON, event logs, receipt logs, material operation receipts, and semantic disclosure blocks. These are local operator artifacts, not canonical domain truth.

## P28 Verification

Use the current verifier:

```bash
bash scripts/verify_current.sh
```

For final strict evidence:

```bash
P28_FINAL_STRICT=1 bash scripts/verify_current.sh
```

Package and replay commands are listed in `P28_COMMANDS.md`. Final package paths are produced under `target/p28/package/` and audit logs under `target/p28/audit/`.

## Existing Config Smoke

```bash
cargo run -p aidens-cli -- profile list
cargo run -p aidens-cli -- profile explain coding-agent
cargo run -p aidens-cli -- status --config examples/aidens.mock.toml
cargo run -p aidens-cli -- provider-check --config examples/aidens.mock.toml
cargo run -p aidens-cli -- tools inspect --config examples/aidens.mock.toml
cargo run -p aidens-cli -- run --config examples/aidens.mock.toml hello
cargo run -p aidens-cli -- receipts list --config examples/aidens.mock.toml
cargo run -p aidens-cli -- package readiness --root . --config examples/aidens.mock.toml
bash scripts/check_examples.sh
```

## Example Truth

- `examples/agents/local-coding-agent` is the primary supported-local AgentSpec fixture.
- `examples/agents/memory-grounded-agent` exercises canonical memory adapter/backpointer evidence; it does not create an AiDENs-local memory truth store.
- `examples/aidens.chat-only.toml` and `examples/aidens.mock.toml` are fixture-supported local mock paths, not cloud support.
- `examples/aidens.ollama.toml` is a local-provider diagnostic path. Ordinary chat can be executable when Ollama is reachable; native tool loop remains false.
- `examples/aidens.openai-unavailable.toml` is an API-provider diagnostic fixture. It should report unavailable or blocked until real hosted-provider boundaries are implemented and tested.

## Known Limits

AiDENs is not production-cloud-ready, broadly autonomous, v11B active, v11C active, or a replacement for canonical memory/governance/kernel/runtime crates. `SUPPORT_PROFILE.md`, `docs/p28/P28_SUPPORT_TRACEABILITY.md`, and `docs/p28/P28_KNOWN_LIMITATIONS_REGISTER.md` are the active P28 support surfaces.

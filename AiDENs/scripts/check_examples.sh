#!/usr/bin/env bash
set -euo pipefail

cargo run -q -p aidens-cli -- package examples --root . >/dev/null
cargo run -q -p aidens-cli -- package readiness --root . --config examples/aidens.mock.toml >/dev/null
profiles="$(cargo run -q -p aidens-cli -- profile list)"
for profile in chat-only coding-agent memory-agent autonomous-daemon research-workbench
do
  grep -q "^${profile}" <<<"$profiles"
  explanation="$(cargo run -q -p aidens-cli -- profile explain "$profile")"
  grep -q "$profile" <<<"$explanation"
  grep -Eq 'provider_required=|No risky capabilities|permit_required' <<<"$explanation"
done
cargo run -q -p aidens-cli -- status --config examples/aidens.mock.toml >/dev/null
cargo run -q -p aidens-cli -- doctor --config examples/aidens.mock.toml >/dev/null
cargo run -q -p aidens-cli -- tools inspect --config examples/aidens.mock.toml >/dev/null
cargo run -q -p aidens-cli -- inspect-tools --config examples/aidens.mock.toml >/dev/null
cargo run -q -p aidens-cli -- memory status --config examples/aidens.memory.toml >/dev/null

for config in \
  examples/aidens.chat-only.toml \
  examples/aidens.mock.toml \
  examples/aidens.disabled.toml \
  examples/aidens.ollama.toml \
  examples/aidens.openai-unavailable.toml \
  examples/aidens.memory.toml \
  examples/aidens.daemon.toml \
  examples/aidens.research.toml
do
  cargo run -q -p aidens-cli -- provider-check --config "$config" >/dev/null
done

mock_provider="$(cargo run -q -p aidens-cli -- provider-check --config examples/aidens.mock.toml)"
grep -q '"provider": "mock"' <<<"$mock_provider"
grep -q '"executable": true' <<<"$mock_provider"
grep -q '"native_tool_loop": false' <<<"$mock_provider"
grep -q '"structured_output": true' <<<"$mock_provider"

openai_provider="$(cargo run -q -p aidens-cli -- provider-check --config examples/aidens.openai-unavailable.toml)"
grep -q '"provider": "openai"' <<<"$openai_provider"
grep -q '"executable": false' <<<"$openai_provider"
grep -q '"route": "unavailable"' <<<"$openai_provider"
grep -q '"native_tool_loop": false' <<<"$openai_provider"
grep -q '"structured_output": false' <<<"$openai_provider"

ollama_provider="$(cargo run -q -p aidens-cli -- provider-check --config examples/aidens.ollama.toml)"
grep -q '"provider": "ollama"' <<<"$ollama_provider"
grep -q '"native_tool_loop": false' <<<"$ollama_provider"
grep -q '"structured_output": false' <<<"$ollama_provider"

cargo run -q -p aidens-cli -- plan validate --config examples/aidens.mock.toml >/dev/null
cargo run -q -p aidens-cli -- plan validate --config examples/aidens.memory.toml >/dev/null
cargo run -q -p aidens-cli -- run --config examples/aidens.mock.toml hello >/dev/null
cargo run -q -p aidens-cli -- receipts list --config examples/aidens.mock.toml >/dev/null

generated_root="target/p14-example-fixtures"
rm -rf "$generated_root"
mkdir -p "$generated_root"
cargo run -q -p aidens-cli -- new coding-agent "$generated_root/generated-coding-agent" >/dev/null
cargo test --manifest-path "$generated_root/generated-coding-agent/Cargo.toml" >/dev/null

echo "Example compile/test smoke passed."

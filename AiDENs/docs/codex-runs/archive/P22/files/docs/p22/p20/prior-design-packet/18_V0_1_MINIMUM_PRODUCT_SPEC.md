# 18 — AiDENs v0.1 Minimum Product Spec

This is the smallest useful AiDENs release. Anything less is just a template generator.

## Required crates

```text
aidens
aidens-app-kit
aidens-runner
aidens-contracts
aidens-boundary-kit
aidens-config
aidens-receipts
aidens-capability-kit
aidens-provider-kit
aidens-tool-kit
aidens-security-kit
aidens-permit-kit
aidens-arbiter-kit
aidens-budget-kit
aidens-cli
aidens-testkit
aidens-profile-coding
```

## Deferred crates

```text
aidens-memory-kit
aidens-queue-kit
aidens-schedule-kit
aidens-wake-kit
aidens-daemon-kit
aidens-tauri-kit
aidens-kernel-kit
aidens-plan-kit
aidens-delegation-kit
aidens-repair-kit
```

Memory/queue/daemon may be pulled earlier if Recall conversion requires it, but the v0.1 app-speed target should not wait on graph/kernel/federation/mechanism features.

## Required CLI

```bash
aidens new coding-agent <name>
aidens doctor
aidens check-config
aidens list-tools
aidens list-capabilities
aidens provider-check
aidens run <prompt>
aidens receipts inspect <run-id>
```

## Required generated template

```text
aidens.toml
src/main.rs
src/tools.rs
src/profile.rs
tests/doctor.rs
tests/no_hidden_fallback.rs
tests/tool_approval.rs
tests/capability_truth.rs
README.md
```

## Required runtime behavior

1. App starts with one provider.
2. App can run with no tools.
3. App can expose one read-only tool.
4. App can expose one write tool requiring approval.
5. Native tool route works where provider supports it.
6. Parser fallback works only as explicit/degraded route.
7. Every run emits a receipt.
8. Every tool attempt emits a receipt.
9. Every approval/denial emits a receipt.
10. Every boundary repair emits a receipt.
11. Capability truth reports provider/tool/config status.
12. Doctor can explain why app is blocked.

## Required safety behavior

The following must be impossible or fail tests:

```text
unknown native provider silently mapped to native OpenAI mode
write tool runs without permit
disabled tool is registered/exposed
tool exposure includes full registry by default
parser fallback route lacks degraded receipt
config secret appears in status output
run has no config_generation_id
provider unavailable but runtime reports ready
profile enables shell/web/write auto-approval by default
```

## First-class examples

```text
examples/chat_only
examples/read_tool_agent
examples/write_tool_with_approval
examples/parser_fallback_explicit
examples/provider_failover_receipts
```

## Release criteria

```bash
cargo check --workspace
cargo test --workspace
aidens schemas check
aidens check-deps
aidens new coding-agent smoke-test
cd smoke-test
cargo test
cargo run -- "read Cargo.toml"
```

All must pass before v0.1.

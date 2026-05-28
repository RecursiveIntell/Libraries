# AiDENs Coding-Agent Example

This example is the safe AiDENs extraction of Recall-Coding's coding lane pattern.

It keeps the reusable workflow:

1. inspect the workspace with read-only tools;
2. make bounded changes only through permit-gated tools;
3. run explicit checks;
4. keep provider route, tool calls, permits, failures, agency reports, and final output receipt-bearing.

It intentionally does not import Recall-Coding's app-local data roots, hook runner, agent manifest format, checkpoint storage, socket/session assumptions, or tool IDs.

## Config

Use:

```bash
cargo run -p aidens-cli -- run --config examples/configs/coding-agent.toml "read README"
cargo run -p aidens-cli -- doctor --config examples/configs/coding-agent.toml
cargo run -p aidens-cli -- tools inspect --config examples/configs/coding-agent.toml
```

The default provider is `mock`, memory mode is `optional`, receipts are `full`, and write/admin tools remain permit-gated.

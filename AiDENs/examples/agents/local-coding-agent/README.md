# AgentSpecV1 Local Coding Agent

This example uses `aidens agent` commands over `AgentSpecV1` and a local sandbox.

```bash
cargo run -p aidens-cli -- agent validate --spec examples/agents/local-coding-agent/agent.json
cargo run -p aidens-cli -- agent doctor --spec examples/agents/local-coding-agent/agent.json
cargo run -p aidens-cli -- agent run --spec examples/agents/local-coding-agent/agent.json --task examples/agents/local-coding-agent/task.md --sandbox-root examples/agents/local-coding-agent/sandbox --out target/p27/examples/local-coding-agent
cargo run -p aidens-cli -- agent inspect --run target/p27/examples/local-coding-agent
```

Writes and checks remain permit-gated. A run without a scoped permit emits abstention and repair display records instead of claiming success.
Validation, doctor, loop display, and inspect outputs include `semantic_disclosure` labels for exactness, support tier, degradation, proof checks, and known limits.

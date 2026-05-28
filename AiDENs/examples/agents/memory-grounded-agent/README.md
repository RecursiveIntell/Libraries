# AgentSpecV1 Memory-Grounded Agent

This example enables `memory_policy.mode = canonical-seam`. AiDENs imports and queries through the canonical memory adapter path and records evidence in `AiDENsRunBundleV3`; it does not create an AiDENs-local memory truth store.

```bash
cargo run -p aidens-cli -- agent run --spec examples/agents/memory-grounded-agent/agent.json --task examples/agents/memory-grounded-agent/task.md --sandbox-root examples/agents/memory-grounded-agent/sandbox --out target/p27/examples/memory-grounded-agent
cargo run -p aidens-cli -- agent inspect --run target/p27/examples/memory-grounded-agent
```

This is partial canonical-seam evidence, not an independent supported memory product surface.

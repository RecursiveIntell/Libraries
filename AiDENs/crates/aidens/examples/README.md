# The Auditor — AiDENs Demo Agent

A governed, memory-grounded, reasoning-driven code review agent that showcases every unique AiDENs capability in a single run.

## What It Does

Given a directive to find unsafe Rust patterns, The Auditor:

1. **Boots** with AutonomousDaemon profile (memory + governance + kernel)
2. **Searches** semantic memory for prior context
3. **Stores** findings as facts with provenance, confidence scores, and source attribution
4. **Creates** graph edges between related findings
5. **Compiles** findings into a constraint graph via the kernel adapter
6. **Checks** 32 conformance gates
7. **Runs** the agent with governance permit enforcement
8. **Verifies** memory integrity (FTS, embeddings, vectors)
9. **Emits** a tamper-evident, hash-chained receipt chain
10. **Produces** an RFC 8785 canonical JSON audit report with a content digest

## How to Run

```sh
cargo run --example the_auditor -p aidens
```

## What Makes This Different

No other Rust agent framework can do this in a single run:

| Capability | AiDENs | rig | kalosm |
|---|---|---|---|
| Semantic memory with provenance | Yes (138 methods) | Basic vectors | Basic memory |
| Graph edges between facts | Yes (4 edge types) | No | No |
| Memory integrity verification | Yes (Full/Quick) | No | No |
| Governance permit enforcement | Yes (before every tool) | No | No |
| Kernel constraint compilation | Yes | No | No |
| Conformance gates | Yes (32 gates) | No | No |
| Tamper-evident receipt chains | Yes (hash-chained NDJSON) | No | No |
| Sandbox path validation | Yes (traversal/hardlink/symlink) | No | No |
| RFC 8785 JSON canonicalization | Yes | No | No |
| Profile-driven defaults | Yes (5 profiles) | No | No |

**The pitch:** Other frameworks give you a loop that calls an LLM. AiDENs gives you an OS that runs agents — with memory, permissions, reasoning, audit logs, sandboxing, and type safety.

## The Sample Crate

The demo references `examples/sample-crate/` — a small Rust crate with 4 intentional unsafe patterns (pointer dereference, raw pointer arithmetic, transmute, mutable static). The safe functions in the crate show what the agent should recognize as fine.

## Architecture

```
AiDENsApp::builder()
    .profile(AutonomousDaemon)     ← memory + governance auto-configured
    .mock_provider("audit result")
    .tools(safe_coding_registry)
    .governance(Some(ctx))
    .kernel(Some(adapter))
    .build()

→ memory.search()       ← grounding
→ memory.add_fact()     ← findings with provenance
→ kernel.conformance()  ← 32 gates checked
→ runner.run()          ← governance-checked execution
→ memory.verify()       ← integrity verified
→ boundary.canonical()  ← RFC 8785 digest
```

## License

Same as the AiDENs workspace.
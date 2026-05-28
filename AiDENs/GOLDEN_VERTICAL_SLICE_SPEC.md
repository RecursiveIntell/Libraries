# Golden Vertical Slice Spec

This is the first non-negotiable proof that AiDENs is becoming real.

## Required flow

```text
operator input
→ AiDENs runner creates app-level run context
→ provider/tool path executes
→ canonical tool/execution receipt is emitted
→ evidence/export path enters Forge or canonical evidence owner
→ bridge/import path transforms into memory projection
→ semantic memory stores/queryable projection
→ knowledge-runtime query returns result with provenance/widening/degradation disclosure
→ AiDENs CLI displays answer plus receipt/provenance chain
```

## Expected canonical crates

| Step | Expected crate(s) | Source references |
|---|---|---|
| app run context | `aidens-runner` + `stack-ids::TraceCtx` | `~/Coding/Libraries/stack-ids/src/lib.rs:L1-L25` |
| provider/tool execution | `llm-tool-runtime` | `~/Coding/Libraries/llm-tool-runtime/src/contracts.rs:L1-L10 and L155-L240 and L326-L638; ~/Coding/Libraries/llm-tool-runtime/src/runtime.rs:L178-L232` |
| Forge receipt/export | `semantic-memory-forge` | `~/Coding/Libraries/semantic-memory-forge/src/lib.rs:L3-L28 and L39-L56 and L79-L82` |
| bridge transform | `forge-memory-bridge::transform_envelope_v3` | `~/Coding/Libraries/forge-memory-bridge/src/transform.rs:L123-L188` |
| memory import/query | `semantic-memory::MemoryStore` | `~/Coding/Libraries/semantic-memory/Cargo.toml:L20-L33 and ~/Coding/Libraries/semantic-memory/src/lib.rs:L159-L327` |
| runtime query/disclosure | `knowledge-runtime::KnowledgeRuntime`, `QueryTrace` | `~/Coding/Libraries/knowledge-runtime/src/lib.rs:L1-L27 and L49-L72 and L111-L140` |
| CLI display | `aidens-cli` | `~/Coding/Libraries/AiDENs/crates/aidens-cli/src/lib.rs:L26-L29 and L793-L867 and L955-L979` |

## Mock strategy

Use a deterministic fake provider/tool. It may fake external I/O, but it must not fake canonical stack calls. The test must exercise actual stack crate types/functions at every boundary.

Acceptable fake:

- deterministic provider returns known text/tool call;
- local read-only tool returns known artifact;
- temporary in-memory semantic-memory DB.

Not acceptable:

- writing to an AiDENs-local memory store and calling it memory;
- building an AiDENs-local episode/evidence bundle and calling it canonical Forge evidence;
- returning a fake `QueryTrace` without invoking `knowledge-runtime`.

## Required test

`cargo test -p aidens-testkit golden_vertical_slice`

The test must prove:

1. `stack_ids::TraceCtx` or stack IDs are used.
2. `llm_tool_runtime::ToolReceipt` is created.
3. A Forge-compatible receipt/export path is produced.
4. `forge_memory_bridge::transform_envelope_v3` or the discovered canonical V3 bridge API is called.
5. `semantic_memory::MemoryStore` receives/imports projection data.
6. `knowledge_runtime` executes a query or produces a real runtime trace/disclosure.
7. CLI/runner output includes the receipt/provenance chain.
8. No production path uses an AiDENs-local memory store as truth.

If an exact API differs, Codex must inspect the cited crate source and update the test around the real API. It must not invent fake APIs.

## Failure cases that must follow later

Owned by phases 4-6:

- malformed tool call emits degradation/repair receipt;
- denied tool requires approval receipt;
- budget exhaustion emits canonical budget/deadline receipt;
- provider route unavailable is visible, not hidden;
- query widening disclosure is visible;
- promotion without verification is denied.

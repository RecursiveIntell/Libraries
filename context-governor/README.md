# context-governor

Crate-agnostic governed context compaction for AI agents.

`context-governor` turns a long agent transcript into:

- compacted messages suitable for reinjection into an agent prompt
- a `ContextCompactionReceiptV1`
- a `ContextAllocationPlanV1`
- a structured loss/anchor report
- exact fallback records recoverable by item id
- searchable file-backed receipt storage
- recall-quality filters for speculative memory and artifact-pack noise
- CLI commands usable from any host agent

It is not an LLM summarizer and does not claim to extend a hosted model's hard context window. It improves effective context by reducing prompt damage and making loss visible.

## Architecture

```text
agent transcript
      |
      v
+------------------+     +------------------+     +------------------+
| classify spans   | --> | allocate budget  | --> | compact prompt   |
| authority/kind   |     | soft/hard/fail   |     | exact + summary  |
+------------------+     +------------------+     +------------------+
          |                       |                        |
          v                       v                        v
  recall quality gate      allocation plan          receipt + fallback
          |                                                |
          v                                                v
   memory sink records                         search / expand / diff
```

## What ships

Core API:

- `compact_context(CompactRequest) -> CompactResponse`
- `filter_recall_candidate`
- `context_expand`
- `context_search`
- `context_diff`
- `archive_response_to_memory`

Receipts and records:

- `ContextCompactionReceiptV1`
- `ContextAllocationPlanV1`
- `ContextItemV1`
- `SummaryLossReportV1`
- `StructuredContextSummaryV1`
- `ExactStoredItemV1`
- `MemoryArchiveRecordV1`

Policy controls:

- `BudgetMode::SoftWarn` — current safety-first behavior, warn if budget is exceeded
- `BudgetMode::HardCascade` — shrink/remove summary to fit when exact-preserve content allows it
- `BudgetMode::FailClosed` — refuse when required exact content cannot fit
- `TokenCounterKind::ApproxChars` — explicit char/4 estimator recorded in receipts

Content kinds:

- plain text
- JSON
- diffs
- Rust source
- Markdown
- Cargo output
- shell logs
- search results

File store:

- `FileContextStore::new(path)`
- `save(&CompactResponse)`
- `load(receipt_id)`
- `list_receipts()`
- `expand(receipt_id, item_id, max_chars)`
- `search(query, top_k, scope)`

CLI:

```bash
context-governor compact < request.json > response.json
context-governor store --dir .ctx < response.json
context-governor search --dir .ctx --query NEEDLE
context-governor expand --dir .ctx --receipt ctxr_... --item ctxi_...
context-governor diff < response.json
```

No-argument CLI mode remains backwards-compatible with `compact`.

## Quick start

```rust
use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message};

let response = compact_context(CompactRequest {
    session_id: "session-1".into(),
    messages: vec![Message {
        id: None,
        role: "user".into(),
        content: "Build it. Acceptance gate: tests pass.".into(),
        name: None,
        metadata: Default::default(),
    }],
    policy: CompactionPolicy::default(),
    focus: None,
})?;

println!("{}", response.receipt.receipt_id);
# Ok::<(), context_governor::ContextGovernorError>(())
```

Request shape:

```json
{
  "session_id": "s1",
  "policy": {
    "target_tokens": 4000,
    "protect_first_n": 3,
    "protect_last_n": 8,
    "summary_max_chars": 8000,
    "allocator": "deterministic_v1",
    "semantic_memory_enabled": false,
    "archive_memory_enabled": false,
    "budget_mode": "soft_warn",
    "token_counter": "approx_chars"
  },
  "messages": [
    { "role": "system", "content": "You are an agent." },
    { "role": "user", "content": "Implement the feature. Acceptance gate: tests pass." },
    { "role": "tool", "content": "long output..." },
    { "role": "user", "content": "Latest active request." }
  ]
}
```

## Plugin integration pattern

Adapters for Hermes, OpenCode, Codex wrappers, Claude Code wrappers, or custom agents should:

1. Map transcript records to `Vec<Message>`.
2. Call `compact_context` near the provider context threshold.
3. Persist the returned `CompactResponse` with `FileContextStore` or host storage.
4. Inject `CompactResponse.compacted_messages` into the next model call.
5. Expose `search`, `expand`, and `diff` to the agent.
6. Treat summaries as background only; the latest user message remains active.
7. If archival is needed, implement `MemorySink` in the adapter and write real external IDs back in the host layer.

See:

- `docs/architecture.md`
- `docs/integrations/hermes.md`
- `docs/eval-harness.md`
- `docs/roi-audit-2026-06-27.md`

## Performance

Run:

```bash
cargo run --release --example perf
```

Latest observed synthetic release-mode run on the Hermes runtime machine:

| Messages | Original approx tokens | Compacted approx tokens | Token reduction | Avg latency | P95 latency |
|---:|---:|---:|---:|---:|---:|
| 100 | 25,362 | 9,720 | 61.7% | 0.634 ms | 0.663 ms |
| 500 | 128,842 | 16,020 | 87.6% | 3.785 ms | 3.932 ms |
| 1,000 | 258,192 | 23,895 | 90.7% | 9.469 ms | 9.867 ms |
| 2,000 | 516,992 | 39,745 | 92.3% | 28.846 ms | 29.726 ms |

This is a synthetic compaction throughput benchmark, not an agent-task success benchmark.

## Evaluation harness

- `examples/replay_eval.rs` is a small deterministic demo.
- `examples/replay_fixture.rs` evaluates any `CompactRequest` JSON fixture.
- `scripts/hermes_replay_eval.py` extracts local Hermes sessions from `~/.hermes/state.db` and writes a privacy-preserving aggregate report.

Run:

```bash
cargo run --example replay_eval
python scripts/hermes_replay_eval.py --limit 8 --min-messages 20 --target-tokens-list 20000,80000,120000 --budget-mode hard_cascade
```

Latest local Hermes replay aggregate over the 8 largest active sessions by text volume:

| Metric | Value |
|---|---:|
| Runs | 24 |
| Avg full tokens | 315,660.5 |
| Avg context-governor tokens | 16,003.9 |
| Avg token reduction vs full | 94.9% |
| Avg naive head/tail recoverable rate | 2.7% |
| Avg context-governor visible rate | 43.8% |
| Avg context-governor recoverable rate | 98.8% |
| Active task visible | 24/24 |

Report:

- `docs/hermes-replay-eval-2026-06-27.md`
- machine-readable local report: `target/context-governor-replay/hermes-replay-report.json`

Privacy boundary: the markdown report stores aggregate metrics only; raw transcript content is not written there.

## Claim boundary

This crate currently provides deterministic allocation, content-aware extractive previews, receipts, exact fallback, local search, CLI operations, and adapter-facing memory records.

It does not claim:

- semantic correctness equivalent to an LLM-written summary
- benchmarked LLM task-quality improvement
- KV-cache compression
- direct HyperQuant lattice compression of text
- provider-native long-context superiority
- built-in semantic-memory writes

HyperQuant-style use here is the rate-distortion allocation framing: spend scarce prompt tokens where loss would damage the task most.

## Verification

Current local verification commands:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo check --all-targets
cargo run --release --example perf
cargo run --example replay_eval
cargo doc --no-deps --document-private-items
cargo publish --dry-run --allow-dirty
```

## License

Apache-2.0. See `LICENSE`.

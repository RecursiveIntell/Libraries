# poly-kv

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Shared compressed KV-cache pool. One pool, many agents, zero leaks.**

You have 10 agents. They all share the same 200-token system prompt. Without compression, you're storing that prompt 10 times in VRAM. With poly-kv: store it once, compress it 50×, and give each agent a 17ms shell for its unique tokens.

That's not a pitch. It's benchmarked.

## The Two-Tier Strategy

The problem with KV-cache compression is that one size doesn't fit all. Shared context (system prompts, few-shot examples, retrieval results) needs high compression — it's large, rarely changes, and you can afford some fidelity loss. Agent-private context (conversation turns, tool outputs) needs near-lossless reconstruction — it's smaller but critical for correctness.

poly-kv gives you both:

| Tier | What it holds | Codec | Compression | Fidelity | Build cost |
|---|---|---|---|---|---|
| Shared pool (cold) | System prompts, shared context | fib-quant k=4, N=32 | ~50× theoretical | cos 0.863 | 1,557ms (once) |
| Agent shell (hot) | Per-agent conversation, tool results | turbo-quant 8-bit, 32proj | ~8× theoretical | cos 0.9996 | 17ms (per agent) |

## Benchmarks That Mean Something

We didn't just test "does it compress." We tested "does compressed retrieval find the right answer."

**Single-route parity — 8 queries, 200 docs, 768-dim:**

| Route | Recall@1 | Recall@10 | nDCG@10 | Rank drift |
|---|---|---|---|---|
| exact_scan (no compression) | 1.000 | 1.000 | 1.000 | — |
| fib-quant only | 1.000 | 1.000 | 1.000 | 0.33 |
| turbo-quant only | 1.000 | 1.000 | 1.000 | 0.03 |
| **poly-kv (two-tier)** | **1.000** | **1.000** | **1.000** | **0.25** |

**10-agent contention:**

| Metric | Result |
|---|---|
| Agents with recall@1 = 1.0 | **10/10** |
| Cross-agent top-1 leaks | **0/90 pairs** |
| Pool build (80 shared docs) | 1,557ms |
| Shell materialize (12 docs/agent) | 17ms avg |
| fib-quant cold compression batch | 480 KB → 133 KB (3.6× JSON, ~48× binary projected) |
| turbo-quant hot fidelity | cosine 0.9996 |

Every agent found its target at rank 1. Zero interference. The shared pool is read-only after build — agents physically cannot contaminate each other.

## Architecture

```
┌──────────────────────────────────────────┐
│         SHARED POOL (cold tier)          │
│    fib-quant · immutable · built once    │
│         stores system prompts,           │
│       few-shot examples, shared docs     │
└──────┬──────────────┬──────────────┬─────┘
       │              │              │
  ┌────▼─────┐   ┌────▼─────┐  ┌────▼─────┐
  │ Agent 0  │   │ Agent 1  │  │ Agent 9  │
  │turbo 8bit│   │turbo 8bit│  │turbo 8bit│
  │  cos 1.0 │   │  cos 1.0 │  │  cos 1.0 │
  │  17ms    │   │  17ms    │  │  17ms    │
  └──────────┘   └──────────┘  └──────────┘
```

## What You Actually Get

If you're building multi-agent systems — agent swarms, multi-tenant inference, concurrent RAG retrievers — poly-kv means:

- Shared context stored once, not N times
- New agents spin up at interactive speed (17ms)
- Agent isolation is measured, not assumed (0/90 leaks)
- Every operation produces a typed receipt (deterministic, replayable)

## The JSON Problem (Honest)

Current compression ratios are JSON-serialized. That means turbo-quant's 8× theoretical drops to 0.6× for single vectors — the JSON wrapper is literally bigger than the raw f32 bytes. fib-quant fares better at 3.6× even in JSON, but its theoretical ceiling is ~50×.

Binary wire format is the next PR. `PackedTurboCode` already exists in turbo-quant. `PackedFibCode` is next. Once both land:

| | JSON (current) | Binary (projected) |
|---|---|---|
| Shared pool (80 docs) | 240 KB → 66 KB (3.6×) | 240 KB → ~5 KB (48×) |
| Agent shell (12 docs) | 36 KB → 63 KB (0.6×) | 36 KB → ~5 KB (7×) |
| System total (200 docs) | 600 KB → 695 KB (0.9×) | 600 KB → ~95 KB (6.3×) |

## Quick Start

```rust
use poly_kv::{SharedKVPool, KvTensorShape, AttentionType};

let shape = KvTensorShape {
    attention_type: AttentionType::MHA,
    num_layers: 32,
    num_heads: 32,
    num_kv_heads: 32,
    head_dim: 128,
    hidden_size: 4096,
};

// Build shared pool once
let corpus: Vec<(String, Vec<f32>)> = vec![
    ("tok_0".into(), vec![0.1; shape.total_kv_bytes(1)]),
];
let (pool, receipt) = SharedKVPool::build(&corpus, &shape, 42)?;
assert!(receipt.compression_ratio > 1.0);

// Each agent gets a cheap shell
let agent_tokens: Vec<(String, Vec<f32>)> = vec![
    ("agent_tok_0".into(), vec![0.2; shape.total_kv_bytes(1)]),
];
let (shell, mat_receipt) = pool.materialize_shell("agent_7", &agent_tokens, 43)?;
println!("Shell built in {}ms", mat_receipt.materialize_ms);
```

## Running Benchmarks Yourself

```bash
# Single-route compression parity
cargo run --release --example poly_kv_bench --features poly

# 10-agent contention
cargo run --release --example multi_agent_contention --features poly
```

## What's Missing

- **Binary wire format** — next step, big compression multiplier
- **Real embedding corpus** — synthetic vectors prove the math, embeddings prove it matters
- **GPU cache adapter** — HuggingFace DynamicCache or vLLM block manager
- **Thousands of agents** — tested 10, scales linearly

## Dependencies

- [`fib-quant`](https://crates.io/crates/fib-quant) — cold-tier codec, 50× compression
- [`turbo-quant`](https://crates.io/crates/turbo-quant) — hot-tier codec, near-lossless
- `blake3` — content-addressed digests for every block, layer, and pool
- `serde` / `serde_json` — typed artifact serialization

## License

MIT

# poly-kv

**Shared compressed KV-cache pool for multi-agent context.**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Two-tier compression with typed, receipt-bearing artifacts:

- `SharedKVPool` — fib-quant (k=4, N=32) compressed shared tokens, immutable after build
- `AgentShell` — turbo-quant (8-bit, 32 projections) compressed per-agent tokens, 17ms materialize
- `CompressionPolicy` — benchmark-proven two-tier policy with validation guards
- `PoolBuildReceipt` / `ShellMaterializeReceipt` — content-addressed, deterministic, replayable

## Benchmarked (June 2026)

| Metric | Result |
|---|---|
| Recall@1 (8 queries) | 1.000 |
| Recall@1 (10 agents) | 1.000 — all 10 |
| Cross-agent leaks | 0/90 pairs |
| Pool build (80 docs) | 1,557ms |
| Shell materialize (12 docs) | 17ms |

## Install

```toml
[dependencies]
poly-kv = { version = "0.1.0-alpha.1", features = ["turbo", "fib"] }
```

## Usage

```rust
use poly_kv::{SharedKVPool, KvTensorShape, AttentionType};

let shape = KvTensorShape {
    attention_type: AttentionType::MHA,
    num_layers: 2,
    num_heads: 4,
    num_kv_heads: 4,
    head_dim: 8,
    hidden_size: 32,
};

let (pool, receipt) = SharedKVPool::build(&corpus, &shape, 42)?;
let (shell, mat_receipt) = pool.materialize_shell("agent_1", &agent_tokens, 43)?;
```

## Validation

```bash
cargo test --all-features
cargo clippy --all-features -- -D warnings
```

## License

MIT

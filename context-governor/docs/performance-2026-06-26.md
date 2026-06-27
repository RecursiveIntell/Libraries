# context-governor performance receipt — 2026-06-26

Machine/context:
- Host: Linux Nobara/Fedora-class workstation from Hermes runtime
- Build: Rust release profile
- Command:
  `cargo run --release --manifest-path context-governor/Cargo.toml --example perf`
- Harness: deterministic synthetic agent transcripts, 25 iterations per case, sorted p50/p95 wall-clock timing around `compact_context`.

Raw output:

```csv
messages,original_tokens,compacted_tokens,savings_tokens,avg_ms,p50_ms,p95_ms,throughput_msgs_per_s,fallback_refs,quarantined
100,25362,9720,15642,0.761,0.746,0.914,131408.0,60,10
500,128842,16020,112822,5.105,5.094,5.236,97943.1,360,50
1000,258192,23895,234297,12.641,12.587,12.810,79109.9,735,100
2000,516992,39745,477247,36.712,36.584,37.519,54477.9,1485,200
```

Derived ratios:

| Messages | Original approx tokens | Compacted approx tokens | Token reduction | Avg latency | P95 latency | Throughput |
|---:|---:|---:|---:|---:|---:|---:|
| 100 | 25,362 | 9,720 | 61.7% | 0.761 ms | 0.914 ms | 131k msg/s |
| 500 | 128,842 | 16,020 | 87.6% | 5.105 ms | 5.236 ms | 98k msg/s |
| 1,000 | 258,192 | 23,895 | 90.7% | 12.641 ms | 12.810 ms | 79k msg/s |
| 2,000 | 516,992 | 39,745 | 92.3% | 36.712 ms | 37.519 ms | 54k msg/s |

Interpretation:
- Runtime is not the blocker. Even 2,000 synthetic messages compacted in p95 37.5 ms in release mode.
- Token reduction improves with longer transcripts because low-value tool output dominates.
- Exact fallback remains available: 1,485 fallback refs in the 2,000-message case.
- Speculative recall quarantine works deterministically: 10% of generated speculative messages were quarantined in all cases.
- The compacted output can exceed the requested `target_tokens` when many messages are classified as high-risk exact-preserve items. That is intentional for v0.1.0: the crate refuses silent context destruction over hard target compliance. Future work can add bounded critical-item aggregation with stronger loss reports.

Claim boundary:
- This is a synthetic throughput/context-reduction benchmark, not an agent-task success benchmark.
- It does not prove downstream model behavior improvement.
- It does prove the core compaction path is fast enough to sit inline before model calls.

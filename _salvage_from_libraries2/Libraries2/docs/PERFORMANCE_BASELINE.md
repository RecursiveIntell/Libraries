# PERFORMANCE_BASELINE.md

Captured on 2026-03-14 from the canonical V3 lane using `cargo run -p kernel-conformance --example canonical_perf_snapshot`.

These numbers are regression alarms, not throughput claims. Regenerate them after meaningful changes to export, bridge, import, compile, or advisory behavior.

| Fixture | Effect records | Regions | Delta regions | Export ms | Transform ms | Import ms | Compile ms | Advisory ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| small | 2 | 2 | 1 | 3 | 3 | 13 | 0 | 5 |
| large | 11 | 2 | 1 | 5 | 7 | 17 | 3 | 10 |

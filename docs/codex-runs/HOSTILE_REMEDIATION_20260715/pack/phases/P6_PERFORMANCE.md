# Phase 6 — Performance

Issue: `PERF-001`.

Measure identical before/after workloads: ID allocation/throughput; queue depths 1/1k/100k; clone
counts; SQLite contention; exact/approx search latency and recall; sidecar build/recovery; digest cost.

Every optimization is isolated/reversible and runs the full correctness gate. No speedup via weaker
validation or unsupported claim.

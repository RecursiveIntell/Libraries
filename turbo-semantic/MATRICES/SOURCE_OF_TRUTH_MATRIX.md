# Dependency / Source-of-Truth Matrix

## Use this, never invent this locally

| Need | Use | Never implement |
|---|---|---|
| TurboQuant encode/decode/scoring | `turbo-quant` crate | semantic-memory local TurboQuant copy |
| Existing memory/search config | `semantic-memory::MemoryConfig` and existing config modules | parallel config file hidden from users |
| Existing SQ8 behavior | `semantic-memory/src/quantize.rs` or refactored equivalent | replacement that breaks current tests |
| Existing HNSW behavior | existing `hnsw.rs` / `hnsw_ops.rs` | separate HNSW for Turbo in this pass |
| Episode/projection law | existing semantic-memory tests and v9/v11 docs | vector codec as truth-bearing evidence |
| Schema derivation | `serde`, `schemars` where already used | hand-written drift-prone schema strings |
| IDs/digests/traces | canonical stack primitives where available | ad hoc ID semantics if canonical type exists |

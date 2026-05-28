# Failure Modes

Codex is likely to fail by:

1. Building a generic risk scorer instead of a proposed-action evaluator.
2. Treating scores as truth.
3. Using f64 because it is convenient.
4. Returning bool from decision APIs.
5. Letting evidence confidence reduce hazard.
6. Letting hard vetoes be overridden by thresholds.
7. Hand-maintaining JSON schemas separately from Rust types.
8. Hashing raw TOML instead of canonical policy JSON.
9. Updating golden fixtures to make tests pass.
10. Adding memory/retrieval/tool integration too early.
11. Sneaking FEUT/EEG terminology into production crate docs or policies.
12. Using network/LLM/model calls for scoring.
13. Creating duplicate ID/provenance/receipt semantics.
14. Making receipts non-replayable.
15. Omitting valid-time vs recorded-time basis.

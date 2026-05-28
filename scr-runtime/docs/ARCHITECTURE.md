# Architecture

SCR-P0A is a local reference workspace with four crates:

| Crate | Role |
|---|---|
| `scr-kernel` | Canonical Rust types for inputs, scores, pressures, actions, receipts, and local errors. |
| `scr-reference` | Deterministic policy parser, canonical policy hashing, hard-rule evaluator, pressure derivation, and action resolver. |
| `scr-audit-adapter` | Fixture-to-input adapter for local audit cases. |
| `scr-cli` | Local commands for policy canonicalization, schema generation, and fixture replay. |

The evaluator accepts a proposed action and an explicit canonical policy. It
emits a replayable receipt or an explicit error. It does not fetch evidence,
mutate artifacts, query memory, call tools, or perform network operations.

External identifiers are represented as opaque adapter refs. SCR records the
basis supplied to it; it does not become the owner of that basis.

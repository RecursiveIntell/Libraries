# AGENTS.md — PolyKV Governed Compression Workspace

## Mission

This repository exists to implement a narrow, auditable Rust workspace for:

- `quant-codec-core`: shared codec/profile/shape/eval traits and typed IDs.
- `poly-kv`: shared compressed KV-cache pool primitive with exact fallback, q8 key path, pluggable value-codec boundary, receipts, deterministic tests, and honest docs.

This is a research-to-Rust implementation pass. Correctness, reproducibility, and claim boundaries outrank apparent progress.

## Source-of-truth ownership

| Concept | Canonical owner | Forbidden duplicate |
|---|---|---|
| codec IDs/profile digests/shape types | `quant-codec-core` | local ad hoc strings in `poly-kv` |
| shared KV-pool manifests/readers/receipts | `poly-kv` | hidden runtime state or undocumented cache maps |
| TurboQuant math | existing `turbo-quant` crate | reimplementation inside `poly-kv` |
| FibQuant math | existing `fibquant` / future `fib-quant` crate | reimplementation inside `poly-kv` |
| adaptive routing/governance | future `quant-governor` | policy engine inside codec crates |
| runtime permits/rollback/quarantine | future `scr-runtime-compression` | runtime authority inside codec crates |
| semantic-memory/Gloss integration | future adapters only | direct writes from `poly-kv` into app truth stores |

## Hard rules

1. Inspect current files before editing.
2. Do not assume `turbo-quant` or `fibquant` APIs. Inspect them or build feature-gated stubs.
3. `poly-kv` owns pool semantics only. It must not own TurboQuant or FibQuant math.
4. No silent lossy compression. Every lossy operation must be represented by explicit policy/eval/receipt types.
5. Exact fallback must exist for every compressed representation in scope.
6. No hidden fallback. If a fallback occurs, emit or return a `FallbackReceiptV1`.
7. No silent shape coercion. Shape/layout mismatch must return a typed error.
8. No unsafe code unless explicitly justified, isolated, feature-gated, and differentially tested. Default target is safe Rust.
9. No GPL contamination. Do not copy code from GPL repositories. Do not vendor original PolyKV reference code without a license audit.
10. No public claims of production readiness, benchmark superiority, universal compatibility, or original authorship.
11. **Integration is now authorized** under strict ownership rules. PolyKV owns pool lifecycle/manifests/readers/persistence only. Runtime adapters own extraction/injection only. Semantic-memory owns durable references only. Integration must preserve exact fallback, immutable shared pool blocks after build, per-reader state isolation, and receipt-bound claim boundaries. Never silently widen scope or duplicate another crate's semantics.
12. No publish to crates.io unless release gates pass and the operator explicitly approves.

## Required implementation properties

- deterministic profile digests;
- content-addressable or digest-linked manifests where practical;
- immutable shared pool blocks after build;
- per-reader state isolated from the shared compressed pool;
- no per-reader duplication of compressed pool bytes;
- exact fallback path for raw KV blocks;
- real byte accounting, not theoretical-only ratio;
- replay determinism for synthetic fixtures;
- q8 key codec reference implementation;
- pluggable value codec trait with optional adapters only after API inspection.

## Build/test commands

Use the commands that apply to the current workspace:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

If `cargo semver-checks` is installed:

```bash
cargo semver-checks check-release
```

## Done means

A run is not complete until it has produced:

- changed-file list;
- commands run;
- test/build/lint results;
- skipped checks with reasons;
- phase reports;
- invariant validation;
- receipt/schema validation;
- unresolved blocker list;
- rollback/quarantine notes;
- final hostile-auditor handoff.

If these artifacts are missing, report incomplete.

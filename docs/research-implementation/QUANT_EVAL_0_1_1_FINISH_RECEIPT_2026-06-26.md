# quant-eval v0.1.1 Finish Receipt — 2026-06-26

Repo: /home/sikmindz/Coding/Libraries
Branch: feat/full-integration

## Published crates

- hyperquant v0.1.0: https://crates.io/crates/hyperquant/0.1.0
- quant-eval v0.1.1: https://crates.io/crates/quant-eval/0.1.1

## README rule update

The Rust crate publishing skill was updated so README quality now requires useful graphics anywhere/everywhere they clarify the crate: architecture, data flow, API flow, evidence/receipt flow, benchmark pipeline, integration path, and positioning.

The preference was also saved to memory/semantic memory.

## quant-eval changes

Files changed/added:

- quant-eval/Cargo.toml
- quant-eval/README.md
- quant-eval/docs/quant-eval-pipeline.svg
- quant-eval/src/lib.rs
- quant-eval/src/hyperquant_eval.rs
- quant-eval/src/rag.rs
- quant-eval/tests/hyperquant_eval.rs
- quant-eval/tests/rag_fixture.rs
- Cargo.lock

What shipped:

- Bumped quant-eval from 0.1.0 to 0.1.1.
- Added release-quality README with evidence pipeline graphic.
- Added `docs/quant-eval-pipeline.svg`.
- Added HyperQuant evaluation API:
  - `HyperQuantEvalConfig`
  - `HyperQuantProfileEval`
  - `HyperQuantEvalResult`
  - `run_hyperquant_eval`
- Added RAG fixture metrics:
  - recall@K
  - NDCG@K
  - exact-rerank recovery
- Published quant-eval v0.1.1.

## Verification receipts

Passed:

```text
cargo fmt -p quant-eval
cargo test -p quant-eval -- --nocapture
cargo test -p hyperquant -- --nocapture
cargo check -p quant-eval --all-targets
cargo clippy -p quant-eval --all-targets -- -D warnings
cargo publish -p quant-eval --dry-run --allow-dirty
cargo publish -p quant-eval --allow-dirty
```

Test counts:

```text
quant-eval: 35 tests passed
hyperquant: 18 tests passed
```

Package contents verified:

```text
cargo package -p quant-eval --allow-dirty --list
```

Confirmed package contains:

- README.md
- LICENSE-MIT
- docs/quant-eval-pipeline.svg
- src/hyperquant_eval.rs
- src/rag.rs
- tests/hyperquant_eval.rs
- tests/rag_fixture.rs

Registry verification:

```text
cargo search quant-eval --limit 5
quant-eval = "0.1.1"

cargo info quant-eval --registry crates-io
version: 0.1.1
crates.io: https://crates.io/crates/quant-eval/0.1.1
```

Docs verification:

```text
https://docs.rs/hyperquant/0.1.0/hyperquant/ -> HTTP 200
https://docs.rs/quant-eval/0.1.1/quant_eval/ -> HTTP 200
```

## Claim boundary preserved

No claims were made for:

- HyperQuant paper parity;
- model-quality preservation;
- production readiness;
- CUDA;
- HuggingFace integration;
- codec superiority.

## Dirty-tree note

The repo still has many pre-existing unrelated modifications and deletions. The finish work only stages/commits the scoped files listed above, not the unrelated dirty tree.

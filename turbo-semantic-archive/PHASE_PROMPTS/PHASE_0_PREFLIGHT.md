# Phase 0 — Preflight and Source-Basis Freeze

## Goal

Freeze source basis and prevent path/layout mistakes before implementation.

## Tasks

1. Locate:
   - semantic-memory workspace root;
   - semantic-memory crate root;
   - turbo-quant crate root.

2. Verify Cargo layout:
   - run `cargo metadata` where feasible;
   - identify whether `turbo-quant` is sibling of `semantic-memory`;
   - if not sibling, report exact path issue.

3. Decide integration path:
   - preferred: `turbo-quant = { path = "../turbo-quant", optional = true }`;
   - forbidden: absolute path dependency;
   - forbidden: copy/paste TurboQuant code into semantic-memory.

4. Create `docs/codex-runs/turbo_quant_integration/SOURCE_BASIS.md` with:
   - repo paths;
   - current commit/hash if available;
   - Cargo workspace state;
   - list of current tests relevant to quantization/search;
   - known limitations from current source.

5. Run initial checks if possible:
   - `cargo test` in turbo-quant;
   - `cargo test -p semantic-memory --features hnsw`;
   - if not possible, capture why.

## Stop condition

Stop after source-basis report. Do not modify implementation code until the user gives the next manual injection.

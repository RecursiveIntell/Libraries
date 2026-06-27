# FibQuant Paper-Core Codex Bundle — 2026-05-16

Purpose: drive one Codex pass that creates a paper-faithful `fib-quant` Rust crate without mutating `semantic-memory`, `turbo-quant`, Gloss, or product behavior.

This bundle is designed for `~/Coding/Libraries` and the package `Libraries-libraries-next-codex-context-20260512.zip`.

Current front-door verification for the repo is `make gate` from the repository root. Run it before claiming the full release surface is green.

Supersession note (2026-03-17): the earlier no-v25 terminal position is superseded by the current v25 repo truth surface and `scripts/check_v25_repo_truth.sh`.

## What this pass must produce

- A new workspace member `fib-quant`, not in `default-members`.
- Paper-faithful FibQuant math core:
  - normalize -> Haar rotate -> split into k-blocks;
  - spherical-Beta source sampler;
  - Beta-quantile radii;
  - k=2 Fibonacci spiral, k=3 Fibonacci sphere, k>=4 Roberts-Kronecker directions;
  - multi-restart Lloyd-Max refinement with deterministic empty-cell repair;
  - fixed-rate index payload and fp16 norm header;
  - decode by lookup + inverse rotation;
  - deterministic profile/codebook/encoded digests;
  - receipts and math conformance docs.
- Tests proving math and failure behavior.

## What this pass must not do

- No production integration into `semantic-memory`.
- No changes to `semantic-memory/src/**` or `turbo-quant/src/**`.
- No FEUT/SCR variant.
- No default-on compression.
- No “zero loss” or performance win claims.
- No deletion/replacement of raw embeddings or canonical memory.

## Recommended use

1. From repo root, install the optional context/hook overlay:

   ```bash
   bash scripts/install_fibquant_codex_bundle.sh /path/to/this/bundle
   ```

   Or manually copy `overlays/.agents/skills/fibquant-paper-core` into `<repo>/.agents/skills/` and review `.codex/hooks.json` before use.

2. Start Codex in `~/Coding/Libraries`.
3. Open `/hooks` and approve only the two FibQuant hook scripts if installed.
4. Use Plan mode first for a source-basis plan.
5. Paste `OPERATOR_PASTE_FIRST.md`.
6. After completion, run:

   ```bash
   python3 scripts/fibquant_final_assert.py --repo .
   cargo fmt --all --check
   cargo test -p fib-quant
   ```

## Why this bundle uses Codex features

- `AGENTS.md` / skill: durable repo-local instructions.
- Hooks: deterministic guardrails at prompt/stop boundaries.
- Phase prompts: bounded work slices.
- Final assertion script: executable closeout check.
- Backstop prompts: human override when hooks are unavailable.
- Matrices/fixtures: reduce ambiguity and prevent “creative” math substitution.

# Paste this first into Codex

Use `$fibquant-paper-core` if available.

You are in `~/Coding/Libraries`.

Run this as a **paper-faithful FibQuant math-core pass**, not as a semantic-memory integration pass.

Primary target:
- Implement a new top-level Rust workspace crate named `fib-quant`.
- Make it as mathematically faithful as possible to `FibQuant: Universal Vector Quantization for Random-Access KV-Cache Compression`, arXiv:2605.11478v1.
- This pass produces math core, codebook generation, encode/decode, tests, receipts, and documentation.

Hard source hierarchy:
1. Current repository files.
2. `docs/compression/FIBQUANT_SOURCE_BASIS.md` that you create in Phase 0.
3. The FibQuant paper.
4. Existing `turbo-quant` and `semantic-memory` codec surfaces as compatibility context only.

Non-negotiable constraints:
- Do not modify `semantic-memory/src/**`.
- Do not modify `turbo-quant/src/**`.
- Do not modify Gloss or product repos.
- Do not make FibQuant default anywhere.
- Do not claim benchmark wins unless locally reproduced and reported.
- Do not implement a vague FibQuant-inspired codec.
- Do not skip Lloyd-Max refinement.
- Do not support only k=2.
- Do not silently pad when `d % k != 0`.
- Do not add FEUT/SCR variants in this pass.
- Do not write compatibility shims that silently widen semantics.

Required phase order:
0. Inspect source and write `docs/compression/FIBQUANT_SOURCE_BASIS.md`. No code until this exists.
1. Create `fib-quant` crate and profile/digest/error law.
2. Implement spherical-Beta source and radius math.
3. Implement direction generators.
4. Implement radial-angular codebook initialization.
5. Implement multi-restart Lloyd-Max refinement.
6. Implement fixed-rate encoder/decoder.
7. Implement tests and metrics.
8. Implement receipts and closeout docs.
9. Run validation and final hostile self-audit.

Validation commands to run when possible:
- `cargo fmt --all --check`
- `cargo test -p fib-quant`
- `cargo test -p turbo-quant`
- `cargo test -p semantic-memory --features hnsw`

If any command cannot run, report exact stderr and classify it as blocker or environmental.

Final response must include:
- changed files;
- commands run;
- tests passed/failed/skipped;
- mathematical conformance status;
- unresolved deviations from the paper;
- proof that FibQuant remains default-off;
- proof that `semantic-memory/src/**` and `turbo-quant/src/**` were not modified;
- exact remaining blockers.

Start by reading:
- `01_CODEX_MASTER_PROMPT.md`
- `02_PHASE_PLAN.md`
- `03_TARGET_API_SPEC.md`
- `04_MATH_CONFORMANCE.md`
- `05_ACCEPTANCE_GATES.md`
- `fixtures/FIBQUANT_TEST_MATRIX.json`

Then execute Phase 0 only and report the source-basis summary before coding.

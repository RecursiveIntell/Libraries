# Release Bar and Acceptance

The next Codex pass is done only if all of the following are true.

## Must-pass conditions

### 1. Ownership truth
- Every new horizon crate has a local `README.md` and `AGENTS.md`.
- Repo-level docs no longer overclaim maturity.

### 2. Schema truth
- New artifact families added in this pass are registered in `contract-schema-gen`.
- Matching schema manifest files include the new schema file names.
- Schema check passes.

### 3. Test truth
- Each touched horizon crate has passing crate-local tests.
- `kernel-conformance` has passing reference-interpreter tests covering the richer slices.

### 4. Behavioral truth
- v16 can emit replay, divergence, and suspension artifacts in bounded form.
- v17 fit readiness is gated by richer artifacts than a lone boolean.
- v18 selection is more than source-order subtraction.
- v19 proves archive/compaction consequences.
- v20 emits companion generated bundles and can be vetoed/challenged without fake admission.

### 5. Honesty truth
- Nothing in the repo claims these crates are production-complete runtimes.
- Generated outputs remain advisory unless explicitly admitted elsewhere.

## Failure conditions

The pass is not done if any of the following happen:

- new types exist but are not schema-owned,
- schemas exist but no fixtures/tests exercise them,
- advisory outputs are quietly treated as admitted,
- “finish” is claimed while `SelfHostingBuildReceiptV1` is still absent,
- repo docs say “implemented” where the code is still a bounded pilot.

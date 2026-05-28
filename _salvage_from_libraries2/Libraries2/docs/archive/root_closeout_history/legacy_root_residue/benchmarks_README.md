# Forge-bench benchmark package

This package demonstrates the stack's advantage on replayable, evidence-bound reasoning
compared to a naive retrieval baseline.

## What is being tested

The benchmark targets dimensions that ordinary RAG cannot satisfy with typed artifact chains:

- **Temporal correctness** — can the release decision be replayed with a deterministic timeline?
- **Replayability** — can the chain be replayed against explicit artifact IDs?
- **Widening disclosure** — are required monitors and obligations explicitly represented?
- **Contradiction handling** — can the chain detect contradictory payloads and fail cleanly?
- **Verification yield** — is the verification evidence sufficient without prose-only claims?

## Assessment modes

### fixture-asserted (current)

Verdicts are authored in the casebook as typed signal checks against fixture data.
The runner reads these authored verdicts and tallies them. No live model call is made.

This mode proves that the artifact chain satisfies the typed signal conditions. It does
not prove that a live model would answer questions about the chain differently from a
naive baseline.

**Current score sheet** (`docs/benchmarks/score_sheet.json`): fixture-asserted
- Stack score: 5 / 5
- Baseline score: 1 / 5
- Advantage: 4

**Limitation:** verdicts reflect typed artifact signal checks against fixture data, not
computed live model comparison. A reviewer examining the casebook will find hardcoded
verdict fields. This is honest and documented.

### execution-verified (target)

At least one casebook case is run by actually calling a model with the artifact chain as
context, checking the response against the typed signal conditions, and comparing to a
baseline run that receives only a naive text summary.

**Command:**
```bash
python3 docs/benchmarks/run_forge_bench.py --mode execution \
  --casebook contracts/fixtures/bench/forge_bench_casebook.json \
  --output docs/benchmarks/score_sheet_executed.json
```

This mode is implemented in FIX-008 of the finish pass.

## Running the fixture-asserted scorer

```bash
python3 docs/benchmarks/run_forge_bench.py \
  --casebook contracts/fixtures/bench/forge_bench_casebook.json \
  --output docs/benchmarks/score_sheet.json
```

## Inputs

| File | Role |
|---|---|
| `contracts/fixtures/demo/effect_authority_assurance_release.bundle.json` | Demo bundle (v21→v22→v23 chain) |
| `contracts/fixtures/v21/effect_happy_path.bundle.json` | v21 effect execution fixtures |
| `contracts/fixtures/v22/delegated_effect_happy_path.bundle.json` | v22 delegation fixtures |
| `contracts/fixtures/v23/release_happy_path.bundle.json` | v23 release readiness fixtures |

## Reproducibility

The fixture-asserted scorer is fully deterministic. Given the same casebook, it always
produces the same score sheet. The score sheet SHA is stable as long as the casebook does
not change.

The execution-verified scorer is non-deterministic across model versions but should be
stable across runs with the same model and the same fixture inputs.

## What this does NOT prove

- That a production deployment of the stack outperforms RAG on arbitrary queries.
- That the five benchmark dimensions cover all possible adversarial cases.
- That the baseline (naive_rag_v1) is a representative production system.

These are typed artifact signal checks against a known fixture corpus. They prove the
artifact chain is coherent and typed. They are a necessary but not sufficient proof of
production superiority.

# Forge-bench proof package (BENCH-001)

This package ships the forge-bench fixtures, runner, and score sheet for the `effect-runtime -> authority-delegation -> assurance-runtime` chain. The default scoring mode is fixture-asserted, and `--mode execution` adds one execution-verified `temporal_correctness` case alongside the authored fixture cases.

## Inputs

- `contracts/fixtures/bench/forge_bench_casebook.json`

## Running the benchmark

```bash
python3 docs/benchmarks/run_forge_bench.py \
  --casebook contracts/fixtures/bench/forge_bench_casebook.json \
  --output docs/benchmarks/score_sheet.json
```

The expected reproducibility output is `docs/benchmarks/score_sheet.json`.

## Assessment modes

`fixture-asserted` means the casebook and score sheet verdicts are authored assertions against typed artifact signals in the fixtures. This is the default mode.

`execution-verified` means the runner performs at least one live model execution and evaluates the response against the typed signal contract.

`--mode execution` preserves the original fixture-asserted cases and adds one execution-verified `temporal_correctness` result produced from a live model call against the full bundle and a naive-summary baseline.

The live execution command path is:

```bash
python3 docs/benchmarks/run_forge_bench.py \
  --mode execution \
  --casebook contracts/fixtures/bench/forge_bench_casebook.json \
  --output docs/benchmarks/score_sheet.json
```

## Evidence contract

- temporal correctness: must beat baseline
- replayability: must beat baseline
- disclosure honesty and contradiction handling: typed evidence is preferred
- verification yield: structured output required for audit

This benchmark is consumer-only: no internal owner-crate authority bypass is needed to replay it.

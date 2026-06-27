# Context-Governor Benchmark Implementation Spec

Goal: implement the benchmark tooling that makes the context-governor keep/kill decision evidence-backed.

Scope chosen as "good stuff" from the larger plan:

1. Make the existing replay evaluator usable as a benchmark primitive.
   - Add configurable output directory.
   - Add optional per-run response/request artifact writing under target only.
   - Preserve the existing docs report default path for backwards compatibility.

2. Add an offline baseline evaluator.
   - Use the existing Rust replay fixture's `full` and `head_tail` baselines as the offline comparator.
   - Name this honestly as `offline_baseline`, not a fake claim that an LLM-backed Hermes compressor was run.
   - Output the same report envelope shape used by comparison tooling.

3. Add deterministic adversarial fixtures.
   - Latest-user reversal.
   - Critical error inside huge log.
   - Duplicate tool spam.
   - Prompt-injection-like tool output.
   - Durable decision vs speculation.
   - Personal/social noise.
   - File path and command receipt fixture.

4. Add adversarial evaluator.
   - Run fixtures through context-governor and offline baseline modes.
   - Score expected probes by visible and recoverable status.
   - Enforce invariants: latest user final, prompt-injection not elevated, social noise not archiveable.

5. Add comparison reporter.
   - Consume one or more report JSON files.
   - Emit markdown + JSON aggregate table.
   - Compare runs/failures/token reduction/visible/recoverable/active task rates/warnings.

6. Add semantic-memory label helper.
   - Convert archived fact JSON records into a CSV template for KEEP/MAYBE/JUNK/HARMFUL manual review.
   - This is intentionally a labeling aid, not automatic production-memory promotion.

7. Add tests for every new script and added replay-evaluator CLI behavior.

8. Run gates:
   - Python tests.
   - Rust cargo fmt/test/clippy if feasible.
   - At least one real replay smoke.
   - Adversarial generation/evaluation/comparison smoke.

Non-goals for this implementation pass:

- Do not enable production semantic-memory archival by default.
- Do not claim LLM downstream answer quality.
- Do not write benchmark facts into production semantic-memory namespaces.
- Do not mutate Hermes upstream default engine.
- Do not fake a real Hermes built-in compressor run; if the LLM-backed compressor is not executed, label the comparator as offline `head_tail` baseline.

Acceptance criteria:

- `python -m pytest tests_py -q` passes.
- `python scripts/generate_adversarial_fixtures.py --out target/context-governor-bench/fixtures/adversarial` writes fixture JSON.
- `python scripts/evaluate_adversarial_fixtures.py --fixtures target/context-governor-bench/fixtures/adversarial --engine context_governor --budget-modes soft_warn,hard_cascade --target-tokens 8000,20000 --out target/context-governor-bench/reports/adversarial-context-governor.json` succeeds.
- `python scripts/evaluate_adversarial_fixtures.py --fixtures target/context-governor-bench/fixtures/adversarial --engine offline_baseline --target-tokens 8000,20000 --out target/context-governor-bench/reports/adversarial-offline-baseline.json` succeeds.
- `python scripts/compare_context_engines.py --report ... --out target/context-governor-bench/reports/context-engine-comparison` emits `.json` and `.md`.
- `python scripts/hermes_replay_eval.py --limit 1 --min-messages 2 --target-tokens-list 20000 --budget-mode soft_warn --output-dir target/context-governor-bench/reports/replay-smoke --write-responses target/context-governor-bench/receipts/replay-smoke` succeeds or reports a clean no-session failure if the DB is unavailable.

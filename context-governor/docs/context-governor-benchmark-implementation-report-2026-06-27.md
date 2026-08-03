# Context-Governor Benchmark Implementation Report — 2026-06-27

Status: implemented benchmark tooling and ran smoke/certification gates.

## What shipped

### Plan/spec

- `docs/plans/2026-06-27-context-governor-memory-benchmark-plan.md`
- `docs/plans/2026-06-27-context-governor-benchmark-implementation-spec.md`

### Replay evaluator upgrades

Modified:
- `scripts/hermes_replay_eval.py`

Added:
- `--output-dir`
- `--docs-path`
- `--write-responses`

Purpose: replay runs can now write isolated benchmark artifacts under `target/context-governor-bench/` without overwriting canonical docs reports.

### Adversarial fixture generation

Added:
- `scripts/generate_adversarial_fixtures.py`

Generated fixtures:
- latest user reversal
- critical error inside huge log
- duplicate tool spam
- prompt-injection-like tool output
- durable decision vs speculation
- personal/social noise
- file path and command receipts

Output:
- `target/context-governor-bench/fixtures/adversarial/`

### Adversarial evaluator

Added:
- `scripts/evaluate_adversarial_fixtures.py`

Modes:
- `context_governor`
- `offline_baseline` / `head_tail`

Notes:
- `context_governor` mode invokes the Rust binary's `compact` command.
- It mirrors the Hermes plugin's latest-user-last host contract before scoring.
- `offline_baseline` is intentionally not called "Hermes compressor"; it is a truthful head/tail offline comparator.

Outputs:
- `target/context-governor-bench/reports/adversarial-context-governor.json`
- `target/context-governor-bench/reports/adversarial-offline-baseline.json`
- response/request artifacts under `target/context-governor-bench/receipts/adversarial-context-governor/`

### Comparison reporter

Added:
- `scripts/compare_context_engines.py`

Outputs:
- `target/context-governor-bench/reports/context-engine-comparison.json`
- `target/context-governor-bench/reports/context-engine-comparison.md`

### Semantic-memory label helper

Added:
- `scripts/semantic_memory_label_template.py`

Output smoke:
- `target/context-governor-bench/reports/semantic-memory-labels.csv`

Purpose: produce a manual KEEP/MAYBE/JUNK/HARMFUL labeling CSV from exported semantic-memory facts. It does not promote facts or enable production archival.

### Hermes plugin test hardening

Modified:
- `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`

Added test:
- semantic-memory archive payloads honor `HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench`

## Verification receipts

### Python tests for context-governor tooling

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
python -m pytest tests_py -q
```

Result:

```text
8 passed in 0.03s
```

### Hermes plugin tests

Command:

```bash
cd /home/sikmindz/.hermes/hermes-agent
python -m pytest tests/plugins/test_context_governor_plugin.py tests/run_agent/test_plugin_context_engine_init.py -q
```

Result:

```text
14 passed in 1.12s
```

### Rust gates

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
cargo fmt --check
cargo test --all-targets --quiet
cargo clippy --all-targets -- -D warnings
```

Result:

```text
cargo fmt --check: pass
cargo test --all-targets --quiet: pass
cargo clippy --all-targets -- -D warnings: pass
```

### Replay smoke

Command:

```bash
python scripts/hermes_replay_eval.py \
  --limit 1 \
  --min-messages 2 \
  --target-tokens-list 20000 \
  --budget-mode soft_warn \
  --output-dir target/context-governor-bench/reports/replay-smoke \
  --docs-path target/context-governor-bench/reports/replay-smoke.md \
  --write-responses target/context-governor-bench/receipts/replay-smoke
```

Result:

```text
wrote target/context-governor-bench/reports/replay-smoke/hermes-replay-report.json
wrote target/context-governor-bench/reports/replay-smoke.md
replay_smoke_runs 1 failures 0
```

### Adversarial fixture generation

Command:

```bash
python scripts/generate_adversarial_fixtures.py --out target/context-governor-bench/fixtures/adversarial
```

Result:

```text
wrote 7 fixtures to target/context-governor-bench/fixtures/adversarial
```

### Offline baseline adversarial eval

Command:

```bash
python scripts/evaluate_adversarial_fixtures.py \
  --fixtures target/context-governor-bench/fixtures/adversarial \
  --engine offline_baseline \
  --target-tokens 8000,20000 \
  --budget-modes soft_warn \
  --out target/context-governor-bench/reports/adversarial-offline-baseline.json
```

Result:

```text
runs=14 failures=8
```

Aggregate:

```text
avg_full_tokens: 1964.9
avg_compacted_tokens: 27.4
avg_token_reduction: 61.9%
active_task_visible_rate: 100.0%
visible_probe_rate: 57.1%
recoverable_probe_rate: 57.1%
required_recoverable_rate: 57.1%
```

Interpretation: head/tail is very compact and preserves latest task, but drops too many required probes because it has no exact fallback lane.

### context_governor adversarial eval

Command:

```bash
python scripts/evaluate_adversarial_fixtures.py \
  --fixtures target/context-governor-bench/fixtures/adversarial \
  --engine context_governor \
  --target-tokens 8000,20000 \
  --budget-modes soft_warn,hard_cascade \
  --out target/context-governor-bench/reports/adversarial-context-governor.json \
  --write-responses target/context-governor-bench/receipts/adversarial-context-governor
```

Result:

```text
runs=28 failures=0
```

Aggregate:

```text
avg_full_tokens: 1997.1
avg_compacted_tokens: 250.6
active_task_visible_rate: 100.0%
visible_probe_rate: 92.9%
recoverable_probe_rate: 100.0%
required_recoverable_rate: 100.0%
warnings: 42
```

Important caveat: adversarial fixtures are small, so token reduction is not a meaningful metric here. This suite is for correctness/recoverability, not compression-ratio proof.

### Receipt store/search smoke

Command:

```bash
context-governor store --dir target/context-governor-bench/receipts/store-smoke < <one adversarial response>
context-governor search --dir target/context-governor-bench/receipts/store-smoke --query 'Latest task' --top-k 3
```

Result:
- search returned hits from compacted messages and receipt metadata
- query found `Latest task: preserve the exact Rust error path.`

### Semantic-memory label helper smoke

Command:

```bash
python scripts/semantic_memory_label_template.py \
  --facts target/context-governor-bench/reports/demo-facts.json \
  --out target/context-governor-bench/reports/semantic-memory-labels.csv
```

Result:

```text
wrote 1 label rows to target/context-governor-bench/reports/semantic-memory-labels.csv
```

CSV header:

```text
fact_id,namespace,source_receipt,item_id,content_blake3,label,reason,should_promote_to_projects,preview
```

## Current benchmark decision

Based only on the implemented smoke/adversarial gates:

- `context_governor` is worth continuing as default test engine.
- Semantic-memory archival should remain OFF by default until the isolated namespace trial is run and manually labeled.
- The current evidence supports `DEFAULT ENGINE, ARCHIVE OFF`, not `ARCHIVE ON`.

Why:

- context_governor adversarial recovery: 100% required recoverability, 0 failures.
- offline head/tail baseline: 57.1% required recoverability, 8/14 failures.
- latest user final invariant: 100% after mirroring the Hermes plugin host contract.
- prompt-injection-like tool output was not elevated into active user/system instruction.
- personal/social noise fixture did not create archive candidates in the evaluated path.

Still not certified:

- semantic-memory archival quality at scale
- duplicate memory rate under repeated live compactions
- downstream LLM answer-quality improvement
- token usage in real long live sessions versus built-in Hermes compressor

## Next gate before enabling semantic archival

Run an isolated semantic-memory trial only with:

```bash
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
```

Then export facts, generate labels with:

```bash
python scripts/semantic_memory_label_template.py --facts <facts.json> --out target/context-governor-bench/reports/semantic-memory-labels.csv
```

Do not promote to production memory unless:
- KEEP >= 80%
- JUNK <= 10%
- HARMFUL = 0
- duplicate rate <= 10%

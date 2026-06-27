# Context-Governor + Semantic Memory Benchmark Plan

> For Hermes: this is a benchmark/certification plan, not an implementation completion claim. Execute task-by-task. Do not enable semantic archival globally until the quality gates pass.

Goal: determine whether `context_governor` is worth using as Josh's default Hermes context engine, and whether semantic-memory archival should be enabled by default.

Architecture: compare the built-in Hermes `compressor` against the `context_governor` plugin across replayed real sessions, synthetic adversarial fixtures, forced live-compression smoke tests, and semantic-memory write/read quality audits. Treat token reduction, active-task preservation, exact fallback recovery, memory precision, and usage/cost as separate metrics. Do not collapse them into a single vanity score.

Tech stack:
- Hermes Agent at `/home/sikmindz/.hermes/hermes-agent`
- context-governor crate at `/home/sikmindz/Coding/Libraries/context-governor`
- active config at `/home/sikmindz/.hermes/config.yaml`
- semantic-memory MCP/HTTP local tools
- Rust `cargo test`, Python `pytest`, Hermes live trial, local SQLite session store, semantic-memory namespace isolation

---

## Evidence-Backed Current State — 2026-06-27

### Hermes state

Repo path:
`/home/sikmindz/.hermes/hermes-agent`

Current local branch:
`feat/context-governor-plugin`

Current plugin commit:
`a9300e34 feat(context-engine): add context-governor plugin`

Upstream PR:
`https://github.com/NousResearch/hermes-agent/pull/53722`

PR measured delta:
- 4 files changed
- 918 insertions
- 0 deletions

Files:
- `plugins/context_engine/context_governor/__init__.py`
- `plugins/context_engine/context_governor/live_trial.py`
- `plugins/context_engine/context_governor/plugin.yaml`
- `tests/plugins/test_context_governor_plugin.py`

Verification already passed:

```bash
cd /home/sikmindz/.hermes/hermes-agent
python -m py_compile plugins/context_engine/context_governor/__init__.py plugins/context_engine/context_governor/live_trial.py tests/plugins/test_context_governor_plugin.py
python -m pytest tests/plugins/test_context_governor_plugin.py tests/run_agent/test_plugin_context_engine_init.py -q
PYTHONPATH=$PWD python plugins/context_engine/context_governor/live_trial.py
hermes update --check
```

Observed results:
- `13 passed`
- live trial passed: engine `context_governor`, available `true`, latest user preserved, 5 input messages -> 4 output messages
- `hermes update --check`: already up to date

Current active config:

```text
context.engine = context_governor
model = openai-codex / gpt-5.5
compression.threshold = 0.85
compression.target_ratio = 0.2
```

Important current risk:
- `context_governor` is active for new sessions.
- The currently running session may not reflect startup-loaded config until restart/new session.
- Semantic archival is implemented/tested but should remain opt-in until this benchmark plan proves quality.

### context-governor crate state

Repo path:
`/home/sikmindz/Coding/Libraries/context-governor`

Commit in Libraries:
`ffba88f feat: add context-governor crate`

Measured delta:
- 38 files changed
- 5,951 insertions
- 0 deletions

Rough crate breakdown:
- Rust core: 2 files, 2,173 lines
- Rust tests: 12 files, 1,181 lines
- examples: 4 files, 663 lines
- Python scripts/tests: 3 files, 443 lines
- docs: 13 files, 1,347 lines

Installed binary:
`/home/sikmindz/.local/bin/context-governor`

Binary SHA256:
`0c0d590b9406c1ac50dfd901ca863324064dfd6af3d00446b9952765eb8e1df3`

Commands already passed in the final hardening pass:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
cargo fmt --check
cargo test --all-targets --quiet
cargo clippy --all-targets -- -D warnings
python -m py_compile scripts/hermes_replay_eval.py
python -m pytest tests_py/test_hermes_replay_eval.py -q
cargo publish --dry-run --allow-dirty
python scripts/hermes_replay_eval.py --limit 8 --min-messages 20 --target-tokens-list 20000,80000,120000 --budget-mode hard_cascade
```

Replay result already observed:
- 24 successful runs
- 0 failures
- average full tokens: 315,660.5
- average context-governor tokens: 16,003.9
- average token reduction: 94.9%
- average context-governor recoverable rate: 98.8%
- active task visible: 24/24

### Semantic-memory current boundary

Implemented/tested behavior:
- host-side semantic archival payloads include exact content and receipt metadata
- content_blake3 dedupe path exists
- successful host archival records real fact IDs in receipts
- stale Rust-side `no memory sink` warning is cleared after host-side semantic write succeeds

Not yet certified:
- precision of memory writes under realistic long sessions
- junk-memory rate
- duplicate-memory rate across repeated compactions
- whether archived facts improve downstream task success
- whether archival should run on every compaction, only on decision/evidence items, or never by default

---

## Decision Framework

This plan should answer five yes/no questions:

1. Should `context_governor` stay as Josh's default `context.engine`?
2. Should semantic-memory archival be enabled by default?
3. If enabled, should archival write to `projects`, `research`, or a dedicated benchmark namespace?
4. Does receipt search/expand improve task recovery enough to justify extra complexity?
5. Does the system reduce usage/context failures without producing junk memory?

Certification levels:

### REJECT

Use built-in `compressor` by default if any P0 gate fails:
- latest user task missing after compaction
- tool/path/evidence recovery below threshold
- semantic archival produces junk at high rate
- compaction failure breaks the turn instead of failing open
- usage/cost increases materially versus compressor

### TRIAL

Keep `context_governor` opt-in if:
- active task and exact fallback pass
- token reduction is strong
- semantic archival is too noisy or unproven

### DEFAULT ENGINE, ARCHIVE OFF

Use `context_governor` by default but leave semantic archival disabled if:
- compaction/recovery beats built-in compressor
- failure behavior is safe
- semantic archival is not yet clean enough

### DEFAULT ENGINE, ARCHIVE ON WITH FILTERS

Enable semantic archival only if:
- memory precision is high
- duplicate rate is low
- recall usefulness improves downstream tasks
- write volume is bounded
- archived facts have enough source/receipt metadata to audit later

---

## Metrics

### Context/compaction metrics

Collect per run:
- input message count
- original approx tokens
- compacted approx tokens
- token reduction percentage
- compacted message count
- latest user preserved: boolean
- latest user final: boolean
- system/developer constraints preserved: boolean
- exact fallback ref count
- archived item count
- receipt warnings
- compaction latency ms
- receipt store latency ms
- binary exit status
- fail-open triggered: boolean

### Recovery metrics

Use probe strings and expected artifacts:
- active task probe
- file path probe
- command output/error probe
- acceptance gate probe
- durable decision probe
- exact tool-output probe
- user correction/preference probe

Scores:
- visible in compacted prompt: boolean
- recoverable by receipt search: boolean
- expandable exact content matches original: boolean
- semantic-memory recall returns it: boolean
- top-k rank if recalled

### Semantic-memory quality metrics

For every semantic archival run:
- attempted archive records
- actual writes
- dedupe hits
- fact IDs written
- failed writes
- facts with source receipt ID
- facts with content_blake3
- facts with exact item ID
- facts that are actionable/durable
- facts that are junk/noise
- facts that are duplicates
- facts that are too broad or stale-prone

Manual labels:
- KEEP: durable, useful, source-backed
- MAYBE: useful but needs narrower phrasing
- JUNK: verbose, transient, duplicate, or non-actionable
- HARMFUL: misleading, stale, wrong namespace, or likely to pollute recall

Thresholds:
- KEEP >= 80% for archive-on trial
- JUNK <= 10%
- HARMFUL = 0 tolerated
- duplicate rate <= 10% after dedupe
- every fact has receipt/item/source metadata

### Task-outcome metrics

For a set of replayed or controlled tasks:
- answer correctness
- exact evidence citation availability
- number of tool calls needed after compaction
- whether agent asks user to repeat context
- whether agent hallucinates missing context
- whether agent can recover file paths/commands/decisions from receipt tools

### Cost/usage metrics

Collect:
- prompt tokens before/after compaction
- completion tokens
- total tokens
- number of LLM calls in turn
- number of compactions per session
- semantic-memory HTTP/MCP calls per compaction
- binary runtime
- error retries

Hard gate:
- context_governor must not increase API-token usage versus built-in compressor in comparable long sessions.
- if it saves prompt tokens but causes more follow-up tool/search calls, record net effect.

---

## Benchmark Matrix

### Engines

1. Built-in Hermes compressor
   - `context.engine=compressor`

2. context-governor, archival off
   - `context.engine=context_governor`
   - `HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=false`
   - `HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=false`

3. context-governor, semantic archival on
   - `context.engine=context_governor`
   - `HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true`
   - `HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true`
   - benchmark namespace only, not production namespace initially

### Budget modes

- `soft_warn`
- `hard_cascade`
- `fail_closed` only in non-interactive fixture tests, never in normal Hermes traffic until proven

### Target budgets

Use:
- 8,000
- 20,000
- 80,000
- 120,000

Why:
- 8k catches strict-small-context behavior
- 20k matches earlier hard-cascade eval
- 80k/120k matches large-context practical use

### Workload classes

1. Real Hermes long sessions
   - largest sessions by text volume
   - sessions with code edits
   - sessions with failed tests then fixes
   - sessions with semantic-memory discussion
   - sessions with user corrections/preferences

2. Synthetic adversarial fixtures
   - huge noisy tool logs with one critical error path
   - repeated duplicate tool outputs
   - conflicting decisions
   - stale task followed by latest user reversal
   - prompt-injection-like tool output
   - many file paths, only some relevant
   - large social/emotional content that should NOT become durable memory

3. Live smoke sessions
   - new Hermes process with context_governor active
   - force compression near threshold
   - verify receipt tools work in-process
   - verify no startup crash

4. Semantic-memory archival trials
   - isolated namespace first: `context_governor_bench`
   - then `projects_staging`
   - never direct production `projects` until gates pass

---

## Phase 0 — Safety Baseline and Rollback

### Task 0.1: Record repo/config state

Objective: capture exact state before benchmarking.

Files:
- Read: `/home/sikmindz/.hermes/config.yaml`
- Read: `/home/sikmindz/.hermes/hermes-agent`
- Read: `/home/sikmindz/Coding/Libraries/context-governor`

Commands:

```bash
cd /home/sikmindz/.hermes/hermes-agent
git status -sb
git rev-parse HEAD
git rev-parse origin/main
hermes --version
hermes update --check

cd /home/sikmindz/Coding/Libraries/context-governor
git status -sb
git rev-parse HEAD
sha256sum ~/.local/bin/context-governor

cp /home/sikmindz/.hermes/config.yaml /home/sikmindz/.hermes/config.yaml.context-governor-bench-pre
```

Expected:
- Hermes repo shows only intentional branch/stash state
- `hermes update --check` says already up to date
- binary exists
- config backup created

### Task 0.2: Create benchmark output directories

Objective: keep all outputs auditable and separate from production memory.

Create:
- `/home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/`
- `/home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/reports/`
- `/home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/fixtures/`
- `/home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/receipts/`

Command:

```bash
mkdir -p \
  /home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/reports \
  /home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/fixtures \
  /home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/receipts
```

Expected:
- directories exist

### Task 0.3: Define rollback commands

Objective: make rollback one command, not a panic.

Commands:

```bash
# revert active engine only
hermes config set context.engine compressor
hermes config set compression.threshold 0.85
hermes config set compression.target_ratio 0.2

# restore full config if needed
cp /home/sikmindz/.hermes/config.yaml.context-governor-bench-pre /home/sikmindz/.hermes/config.yaml

# clear context-governor env flags for new shells
unset HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED
unset HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED
unset HERMES_CONTEXT_GOVERNOR_BUDGET_MODE
unset HERMES_CONTEXT_GOVERNOR_TARGET_TOKENS
```

Gate:
- rollback must restore `context.engine=compressor` and a new Hermes session starts normally

---

## Phase 1 — Static Correctness and Unit Gates

### Task 1.1: Re-run Hermes plugin unit tests

Objective: prove rebased plugin still passes local tests before benchmark.

Command:

```bash
cd /home/sikmindz/.hermes/hermes-agent
python -m py_compile plugins/context_engine/context_governor/__init__.py plugins/context_engine/context_governor/live_trial.py tests/plugins/test_context_governor_plugin.py
python -m pytest tests/plugins/test_context_governor_plugin.py tests/run_agent/test_plugin_context_engine_init.py -q
```

Expected:
- py_compile exits 0
- pytest reports `13 passed`

### Task 1.2: Re-run Rust crate gates

Objective: prove compactor binary still builds/tests cleanly.

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
cargo fmt --check
cargo test --all-targets --quiet
cargo clippy --all-targets -- -D warnings
python -m py_compile scripts/hermes_replay_eval.py
python -m pytest tests_py/test_hermes_replay_eval.py -q
```

Expected:
- all commands pass

### Task 1.3: Verify binary matches current source

Objective: prevent benchmarking stale binary.

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
cargo build --release --quiet
cp target/release/context-governor ~/.local/bin/context-governor
sha256sum ~/.local/bin/context-governor
context-governor --help
```

Expected:
- binary help lists compact/store/expand/search/diff
- sha256 recorded into benchmark report

---

## Phase 2 — Replay Benchmark: Engine Quality Without Semantic Writes

### Task 2.1: Run existing context-governor replay matrix

Objective: reproduce the current 24-run baseline.

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
python scripts/hermes_replay_eval.py \
  --limit 8 \
  --min-messages 20 \
  --target-tokens-list 20000,80000,120000 \
  --budget-mode hard_cascade
```

Expected:
- writes `target/context-governor-replay/hermes-replay-report.json`
- writes `docs/hermes-replay-eval-2026-06-27.md`
- 24 successful runs
- 0 failures
- active task visible 24/24

### Task 2.2: Add soft_warn matrix

Objective: compare safer live mode against hard_cascade.

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
python scripts/hermes_replay_eval.py \
  --limit 8 \
  --min-messages 20 \
  --target-tokens-list 20000,80000,120000 \
  --budget-mode soft_warn \
  --output-dir target/context-governor-bench/reports/soft-warn
```

Expected:
- 24 runs
- no turn-breaking failures
- report copied into benchmark report directory

If `--output-dir` does not exist in the script yet:
- add it as a small TDD task before running this matrix
- do not overwrite the prior canonical docs report for every variant

### Task 2.3: Add built-in compressor comparison harness

Objective: compare against Hermes' built-in compressor instead of only measuring context-governor internally.

Modify or create:
- `scripts/hermes_compressor_replay_eval.py`
- `tests_py/test_hermes_compressor_replay_eval.py`

Required behavior:
- read same sessions/fixtures as `hermes_replay_eval.py`
- run built-in compressor via Hermes `ContextCompressor` or closest public function
- produce same report schema fields where possible:
  - full_tokens
  - compacted_tokens
  - visible probes
  - recoverable probes if possible
  - active_task_visible
  - failures

TDD steps:
1. Write test with a tiny fake transcript and expected report shape.
2. Run test and verify failure.
3. Implement minimal harness.
4. Run test and verify pass.
5. Run against real sessions.

Commands:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
python -m pytest tests_py/test_hermes_compressor_replay_eval.py -q
python scripts/hermes_compressor_replay_eval.py \
  --limit 8 \
  --min-messages 20 \
  --target-tokens-list 20000,80000,120000 \
  --output-dir target/context-governor-bench/reports/builtin-compressor
```

Expected:
- report exists
- schema aligns with context-governor report enough for side-by-side comparison

### Task 2.4: Create comparison report

Objective: produce a single table comparing compressor vs context_governor.

Create:
- `scripts/compare_context_engines.py`
- `tests_py/test_compare_context_engines.py`

Inputs:
- built-in compressor report JSON
- hard_cascade context-governor report JSON
- soft_warn context-governor report JSON

Output:
- `target/context-governor-bench/reports/context-engine-comparison.md`
- `target/context-governor-bench/reports/context-engine-comparison.json`

Required table columns:
- engine
- mode
- runs
- failures
- avg full tokens
- avg compacted tokens
- avg token reduction
- active task visible
- avg visible probe rate
- avg recoverable probe rate
- avg latency if available
- warnings count

Command:

```bash
python scripts/compare_context_engines.py \
  --builtin target/context-governor-bench/reports/builtin-compressor/report.json \
  --governor-hard target/context-governor-bench/reports/hard-cascade/report.json \
  --governor-soft target/context-governor-bench/reports/soft-warn/report.json \
  --out target/context-governor-bench/reports/context-engine-comparison
```

Gate:
- report explicitly says whether context_governor beats compressor on active-task preservation and recoverability

---

## Phase 3 — Adversarial Fixture Suite

### Task 3.1: Build fixture generator

Objective: create deterministic fixtures that catch the failures Josh cares about.

Create:
- `scripts/generate_adversarial_fixtures.py`
- `tests_py/test_generate_adversarial_fixtures.py`

Fixtures to generate:
1. `latest_user_reversal.json`
   - old task says do X
   - latest user says stop X, do Y
   - expected: latest user final and visible

2. `critical_error_in_huge_log.json`
   - 50k chars tool log
   - one `error[E0425]` with file path
   - expected: error/path recoverable

3. `duplicate_tool_spam.json`
   - repeated identical tool outputs
   - expected: duplicates omitted or receipt-only

4. `prompt_injection_tool_output.json`
   - tool output contains fake instruction to ignore user
   - expected: no elevation into active instruction

5. `durable_decision_vs_speculation.json`
   - contains real decision and speculative maybe-language
   - expected: decision preserved/archive candidate, speculation quarantined

6. `personal_social_noise.json`
   - emotional/social transient content
   - expected: not archived as durable project fact

7. `file_path_and_command_receipts.json`
   - many paths and command outputs
   - expected: relevant paths recoverable

Command:

```bash
python scripts/generate_adversarial_fixtures.py \
  --out target/context-governor-bench/fixtures/adversarial
```

Expected:
- JSON fixtures generated
- each fixture includes expected probe metadata

### Task 3.2: Run adversarial fixtures through context-governor

Objective: prove exact fallback and latest-user invariants on controlled failures.

Create:
- `scripts/evaluate_adversarial_fixtures.py`
- `tests_py/test_evaluate_adversarial_fixtures.py`

Command:

```bash
python scripts/evaluate_adversarial_fixtures.py \
  --fixtures target/context-governor-bench/fixtures/adversarial \
  --engine context_governor \
  --budget-modes soft_warn,hard_cascade \
  --target-tokens 8000,20000 \
  --out target/context-governor-bench/reports/adversarial-context-governor.json
```

Gate:
- latest user final: 100%
- critical file/error probes recoverable: >= 95%
- prompt-injection tool output never becomes active instruction: 100%
- social noise archived: 0 unless explicitly durable/user-approved

### Task 3.3: Run adversarial fixtures through built-in compressor

Objective: quantify whether context_governor improves over baseline.

Command:

```bash
python scripts/evaluate_adversarial_fixtures.py \
  --fixtures target/context-governor-bench/fixtures/adversarial \
  --engine compressor \
  --target-tokens 8000,20000 \
  --out target/context-governor-bench/reports/adversarial-compressor.json
```

Gate:
- comparison report shows where each engine wins/loses
- no hidden claim that one is globally superior unless metrics show it

---

## Phase 4 — Receipt Search/Expand Certification

### Task 4.1: Store receipts for replay outputs

Objective: verify search/expand works on persisted receipts, not just in-memory responses.

Command:

```bash
cd /home/sikmindz/Coding/Libraries/context-governor
# For each response JSON generated by fixture/replay scripts:
context-governor store \
  --dir target/context-governor-bench/receipts \
  < path/to/response.json
```

If response files are not currently emitted individually:
- add `--write-responses DIR` to the replay/eval scripts
- test it with a one-fixture run first

### Task 4.2: Search for known probes

Objective: prove receipt search finds exact fallback content.

Command pattern:

```bash
context-governor search \
  --dir target/context-governor-bench/receipts \
  --query 'error[E0425]' \
  --top-k 5

context-governor search \
  --dir target/context-governor-bench/receipts \
  --query '/src/lib.rs' \
  --top-k 5
```

Expected:
- correct receipt/item appears in top 5
- result includes receipt ID and item ID

### Task 4.3: Expand found items and verify exact match

Objective: ensure fallback content is byte/substring recoverable.

Command pattern:

```bash
context-governor expand \
  --dir target/context-governor-bench/receipts \
  --receipt <receipt_id> \
  --item <item_id> \
  --max-chars 8000
```

Gate:
- expanded text includes exact probe strings
- content_blake3 matches receipt metadata if exposed
- no unrelated item expands under same ID

---

## Phase 5 — Semantic-Memory Archival Trial in Isolated Namespace

### Task 5.1: Add benchmark namespace override

Objective: prevent test archival from polluting production `projects` memory.

Modify:
- `/home/sikmindz/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`
- `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`

Add env/config knob:

```text
HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
```

Test requirements:
- default remains current behavior
- env override changes archive payload namespace
- payload source still includes receipt/item IDs

Command:

```bash
cd /home/sikmindz/.hermes/hermes-agent
python -m pytest tests/plugins/test_context_governor_plugin.py -q
```

Expected:
- plugin tests pass

### Task 5.2: Run one forced archival smoke

Objective: prove real semantic-memory writes happen with real fact IDs.

Environment:

```bash
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
export HERMES_CONTEXT_GOVERNOR_BUDGET_MODE=soft_warn
```

Run:

```bash
cd /home/sikmindz/.hermes/hermes-agent
PYTHONPATH=$PWD python plugins/context_engine/context_governor/live_trial.py \
  > /home/sikmindz/Coding/Libraries/context-governor/target/context-governor-bench/reports/live-archive-smoke.json
```

Then verify through semantic-memory MCP/tooling:
- search namespace `context_governor_bench` for the exact decision/probe string
- fetch fact by ID if receipt reports one

Expected:
- receipt has non-empty semantic_memory_fact_ids if archival candidates exist
- `sm_get_fact` returns exact source-backed archive content
- no writes land in production `projects` namespace during smoke

### Task 5.3: Run archival on adversarial fixtures

Objective: measure junk-memory rate before enabling globally.

Command pattern:

```bash
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
python scripts/evaluate_adversarial_fixtures.py \
  --fixtures target/context-governor-bench/fixtures/adversarial \
  --engine context_governor \
  --semantic-archive on \
  --out target/context-governor-bench/reports/adversarial-semantic-archive.json
```

Gate:
- no prompt-injection content archived as instruction/policy
- no social transient content archived as project state
- all facts include receipt ID, item ID, content_blake3

### Task 5.4: Manually label archived facts

Objective: quantify memory quality.

Create:
- `target/context-governor-bench/reports/semantic-memory-labels.csv`

Columns:
- fact_id
- namespace
- source_receipt
- item_id
- content_blake3
- label: KEEP/MAYBE/JUNK/HARMFUL
- reason
- should_promote_to_projects: yes/no

Manual process:
1. List facts in `context_governor_bench`.
2. Fetch each fact.
3. Label it.
4. Count rates.

Gate:
- KEEP >= 80%
- JUNK <= 10%
- HARMFUL = 0

If gate fails:
- keep semantic archival disabled
- improve filters before another run

---

## Phase 6 — Downstream Recall Usefulness Test

### Task 6.1: Build recall challenge set

Objective: test whether archived facts help later tasks, not just whether writes occurred.

Create:
- `target/context-governor-bench/fixtures/recall_challenges.json`

Each challenge includes:
- question/prompt
- expected fact ID or receipt item
- expected answer fragment
- allowed retrieval tools
- pass/fail criteria

Example challenge classes:
- “What command failed and what file path did it cite?”
- “What did we decide about HyperQuant text compression?”
- “Which latest-user reversal should override older task context?”
- “Which content should NOT be treated as durable project memory?”

### Task 6.2: Run recall challenges with memory off vs archive on

Objective: measure downstream practical impact.

Modes:
1. No semantic-memory archive; receipts only
2. Semantic archive in `context_governor_bench`; receipts available
3. Semantic archive only; receipt tools disabled if feasible

Metrics:
- correct answer rate
- evidence cited
- tool calls used
- false recall/hallucination count
- asks user to repeat context: yes/no

Gate:
- archive-on must improve answer correctness or reduce recovery cost without increasing false recall

---

## Phase 7 — Live Hermes Trial

### Task 7.1: Start a fresh Hermes session with context_governor active and archive off

Objective: test real startup-loaded behavior.

Precondition:

```bash
hermes config set context.engine context_governor
unset HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED
unset HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED
```

Run in a new shell/session:

```bash
hermes chat -q "Run a context-governor smoke: say what context engine is active, then stop."
```

Expected:
- no startup crash
- response completes

### Task 7.2: Force a controlled compression

Objective: verify live compression does not break a real turn.

Use a generated long prompt with embedded probes:
- latest user task
- file path
- error string
- decision string

Command idea:

```bash
python /home/sikmindz/Coding/Libraries/context-governor/scripts/make_live_compression_prompt.py \
  --out /tmp/context-governor-live-prompt.txt
hermes chat -q "$(cat /tmp/context-governor-live-prompt.txt)"
```

If shell quoting is unsafe, use a small Python subprocess wrapper.

Expected:
- Hermes completes
- no provider role alternation error
- receipt file created under `~/.hermes/context-governor/receipts`
- latest task answered correctly

### Task 7.3: Repeat with semantic archive enabled in benchmark namespace

Objective: verify live archive path, still isolated.

Environment:

```bash
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
```

Run same controlled prompt.

Expected:
- response completes
- semantic fact IDs appear in receipt if candidates exist
- facts land only in `context_governor_bench`

Gate:
- no production memory pollution
- no runtime exceptions

---

## Phase 8 — Usage and Cost Watch

### Task 8.1: Capture baseline usage with built-in compressor

Objective: compare usage over equivalent tasks.

Config:

```bash
hermes config set context.engine compressor
```

Run 3 controlled long-session tasks with same prompt class.

Record:
- prompt tokens
- completion tokens
- total tokens
- wall time
- any compression calls
- whether answer was correct

### Task 8.2: Capture usage with context_governor archive off

Config:

```bash
hermes config set context.engine context_governor
unset HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED
unset HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED
```

Run the same 3 tasks.

Gate:
- no material token blowthrough vs compressor
- no repeated compaction loop
- no extra LLM summarization calls for compaction

### Task 8.3: Capture usage with archive on

Config/env:

```bash
hermes config set context.engine context_governor
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED=true
export HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_NAMESPACE=context_governor_bench
```

Run the same 3 tasks.

Gate:
- archival adds bounded local memory calls only
- no model-token blowthrough
- no repeated archive-write storm

---

## Phase 9 — Report and Decision

### Task 9.1: Generate final benchmark report

Create:
- `docs/context-governor-benchmark-2026-06-27.md`
- `target/context-governor-bench/reports/final-summary.json`

Required sections:
1. Current config and binary SHA
2. Engines compared
3. Replay metrics
4. Adversarial metrics
5. Receipt search/expand metrics
6. Semantic-memory quality labels
7. Usage/cost comparison
8. Failure modes observed
9. Decision: REJECT / TRIAL / DEFAULT ARCHIVE OFF / DEFAULT ARCHIVE ON WITH FILTERS
10. Rollback command

### Task 9.2: Make the keep/kill decision

Decision rules:

Keep `context_governor` default if:
- active task visible/final = 100%
- exact fallback recovery >= 95%
- no turn-breaking compaction failures
- replay token reduction materially beats compressor OR preserves more critical probes at similar token size
- live usage does not increase materially

Enable semantic archival only if:
- KEEP >= 80%
- JUNK <= 10%
- HARMFUL = 0
- duplicate rate <= 10%
- recall challenge correctness improves or recovery cost decreases

Otherwise:
- leave `context_governor` on with archive off, or revert to compressor if compaction gates fail

### Task 9.3: Save results to semantic memory

Only save final durable findings, not raw dump.

Save a compact project-state fact containing:
- final decision
- exact benchmark report path
- binary SHA
- core metrics
- whether semantic archival remains disabled/enabled
- rollback command

Do not save:
- raw transcripts
- per-fact label dump
- temporary test outputs

---

## Phase 10 — Cleanup

### Task 10.1: Delete benchmark namespace if rejected

If semantic archival is rejected:

```python
# Use semantic-memory delete namespace tool or MCP equivalent
sm_delete_namespace(namespace="context_governor_bench")
```

Gate:
- namespace no longer lists facts

### Task 10.2: Promote only approved facts if accepted

If semantic archival is accepted:
- manually promote only KEEP facts that should live in production namespace
- use supersession or add_fact with evidence refs
- keep benchmark namespace as audit archive only if useful

### Task 10.3: Restore desired config

If decision is DEFAULT ARCHIVE OFF:

```bash
hermes config set context.engine context_governor
unset HERMES_CONTEXT_GOVERNOR_SEMANTIC_MEMORY_ENABLED
unset HERMES_CONTEXT_GOVERNOR_ARCHIVE_MEMORY_ENABLED
```

If decision is DEFAULT ARCHIVE ON WITH FILTERS:
- set only after adding namespace/filter config support and tests
- document exact env/config required

If decision is REJECT:

```bash
hermes config set context.engine compressor
```

---

## Non-Claims Until This Plan Passes

Do not claim:
- context_governor is globally better than Hermes compressor
- semantic-memory archival improves real task success
- HyperQuant improves prompt text compression
- memory writes are production-safe
- provider-token accounting is exact
- PR readiness implies operational default readiness

Allowed current claim:
- context_governor is implemented, locally active, unit-tested, live-smoke-tested, and has replay evidence showing strong token reduction and high recoverability on the tested local matrix.

---

## Immediate Next Command Sequence

Run this first if executing the plan now:

```bash
cd /home/sikmindz/.hermes/hermes-agent
python -m pytest tests/plugins/test_context_governor_plugin.py tests/run_agent/test_plugin_context_engine_init.py -q

cd /home/sikmindz/Coding/Libraries/context-governor
cargo fmt --check
cargo test --all-targets --quiet
cargo clippy --all-targets -- -D warnings
python -m pytest tests_py/test_hermes_replay_eval.py -q
mkdir -p target/context-governor-bench/{reports,fixtures,receipts}
python scripts/hermes_replay_eval.py --limit 8 --min-messages 20 --target-tokens-list 20000,80000,120000 --budget-mode hard_cascade
```

Stop after this if any gate fails. Do not continue to semantic-memory archival until static and replay gates pass.

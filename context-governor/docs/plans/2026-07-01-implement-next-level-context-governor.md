# context-governor Next-Level Implementation Plan

> For Hermes: implement directly with TDD gates; preserve existing dirty work and do not delete prior audit artifacts.

Goal: complete the remaining next-level context-governor work after P0 boundary-audit adapter wiring: same-transcript comparisons, historical answerability replay, reducer/token-counter/index lifecycle proof, semantic-memory archive truth surfaces, and host-integration documentation.

Architecture: keep the Rust crate deterministic and host-agnostic. Put private/Hermes transcript harvesting in scripts under `scripts/` and local aggregate reports under `target/`. Do not write private transcript content to public docs. Treat external competitors as optional adapters that emit explicit `unsupported` receipts when unavailable.

Tech stack: Rust crate + CLI; Python benchmark/eval scripts; pytest and cargo tests.

## Phase 0 — preserve current state

Files already dirty before this pass include README/docs/src/lib.rs/tests/token_counter.rs plus new certification/task-success files. Do not revert them.

Verification before edits:
- `git status --short`
- inspect `scripts/task_success_eval.py`, `scripts/hermes_replay_eval.py`, `scripts/compare_context_engines.py`, `src/lib.rs`, `tests_py/*`.

## Phase 1 — Same-transcript cross-engine benchmark

Objective: produce a public-safe JSON/markdown comparison over identical synthetic fixture families.

Files:
- Create: `scripts/compare_context_engines_live.py`
- Modify: `tests_py/test_benchmark_tooling.py`

Required behavior:
- Generate at least 3 fixture families: coding log, file-search/tool output, plan+acceptance gates.
- Evaluate: `full`, `head_tail`, `context_governor`.
- Attempt optional external engines: `squeez`, `ogham`, `headroom`, `llmlingua`.
- If external engine is not installed/callable, record `status=unsupported` with a reason; do not hide it.
- Emit private machine JSON and public-safe aggregate markdown under `target/context-governor-comparisons/` by default.
- Metrics: p50/p95 latency, input/output approx tokens, token reduction, visible anchor rate, recoverable anchor rate, answerability, incorrect-action risk, safety warnings.

TDD:
1. Add pytest that calls fixture generation/evaluation in-process and asserts all required engines appear, unsupported reasons are explicit, and markdown contains no raw fixture marker text.
2. Run focused pytest and verify failure.
3. Implement script.
4. Run pytest and a real script smoke.

## Phase 2 — Historical Hermes coding-task answerability replay

Objective: aggregate-only answerability replay over local Hermes sessions without public raw transcript content.

Files:
- Create: `scripts/hermes_task_replay_eval.py`
- Modify: `tests_py/test_certification_tooling.py`
- Modify: `scripts/certify_all.py`

Required behavior:
- Sample up to 10 large active Hermes coding sessions from `~/.hermes/state.db`.
- Generate redacted/hashed operational questions from acceptance gates, file paths, errors, decisions, active user task, and blockers.
- Score `full`, `head_tail`, and `context_governor` using deterministic term-presence/recoverability against compacted output/exact fallback.
- Emit aggregate JSON and markdown only; raw terms remain hashed/truncated in machine-local target output.
- Certification quick mode should run a bounded historical eval if the DB exists, otherwise record skipped/not-required.

TDD:
1. Add pytest with a temporary SQLite state.db containing synthetic sessions.
2. Verify RED on missing script/API.
3. Implement script and certification gate.
4. Run pytest and real script smoke.

## Phase 3 — Reducer/token-counter/index/archive surfaces

Objective: complete product-grade proof surfaces for content reducers, token counter disclosure, receipt store lifecycle, and semantic-memory archive truth.

Files:
- Modify: `src/lib.rs`
- Modify/add Rust tests:
  - `tests/content_compression.rs`
  - `tests/token_counter.rs`
  - `tests/store.rs`
  - `tests/memory_sink.rs`

Required behavior:
- Reducer tests cover cargo logs, diffs, JSON, search/read-file output, Rust/code/markdown anchors.
- `context_diff` reports content-kind reduction counts.
- Token counter supports an exact-provider-labelled mode only when implemented; otherwise receipts explicitly disclose approximate mode and warning. Do not fake native tokenizer claims.
- Store lifecycle exposes index status/receipt count/store bytes via a serializable status method and proves search across 1,000 synthetic receipts under a declared local threshold.
- Semantic-memory archive remains host-owned; if no sink is supplied, warning and empty IDs are explicit. Add manufactured-confidence fixture ensuring LLM hearsay is not promoted as a confident fact.

TDD:
1. Add failing tests for each acceptance surface.
2. Implement minimal Rust changes.
3. Run focused cargo tests.

## Phase 4 — Host integration and docs

Files:
- Modify: `README.md`
- Modify: `docs/integrations/hermes.md`
- Create/update: `docs/integrations/host-adapters.md`
- Modify: `CHANGELOG.md`

Required behavior:
- Document clean install/build/certify path.
- Document Hermes as stable first host and Codex/OpenCode as planned wrappers unless actually implemented.
- Include benchmark/certification artifact paths and claim boundary.
- No public claim of global superiority over competitors without identical-input external receipts.

## Final verification

Run:
- `python3 -m pytest tests_py -q`
- `cargo fmt --check`
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `python3 scripts/compare_context_engines_live.py --quick --out-dir target/context-governor-comparisons/final`
- `python3 scripts/hermes_task_replay_eval.py --limit 10 --out-dir target/historical-answerability/final`
- `python3 scripts/certify_all.py --quick --skip-hermes`

Report:
- changed files
- commands run and actual results
- generated artifact paths
- remaining claim boundary

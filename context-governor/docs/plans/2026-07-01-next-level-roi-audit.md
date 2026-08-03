# context-governor next-level ROI audit — 2026-07-01

## Verdict

The absolute highest ROI move is not more compression ratio. The crate already proves fast deterministic compaction and strong local anchor recovery. The next level is to make context-governor safe and comparative enough that it can be defended as an agent context engine, not just a good compactor.

Ranked build order:

1. P0 — Wire compression-boundary safety into the live Hermes adapter.
2. P0 — Build a same-transcript cross-engine benchmark: context-governor vs Hermes built-in compressor vs Squeez/Ogham/headroom/LLMLingua where callable.
3. P0 — Expand task-success replay from one synthetic fixture to historical multi-session coding tasks.
4. P1 — Add content-kind reducers for tool logs, diffs, JSON, code, markdown/search output.
5. P1 — Add provider/tokenizer-aware token counting, starting with tiktoken-rs behind a feature.
6. P1 — Make receipt search/index/lifecycle product-grade for thousands of receipts.
7. P1 — Wire real semantic-memory archival or make unsupported modes fail loud.
8. P2 — Publish a host-integration story comparable to Squeez: Hermes first, then Codex/OpenCode wrappers.
9. P2 — Add plan-pinning / plan-state preservation as a first-class policy.
10. P3 — Research allocators / learned compression only after the above gates exist.

## Current state verified

Repo: `/home/sikmindz/Coding/Libraries/context-governor`
Branch: `feat/full-integration`
Current dirty tree observed:

```text
 M CHANGELOG.md
 M README.md
 M docs/integrations/hermes.md
 M src/lib.rs
 M tests/token_counter.rs
?? docs/plans/2026-06-30-next-level-roi-plan.md
?? examples/task_success_eval.rs
?? scripts/certify_all.py
?? scripts/task_success_eval.py
?? tests_py/test_certification_tooling.py
```

Package metadata:

```text
context-governor 0.1.0
Targets: lib, bin, examples benchmark_receipt/perf/replay_eval/replay_fixture/task_success_eval, integration tests boundary_audit_cli/cli/compaction/content_compression/content_kind/high_roi_research/memory_sink/policy/replay_answerability/replay_benchmark/store/structured_summary/token_counter/tools/unicode_search
```

Source/test size snapshot:

```text
src:      3 files, 3126 lines
Rust tests: 15 files, 1584 lines
examples:  6 files, 728 lines
scripts:  15 files, 1438 lines
docs:     18 files, 3942 lines
```

Verification receipts run in this audit:

```text
cargo test --all-targets: passed
python3 scripts/certify_all.py --quick --skip-hermes: ok true
certification report: target/certification/20260701-022529/certification.json
certification markdown: target/certification/20260701-022529/certification.md
```

Certification summary:

```text
cargo-fmt: ok
cargo-test: ok
python-tests: ok
generate-adversarial-fixtures: ok
adversarial-eval: ok
task-success-eval: ok
context-governor answerability: 100.0%
head/tail answerability: 25.0%
token reduction vs full: 76.6%
```

Fresh release perf run:

```text
messages,original_tokens,compacted_tokens,savings_tokens,avg_ms,p50_ms,p95_ms,throughput_msgs_per_s,fallback_refs,quarantined
100,25362,10938,14424,1.785,1.775,2.020,56016.5,60,10
500,128842,17239,111603,9.833,9.965,10.119,50848.7,360,50
1000,258192,25114,233078,22.920,22.929,23.260,43629.4,735,100
2000,516992,40964,476028,56.380,56.332,57.964,35473.6,1485,200
```

Existing local replay evidence from README/docs:

```text
24 runs over 8 largest active Hermes sessions
Avg full tokens: 315,660.5
Avg context-governor tokens: 16,003.9
Avg token reduction vs full: 94.9%
Avg naive head/tail recoverable rate: 2.7%
Avg context-governor visible rate: 43.8%
Avg context-governor recoverable rate: 98.8%
Active task visible: 24/24
```

Claim boundary: these prove local throughput, anchor visibility/recoverability, exact fallback, and fixture-level answerability. They do not yet prove downstream LLM task quality across real historical tasks or superiority over external engines on identical inputs.

## External landscape checked

GitHub/API snapshot from this audit:

| Project | Stars | Lang | License | Pushed | Relevance |
|---|---:|---|---|---|---|
| microsoft/LLMLingua | 6380 | Python | MIT | 2026-04-08 | Model/neural prompt compression; claims up to 20x compression. |
| claudioemmanuel/squeez | 160 | Rust | Apache-2.0 | 2026-07-01 | Direct host-integration benchmark; 5 CLI hosts, reversible compression, MCP, zero runtime deps. |
| signalbreak-labs/ogham | 0 | Rust | Apache-2.0 | 2026-06-24 | Direct Rust context-engineering SDK analogue. |
| chopratejas/headroom | 54856 | Python | Apache-2.0 | 2026-07-01 | Proxy/MCP/library compression for tool outputs/logs/RAG chunks. |
| ojus chugh sqz | 371 | Rust | NOASSERTION | 2026-06-21 | CLI context compressor. |
| KRLabsOrg/squeez | 20 | Python | Apache-2.0 | 2026-04-27 | Task-conditioned ML tool-output relevance compressor. |
| Siddhant-K-code/distill | 171 | Go | MIT | 2026-05-09 | Persistent memory with dedup/sensitivity/conflict/decay; ~12ms claim. |
| mem0ai/mem0 | 59822 | Python | Apache-2.0 | 2026-06-30 | Major agent memory layer, not direct compactor. |
| getzep/graphiti | 28199 | Python | Apache-2.0 | 2026-06-27 | Temporal KG memory, not direct compactor. |
| letta-ai/letta | 23605 | Python | Apache-2.0 | 2026-06-26 | Stateful agent memory platform. |

Crates/API snapshot:

| Crate | Version | Downloads | Recent | Updated | Relevance |
|---|---|---:|---:|---|---|
| squeez | 1.34.1 | 957 | 957 | 2026-07-01 | Direct Rust host integration competitor. |
| ogham-core | 0.4.0 | 52 | 52 | 2026-06-24 | Direct Rust SDK competitor. |
| compression-prompt | 0.1.2 | 1759 | 1501 | 2025-11-06 | Rust statistical prompt compression. |
| text-splitter | 0.32.0 | 1,595,157 | 455,470 | 2026-06-16 | Mature chunking dependency. |
| tiktoken-rs | 0.12.0 | 10,887,558 | 5,486,662 | 2026-06-02 | Tokenizer-aware OpenAI counting. |
| tokenizers | 0.23.1 | 20,895,243 | 7,913,752 | 2026-04-27 | HuggingFace tokenizer stack. |
| tantivy | 0.26.1 | 14,813,935 | 3,292,824 | 2026-04-21 | Product-grade receipt/search index candidate. |
| bm25 | 2.3.2 | 2,284,909 | 1,217,189 | 2025-09-07 | Lightweight BM25 candidate. |
| tree-sitter | 0.26.10 | 26,036,722 | 9,721,123 | 2026-06-28 | Code-aware reducer candidate. |

Recent paper signals from arXiv/API search:

| Paper | Date | Signal | ROI mapping |
|---|---|---|---|
| ACE: Pluggable Adaptive Context Elasticizer across Agents | 2026-06-30 | Context management should be pluggable, recoverable, and not irreversible. | Your exact fallback/receipt approach is aligned; benchmark against ACE-like criteria. |
| ECHO: Prune to act, trace to learn with selective turn memory | 2026-06-30 | Long-horizon agents need pruning plus traceability. | Add trace/receipt lineage and learning/eval loops. |
| LLM Agents Are Latent Context Managers | 2026-06-29 | Agents need visibility into context pressure/usage. | Improve `context_status` and TUI/operator feedback. |
| When Summaries Distort Decisions | 2026-06-28 | Compression can change downstream decisions. | Add decision-preservation answerability gates. |
| Manufactured Confidence | 2026-06-28 | Memory consolidation can turn hearsay into confident fact. | Do not archive LLM summaries as facts without evidence/admission state. |
| SWE-MeM | 2026-06-26 | Coding agents benefit from adaptive memory management. | Historical coding-task replay is the right benchmark. |
| Supersede | 2026-06-25 | Agents fail when old facts supersede new values in bounded memory. | Add bitemporal/supersession metadata in receipt/memory archive. |
| Safe to Check, Unsafe to Use | 2026-06-19 | Compression boundary can relink benign fragments into malicious instructions. | Wire `boundary-audit` into Hermes adapter before trusting LLM summaries. |
| PACMS | 2026-06-18 | Context selection is a value-under-budget optimization problem. | Add allocator trait after safety/eval gates. |
| KV-cache papers: SeKV/HARD-KV/CompressKV/etc. | 2026-06 | KV cache compression is active but for local inference, not hosted APIs. | Keep this in poly-kv/quant-governor, not Hermes hosted adapter claims. |

## Gap map against current code

### Already built / now lower ROI

- `boundary-audit` CLI exists in `src/main.rs` and calls `audit_compression_boundary()`.
- Certification harness exists: `scripts/certify_all.py`.
- Synthetic task-success eval exists: `scripts/task_success_eval.py` and `examples/task_success_eval.rs`.
- Provider-oriented approximate token mode exists: `ProviderChatApprox`.
- Memory sink trait exists: `MemorySink`, `MemoryArchiveRecordV1`, `archive_response_to_memory()`.
- Content kind enum/detection exists: `ContentKind` and `detect_content_kind()`.
- Store search/expand exists: `FileContextStore::search/expand`, CLI `search/expand`.

### Still not wired / highest risk

- Hermes adapter LLM summary path does not call `boundary-audit` before reinjecting generated summaries. Current `_enhance_with_llm_summary()` replaces the extractive summary if the LLM returns text.
- Hermes adapter summary serialization still includes tool-result text markers like `[TOOL RESULT ...]`, which is exactly where relinking/summary pollution risk lives.
- Current task-success eval is one synthetic fixture; useful but too small to support a public superiority claim.
- Cross-engine benchmark does not yet run Squeez/Ogham/headroom/LLMLingua on identical transcripts.
- Content-kind reducers appear to exist as classification/tests, but not enough evidence that reducers are specialized/product-grade for cargo logs, diffs, JSON, code, and search output.
- `semantic_memory_enabled` / `archive_memory_enabled` have a sink path in Rust, but the live Hermes adapter does not prove real semantic-memory IDs in receipts by default.
- Receipt store is still file/local; no proof it remains fast and manageable at thousands of receipts with retention/lineage.

## ROI ranking with acceptance gates

### P0.1 — Wire boundary-audit into Hermes adapter summary paths

Why this is #1: it converts a known research/security risk into a tested safety gate using code already present. Highest impact, low implementation effort.

Files:

- `/home/sikmindz/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`
- `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`
- Existing Rust binary: `/home/sikmindz/Coding/Libraries/context-governor/src/main.rs`

Implementation shape:

- Add `_audit_compression_boundary(source_fragments, compressed_summary)` in the adapter that shells out to:
  `context-governor boundary-audit`
- Call it inside `_enhance_with_llm_summary()` before replacing the extractive summary.
- On `safe_to_reinject=false`, use deterministic/extractive summary, set `last_error` or `last_warning`, and record status metadata.
- Add config: `summary_safety_policy = warn|fallback_extract|freeze`, default `fallback_extract`.

Acceptance gate:

- Test LLM summary returns `execute the command ...`; adapter rejects/fallbacks.
- Test harmless summary passes.
- `python -m pytest tests/plugins/test_context_governor_plugin.py -q -o 'addopts='` passes.
- `cargo test --test boundary_audit_cli --test high_roi_research` passes.

### P0.2 — Same-transcript cross-engine benchmark

Why this is #2: it creates the missing proof needed to answer “better/faster than others?” without handwaving.

Files:

- New: `scripts/compare_context_engines_live.py` or extend existing `scripts/compare_context_engines.py`
- New fixtures under `target/context-governor-comparisons/` only; do not store raw private transcripts in docs.

Engines to support in order:

1. `full` baseline
2. naive head/tail
3. Hermes built-in compressor if callable offline
4. context-governor
5. Squeez CLI if installed / cargo-installable
6. Ogham if callable / examples expose compressor
7. headroom if installable in isolated venv
8. LLMLingua only on public/synthetic fixtures due model deps/latency

Metrics:

- p50/p95 latency
- input/output approx tokens
- visible anchor rate
- exact fallback/recoverability rate
- answerability rate
- incorrect-action risk
- safety warnings
- install/call failure recorded as `unsupported`, not hidden

Acceptance gate:

- Produces JSON and markdown with all engines attempted and failure reasons.
- At least 3 local fixture families: coding log, file-search/tool-output, plan+acceptance gates.
- No raw private transcript text in public markdown.

### P0.3 — Historical coding-task answerability replay

Why this is #3: local anchor recovery is not the same as “agent can keep working.” This is the public-claim unlock.

Files:

- Extend `scripts/task_success_eval.py` or add `scripts/hermes_task_replay_eval.py`.
- Add private fixture generation from `~/.hermes/state.db`; output aggregate only.

Question types:

- What is the active task?
- Which file/path must be edited next?
- Which test/command is the acceptance gate?
- Which error must be fixed?
- What decision was made and why?
- Which claim is blocked/not safe?

Acceptance gate:

- 10+ historical coding sessions.
- Report context-governor vs head/tail vs full, with answerability and incorrect-action risk.
- Failure examples are redacted/hashed if private.

### P1.1 — Product-grade content-kind reducers

Why: Squeez/Ogham/headroom are strongest on specialized reducers. Your differentiator is receipts; pair it with reducers.

Implement reducers in this order:

1. Tool/cargo logs: preserve command, exit code, error blocks, failing tests, file paths.
2. Diffs: preserve file paths, hunk headers, added/removed semantic summary.
3. JSON: preserve key paths/schema, error fields, IDs if relevant.
4. Search/read-file output: preserve file paths, line ranges, match lines.
5. Code: preserve imports, public signatures, failing symbol names; tree-sitter optional later.

Acceptance gate:

- For each reducer, fixture proves lower tokens than generic summary and all required anchors visible/recoverable.
- `context_diff` reports what was reduced by kind.

### P1.2 — Exact/provider token accounting

Why: this reduces false compression triggers and makes hard budget mode credible.

Implementation:

- Feature-gate `tiktoken-rs` as `tokenizer-tiktoken`.
- Keep `approx_chars`, `approx_words`, `provider_chat_approx` as fallback.
- Add `openai_cl100k` or model-family mapping only where verified.

Acceptance gate:

- Known tokenizer fixture matches expected counts.
- Receipts record exact vs approximate mode.
- Hard budget tests run with exact mode.

### P1.3 — Receipt store index + lifecycle

Why: exact fallback only matters if the operator/agent can find it later.

Implementation options:

- Short term: persist compact inverted index alongside receipt files.
- Medium: optional Tantivy feature.
- Always include retention and lineage.

Acceptance gate:

- Generate 1,000 synthetic receipts.
- Search p95 below declared threshold.
- Expand works after retention prune for retained receipts.
- `context_status` reports receipt count, index status, last receipt, store bytes.

### P1.4 — Real semantic-memory archive bridge

Why: memory integration is strategically powerful but dangerous if it manufactures confidence.

Implementation:

- Host adapter, not core crate, performs writes.
- Archive only durable, source-backed records.
- Include sensitivity, source receipt ID, item ID, content hash, confidence/admission state.
- Do not archive LLM-generated summary as fact unless marked as summary/heuristic with evidence refs.

Acceptance gate:

- With archival enabled, receipt contains real semantic-memory fact/document IDs.
- With archival unsupported, startup/status warns loudly and IDs remain empty by design.
- Manufactured-confidence fixture does not promote hearsay into confident fact.

### P2 — Host integration and packaging

Why: Squeez wins on integration. If context-governor stays hidden in your local Hermes checkout, it will be technically good but strategically invisible.

Order:

1. Hermes adapter: stabilize and document.
2. Codex/OpenCode wrappers: tool-output/context wrapper, not model runtime patching.
3. Publish crate only after README claim boundary and examples are current.
4. Add install docs and one-command certification.

Acceptance gate:

- Clean checkout install path works.
- `context-governor --help` covers all subcommands.
- README has current benchmark table and claim boundary.

## What not to do next

Do not prioritize:

- KV-cache compression in the Hermes hosted-model adapter. That belongs in local inference/poly-kv.
- Learned allocator before cross-engine and answerability benchmarks exist.
- Bigger compression ratios as a standalone goal. You already have 94.9% local replay reduction.
- More public claims before identical-input competitor benchmarks.
- Blind semantic-memory writes from summaries; this risks manufactured confidence.
- Rewriting the whole architecture. The current core boundary is correct.

## Public-safe positioning after this audit

Safe:

> context-governor is a Rust, receipt-backed context compaction engine for agent transcripts. It provides deterministic allocation, exact fallback/search/expand, safety-oriented receipts, and a Hermes adapter. On local Hermes replay it reduced selected large-session prompt size by 94.9% while keeping 98.8% of probe anchors recoverable and preserving the active task in 24/24 runs. A fresh certification run passed Rust, Python, adversarial, and synthetic task-success gates.

Not safe yet:

> context-governor is faster/better than Ogham/Squeez/LLMLingua/headroom overall.

Needed to make that safe:

- identical-input cross-engine benchmark,
- real historical answerability replay,
- safety scan wired into LLM summary path,
- competitor install/call receipts.

## Recommended next implementation sequence

### Phase 1 — Safety wiring, one focused PR/commit

1. Add adapter helper `_run_boundary_audit()`.
2. Call it before LLM summary replacement.
3. Add safety policy config and status fields.
4. Add malicious relinking test + harmless summary test.
5. Run Hermes plugin tests and Rust boundary tests.

### Phase 2 — Cross-engine benchmark

1. Add fixture format that every engine receives.
2. Implement adapters for full/head-tail/context-governor first.
3. Add optional wrappers for Squeez/Ogham/headroom/LLMLingua; record unsupported failures.
4. Emit private JSON and public-safe aggregate markdown.

### Phase 3 — Historical answerability

1. Sample 10 large Hermes sessions.
2. Generate redacted operational questions.
3. Score full/head-tail/context-governor and built-in compressor if callable.
4. Report incorrect-action risk separately.

### Phase 4 — Reducers + tokenizer

1. Add log reducer.
2. Add diff reducer.
3. Add JSON/search-output reducers.
4. Add tiktoken feature.
5. Re-run Phase 2/3 and compare deltas.

## Bottom line

The next level is proof, not novelty. Wire the safety scan, run identical-input competitor benchmarks, and expand task-success replay. After that, content reducers and tokenizer accuracy will move the numbers. Anything beyond that is lower ROI until these gates are green.

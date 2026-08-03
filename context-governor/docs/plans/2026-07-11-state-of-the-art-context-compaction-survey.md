# State of the art for deterministic agent-context compaction (2024–2026)

**Research-council review — 2026-07-11**

## Executive decision

The best next version of `context-governor` is **not** an LLMLingua port and not an LLM summarizer. It is a deterministic, task-conditioned **step/item selector plus typed extractive reducers**, backed by exact recovery, with hierarchical receipt retrieval. The highest-value research result is that agent traces are structurally different from prose: token-level deletion can destroy tool/action grammar even at apparently good compression ratios [AGORA]. This validates the crate's message/item-level direction and argues against token pruning in the default path.

Recommended order:

1. Replace fixed priority + sequential fit with a deterministic **query/task-conditioned submodular selector** over whole messages/steps.
2. Add an **always-keep structural floor** for system/tool schemas, current plan, action/result pairing, latest error, active user request, and acceptance commands.
3. Make reducers emit **typed symbolic records** rather than coherent prose: entities, paths, commands, decisions, qualifiers, errors, action/result links, source IDs.
4. Add a deterministic **hierarchical temporal receipt index** and retrieval-conditioned rehydration.
5. Add repeated-compaction lineage, supersession, and plan persistence tests.
6. Keep neural compression as an optional, untrusted adapter whose output must pass boundary/fidelity audits and retain exact fallback.

This is the strongest quality/privacy/determinism trade-off. Learned compressors can improve benchmark quality in selected settings, but they add model downloads, hardware variance, privacy exposure, and nondeterministic/version-sensitive behavior. They should be benchmark baselines or opt-in candidates, not the authoritative core.

## Current implementation: verified mapping

The Rust crate already has the right safety substrate:

- `src/lib.rs::classify_message` classifies messages and content kinds.
- `src/lib.rs::score_items` assigns fixed task/risk scores.
- `src/lib.rs::allocate_items` performs a sequential budget allocation.
- `src/lib.rs::build_structured_summary` extracts task anchors.
- `src/lib.rs::content_aware_preview` provides shallow typed reducers.
- `src/lib.rs::build_exact_store`, `context_search`, and `context_expand` preserve and recover exact text.
- `ContextCompactionReceiptV1`, `SummaryLossReportV1`, and hard/soft/fail-closed modes provide governance.
- `src/high_roi.rs::audit_compression_boundary` detects a narrow set of post-compression instruction hazards.
- `src/high_roi.rs::select_retrieval_route` is a deterministic routing seed.
- `agent-memory-kits/shared/scripts/context-governor-compact.py` currently fixes `deterministic_v1`, `approx_chars`, head=2, tail=8, disables memory, stores the receipt, and deliberately fails open.

The largest technical gaps are: (a) relevance is not actually conditioned on `focus`—`score_items` adds the same +25 to every non-quarantined item; (b) allocation is order-dependent rather than globally optimized; (c) item granularity is one message, so action/result and plan-state relationships are not represented; (d) typed reducers are previews, not fidelity-preserving schemas; (e) search is literal substring over one response, not hierarchical/session-lineage retrieval; (f) token counting remains approximate.

## Ranked method matrix

Ratings are prospective for this codebase, not universal benchmark claims. **Quality gain** means expected downstream agent-task gain over the current deterministic baseline. Latency and effort use Low/Medium/High; privacy and determinism use High/Medium/Low. “Model dependency” distinguishes no model, optional local model, and required model/API.

| Rank | Method | Expected quality gain | Added latency | Model dependency | Privacy | Determinism | Effort | Council decision |
|---:|---|---|---|---|---|---|---|---|
| 1 | Structural floor + step/action-result grouping | **Very high** | Low | None | High | High | Medium | **Adopt now** |
| 2 | Task-conditioned deterministic submodular selection | **High** | Low–medium | None (lexical); optional embeddings later | High | High | Medium | **Adopt now** |
| 3 | Typed symbolic/extractive reducers | **High** | Low | None | High | High | Medium–high | **Adopt now** |
| 4 | Retrieval-conditioned exact rehydration | **High** | Low–medium | None for BM25/FTS | High | High | Medium | **Adopt now** |
| 5 | Hierarchical temporal receipts + plan/supersession state | High on long horizons | Low–medium | None | High | High | Medium–high | **Adopt next** |
| 6 | Rate–distortion/knapsack allocation over representation choices | Medium–high | Low–medium | None if distortion proxies are fixed | High | High | Medium | **Adopt after 1–4** |
| 7 | Query-aware LongLLMLingua/RECOMP-style compression | Medium–high on QA/RAG; uncertain on agents | High | Required local compressor model | Medium–high if local | Medium | High | **Optional benchmark lane** |
| 8 | LLMLingua-2 token classifier | Medium on prose/RAG; agent risk | Medium | Required encoder model | Medium–high if local | Medium | High | **Do not default; guarded experiment** |
| 9 | LLM hierarchical summaries / RAPTOR-like tree | Medium on retrieval; drift risk under repeated rewrites | High | Required LLM | Low for hosted, medium for local | Low | High | **Use only as untrusted projection** |
| 10 | Generic LLM checkpoint summary | Unstable: can help or silently change decisions | Very high | Required LLM/API | Low–medium | Low | Low–medium | **Fallback only, audited** |
| 11 | Token-entropy pruning / Selective Context on agent traces | Negative-to-medium | Medium | Small causal LM | Medium–high if local | Medium | High | **Reject for authoritative agent context** |
| 12 | KV-cache eviction/quantization | No prompt-quality gain in hosted API path | Runtime-specific | Requires inference control | Depends on host | Often medium | Very high | **Separate project/layer** |

## What the strongest methods actually show

### 1. Preserve agent grammar at step granularity

AGORA reports that two token-level compressor families collapsed to mean reward ≤0.05 across 17 agent settings despite 1.3–13.3× compression. The paper attributes this to **action-grammar destruction**: identifiers, brackets, and action verbs can look low-information to generic language compressors. Its step-level compressor combines a structural parser, an always-keep floor, and a learned relevance scorer; the structural floor was the dominant component [AGORA].

**Integration:**

- Add `ContextStepV1` beside `ContextItemV1`: `user_intent`, `assistant_reasoning_or_action`, `tool_call`, `tool_result`, `state_delta`, and source indices.
- In `classify_messages`, pair tool calls/results by metadata ID where present and conservatively by adjacency otherwise.
- Add `StructuralFloorV1` to `CompactionPolicy`: preserve role/schema delimiters, active user turn, system constraints, latest unresolved error, current plan, acceptance gate, and incomplete action/result pairs.
- Never prune inside JSON tool-call syntax. Reduce tool payload fields only through a typed reducer and retain the exact blob.
- Extend receipts with `step_id`, pairing confidence, and floor reasons.

This is deterministic and captures most of AGORA's reported benefit without importing its 125M scorer.

### 2. Task-aware, diversity-aware selection rather than recency

LongLLMLingua conditions compression on the question and reports improved long-context QA plus lower token use [LongLLMLingua]. RECOMP trains extractive/abstractive compressors for downstream utility and may return no retrieved text when it adds no value [RECOMP]. PACMS frames an agent's mixed pool (history, memory, tool output) as budgeted submodular selection rather than recency truncation [PACMS].

**Integration:** replace the current equal `focus` bonus and sequential fit with deterministic marginal-gain selection:

`gain(i | S) = authority(i) + task_relevance(i) + coverage(i,S) + recency(i) + recoverability(i) - redundancy(i,S) - rate_penalty(i)`.

- `task_relevance`: BM25/weighted term overlap between latest user/focus/current plan and item fields.
- `coverage`: reward first coverage of entity/path/command/error/decision/action IDs.
- `redundancy`: deterministic shingles/MinHash or exact normalized-line overlap.
- `recoverability`: lower visible-context priority only when an indexed exact fallback is verified available.
- stable tie-break: authority, source index, then BLAKE3 ID.
- representation choices per item: exact, typed extract, recovery pointer, omit. Run a deterministic knapsack or lazy-greedy pass over measured token costs.

Add an `AllocatorMode::SubmodularV1`; retain `DeterministicV1` for compatibility. Emit each feature component and selected marginal gain in `ContextAllocationPlanV1` so the choice is auditable.

### 3. Symbolic re-expression beats fluent summary where fidelity matters

“Context Compression Is Not One Thing” reports that a readable entity–relation format beat matched-budget deletion/truncation/random baselines by 13–20 F1 points and beat coherent prose summarization on its hardest multi-hop dataset [Telegraph]. The result is for small-model QA, not coding agents, but it directly supports structured anchors over prose. “When Summaries Distort Decisions” finds plausible summaries can alter downstream decisions through decontextualization and compressor dependence [DecisionFidelity].

**Integration:** evolve `StructuredContextSummaryV1` into typed records:

- `TaskState { objective, status, next_action, blocked_by, source_ids }`
- `Decision { proposition, status, rationale_anchor, qualifiers, supersedes, source_ids }`
- `CommandOutcome { command, exit_code, failing_tests, primary_errors, source_ids }`
- `FileState { path, operation, symbols, unresolved_changes, source_ids }`
- `EvidenceClaim { subject, relation, object, qualifiers, polarity, observed_at, source_ids }`
- `RecoveryPointer { item_id, content_hash, store_generation }`

Render these deterministically in a compact line protocol. Crucially, keep qualifiers and negation attached to claims. Do not convert evidence into a fluent narrative.

### 4. Retrieval-conditioned compression and exact rehydration

RECOMP's useful insight for this project is not its neural implementation but **selective augmentation**: irrelevant retrieved content may contribute nothing and should be omitted [RECOMP]. HiGMem first navigates event summaries, then inspects selected turns [HiGMem]. MemForest uses a hierarchical temporal tree and localized updates rather than rewriting global state [MemForest]. RAPTOR recursively clusters/summarizes document chunks and retrieves at multiple abstraction levels [RAPTOR].

**Integration:**

- Store a session tree: session → epoch/checkpoint → step → exact item.
- Index typed summaries and exact items separately with BM25/FTS; use summaries only for navigation and exact items for authoritative reinjection.
- Add `context_rehydrate(query, budget, authority_floor, receipt_lineage)` returning a new retrieval receipt with candidate scores, rejected IDs, exact hashes, and token cost.
- Rehydrate before generation, not only after the model notices a missing detail.
- Link `parent_session_id` and add `parent_receipt_id`, `compaction_generation`, `supersedes`, and retention status.
- Prefer append/localized update; do not repeatedly summarize prior summaries.

RAPTOR's LLM-generated hierarchy should not be copied literally into the deterministic core. Use its navigation shape with extractive/typed nodes and exact leaves.

### 5. Plan persistence and trajectory compaction

CAT treats context management as an agent-callable tool with stable task semantics, condensed long-term memory, and high-fidelity short-term interactions; it trains a compressor from trajectory-level supervision [CAT]. “Plans Don't Persist” reports that plan signal can decay sharply after action/observation steps and argues that plans must remain explicit context state [Plans]. Cost-aware skill rewriting likewise finds that sparse API/code/workflow/rule anchors can reduce total downstream cost even when they make the retained document longer [Skills].

**Integration:**

- Add a canonical `PlanStateV1` outside ordinary message aging: objective, ordered steps, completed/cancelled state, acceptance commands, blockers, and source hashes.
- Preserve it verbatim or as typed extracts across every compaction generation.
- Detect updates/supersession rather than collecting every stale plan.
- Expose explicit `compact_now(focus)` / `checkpoint_now()` through host adapters, but keep automatic hard-budget operation.
- Evaluate repeated compaction (1, 2, 4, 8 generations), not one-shot compression.

Do not adopt CAT's learned compressor as a prerequisite. Adopt the stable/long-term/short-term workspace separation and explicit context-maintenance action.

### 6. Rate–distortion allocation

The useful information-theoretic formulation is operational rather than a claim that semantic distortion is exactly measurable:

- **Rate** = provider-token cost of each representation.
- **Distortion proxy** = expected task loss from changing exact → typed extract → pointer → omit.
- **Constraints** = structural floor, authority, privacy, exact-store availability, hard budget.

Use a deterministic multiple-choice knapsack over representation variants. Distortion should be a versioned, inspectable sum of risk features (authority, task overlap, uniqueness, age, unresolved state, action grammar, exact recoverability). Calibrate weights only against held-out historical task-success tests. Do not call the result “optimal semantic compression”; it is optimal only under the declared proxy.

## LLMLingua-family assessment

- **LLMLingua**: coarse-to-fine budget control and token compression; paper reports up to 20× with little loss on its tested tasks [LLMLingua].
- **LongLLMLingua**: question-aware compression and position reordering; reports up to +21.4% on NaturalQuestions at ~4× fewer tokens and 1.4–2.6× end-to-end speedup for tested 10k-token prompts [LongLLMLingua].
- **LLMLingua-2**: distills GPT-generated compression labels and trains a bidirectional token classifier for task-agnostic extractive compression [LLMLingua2]. Official implementation is Python/MIT [LLMLinguaCode].
- **Selective Context**: self-information-based pruning; reports 50% context-cost reduction with memory/latency benefits and small aggregate quality declines on its prose tasks [SelectiveContext]. Its public repository does not declare a license in GitHub metadata as checked 2026-07-11 [SelectiveCode].

**Why these are not the default here:** they are trained/evaluated chiefly on prose, QA, summarization, or ordinary conversations; agent syntax and tool protocol are brittle; they require model inference and model-specific tokenization; exact reruns can vary with model/runtime versions; and the strongest current agent-specific evidence is negative for generic token-level methods [AGORA].

A guarded experiment may apply LLMLingua-2 only **inside low-authority plain-text payloads**, after freezing code, paths, IDs, numbers, negation, quoted evidence, tool schemas, and action grammar. The output remains an untrusted derivative with exact fallback. It must beat typed reducers on downstream action correctness, not just token count.

## Hosted prompt compaction versus KV-cache compression

These are separate engineering layers:

| Surface | Input/output | Available to this project? | Relevant techniques |
|---|---|---|---|
| Hosted API prompt compaction | Text/chat messages → fewer text/chat tokens | **Yes** | selection, typed extraction, retrieval, LLMLingua-style text pruning, summaries, receipts |
| Local model KV-cache compression | Internal per-layer key/value tensors → smaller/evicted/quantized cache | **Only when controlling inference runtime** | token eviction, KV quantization, cache merging, layer/head budgets |

`context-governor` and `context-governor-compact.py` operate before a hosted request. They cannot inject, quantize, or retain OpenAI/Anthropic/provider KV tensors. KV-cache papers may improve local serving throughput or effective cache capacity, but they do not validate better hosted prompt compaction and must not be used to claim an extended hosted context window. Keep KV experiments in the local-inference/poly-kv/quant-governor layer.

## Techniques not to adopt

1. **Generic token-level entropy pruning for tool/agent messages.** Negative agent evidence shows action grammar can be destroyed [AGORA].
2. **Blind recursive abstractive summaries.** Repeated rewrites accumulate drift, lose qualifiers, and can change decisions [DecisionFidelity]. Exact leaves plus typed navigational nodes are safer.
3. **LLM-generated summary as authoritative memory.** It is a derived claim set, not evidence. Store as an untrusted projection with source links, or do not store it.
4. **Pre-compression security filtering only.** Compression can relink separated benign fragments into an actionable malicious instruction; the 2026 Relink paper reports 86.9% relink/backend-action rate versus 17.0% clean controls across its tested benchmarks [Relink]. Scan the emitted context and default to deterministic fallback.
5. **Compression ratio as the primary objective.** Optimize task completion, incorrect-action rate, evidence/qualifier preservation, exact-recovery success, and total end-to-end cost.
6. **Recency-only truncation.** Early plans, constraints, and decisions remain operationally necessary [Plans].
7. **Flat vector similarity as the only retrieval strategy.** It over-retrieves superficially similar turns and ignores temporal/supersession structure [HiGMem].
8. **Learned salience before an eval gate.** A scorer can improve ranking but makes the core model-dependent and harder to reproduce. First establish deterministic baselines and historical task labels.
9. **Hosted summary APIs for private transcripts by default.** This breaks the crate's local privacy posture and introduces provider retention/policy exposure.
10. **Unlicensed code ports.** Selective Context's public repo lacked a declared license in the GitHub metadata checked; use the paper's ideas, not its code [SelectiveCode].
11. **Claiming submodular or rate–distortion optimality without declared proxies.** The guarantee applies to the formalized objective, not true semantic/task loss.
12. **Treating exact fallback as sufficient when it is not proactively retrieved.** Recoverability is valuable only if search is indexed, available after retention, and invoked before an incorrect action.

## Concrete file/API roadmap

### P0 — deterministic quality core

**`src/lib.rs`**

- Add `ContextStepV1`, `PlanStateV1`, `RepresentationVariantV1`, `SelectionFeatureV1`.
- Refactor `classify_messages` → `group_steps` then `classify_steps`.
- Replace the no-op focus bonus in `score_items` with tokenized BM25/weighted overlap.
- Add `AllocatorMode::SubmodularV1` and deterministic lazy-greedy selection.
- Add a multiple-choice representation budget pass (`exact`, `typed`, `pointer`, `omit`).
- Add stable tie-breaking and feature-level selection receipts.
- Preserve provider/tool message validity; reject orphan tool results or malformed action pairs.

**Acceptance:** adversarial fixtures prove active task, plan, command syntax, identifiers, paths, numbers, negation, error, and acceptance command remain visible or proactively rehydrated; output is byte-identical across repeated runs except explicitly nondeterministic receipt/time IDs.

### P1 — typed reducers

**`src/lib.rs` or new `src/reducers/{mod,json,diff,logs,code,markdown}.rs`**

- Replace `content_aware_preview` with reducers that return typed records plus source spans.
- JSON: schema/keys, selected values, error objects, IDs, pagination, and truncation receipt.
- Logs: command, exit status, failing tests, first causal error and bounded context, final result.
- Diff/code: files, hunks, signatures/symbols, compiler locations; optional Tree-sitter later.
- Markdown/search: hierarchy, matched passages, paths, source locations.

**Acceptance:** each reducer beats head/tail and generic preview at equal token budget on answerability/action fixtures and round-trips every omitted span through exact fallback.

### P1 — indexed retrieval and hierarchy

**Store/search code in `src/lib.rs` should be split into `src/store.rs` / `src/retrieval.rs`.**

- Add session/receipt/step hierarchy and parent/supersession fields.
- Add deterministic BM25/FTS candidate generation; semantic embeddings remain optional second-stage candidates.
- Add `context_rehydrate` API/CLI with a `RetrievalReceiptV1`.
- Verify exact-store availability before assigning recoverability credit.

**Acceptance:** fixed query outputs are stable; 1k+ receipt benchmark meets a declared p95; retained exact items expand after index rebuild; missing/pruned items fail loudly.

### P1 — adapter policy surface

**`agent-memory-kits/shared/scripts/context-governor-compact.py`**

- Add CLI/env controls for allocator, budget mode, token counter, focus/current task, privacy mode, and fail-open/fail-closed behavior.
- Stop silently swallowing every compaction/store failure in machine-readable mode; emit a structured warning/error receipt while preserving host fail-open semantics when requested.
- Pass provider/model tokenizer profile instead of hard-coding `approx_chars`.
- Permit a retrieval index/memory sink only when it produces durable IDs; otherwise report unsupported.
- Return compacted messages (not only a human status) through an explicit host integration contract.

### P2 — optional neural lane

Create a host-side `CandidateCompressor` interface; do not add model dependencies to the Rust default feature set. Candidate output must include source spans and then pass:

1. exact-anchor validation;
2. action/tool grammar validation;
3. `audit_compression_boundary` (expanded beyond four phrases);
4. decision/qualifier/negation fidelity checks;
5. task-success benchmark;
6. exact fallback receipt.

## Evaluation protocol required before adoption

Use identical transcripts and equal provider-token budgets across:

1. full context;
2. head/tail;
3. current deterministic allocator;
4. structural-floor + submodular selector;
5. typed reducers;
6. optional LLMLingua-2/LongLLMLingua;
7. host LLM summary.

Measure:

- downstream task/answer correctness;
- **incorrect action** and malformed tool-call rate;
- plan/constraint/qualifier/negation preservation;
- visible versus proactively rehydrated evidence;
- exact expansion success and index availability;
- repeated-compaction drift at generations 1/2/4/8;
- prompt tokens using the target provider tokenizer;
- compactor p50/p95 latency and total request latency;
- local RAM/model footprint and model-call cost;
- privacy boundary (local deterministic, local model, hosted model);
- reproducibility across 10 runs and version-pinned environments;
- compression-boundary attack success.

Do not publish “best” claims until this same-input test exists. Paper-reported gains are evidence to test a method, not evidence that it improves this coding-agent corpus.

## Implementation/license check (2026-07-11)

- Microsoft `LLMLingua`: Python, MIT, active repository [LLMLinguaCode].
- RAPTOR reference implementation: Python, MIT [RAPTORCode].
- Selective Context: Python; GitHub API returned no declared SPDX license [SelectiveCode].
- AGORA search found `ranranrannervous/agoracompression`, MIT; this is early 2026 code and should be treated as a research reference, not a dependency [AGORACode].
- No mature Rust implementation was identified for LLMLingua/AGORA in this review. Porting model inference would be high effort and would weaken the zero-model default.

## Source notes and claim boundary

2026 items below are recent preprints. Their reported numbers have not been independently reproduced here. They should shape tests and architecture, not be copied into product claims. Repository license/activity checks were performed against GitHub API metadata on 2026-07-11.

[LLMLingua]: https://arxiv.org/abs/2310.05736
[LongLLMLingua]: https://arxiv.org/abs/2310.06839
[LLMLingua2]: https://arxiv.org/abs/2403.12968
[LLMLinguaCode]: https://github.com/microsoft/LLMLingua
[SelectiveContext]: https://arxiv.org/abs/2310.06201
[SelectiveCode]: https://github.com/liyucheng09/Selective_Context
[RECOMP]: https://arxiv.org/abs/2310.04408
[RAPTOR]: https://arxiv.org/abs/2401.18059
[RAPTORCode]: https://github.com/parthsarthi03/raptor
[AGORA]: https://arxiv.org/abs/2605.26596
[AGORACode]: https://github.com/ranranrannervous/agoracompression
[CAT]: https://arxiv.org/abs/2512.22087
[Plans]: https://arxiv.org/abs/2606.22953
[PACMS]: https://arxiv.org/abs/2606.20047
[Telegraph]: https://arxiv.org/abs/2606.14875
[DecisionFidelity]: https://arxiv.org/abs/2606.29251
[Relink]: https://arxiv.org/abs/2606.21732
[Skills]: https://arxiv.org/abs/2606.09421
[HiGMem]: https://arxiv.org/abs/2604.18349
[MemForest]: https://arxiv.org/abs/2605.23986

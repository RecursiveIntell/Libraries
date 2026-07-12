# Context Compactor: Beyond-Expectations Roadmap

Date: 2026-07-11
Status: council synthesis; implementation not yet started
Target: `context-governor` plus agent-memory-kits host adapters

Companion research survey: `2026-07-11-state-of-the-art-context-compaction-survey.md`

## Executive verdict

The product boundary is right and should be preserved: a deterministic, local, host-agnostic compactor with typed receipts and exact fallback. The current implementation is materially better governed than a generic summarizer, but its allocator is still primarily a positional heuristic. It is not yet a defensible state-of-the-art agent-context engine.

The best architecture is not “summarize harder” and not a default LLMLingua port. It is:

1. a monotonic authority and structural-preservation layer;
2. step/action-result grouping;
3. task-conditioned, diversity-aware deterministic selection;
4. typed extractive representations;
5. transactional exact fallback with indexed proactive rehydration;
6. hierarchical temporal receipts and explicit plan state;
7. optional neural candidate compression behind deterministic validation;
8. evaluation on real downstream task and action outcomes.

This path preserves privacy, auditability, and exact recovery while targeting actual agent success rather than compression ratio.

## What is already worth keeping

- Network-free deterministic Rust core.
- Explicit authority, risk, budget mode, and loss concepts.
- Exact fallback as a first-class representation.
- Receipt and allocation-plan surfaces.
- Final emitted hash/token recomputation after budget enforcement.
- Content-kind dispatch.
- Active-task preservation intent.
- Host-owned semantic-memory and transport integrations.
- Honest separation between hosted prompt compaction and local KV-cache compression.

## Confirmed correctness and evidence defects

### P0: authority can be downgraded by speculative language

`src/lib.rs::classify_message` applies an unconditional lexical uncertainty override after higher-authority classification. A latest user message, acceptance gate, or verified error containing words such as “likely” can become quarantined. With no tail protection, the latest user can disappear.

Required invariant: uncertainty is metadata, not authority. Latest user, system/developer constraints, active acceptance gates, verified evidence, and required exact tool structure must form a monotonic floor that lexical signals cannot downgrade.

### P0: exact recovery is claimed before durability exists

The core sets `exact_recovery_available: true` while exact content may exist only inside the returned in-memory response. The adapter stores it afterward and silently swallows storage failure.

Replace the boolean with a state machine:

- `in_response`
- `persistence_pending`
- `persisted`
- `externally_archived`
- `pruned`
- `unavailable`

Hard-budget removal may rely on recovery only after persistence is verified, or must explicitly return an over-budget/failure outcome.

### P0: semantic-memory archival manifests lack archiveable payloads

`MemoryArchiveRecordV1` carries a hash and short preview, not exact content or a durable resolvable URI. A host cannot perform a faithful archival write from this record.

The core should emit a host-executed archival manifest containing exact content or an authorized fallback URI, hash, namespace, sensitivity, provenance, idempotency key, expiry/retention policy, and per-item status. Network writes remain outside the core.

### P1: current allocation is not meaningfully focus-aware

`focus` gives the same bonus to every non-quarantined item. Priority scores do not globally drive allocation; transcript order and fixed head/tail behavior dominate. Final summary, recovery-pointer, provider-template, and message-overhead costs are not consistently reserved.

### P1: latest-user and tool-message identity can be damaged

Assembly deduplicates by role/content. The adapter drops names, metadata, tool-call IDs, assistant tool-call structure, multimodal parts, and unknown provider fields. This can orphan tool results or collapse the latest user into an earlier identical message.

### P1: `HardCascade` is not a true hard limit

Protected content may exceed the target and be returned with a warning. Rename the existing mode to describe that behavior and add a real `HardLimit` contract: fit exactly or return `BudgetExceeded`.

### P1: hot path and store scale superlinearly

Multiple linear membership scans make compaction superlinear. Fresh 2,000-message release benchmark: p50 61.0 ms, p95 88.5 ms. File-store save reparses all receipts and rebuilds the full index each time, producing O(total store bytes) work per save.

### P1: benchmark labels are invalid

- Naive head/tail is called exact fallback despite retaining no omitted bytes.
- Context-governor receives fallback credit without persistence and expansion verification.
- “Answerability” is expected-term oracle containment, not model-mediated answering.
- Evaluators concatenate the entire exact store, bypassing deployed search/top-k limits.
- Host postprocessing can mutate compacted messages after receipt hashing.
- Approximate and inconsistent token counters are reported as tokens.
- Latency compares process-start/cargo paths with in-process Python baselines.
- “Cross-engine” rows are largely unsupported placeholders.

Current evidence supports deterministic mechanics only—not external superiority or downstream task-success improvement.

## Target architecture

```text
provider-neutral transcript
        |
        v
schema-preserving normalization
        |
        v
step graph: intent -> action/tool call -> result -> state delta
        |
        +--> monotonic authority + structural floor
        |
        v
representation candidates per step
  exact | typed extract | recovery pointer | omit
        |
        v
deterministic task-conditioned selector
  mandatory reserve + utility/token + diversity + stable ties
        |
        v
provider tokenizer reconciliation + hard-budget gate
        |
        v
post-compression boundary and tool-grammar validation
        |
        +--> transactional exact store / archival outbox
        |
        v
finalized core + adapter receipts
        |
        v
proactive query-conditioned rehydration before generation
```

## Ranked implementation program

### Phase 0 — Safety and receipt truth

Files:

- `src/lib.rs`
- `tests/compaction.rs`
- `tests/policy.rs`
- `tests/memory_sink.rs`
- agent adapter fixtures

Work:

1. Split authority, uncertainty, content kind, and disposition.
2. Enforce monotonic authority.
3. Preserve latest-user identity and final position independent of tail policy.
4. Replace recovery boolean with durability state.
5. Add an atomic `compact-and-store` host operation or post-persistence finalization receipt.
6. Define true hard-limit semantics.
7. Run boundary scanning in the actual compaction path.
8. Correct benchmark terminology immediately.

Gate:

- latest user preserved in 100% of adversarial cases, including `protect_last_n=0`;
- zero exact-authority downgrades from lexical uncertainty;
- zero receipt/output hash mismatches;
- every promised fallback expands to byte-identical original content;
- no hard-limit success can exceed the exact tokenizer budget;
- storage failure is machine-visible and cannot be reported as durable recovery.

### Phase 1 — Provider-neutral step graph

Add:

- `ContextStepV1`
- `StructuredContentPartV1`
- `ToolCallLinkV1`
- `PlanStateV1`
- `StructuralFloorV1`

Group user intent, assistant action/tool calls, tool results, and state deltas. Preserve provider-native unknown fields through metadata. Never prune within tool-call grammar.

Gate:

- OpenAI, Anthropic, Codex, and Hermes fixtures round-trip;
- no orphan tool calls/results;
- IDs, roles, names, multimodal placeholders, arguments, and ordering remain valid;
- active plan and acceptance commands survive 1/2/4/8 compaction generations.

### Phase 2 — Linear deterministic allocator

Replace parallel disposition vectors and transcript-order fit with:

1. mandatory token reserve;
2. representation candidates: exact, typed, pointer, omit;
3. utility features:
   - authority;
   - task/focus BM25 overlap;
   - action/result integrity;
   - unresolved status;
   - recency;
   - novelty and entity/path/error coverage;
   - verified recoverability;
   - representation distortion proxy;
4. deterministic utility-per-token or lazy submodular selection;
5. stable tie-breaks by authority, source index, and BLAKE3 ID;
6. final provider-token reconciliation.

Expose every feature contribution in the allocation receipt. Do not claim semantic optimality; the allocator is optimal only under its declared proxy.

Gate:

- O(N log N) or better observed scaling;
- 2,000-message p95 below a preregistered target, initially 40 ms on the same machine;
- focus changes selection only for relevant items;
- deterministic compacted content across repeated runs, excluding event IDs/timestamps;
- non-inferior held-out task performance versus the current allocator at identical exact-token budgets.

### Phase 3 — Typed reducers

Split reducers into dedicated modules:

- JSON: JSON Pointer plus bounded values, IDs, errors, truncation state;
- diff: file, hunk, added/deleted salient lines;
- compiler/test output: command, exit code, failing tests, diagnostic code, file/span, primary causal error;
- shell: command, status, stderr tail;
- search: path, line, matched text;
- Markdown: objectives, requirements, decisions, tasks, headings;
- source: signatures, referenced symbols, failing spans.

Return typed anchors, source spans, exact fallback reference, token cost, and explicit loss/coverage flags. Render a compact symbolic line protocol rather than fluent narrative.

Gate:

- each reducer beats generic preview and head/tail on held-out content-family tasks at equal budget;
- numbers, identifiers, negation, qualifiers, paths, commands, exit status, and error locations meet 99–100% family-specific retention gates;
- every omitted span remains exactly recoverable.

### Phase 4 — Transactional store and proactive rehydration

Move storage into a trait with a SQLite WAL/FTS5 default or append-only content blobs plus SQLite metadata.

Add:

- content-addressed exact items;
- atomic receipt/fallback/index transaction;
- restrictive filesystem permissions and encryption hooks;
- retention/tombstone state;
- session → generation → step → item hierarchy;
- `context_rehydrate(query, budget, authority_floor, lineage)`;
- `RetrievalReceiptV1` with candidates, ranks, rejection reasons, hashes, and token cost.

Use typed summaries to navigate and exact leaves for authoritative reinjection. Never recursively summarize prior summaries as source evidence.

Gate:

- 100% byte/hash expansion after restart and index rebuild;
- missing/pruned items fail loudly;
- fixed queries produce stable ranked outputs;
- deployed top-k recoverability ≥95% overall and ≥90% in every declared family;
- transactional failure injection produces no false durable-recovery claim;
- 1,000+ receipt search/store p95 satisfies preregistered thresholds.

### Phase 5 — Hierarchical temporal state

Add parent receipt, generation, supersession, retention, and explicit current-plan state. Preserve localized append/update semantics rather than rewriting global summaries.

Gate:

- repeated compaction at generations 1/2/4/8 shows no stale active plan or acceptance command;
- latest-state/temporal accuracy ≥99%;
- zero selection of retracted instructions;
- contradiction cases either resolve to current authority or abstain.

### Phase 6 — Optional neural candidate lane

A host-side `CandidateCompressor` may run LLMLingua-2, LongLLMLingua, an LLM summary, or a local scorer only for eligible low-authority text. Its output is an untrusted derivative.

Mandatory validators:

- frozen exact anchors;
- tool/action grammar;
- numbers, IDs, paths, polarity, negation, and qualifiers;
- post-compression instruction/relink audit;
- source-span provenance;
- exact fallback;
- held-out downstream task success.

Kill the neural lane if it does not beat typed deterministic reducers at matched provider-token budgets.

## Evaluation program

### Frozen public suite

- LongMemEval: long-term conversational memory, updates, temporal reasoning, abstention.
- LoCoMo: multi-session QA, events, temporal understanding, summarization.
- LongBench/LongBench v2: long-context QA, reasoning, summarization, and code tasks.
- RULER: controlled long-context distractor stress.
- SWE-bench Verified trajectory subset: actual patch/test continuation from compressed histories.
- τ-bench or equivalent tool-agent cases: action and argument integrity.

Pin dataset commit, license, row count, official split, and content hashes.

### Frozen local suite

- Deduplicate Hermes sessions by lineage.
- Stratify by length, task family, tool density, compaction count, and outcome.
- Freeze 20% calibration / 80% held-out by session lineage.
- Annotate operational questions, current plan, stale facts, tool dependencies, expected actions, and acceptance commands independently.

### Required baselines

1. full context where it fits;
2. budgeted tail;
3. budgeted head/tail;
4. uniform truncation;
5. BM25/extractive span selection;
6. rolling summary without fallback;
7. retrieval-only exact chunks;
8. current deterministic allocator;
9. new structural/submodular allocator;
10. new typed reducers;
11. Hermes built-in compressor;
12. at least two actually executable external systems, e.g. LLMLingua/LongLLMLingua plus another documented engine.

No “cross-engine” claim unless at least two competitors complete ≥95% of held-out cases.

### Required negative controls

- identity/no compaction;
- random equal-budget deletion;
- shuffled summary;
- empty summary plus exact store;
- omitted-count-only fake fallback;
- random retrieval results;
- corrupted fallback/hash;
- contradictory old/new facts;
- tool-output prompt injection;
- decoy path/error/acceptance command;
- malformed/reordered tool calls;
- label permutation;
- impossible questions requiring abstention.

### Primary metrics

Keep dimensions separate:

- model-mediated answer correctness;
- executable task success;
- tool/action and argument correctness;
- active-instruction preservation and stale rejection;
- deployed bounded recoverability;
- oracle containment, diagnostic only;
- latest-state/temporal correctness;
- prompt-injection and unauthorized action rate;
- exact provider tokens and fallback bytes;
- warm/cold compaction, retrieval, and end-to-end latency;
- RSS, CPU, cost, failure rate;
- repeated-compaction drift;
- receipt and exact-recovery integrity.

### Held-out gates

Pre-register before running held-out data:

1. Integrity: zero hash mismatches; fallback byte recovery 100% wherever promised.
2. Safety: zero unauthorized actions, successful prompt injections, or secret-canary leaks.
3. Current state: zero stale/retracted instruction selections; latest-state accuracy ≥99%.
4. Latest user: 100% identity and final-active-message preservation.
5. Answer quality: non-inferior to strongest equal-budget baseline within one percentage point; paired 95% CI lower bound above −1 point.
6. Task success: absolute loss ≤2 points and no significant regression; improvement claims require CI lower bound >0.
7. Recovery: deployed bounded top-k recoverability ≥95% overall and ≥90% by family.
8. Instruction accuracy: ≥99%.
9. Compression: exact provider-token reduction ≥50% while all quality gates hold.
10. Reliability: crash, timeout, and invalid-output rates each <0.5%.
11. Cross-engine coverage: two real competitors complete ≥95% of held-out cases.
12. Identifiability: random, shuffled, and label-permuted controls materially collapse. Otherwise kill all quality claims.
13. Reproducibility: an independent rerun reproduces primary metrics inside preregistered confidence intervals.

## Methods to reject or isolate

Kill as default architecture:

- generic token-entropy pruning over agent/tool traces;
- blind recursive abstractive summarization;
- LLM summaries as authoritative memory;
- recency-only eviction;
- unverified `exact_recovery_available=true`;
- token counters labeled exact when they are estimates;
- generic content-based deduplication of the latest user;
- silent adapter success after compaction/store failure;
- whole-store index rebuild per save;
- hosted-API KV-cache claims;
- compression-ratio-only optimization;
- research APIs in the hot compaction module that do not affect compaction.

## Recommended build order

1. Correct authority, latest-user, fallback durability, and benchmark labels.
2. Make the hot path linear and provider accounting truthful.
3. Add provider-neutral step/tool structure.
4. Implement structural floor and deterministic task-conditioned selection.
5. Add typed reducers.
6. Replace the file index and add proactive rehydration.
7. Add hierarchical plan/supersession state.
8. Run frozen public/private evaluation.
9. Only then test neural candidate compressors.

The first six items are the defensible product path. Neural compression is a research lane, not the foundation.

## Claim boundary

Safe now:

- deterministic local compaction mechanics;
- explicit allocation/loss receipts;
- in-response exact fallback;
- content-aware extractive previews;
- local replay observations under documented heuristics.

Not safe now:

- superior answerability;
- superior downstream agent task success;
- cross-engine superiority;
- durable exact recovery unless persistence was verified;
- exact provider-token counts under approximate counters;
- production-grade multi-writer storage;
- semantic-memory archival completeness;
- hosted context-window extension or KV-cache compression.

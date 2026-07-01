# Context-governor / Hermes context engine high-ROI research audit — 2026-06-30

## Executive verdict

The next ROI is not “more compression.” It is making compression safe, measurable, and self-recovering in live agent sessions.

The highest-return path is:

1. Wire the existing high_roi.rs safety primitives into the live Hermes adapter.
2. Add a replay/answerability gate that proves checkpoint summaries preserve task execution, not just string probes.
3. Replace approximate token accounting with provider/tokenizer-aware accounting where feasible.
4. Add content-type compressors for the dominant token sources: tool logs, diffs, JSON, code, and search output.
5. Make receipts and context_search operationally first-class: indexed, retained, compacted, and visible in status.
6. Add semantic-memory archive as a real integration or remove/rename the config knobs that imply it.
7. Harden the new checkpoint-summary path with security scans and explicit loss receipts.

The immediate blocker I already fixed in this session was necessary but not sufficient: the context governor now has `last_real_prompt_tokens` and checkpoint-summary rescue, but that rescue path is still a first implementation. It needs a safety/eval harness before it should be treated as production-proven.

## Evidence gathered

### Local code and verification state

Hermes plugin paths:
- `/home/sikmindz/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`
- `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`
- `/home/sikmindz/.hermes/hermes-agent/agent/context_engine.py`
- `/home/sikmindz/.hermes/hermes-agent/agent/turn_context.py`
- `/home/sikmindz/.hermes/hermes-agent/agent/conversation_compression.py`

Rust crate paths:
- `/home/sikmindz/Coding/Libraries/context-governor/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/context-governor/src/high_roi.rs`
- `/home/sikmindz/Coding/Libraries/context-governor/tests/high_roi_research.rs`
- `/home/sikmindz/Coding/Libraries/context-governor/docs/hermes-replay-eval-2026-06-27.md`
- `/home/sikmindz/Coding/Libraries/context-governor/docs/roi-audit-2026-06-27.md`

Current local scan:
- Plugin adapter: 1,364 lines, 0 TODO, 0 unwrap, 0 panic.
- Rust `src/lib.rs`: 2,161 lines, 0 TODO, 1 unwrap, 0 panic.
- Rust `src/high_roi.rs`: 617 lines, contains safety/eval primitives but is not wired into the live Hermes adapter.
- Plugin tests: 44 `def test_` functions after this session.
- Context-governor crate gates run this session:
  - `cargo test`: passed.
  - `cargo clippy --all-targets -- -D warnings && cargo fmt --check`: passed.
- Hermes plugin gates run this session:
  - `tests/plugins/test_context_governor_plugin.py`: 30 passed.
  - `tests/agent/test_context_engine.py + tests/plugins/test_context_governor_plugin.py`: 56 passed.
  - `tests/agent/test_context_compressor.py`: 120 passed.

Known local replay evidence:
- `/home/sikmindz/Coding/Libraries/context-governor/docs/hermes-replay-eval-2026-06-27.md` reports 24 successful replay runs over large Hermes sessions.
- Avg full tokens: 315,660.5.
- Avg governed tokens: 16,003.9.
- Avg reduction: 94.9%.
- Context-governor recoverable rate: 98.8%.
- Active task visible: 24/24.
- Claim boundary in that doc: this measures anchor survival and exact recoverability, not downstream LLM answer quality.

### External ecosystem and paper evidence

Crates/implementation scan performed via crates.io/GitHub APIs:
- `ogham-core`: v0.4.0, 51 downloads, updated 2026-06-24. Direct Rust context-engineering analogue.
- `tiktoken-rs`: v0.12.0, 10,872,640 downloads, updated 2026-06-02. Mature provider-token accounting dependency.
- `text-splitter`: v0.32.0, 1,593,824 downloads, updated 2026-06-16. Mature token-aware chunking dependency.
- `tokenizers`: v0.23.1, 20,864,005 downloads, updated 2026-04-27. Mature tokenizer stack.
- `bm25`: v2.3.2, 2,281,324 downloads, updated 2025-09-07. Search/scoring dependency candidate.
- `tantivy`: v0.26.1, 14,806,582 downloads, updated 2026-04-21. Full-text index candidate.
- `tree-sitter`: v0.26.10, 26,002,474 downloads, updated 2026-06-28. Code-aware compaction dependency candidate.

Relevant recent papers from arXiv API search:
- arXiv:2606.21732, “Safe to Check, Unsafe to Use: Relinking at the Compression Boundary of LLM Agents” — summarization can relink benign fragments into an actionable malicious instruction after pre-compression filters. Directly applies to checkpoint summaries and LLM enhancement.
- arXiv:2606.22953, “Plans Don’t Persist: Why Context Management Is Load Bearing for LLM Agents” — plans are the stress case for compression because they are written early, used for many steps, and often evicted first. Directly applies to head decay, checkpoint summaries, and replay gates.
- arXiv:2606.20047, “PACMS: Submodular Context Selection as a Pluggable Engine for LLM Agents” — context selection should optimize value under budget, not just recency/summarization. Directly maps to allocator improvements.
- arXiv:2606.11680, “Organize then Retrieve: Hierarchical Memory Navigation for Efficient Agents” — long-horizon agents need temporal/causal organization, not only similarity retrieval. Maps to receipt graph / session tree navigation.
- arXiv:2605.16746, “State Contamination in Memory-Augmented LLM Agents” — toxic/adversarial context can be laundered through summaries. Directly maps to post-summary security checks.
- arXiv:2606.24775, “Are We Ready For An Agent-Native Memory System?” — memory systems need storage/retrieval/update/consolidation lifecycle evaluation, not monolithic task-success only. Maps to memory archive + receipt lifecycle.
- arXiv:2606.24428, “Escaping the Self-Confirmation Trap” — execute-distill-verify reduces bad experience memories. Maps to compression summaries needing verification/admission, not blind reuse.
- arXiv:2606.14875, “Context Compression Is Not One Thing” — structured symbolic re-expression can beat coherent summary at matched budget. Maps to content-kind compressors and structured task facts.
- arXiv:2606.29251, “When Summaries Distort Decisions” — summaries can alter downstream decisions. Maps to answerability/fidelity gates.
- KV-cache papers such as CompressKV/Block-GTQ/UltraQuant/SeKV are relevant to local inference/poly-kv but not directly to hosted Hermes prompt compaction. They should not be used to claim hosted API context extension.

## Highest-ROI improvement matrix

Scoring: ROI is impact / effort / risk. P0 means do before treating this as production-safe. P1 means next high-leverage sprint. P2 means good but not blocking. P3 means research/strategic.

| Priority | Improvement | Why ROI is high | Files likely touched | Acceptance gate |
|---|---|---|---|---|
| P0 | Post-compression safety scan for LLM summaries/checkpoints | Directly addresses relinking and memory laundering risk. LLM summaries are untrusted generated content. | Python adapter + Rust high_roi bridge | Test where distributed source fragments become a malicious summary; adapter blocks or marks unsafe before reinjection. |
| P0 | Wire `audit_compression_boundary()` into checkpoint + LLM enhancement path | The Rust primitive already exists in `src/high_roi.rs`; not using it live wastes prior work. | `src/lib.rs` or CLI JSON command; plugin adapter | Checkpoint summary with “execute command” gets warning/blocked; safe summary passes. |
| P0 | Replay-answerability eval, not just probe recoverability | Current replay proves 98.8% recoverable probes, not whether the model can continue the task correctly. | new script under Hermes or context-governor examples/tests | N fixed historical tasks: model answers same operational questions before/after compaction; report pass/fail. |
| P0 | Explicit summary-loss receipt for checkpoint path | New checkpoint rescue path currently has synthetic `llm_checkpoint_*`, but no stored receipt/loss report. | plugin adapter, maybe Rust receipt type | Every checkpoint includes dropped span count, summary source hash, tail/head counts, safety scan result. |
| P0 | Harden “cannot compact further” loop with a compression state machine | Current fix is functional but should become explicit: deterministic → checkpoint → fail-open/freeze. | plugin adapter | Unit test covers 3 cycles: deterministic no-op, checkpoint rescue, then fail-open with warning if still over budget. |
| P0 | Make ContextEngine ABC declare all real status fields or make callers fully generic | The crash came from hidden required fields. This class of bug will recur for plugins. | `agent/context_engine.py`, `turn_context.py`, tests | A fake minimal context engine missing plugin-only fields cannot crash preflight logging/status. |
| P1 | Provider/tokenizer-aware token counting | Approx chars/4 causes bad thresholds and false “over/under budget.” Mature dependencies exist. | Rust crate policy + plugin config | tiktoken/OpenAI counter matches known fixture within small tolerance; approximate mode still available. |
| P1 | Content-kind compressors: logs, diffs, JSON, code, markdown/search output | Biggest token source is tool output. One generic summary is lower ROI than specialized reducers. | Rust `detect_content_kind`, summary builder, tests | Fixtures show cargo errors, diff hunks, JSON keys, file paths survive while bulk noise shrinks. |
| P1 | Indexed receipt store using Tantivy/BM25 or compact in-memory persisted index | context_search must remain fast as receipts grow. Current file scan is acceptable at 50 receipts, not at thousands. | Rust store + CLI search | Search over 1k receipts under fixed latency threshold; exact expand still works. |
| P1 | Real semantic-memory archive bridge or config rename/removal | Current knobs can imply more integration than exists. Shadow-truth risk. | plugin adapter, memory sink config, Rust memory sink | `semantic_memory_enabled=true` produces fact/document IDs or fails loudly with explicit warning. |
| P1 | Hierarchical receipt/session graph | Papers point to organize-then-retrieve; repeated flat receipts are weak for long sessions. | store schema + status/search tools | Receipts link parent/child/checkpoint; context_search can scope by session lineage. |
| P1 | Plan pinning / plan-aware preservation | “Plans Don’t Persist” says plans are high-risk evictions. Current classifier has acceptance gates but no explicit plan object. | classifier + adapter | Plans/TODOs/acceptance gates from early turns survive repeated compaction or are checkpointed with explicit plan section. |
| P1 | Summary fidelity tests: decision preservation | Financial-summary paper generalizes: summaries can alter decisions. For code agents, “what should we do next?” must not flip. | replay eval fixtures | Before/after summaries preserve keep/kill decisions and blockers on fixed transcripts. |
| P1 | Receipt-backed context_status UX | Operators need to know when compaction rescued vs no-op vs checkpointed. | plugin `get_status`, CLI/TUI status | `context_status` reports last mode, last savings, last safety scan, last checkpoint, searchable receipt count. |
| P1 | Configurable checkpoint policy | Current default `llm_checkpoint_after_compressions=2` is reasonable but blunt. Need model/context-specific policy. | plugin config loader | Config supports mode: off/after_n/ineffective_only/threshold_pct, with tests. |
| P2 | Use `text-splitter` for semantic chunks within giant tool output | Better than naive head/tail for logs/docs. | Rust crate | Long markdown/API output fixture keeps headings + error sections. |
| P2 | Tree-sitter code compressor | Code should compress by symbols/imports/errors, not prose. | optional feature in Rust crate | Python/Rust code fixtures preserve function signatures and error lines. |
| P2 | Sensitivity/redaction scan before summary storage | Prevents LLM summary and receipt store from persisting secrets. | adapter serialization + Rust scan | Secret fixture redacted from summary and warning recorded; exact store handling is explicit. |
| P2 | Model-cost accounting and ROI dashboard | Context compression is partly cost control. Report actual prompt/cache deltas per engine. | Hermes logs/scripts | Benchmark emits p50/p95 latency, prompt tokens, cache ratio, compaction cost. |
| P2 | Golden “bad compression” corpus | Prevent regressions from future allocator tweaks. | tests/fixtures | Corpus covers relinking, stale plan, duplicated tool output, path-only evidence, contradictory facts. |
| P2 | Fail-closed option for unsafe summaries | Default can be fail-open, but high-risk users need freeze-on-unsafe. | plugin policy | Config `unsafe_summary_policy=fail_open|freeze|fallback_extract` tested. |
| P2 | Cross-engine comparison harness | Built-in compressor vs context-governor vs checkpoint policy should be measured on same fixtures. | benchmark script | Outputs comparison table with tokens, recoverability, answerability, latency. |
| P3 | Learnable/submodular allocator | PACMS-style selection could beat heuristics, but first wire evals. | Rust allocator trait | Only after answerability gate exists. |
| P3 | KV-cache semantic retention | Valuable for local inference/poly-kv, not hosted Hermes. Keep separate. | poly-kv/quant-governor, not plugin | Local inference benchmark only; no hosted API claims. |

## Concrete sprint order

### Sprint 0: Safety gate for the fix just shipped

Objective: make the checkpoint rescue safe enough to leave enabled.

Tasks:
1. Add a Rust CLI subcommand or exported JSON path for `audit_compression_boundary()` so the Python adapter can call it without embedding Rust internals.
2. In `_compress_with_checkpoint_summary()` and `_enhance_with_llm_summary()`, run boundary scan on source fragments + generated summary.
3. Add policy handling:
   - `warn`: keep summary but mark status unsafe.
   - `fallback_extract`: discard LLM summary and use deterministic fallback.
   - `freeze`: return original messages and set `last_error`.
4. Store safety result in `context_status` and checkpoint message metadata.

Acceptance gates:
- Malicious relinking fixture blocks/fallbacks.
- Safe summary fixture passes.
- Existing plugin suite still passes.
- Context-governor Rust tests still pass.

### Sprint 1: Replay-answerability harness

Objective: stop measuring only “can search recover exact text” and start measuring whether compaction keeps the agent operational.

Tasks:
1. Select 10 historical Hermes sessions with stable expected answers/actions.
2. Generate questions from acceptance gates, plans, errors, files, and decisions.
3. Run three modes: full transcript, built-in compressor, context-governor checkpoint policy.
4. Score exact/semantic answerability, active-task preservation, and incorrect-action risk.
5. Produce private report; do not store raw transcript text in docs.

Acceptance gates:
- Report includes model/provider, token counts, compaction mode, answerability score, failure examples.
- No raw private transcript content in markdown.

### Sprint 2: Token accounting + content-kind reducers

Objective: reduce unnecessary LLM checkpointing and improve deterministic compression quality.

Tasks:
1. Add token counter feature/profile using `tiktoken-rs` for OpenAI-compatible models where feasible.
2. Add reducers:
   - cargo/test logs: keep command, exit code, first/last errors, failing tests.
   - diffs: keep filenames, hunks, added/removed summaries.
   - JSON: keep keys/schema shape + selected matching values.
   - code: keep imports, public signatures, errors/TODOs.
3. Add fixtures for each.

Acceptance gates:
- Deterministic reducers beat current generic summary on token count while preserving fixture anchors.
- Provider-aware token counter warning disappears when exact mode is configured.

### Sprint 3: Receipt/index/memory lifecycle

Objective: make recovery reliable past a few dozen receipts.

Tasks:
1. Add indexed receipt store or Tantivy/BM25 search.
2. Add parent/child/session lineage fields for checkpoint receipts.
3. Wire semantic-memory archive or explicitly disable/rename config.
4. Add retention policy: max receipts by count/age/size, with index rebuild.

Acceptance gates:
- Search over 1k synthetic receipts under a target latency.
- Expand works after retention pruning for retained receipts.
- Semantic-memory IDs appear when enabled, or config refuses startup if unsupported.

## Blunt keep/kill decisions

Keep:
- ContextEngine plugin route. Correct abstraction boundary.
- Receipt + exact fallback architecture. This is the core differentiator.
- Checkpoint rescue idea. It directly solves compaction exhaustion.
- Rust deterministic core + Python adapter split.
- Existing high_roi.rs primitives. They are useful; wire them.

Kill or defer:
- Any claim that this extends hosted model context windows. It does not; it governs prompt compaction and retrieval.
- KV-cache work inside the Hermes hosted-API adapter. Keep that in local inference/poly-kv.
- More compression ratios as the main metric. You already have 94.9% reduction evidence; now measure correctness and safety.
- Generic “LLM summary makes it better” without fidelity/security receipts.
- Semantic-memory config knobs that do not produce IDs or explicit warnings.

## Current claim boundary

Safe to claim now:
- Context-governor is a receipt-backed prompt compaction engine for Hermes.
- It preserves latest user as active instruction in tested paths.
- It exposes context_search/context_expand/context_status tools in the plugin.
- It has exact fallback for omitted content in stored receipts.
- Local replay report showed 94.9% average token reduction and 98.8% probe recoverability over selected large Hermes sessions.
- The new checkpoint rescue path can reduce a synthetic overgrown session and preserve latest user in tests.

Not safe to claim yet:
- That checkpoint summaries preserve downstream task quality in live model behavior.
- That LLM summaries are safe against relinking/memory laundering.
- That semantic-memory archive is fully wired in live Hermes unless IDs are verified.
- That provider-native token budgets are exact in current default mode.
- That context-governor beats the built-in compressor overall. It likely has capability advantages, but the comparison must include answerability and safety, not just token reduction.

## Recommended next command sequence

1. Add compression-boundary scan to live checkpoint/LLM summary paths.
2. Run:
   - `cd /home/sikmindz/.hermes/hermes-agent && python -m pytest tests/plugins/test_context_governor_plugin.py -q -o 'addopts='`
   - `cd /home/sikmindz/Coding/Libraries/context-governor && cargo test --all-targets && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
3. Build replay-answerability harness before adding more compression algorithms.

## What I did not verify

- I did not run a full Hermes test suite.
- I did not call a live provider for checkpoint-summary answerability; the smoke path used a monkeypatched summary function earlier in this session.
- I did not verify current upstream Ogham GitHub repository contents because the guessed GitHub path returned 404; crates.io metadata did verify `ogham-core` exists and is current.
- I did not inspect every parent `Libraries` dirty file. The context-governor crate lives inside a heavily dirty workspace; this report only scopes context-governor/Hermes context-engine work.

## Implementation closeout — 2026-06-30

Shipped in this fix pass:

- Rust CLI `boundary-audit` command wired to `audit_compression_boundary()`.
- Hermes adapter now runs boundary scans on checkpoint summaries and LLM summary enhancement before reinjection.
- Unsafe summary policies added: `fallback_extract` default, `freeze`, and warning mode support.
- Checkpoint messages now carry `ContextGovernorCheckpointReceiptV1` metadata with source hash, summary hash, dropped span count, head/tail counts, mode, and safety scan result.
- `context_status` now reports last compaction mode, last safety scan, last checkpoint receipt, receipt count, semantic-memory integration state, and token-counter status.
- ContextEngine base contract now declares the real token/defer status fields that core code reads.
- Semantic-memory/archive knobs now expose `unsupported_no_sink` when enabled without a wired memory sink, avoiding silent shadow truth.
- Tool-log summaries now preserve high-value error/failure anchors instead of only line counts.
- Replay-answerability harness added in Rust: `evaluate_replay_answerability()` with `ReplayAnswerabilityQuestion` and baseline scores for full/head-tail/context-governor.
- Installed rebuilt `/home/sikmindz/.local/bin/context-governor` so live Hermes uses the new boundary-audit command.

Verification receipts:

- `python -m pytest tests/plugins/test_context_governor_plugin.py -q -o 'addopts='` → 35 passed.
- `python -m pytest tests/agent/test_context_engine.py tests/plugins/test_context_governor_plugin.py -q -o 'addopts='` → 61 passed.
- `python -m pytest tests/plugins/test_context_governor_plugin.py tests/agent/test_context_engine.py tests/agent/test_context_compressor.py -q -o 'addopts='` → 181 passed.
- `cargo test --all-targets` → passed, including new `boundary_audit_cli` and `replay_answerability` tests.
- `cargo clippy --all-targets -- -D warnings && cargo fmt --check` → passed.
- Live plugin smoke from Hermes repo: context_governor discovered/loaded; malicious checkpoint LLM output fell back to extractive mode; `compressed 26 -> 10 mode checkpoint_fallback_extract safe True`; latest user preserved.

Remaining claim boundary:

- The new replay-answerability harness is deterministic/fixture-level. I did not run a live paid model answerability matrix over private historical sessions in this pass.
- Provider-native tokenization is still not fully exact by default; status now says when `approx_chars` is active.
- Semantic-memory archive is not wired as a real sink; it now reports unsupported instead of implying IDs will appear.


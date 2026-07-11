# Hostile agent-memory integrity benchmark

**Claim boundary:** local executable patterns only; no named competitor was tested. Thresholds were compiled into the harness before execution: 100% pass rate, zero stale retrievals, unsupported admissions, and namespace leakage, with replay equivalence required. `not_tested` is excluded from the denominator and never counted as pass. Latencies are single-run wall-clock microseconds and are descriptive, not a performance claim.

| Subject | Pass/Tested | Not tested | Pass rate | Thresholds met | stale | unsupported | leakage | replay |
|---|---:|---:|---:|---|---:|---:|---:|---|
| semantic_memory_real_memory_store | 8/9 | 0 | 88.9% | false | 0 | 1 | 0 | true |

- `correct_fact_retrieval` **Pass** (0 µs): MemoryStore::list_facts
- `unsupported_model_fact_admission` **Fail** (0 µs): add_fact accepts source labels but performs no evidence admission check
- `conflicting_observations` **Pass** (265 µs): HistoricalAt preserves both observations before the supersession edge became recorded
- `source_retraction_supersession` **Pass** (0 µs): Current uses a real supersedes graph edge and excludes the stale head
- `temporal_as_of_correctness` **Pass** (0 µs): HistoricalAt reconstructs the pre-supersession view while Current selects the new head
- `duplicate_replay_idempotency` **Pass** (544 µs): facts 6->6
- `namespace_isolation` **Pass** (244 µs): namespace-scoped public API
- `prompt_injection_preservation` **Pass** (0 µs): opaque content compared exactly; not passed to an instruction interpreter
- `integrity_rebuild` **Pass** (8414 µs): full_before=true full_after=true

| mutable_latest_value_sqlite_baseline | 5/7 | 2 | 71.4% | false | 0 | 1 | 0 | true |

- `correct_fact_retrieval` **Pass** (19 µs): exact key retrieval
- `unsupported_model_fact_admission` **Fail** (12 µs): ordinary overwrite store has no admission governance
- `conflicting_observations` **Fail** (11 µs): latest retained=true; prior conflict lost
- `source_retraction_supersession` **Pass** (10 µs): latest value replaces old value but lineage is absent
- `temporal_as_of_correctness` **NotTested** (0 µs): baseline has no temporal API
- `duplicate_replay_idempotency` **Pass** (105 µs): rows 5->5
- `namespace_isolation` **Pass** (13 µs): namespace included in primary key
- `prompt_injection_preservation` **Pass** (11 µs): payload compared byte-for-byte; benchmark never executes content
- `integrity_rebuild` **NotTested** (0 µs): baseline intentionally has no governance/rebuild API

| append_only_event_log_baseline | 5/8 | 1 | 62.5% | false | 1 | 1 | 0 | false |

- `correct_fact_retrieval` **Pass** (0 µs): linear scan
- `unsupported_model_fact_admission` **Fail** (0 µs): ungoverned log admits event
- `conflicting_observations` **Pass** (0 µs): both observations preserved without adjudication
- `source_retraction_supersession` **Fail** (0 µs): retraction has no governed interpretation
- `temporal_as_of_correctness` **Pass** (0 µs): event order can reconstruct pre-conflict state
- `duplicate_replay_idempotency` **Fail** (3 µs): events 6->12
- `namespace_isolation` **Pass** (3 µs): filtered linear scan
- `prompt_injection_preservation` **Pass** (0 µs): opaque bytes only
- `integrity_rebuild` **NotTested** (0 µs): ungoverned vector log has no integrity/rebuild API

## Limitations

- The tested public fact append API still has no evidence-admission operation; the unsupported proposal is therefore a real failure, not a simulated rejection.
- Supersession and temporal checks exercise the public `add_graph_edge_at` and `list_facts_with_view` APIs with `HistoricalAt` and `Current`.
- Integrity uses real `verify_integrity(Full)` and `reconcile(RebuildFts)` APIs; vector-artifact rebuild is feature/backend specific and was not claimed.
- MockEmbedder removes network/model variance while exercising the real MemoryStore, SQLite, FTS, deduplication, scoping, and integrity paths.
- Baselines are intentionally minimal local patterns, not products.

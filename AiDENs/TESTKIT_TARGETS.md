# Testkit Targets

Codex must create or update `crates/aidens-testkit` tests so each architecture gate is executable.

| Test | Owning phase | Purpose |
|---|---:|---|
| `source_paths_verified` | 00 | Verifies AiDENs resolves stack crates from `~/Coding/Libraries` sibling paths and never any `Libraries2` stack-ids path. |
| `docs_updated_for_current_dependencies` | 00 | Verifies active docs do not contradict current Cargo stack dependencies. |
| `contract_owner_proof` | 01 | Fails if AiDENs public APIs expose local canonical truth structs. |
| `canonical_id_roundtrip` | 01 | Verifies stack ID usage without local ID wrappers or legacy conversion. |
| `stack_import_smoke` | 02 | Verifies AiDENs imports canonical crates from `libraries`. |
| `adapter_delegation_proof` | 02 | Fails if adapters bypass canonical crates. |
| `golden_vertical_slice` | 03 | Proves operator→tool→canonical receipt sink→Forge→bridge→memory→runtime→CLI using `aidens-cli` plus `tokio` only to drive the real app/runtime surfaces. |
| `malformed_tool_call_degrades` | 04 | Malformed tool output must stop as explicit degradation and persist canonical control evidence through the runner/receipt log. |
| `denied_tool_requires_approval` | 04 | Effectful denied tool must not execute and must emit approval plus canonical control receipt evidence. |
| `budget_exhaustion_receipt` | 04 | Budget/deadline exhaustion must produce app-visible blockage and canonical control receipt evidence. |
| `provider_route_unavailable` | 04 | Provider unavailable must be surfaced and durably recorded, not hidden. |
| `bitemporal_asof_query` | 05 | As-of query goes through semantic-memory/knowledge-runtime semantics. |
| `import_atomicity` | 05 | Bridge import commits all-or-none. |
| `query_widening_disclosure` | 05 | Scope/time/entity widening appears in runtime trace/disclosure. |
| `promotion_denies_without_verification` | 06 | No promotion without canonical verification/control state. |
| `approval_required_for_side_effect` | 06 | Side effects require approval/permit path. |
| `repair_record_backpointer` | 06 | Repair record preserves backpointers/replay lineage. |
| `daemon_namespace_isolated` | 07 | Daemon namespace cannot collide with other apps. |
| `schedule_no_duplicate_storm` | 07 | Duplicate schedule/wake signals are suppressed. |
| `restart_does_not_reenqueue_completed_jobs` | 07 | Restart recovery is idempotent. |
| `kernel_exact_small_slice` | 08 | Kernel adapter calls canonical exact/bounded oracle on small slice. |
| `loopy_nonconvergence_degrades` | 08 | Nonconvergence emits explicit degradation, not fake success. |
| `release_truth_audit` | 09 | Full package audit for no shadow truth and no compatibility surfaces. |

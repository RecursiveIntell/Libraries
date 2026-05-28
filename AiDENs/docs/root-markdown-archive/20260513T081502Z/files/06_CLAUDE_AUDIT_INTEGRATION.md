# Claude Audit Integration

This file normalizes `AiDENs_P29_Hard_Audit_20260507.md` into the super-pass backlog.

## Interpretation caveat

Claude F-001 said no label before package self-replay because the audit snapshot stopped before Injection 6. The operator later created the bundle successfully. Therefore F-001 is **not** treated as source failure. It remains a mandatory final gate after Codex modifies code.

## Integrated findings

|ID|Severity|Area|Finding|Super-pass phase|Status|
|---|---|---|---|---|---|
|CLAUDE-F-001|P0|Release|No labels before package self-replay passes|Phase 15 docs/evidence closure|gate-required-not-product-defect|
|CLAUDE-F-002|Medium|Evidence|Known limitations register is blank template|Phase 15 docs/evidence closure|fixed|
|CLAUDE-F-003|Low|Evidence|128 ambiguous root Markdown files unresolved|Phase 15 docs/evidence closure|quarantined|
|CLAUDE-F-004|Low|Evidence|62 stale codex artifacts accumulating|Phase 15 docs/evidence closure|quarantined|
|CLAUDE-F-005|Medium|Architecture|Canonical boundary not enforced in verify gate|Phase 13 modularization/source ownership|fixed|
|CLAUDE-F-006|Medium|Search|Vector scan threshold is warn-only, no hard block|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-007|Low|Search|`days_since` timestamp parse failure is silent|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-008|High|Pool|Reader timeout error handling not audited at call sites|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-009|Medium|Pool|Silent Mutex poison recovery lacks health-state documentation|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-010|High|HNSW|Separate RwLocks on key maps create TOCTOU window|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-011|High|HNSW|Atomic dirty flag may lack sequentially-consistent ordering|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-012|Medium|HNSW|next_id counter never recycles deleted IDs|Phase 18 search/pool/HNSW hardening|fixed|
|CLAUDE-F-013|Medium|Contracts|budget_millis_consumed not enforced before terminal state|Phase 12 artifact lifecycle/effect enforcement|fixed|
|CLAUDE-F-014|Medium|Runner|Mock response is untyped string, not provider-schema-typed|Phase 06 provider honesty|fixed|
|CLAUDE-F-015|High|Bugs|open_bugs list status is ambiguous (fixed vs quarantined vs deferred)|Phase 19 unaudited-surface quarantine/audit|fixed|
|CLAUDE-F-016|P0|Bugs|Unaudited high-risk layers (forge-pilot, effect-runtime, verification pipeline, federation)|Phase 19 unaudited-surface quarantine/audit|quarantined|
|CLAUDE-F-017|Low|Research|External citations in research docs are unresolved tool artifacts|Phase 15 docs/evidence closure|deferred|
|CLAUDE-F-018|High|v11A|v11A path does not enumerate which bug IDs are confirmed not to affect it|Phase 12 artifact lifecycle/effect enforcement|fixed|
|CLAUDE-F-019|Medium|Runner|Mixed ownership strategies in runner suggest incremental concurrency model|Phase 06 provider honesty|fixed|
|CLAUDE-F-020|Low|Evidence|Audit logs are not hashed at generation time|Phase 15 docs/evidence closure|fixed|

## Claude-specific code actions Codex must not miss

- HNSW: fix/coarsen lock ordering or prove safe multi-lock discipline; address dirty flag ordering; document/recycle deleted ID behavior.
- Search: make large vector scan threshold configurable and able to hard-degrade/block; warn on timestamp parse failures.
- Pool: audit reader timeout handling at every call site; document poisoned-mutex recovery or add health flag.
- Contracts: enforce `budget_millis_consumed` before terminal execution receipt states.
- Runner/provider: replace string mock with typed provider mock surface; ensure local route honesty.
- Bugs/status: replace flat `open_bugs` with `fixed/quarantined/deferred/open-blocking`.
- High-risk surfaces: audit or quarantine forge-pilot, effect-runtime, verification pipeline, federation, attestation, authority-delegation, recursive-kernel-core.
- Evidence: hash external audit logs at generation time or classify as external/degraded.

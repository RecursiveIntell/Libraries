# P24 issue matrix

| ID | Priority | Phase | Title | Owner |
|---|---|---|---|---|
| P24-P0-001 | P0 | 00/01 | Verifier and package self-replay must be non-hanging, timeout-governed, and pruned | scripts / zip.py / CI |
| P24-P0-002 | P0 | 00/08 | Active docs/status drift must be collapsed to one P24 state | docs / operator UX |
| P24-P0-003 | P0 | 02/03 | Run bundle must become typed AiDENsRunBundleV2 with canonical execution context | aidens-contracts / aidens-runner / aidens-cli |
| P24-P0-004 | P0 | 02 | Canonical seam map and no-shadow-truth enforcement | aidens-contracts / aidens-integration-tests |
| P24-P0-005 | P0 | 05 | Memory/runtime seam must have a real vertical slice | aidens-memory-kit / forge-memory-bridge / semantic-memory / knowledge-runtime |
| P24-P0-006 | P0 | 04 | Coding-agent lane must be supported as a local, permit-gated product slice | aidens-tool-kit / aidens-runner / aidens-cli |
| P24-P0-007 | P0 | 07 | Boundary parsing/repair must hard-fail treatment-critical ambiguity | aidens-boundary-kit / aidens-repair-kit / aidens-testkit |
| P24-P0-008 | P0 | 08/09 | Final release package must include evidence manifest and exact support claim | zip.py / docs / CI |
| P24-P1-001 | P1 | 06 | Daemon-safe lane must demonstrate append-only local queue/schedule/wake semantics | aidens-daemon-kit / aidens-queue-kit / aidens-schedule-kit / aidens-wake-kit |
| P24-P1-002 | P1 | 03/08 | aidens-contracts monolith must be stabilized, not casually split | aidens-contracts |
| P24-P1-003 | P1 | 03/09 | Run-bundle byte identity should remain normalized, not falsely exact | aidens-runner / aidens-testkit |
| P24-P1-004 | P1 | 05/07 | Verification-plan and repair display records must route to canonical verification-control owners | aidens-governance-kit / aidens-repair-kit |
| P24-P2-001 | P2 | 08 | Scaffold profiles must stay scaffold unless they receive product tests | aidens-profile-* |

## Detailed issues

### P24-P0-001 — Verifier and package self-replay must be non-hanging, timeout-governed, and pruned

**Priority:** P0  
**Phase:** 00/01  
**Owner:** scripts / zip.py / CI

**Problem:** One local script reference check produced an ok line but did not terminate in this container before timeout. Treat this as a verifier hardening target, not proof of source failure. The next pass must add explicit traversal pruning, watchdogs, and deterministic exits.

**Required change:** Add scripts/p24_verify.sh; wrap each assertion with timeout; prune .git, target*, docs/codex-runs/archive, generated sidecars, package outputs; assert all Python scripts return within bounded time; write verifier receipt JSON.

**Acceptance:** P24 verifier completes in <120s on normal repo; emits p24_verifier_receipt.json; failure messages include command, timeout, cwd, and remediation; no verifier silently hangs.

**Risk if skipped:** A super-pass can falsely pass or never finish if the verifier layer rots.

### P24-P0-002 — Active docs/status drift must be collapsed to one P24 state

**Priority:** P0  
**Phase:** 00/08  
**Owner:** docs / operator UX

**Problem:** The package has P23 handoffs and P23 docs, while STATUS and older root docs still contain P22-era language. Active state must not require archaeology.

**Required change:** Update README, STATUS, RUN_ORDER, SUPPORT_PROFILE, docs/codex-runs/CURRENT_RUN, KNOWN_LIMITATIONS, AGENTS, and package docs to P24; archive or explicitly mark P20-P23 materials historical.

**Acceptance:** No active root/status/operator doc claims P22/P23 as current except historical references; CURRENT_RUN says P24; support profile has supported/partial/scaffold/deferred with P24 evidence links.

**Risk if skipped:** Codex may chase stale pass doctrine and re-open solved surfaces.

### P24-P0-003 — Run bundle must become typed AiDENsRunBundleV2 with canonical execution context

**Priority:** P0  
**Phase:** 02/03  
**Owner:** aidens-contracts / aidens-runner / aidens-cli

**Problem:** P23 produced useful local run-bundle JSON, but the hostile audit explicitly says it is AiDENs-local operator evidence. P24 must bind it to canonical ExecutionContextV1, TraceCtx, AttemptId/TrialId, provider/tool receipts, budget, degradation, and replay linkage.

**Required change:** Define AiDENsRunBundleV2 as a display/operator artifact with canonical backpointers; include semantic_memory_forge::ExecutionContextV1; include TraceCtx/AttemptId/TrialId; digest and verify event log; preserve local support tier honesty.

**Acceptance:** run-test-agent emits V2 bundle; inspect-run validates canonical context fields and event-log digest; test proves same run can be replayed semantically with timestamp/id-normalized comparison.

**Risk if skipped:** AiDENs becomes another local truth surface rather than a consumer of canonical semantics.

### P24-P0-004 — Canonical seam map and no-shadow-truth enforcement

**Priority:** P0  
**Phase:** 02  
**Owner:** aidens-contracts / aidens-integration-tests

**Problem:** AiDENs has many display DTOs and 924 public symbols. It must prove it is not minting canonical memory, identity, verification, repair, or execution semantics locally.

**Required change:** Create P24_CANONICAL_SEAM_MAP.md; add tests/scripts to reject local definitions named EpisodeBundleV*, ExecutionContextV*, EvidenceBundle, ExportEnvelope, ProjectionImportBatch, RepairRecord, VerificationPlan unless aliasing/re-exporting canonical owners; require backpointers on display DTOs.

**Acceptance:** cargo test includes canonical TypeId or compile-time alias proofs; script emits zero unowned canonical-looking types; all local display DTOs carry support_tier + canonical_backpointer or explicit None-with-degradation.

**Risk if skipped:** Adapter convenience silently becomes ownership collapse.

### P24-P0-005 — Memory/runtime seam must have a real vertical slice

**Priority:** P0  
**Phase:** 05  
**Owner:** aidens-memory-kit / forge-memory-bridge / semantic-memory / knowledge-runtime

**Problem:** Memory and runtime adapters exist, but AiDENs is not complete until it can demonstrate export -> bridge -> memory -> query as a consumed canonical path.

**Required change:** Create fixtures/export-envelope-v3/*.json and memory-seam test; call transform_envelope_v3 through CanonicalMemoryAdapter; query semantic and temporal view; expose view/widening/degradation in inspect output.

**Acceptance:** integration test imports one ExportEnvelopeV3 fixture, proves digest/backpointer preservation, and query output discloses view model and degradation; no AiDENs-local memory truth is minted.

**Risk if skipped:** AiDENs remains a local runner with decorative memory adapters.

### P24-P0-006 — Coding-agent lane must be supported as a local, permit-gated product slice

**Priority:** P0  
**Phase:** 04  
**Owner:** aidens-tool-kit / aidens-runner / aidens-cli

**Problem:** P23 landed fixture agent execution. P24 should turn the highest-value profile into a real local lane: read/list/search/status, safe patch proposal, and explicit denial for writes without permit.

**Required change:** Implement local coding-agent fixture project, tool dispatch, patch proposal artifact, patch apply receipt if approved, denial receipt if not approved; no shell/network escape; support profile labels it supported-local.

**Acceptance:** aidens run-coding-agent examples/configs/coding-agent.toml produces bundle with repo-read/search/status receipts, patch proposal or abstention, budget/permit receipts, and final support-tier report.

**Risk if skipped:** AiDENs remains interesting but not operator-useful.

### P24-P0-007 — Boundary parsing/repair must hard-fail treatment-critical ambiguity

**Priority:** P0  
**Phase:** 07  
**Owner:** aidens-boundary-kit / aidens-repair-kit / aidens-testkit

**Problem:** Research says parser/patch boundaries are semantic choke points. Duplicate keys, lenient coercions, JSON patch path ambiguity, or repair without provenance would poison downstream evidence.

**Required change:** Add strict JSON boundary fixtures; reject duplicate keys and unknown treatment-critical fields; require repair provenance with before/after digests, reason, confidence, and canonical repair/control backpointer when risk-bearing.

**Acceptance:** hostile fixtures fail closed; accepted repairs emit RepairRecord display artifact with digest lineage; no repair can alter treatment/outcome/episode identity without explicit failure or verification plan.

**Risk if skipped:** Structured-output repair becomes epistemic rot with a prettier file name.

### P24-P0-008 — Final release package must include evidence manifest and exact support claim

**Priority:** P0  
**Phase:** 08/09  
**Owner:** zip.py / docs / CI

**Problem:** The next package must not merely pass; it must testify what it supports, what is partial, what is scaffold, and which commands prove it.

**Required change:** Emit P24_STATUS_EVIDENCE_MANIFEST.json, P24_FINAL_AUDIT_REPORT.md, P24_KNOWN_LIMITATIONS.md, package report, findings, excluded list, archive report, and normalized run bundle digests.

**Acceptance:** Final handoff contains command transcript, artifact hashes, package SHA-256, test list, support matrix, unresolved risk register, and one-line operator claim.

**Risk if skipped:** User-facing completion becomes vibes instead of release evidence.

### P24-P1-001 — Daemon-safe lane must demonstrate append-only local queue/schedule/wake semantics

**Priority:** P1  
**Phase:** 06  
**Owner:** aidens-daemon-kit / aidens-queue-kit / aidens-schedule-kit / aidens-wake-kit

**Problem:** Daemon/desktop profiles are scaffold-heavy. P24 can promote only daemon-safe local queue if it passes idempotency and receipt gates.

**Required change:** Implement local queue lifecycle: enqueue, lease, start, heartbeat, finish/fail/cancel, wake tick; write queue-hop receipts and duplicate suppression receipts.

**Acceptance:** daemon-safe example runs with no external side effects; replay shows identical job disposition after timestamp/id normalization; duplicate enqueue suppressed with receipt.

**Risk if skipped:** Daemon becomes hand-wavy autonomy, which is the wrong kind of spooky.

### P24-P1-002 — aidens-contracts monolith must be stabilized, not casually split

**Priority:** P1  
**Phase:** 03/08  
**Owner:** aidens-contracts

**Problem:** aidens-contracts is ~10k LOC in one file. This is manageable for a super-pass if it is treated as display/report contract registry, but it needs internal module boundaries or a generated inventory.

**Required change:** Add generated type inventory, section index, and ownership comments; split only if low-risk; avoid moving semantics across crate boundaries during P24 unless tests already cover it.

**Acceptance:** P24_CONTRACT_SURFACE_REPORT.md lists every exported artifact family, owner, support tier, canonical backpointer requirement, schema path, and test path.

**Risk if skipped:** A refactor-focused pass burns time and breaks public surface without improving completion.

### P24-P1-003 — Run-bundle byte identity should remain normalized, not falsely exact

**Priority:** P1  
**Phase:** 03/09  
**Owner:** aidens-runner / aidens-testkit

**Problem:** Known limitations say run bundles are not byte-identical because timestamps/generated IDs vary. P24 should not overpromise; it should define normalized semantic replay.

**Required change:** Add normalizer that strips/replaces timestamps, generated UUIDs, and paths; compare semantic fields and digests; record exact non-normalized differences.

**Acceptance:** replay receipt distinguishes byte-identical=false from semantic_replay=true and lists normalized fields.

**Risk if skipped:** False reproducibility claims are worse than honest partial reproducibility.

### P24-P1-004 — Verification-plan and repair display records must route to canonical verification-control owners

**Priority:** P1  
**Phase:** 05/07  
**Owner:** aidens-governance-kit / aidens-repair-kit

**Problem:** AiDENs can display verification/repair decisions, but canonical verification policy/control/adjudication owns the semantics.

**Required change:** Every risk-bearing output must either include a verification-control backpointer or explicit degraded/no-plan reason. Repair display records must include canonical control/refutation references where available.

**Acceptance:** phase_07 tests prove risk-bearing patch proposal cannot be marked promotable without verification plan or explicit abstention.

**Risk if skipped:** AiDENs may promote risk-bearing actions on local policy wrappers.

### P24-P2-001 — Scaffold profiles must stay scaffold unless they receive product tests

**Priority:** P2  
**Phase:** 08  
**Owner:** aidens-profile-*

**Problem:** profile-daemon/desktop/memory/research are tiny and scan as scaffold. They should not be upgraded by prose.

**Required change:** Support profile must mark each as scaffold/partial unless a runnable example and integration test exist. Coding can be supported-local; daemon-safe may be partial/supported-local if phase 06 lands.

**Acceptance:** No profile README claims production support without test and artifact evidence.

**Risk if skipped:** Profile labels become brochureware.

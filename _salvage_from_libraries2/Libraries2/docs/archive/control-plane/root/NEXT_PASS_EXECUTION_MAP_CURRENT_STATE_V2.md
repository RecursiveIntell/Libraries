# Next Pass Execution Map — Current State V2
Use this as the sequencing document for Claude. It is intentionally biased toward killing fossilization risk and converting documented doctrine into enforced behavior.

## P0 — Must land in the next pass
1. **I004 — forge-memory-bridge / supersedes_claim_version_id mapping**
   - Problem: Claim supersession lineage is currently synthesized with a fresh ClaimVersionId whenever supersedes_claim_id is present. That invents a version ID instead of resolving the actual superseded version lineage.
   - Required fix: Do not mint a fake ClaimVersionId. Either carry a real superseded claim_version_id in the export schema, look it up via a real mapping layer, or leave the field None until real version lineage exists.
   - Acceptance: Supersession import tests prove that the bridge preserves a real prior claim_version_id or intentionally leaves it unset; no code path generates synthetic superseded version IDs from thin air.
1. **I009 — semantic-memory / import_projection_batch default-filling**
   - Problem: import_projection_batch still normalizes malformed records into plausible-looking rows via unwrap_or_default()/default state values for IDs, claim_state, projection_family, subject_entity_id, predicate, recorded_at, alias IDs, evidence handles, and episode IDs.
   - Required fix: Replace semantic default-filling with explicit validation errors for required canonical fields. Reserve defaults only for fields the spec explicitly marks optional.
   - Acceptance: Regression tests assert that missing claim_version_id/claim_id/content/subject/predicate/etc. are rejected, not silently normalized.
1. **I010 — semantic-memory / claim_with_minimal_fields_imports**
   - Problem: A test currently locks in permissive minimal-claim import behavior where only claim_version_id, claim_id, and content are supplied. That hardens the very default-filling behavior the importer should stop doing.
   - Required fix: Replace the permissive test with stricter required-field validation tests that match the canonical logical contract.
   - Acceptance: Test suite fails when required canonical fields are omitted from claim/relation rows and passes only for schema-complete imports.
1. **I014 — semantic-memory / claim_versions preferred_open invariant**
   - Problem: claim_versions has only a non-unique index on (claim_id, preferred_open) WHERE preferred_open = 1, so the database does not enforce the single-preferred-open invariant.
   - Required fix: Add a partial UNIQUE constraint/index or equivalent import-time validation that guarantees at most one preferred_open row per logical claim key.
   - Acceptance: A batch that would create two preferred_open claim versions for the same claim fails atomically; happy-path tests still pass.
1. **I015 — semantic-memory / relation_versions preferred_open invariant**
   - Problem: relation_versions likewise lacks a hard uniqueness guarantee for preferred_open rows, leaving relation version truth-order semantics unenforced at the DB layer.
   - Required fix: Add an equivalent uniqueness rule for relation_versions preferred_open semantics or enforce it transactionally before insert.
   - Acceptance: Importer rejects batches that would create multiple preferred_open relation versions for the same logical relation key.
1. **I020 — knowledge-runtime / KnowledgeRuntime::query trace entry**
   - Problem: query() always generates a fresh TraceCtx and does not accept caller-supplied trace context, which breaks end-to-end trace continuity for upstream orchestrators.
   - Required fix: Add an API path that accepts inbound TraceCtx (or a request object carrying it) and threads it through QueryTrace instead of always generating a fresh root.
   - Acceptance: Runtime query traces can preserve an upstream trace ID/parent span when provided; existing convenience API may still auto-generate when none is supplied.
1. **I021 — knowledge-runtime / SemanticMemoryAdapter::search**
   - Problem: The adapter only pushes namespace upstream. Domain/workspace_id/repo_id remain runtime-side warnings rather than real upstream enforcement.
   - Required fix: Add richer scope pushdown once semantic-memory supports it, or add an explicit scope-filtering stage over returned results that is treated as first-class execution rather than only a warning.
   - Acceptance: Queries with extra scope dimensions are either fully enforced or fail/emit a stronger “not supported” path; no silent widening.
1. **I022 — knowledge-runtime / temporal route execution**
   - Problem: Temporal execution is still best-effort post-filtering over hybrid search; when results lack temporal metadata the runtime degrades to hybrid and only emits a warning.
   - Required fix: Implement true temporal retrieval semantics over imported temporal metadata or a dedicated temporal index/view, rather than only post-filtering opaque SearchResult objects.
   - Acceptance: Temporal queries return time-scoped results without downgrade on data that contains valid_from/valid_to; degradation remains explicit only for genuinely unsupported paths.
1. **I031 — agent-graph / execute_with_retry_* / NodeStart / NodeEnd lineage**
   - Problem: Internal retry helpers reuse one AttemptId and one TrialId across the whole node execution and do not emit per-retry trial lineage. That collapses distinct concrete retries into one trial, which breaks the intended attempt/trial semantics.
   - Required fix: Mint one AttemptId per retry family and a fresh TrialId for each concrete internal retry attempt. Emit retry-aware events/checkpoint metadata per trial.
   - Acceptance: A node with retry policy produces one stable AttemptId and multiple TrialIds across internal retries; tests verify per-trial event/checkpoint lineage.

## P1 — Land immediately after P0 or alongside it if low blast radius
1. **I005 — forge-memory-bridge / ProjectionImportBatchV1.schema_version**
   - Problem: Import-batch version vocabulary is still a single generic schema_version field, which conflates export and import-side schema naming and will become confusing as soon as V2/Vn diverge.
   - Required fix: Rename or split version fields so the bridge batch can distinguish export_schema_version from import_schema_version (or equivalent wording) without ambiguity.
1. **I008 — semantic-memory / ImportBatchPayload / payload structs**
   - Problem: Local mirror payload structs mark many canonically-required fields as Option<_>, which makes the importer structurally tolerant of malformed projection rows before semantic validation runs.
   - Required fix: Move required-field validation up to the import boundary. Keep local mirror structs if needed for dependency hygiene, but explicitly validate all canonically-required fields before building storage rows.
1. **I011 — semantic-memory / import_projection_batch(&str)**
   - Problem: The canonical import path still enters semantic-memory as a JSON string boundary. That is not automatically wrong, but it remains a drift-risk seam and pushes correctness onto local deserialization/validation rather than a stronger shared contract.
   - Required fix: Keep dependency direction clean, but harden the seam with explicit contract fixtures/round-trip tests and stricter validation. Consider extracting a tiny shared import-boundary crate only if it preserves package law.
1. **I017 — semantic-memory / local mirror boundary contract**
   - Problem: semantic-memory intentionally mirrors bridge batch types locally, but the workspace still lacks a dedicated contract harness proving the local mirror stays wire-compatible as fields evolve.
   - Required fix: Add bridge->memory contract fixtures/round-trip tests that serialize real bridge batches and import them via semantic-memory without lossy/default-filled behavior.
1. **I018 — semantic-memory / legacy-only boundary coverage**
   - Problem: The dedicated “import boundary” test suite still focuses on the legacy ImportEnvelope path instead of the canonical projection-batch boundary. Canonical path has tests elsewhere, but the split still biases coverage toward the old seam.
   - Required fix: Create/expand a dedicated canonical batch boundary test suite and demote legacy-boundary tests to compatibility coverage.
1. **I023 — knowledge-runtime / projection.persist**
   - Problem: projection.persist is accepted in config but ignored at runtime; projections remain in-memory only.
   - Required fix: Either implement persistence/rebuild support or reject the flag instead of accepting-and-ignoring it.
1. **I024 — knowledge-runtime / projection rebuild execution**
   - Problem: Projection tracker records invalidation/build/failure state, but the runtime still does not execute rebuilds itself.
   - Required fix: Add explicit rebuild orchestration hooks/jobs or demote the tracker to pure observability until real rebuild execution exists.
1. **I027 — knowledge-runtime / entity_registry_mut**
   - Problem: entity_registry_mut() is deprecated but still publicly callable, so authority-like mutation of the runtime cache is still possible via the old API.
   - Required fix: Remove or gate the deprecated mutator after migrating call sites to refresh_entity_cache(); keep only fenced cache-refresh operations.
1. **I029 — agent-graph / GraphEvent legacy fields**
   - Problem: Graph events still carry public legacy trace_id: String and attempt: u32 fields on the normal event surface. They are phase-labeled, but they remain easy for callers to fossilize against.
   - Required fix: Keep canonical trace_ctx/attempt_id/trial_id primary in docs/examples now, then remove legacy fields after caller migration.
1. **I030 — agent-graph / legacy_trace_id() event emission**
   - Problem: The graph executor still derives and emits legacy trace strings at many event and payload sites, so canonical TraceCtx is present but not yet the uncontested operational normal path.
   - Required fix: Route internal logic and event emission through canonical trace_ctx first, deriving trace_id only at explicit compatibility boundaries.
1. **I032 — agent-graph / attempt field on events/checkpoints**
   - Problem: NodeStart/checkpoint recording currently uses attempt: 0 even when retry policy may perform additional internal attempts. The legacy attempt counter therefore does not reflect real retry count.
   - Required fix: Track and emit actual retry-attempt count whenever internal retry is used, or remove the legacy counter from normal-path semantics faster.
1. **I033 — job-queue / trace_id / attempt_count public fields**
   - Problem: Public queue events and JobContext still expose legacy trace_id and attempt_count fields. They are documented as migration-only but still form a broad compatibility surface.
   - Required fix: Continue migration toward canonical trace_ctx/attempt_id/trial_id, then remove legacy event/context fields once consumers migrate.
1. **I040 — LLM-Pipeline / TraceId**
   - Problem: Deprecated crate-local TraceId remains public, which keeps a legacy trace contract alive in a core control-flow crate.
   - Required fix: Continue migration to stack_ids::TraceCtx and remove TraceId once remaining callers are gone. Avoid adding any new APIs that require TraceId.
1. **I041 — LLM-Pipeline / ExecCtx / ExecCtxBuilder trace fields**
   - Problem: ExecCtx and its builder still carry both trace_id and trace_ctx, and build() always derives/retains the legacy TraceId. The crate is still dual-world rather than trace-native.
   - Required fix: Keep canonical TraceCtx primary and push legacy TraceId behind explicit compatibility boundaries; eventually remove trace_id from ExecCtx once callers migrate.

## P2 / P3 — Keep pressure on but do not let them derail closure
- **I002 — stack-ids / TraceCtx legacy helpers**: Legacy trace compatibility constructors/helpers remain public, which is acceptable for migration but still leaves a public backward-compat surface in the canonical primitive crate.
- **I003 — stack-ids / ScopeKind::Compatibility**: A compatibility-only scope kind remains in the primitive crate. Not wrong now, but it can fossilize if callers keep routing through it.
- **I006 — forge-memory-bridge / recorded_at semantics**: recorded_at is bridge-stamped at transform time, but comments across the stack have historically described it as importer-owned transaction time. The code is now consistent internally, but the semantic contract still needs one unambiguous owner/meaning.
- **I007 — forge-memory-bridge / LegacyImportEnvelopeV1 shim**: Legacy compatibility shim remains public. Acceptable for migration, but still a fossilization risk if callers never leave it.
- **I013 — semantic-memory / trace_id TEXT columns**: Projection tables and import logs still persist only trace_id TEXT fields. This may be acceptable as a canonical trace reference, but the contract is not sharp enough and currently loses parent/baggage semantics.
- **I016 — semantic-memory / ImportEnvelope legacy path**: Legacy ImportEnvelope/ImportRecord path remains public and heavily tested. Acceptable for migration, but still a prominent alternate ingestion surface.
- **I019 — semantic-memory / projection_import_log.schema_version**: projection_import_log still uses generic schema_version naming, mirroring the bridge vocabulary ambiguity and making future export/import divergence harder to reason about.
- **I025 — knowledge-runtime / Forge causal adapter**: The runtime still has no Forge/causal projection adapter, so memory-visible causal projections cannot participate in real query-time orchestration yet.
- **I026 — knowledge-runtime / entity resolution**: Entity resolution is still exact canonical/exact alias only; no fuzzy resolution, candidate expansion, or confidence-ranked matching exists yet.
- **I028 — knowledge-runtime / degraded behavior assertions**: Current tests intentionally prove downgrade/warning behavior for temporal and scope gaps. That is good now, but those tests will fossilize lag unless they are revised when the features land.
- **I034 — job-queue / queue_jobs durable trace storage**: Durable storage still keeps only trace_id TEXT rather than a richer canonical trace reference/TraceCtx representation. Canonical context is reconstructed on load.
- **I035 — job-queue / trace_ctx reconstruction**: Executor reconstructs canonical TraceCtx from persisted trace_id for jobs loaded from DB, which loses parent span/baggage continuity when only the legacy string survives.
- **I036 — job-queue / JobContext::new_direct schema bootstrap**: new_direct() bootstraps a minimal in-memory queue_jobs table that does not mirror the current canonical queue schema, which can let the direct-execution path drift from real queue semantics.
- **I037 — AI-Batch-Queue / build_job()**: The default build_job() helper initializes trace_ctx/attempt_id/trial_id to None. That is convenient, but it makes it easy for callers to enqueue untraced batch items unless they consciously choose build_job_traced().
- **I038 — Tauri-Queue / EmitterConfig include_trace_id / should_keep_trace**: The Tauri bridge still carries a dual include_trace_id/include_trace_ctx configuration model that keeps legacy string-first semantics alive in the UI layer.
- **I039 — Tauri-Queue / legacy trace compat cases**: Integration tests still deliberately validate legacy trace-string compatibility behavior. Useful now, but they will fossilize the old surface if they outlive consumer migration.
- **I042 — LLM-Pipeline / compatibility re-exports**: The crate still foregrounds broad compatibility re-exports, which can keep older API shapes feeling normative longer than intended.

## Do not let Claude do these wrong “fixes”
- Do **not** solve semantic-memory importer drift by making `semantic-memory` depend directly on Forge raw domain types.
- Do **not** hide malformed canonical projection rows behind more defaults.
- Do **not** call the runtime “done” just because it emits honest downgrade warnings.
- Do **not** add new features to legacy import / legacy trace compatibility surfaces.
- Do **not** generate fake superseded version IDs to satisfy type shapes.
- Do **not** keep one TrialId across multiple internal retries.

# Claude Code Prompt — Current State V2

You are working against the **current snapshot audit package**, not against old assumptions.

## Files to read first
1. `MASTER_ISSUE_MATRIX_CURRENT_STATE_V2.md`
2. `NEXT_PASS_EXECUTION_MAP_CURRENT_STATE_V2.md`
3. `FILE_AUDIT_INVENTORY_CURRENT_STATE.md`
4. `CANONICAL_STACK_SPEC_V5.md`
5. `LATEST6.md`
6. `LATEST5.md`

## Mission
Execute the **next closure pass** against the current codebase. Prioritize converting phase-accepted but dangerous debt into enforced canonical behavior, especially where the code still documents the right law but does not enforce it.

## Non-negotiable goals for this pass
- Kill the highest-pressure items from the issue matrix first.
- Preserve package/dependency law while fixing importer and bridge seams.
- Do not invent fake lineage IDs or hide malformed projection rows behind defaults.
- Tighten canonical trace/retry semantics instead of adding more compatibility sugar.
- Treat runtime honesty about downgrade as good, but not as completion.

## Mandatory items for this pass
- I001: snapshot — semantic-memory-forge crate — Expected forge/evidence crate is absent from this snapshot, so end-to-end causal/evidence path cannot be audited or advanced from the archive alone.
- I004: forge-memory-bridge — supersedes_claim_version_id mapping — Claim supersession lineage is currently synthesized with a fresh ClaimVersionId whenever supersedes_claim_id is present. That invents a version ID instead of resolving the actual superseded version lineage.
- I005: forge-memory-bridge — ProjectionImportBatchV1.schema_version — Import-batch version vocabulary is still a single generic schema_version field, which conflates export and import-side schema naming and will become confusing as soon as V2/Vn diverge.
- I006: forge-memory-bridge — recorded_at semantics — recorded_at is bridge-stamped at transform time, but comments across the stack have historically described it as importer-owned transaction time. The code is now consistent internally, but the semantic contract still needs one unambiguous owner/meaning.
- I008: semantic-memory — ImportBatchPayload / payload structs — Local mirror payload structs mark many canonically-required fields as Option<_>, which makes the importer structurally tolerant of malformed projection rows before semantic validation runs.
- I009: semantic-memory — import_projection_batch default-filling — import_projection_batch still normalizes malformed records into plausible-looking rows via unwrap_or_default()/default state values for IDs, claim_state, projection_family, subject_entity_id, predicate, recorded_at, alias IDs, evidence handles, and episode IDs.
- I010: semantic-memory — claim_with_minimal_fields_imports — A test currently locks in permissive minimal-claim import behavior where only claim_version_id, claim_id, and content are supplied. That hardens the very default-filling behavior the importer should stop doing.
- I011: semantic-memory — import_projection_batch(&str) — The canonical import path still enters semantic-memory as a JSON string boundary. That is not automatically wrong, but it remains a drift-risk seam and pushes correctness onto local deserialization/validation rather than a stronger shared contract.
- I012: semantic-memory — trace_ctx handling in import_projection_batch — The importer reduces incoming trace_ctx to trace_id only, losing any richer canonical trace context that may matter for parent/span/baggage continuity.
- I014: semantic-memory — claim_versions preferred_open invariant — claim_versions has only a non-unique index on (claim_id, preferred_open) WHERE preferred_open = 1, so the database does not enforce the single-preferred-open invariant.
- I015: semantic-memory — relation_versions preferred_open invariant — relation_versions likewise lacks a hard uniqueness guarantee for preferred_open rows, leaving relation version truth-order semantics unenforced at the DB layer.
- I017: semantic-memory — local mirror boundary contract — semantic-memory intentionally mirrors bridge batch types locally, but the workspace still lacks a dedicated contract harness proving the local mirror stays wire-compatible as fields evolve.
- I018: semantic-memory — legacy-only boundary coverage — The dedicated “import boundary” test suite still focuses on the legacy ImportEnvelope path instead of the canonical projection-batch boundary. Canonical path has tests elsewhere, but the split still biases coverage toward the old seam.
- I020: knowledge-runtime — KnowledgeRuntime::query trace entry — query() always generates a fresh TraceCtx and does not accept caller-supplied trace context, which breaks end-to-end trace continuity for upstream orchestrators.
- I021: knowledge-runtime — SemanticMemoryAdapter::search — The adapter only pushes namespace upstream. Domain/workspace_id/repo_id remain runtime-side warnings rather than real upstream enforcement.
- I022: knowledge-runtime — temporal route execution — Temporal execution is still best-effort post-filtering over hybrid search; when results lack temporal metadata the runtime degrades to hybrid and only emits a warning.
- I023: knowledge-runtime — projection.persist — projection.persist is accepted in config but ignored at runtime; projections remain in-memory only.
- I024: knowledge-runtime — projection rebuild execution — Projection tracker records invalidation/build/failure state, but the runtime still does not execute rebuilds itself.
- I027: knowledge-runtime — entity_registry_mut — entity_registry_mut() is deprecated but still publicly callable, so authority-like mutation of the runtime cache is still possible via the old API.
- I029: agent-graph — GraphEvent legacy fields — Graph events still carry public legacy trace_id: String and attempt: u32 fields on the normal event surface. They are phase-labeled, but they remain easy for callers to fossilize against.
- I030: agent-graph — legacy_trace_id() event emission — The graph executor still derives and emits legacy trace strings at many event and payload sites, so canonical TraceCtx is present but not yet the uncontested operational normal path.
- I031: agent-graph — execute_with_retry_* / NodeStart / NodeEnd lineage — Internal retry helpers reuse one AttemptId and one TrialId across the whole node execution and do not emit per-retry trial lineage. That collapses distinct concrete retries into one trial, which breaks the intended attempt/trial semantics.
- I032: agent-graph — attempt field on events/checkpoints — NodeStart/checkpoint recording currently uses attempt: 0 even when retry policy may perform additional internal attempts. The legacy attempt counter therefore does not reflect real retry count.
- I033: job-queue — trace_id / attempt_count public fields — Public queue events and JobContext still expose legacy trace_id and attempt_count fields. They are documented as migration-only but still form a broad compatibility surface.
- I040: LLM-Pipeline — TraceId — Deprecated crate-local TraceId remains public, which keeps a legacy trace contract alive in a core control-flow crate.
- I041: LLM-Pipeline — ExecCtx / ExecCtxBuilder trace fields — ExecCtx and its builder still carry both trace_id and trace_ctx, and build() always derives/retains the legacy TraceId. The crate is still dual-world rather than trace-native.

## Execution rules
- Prefer the smallest, highest-leverage code changes that actually improve enforcement.
- Update tests whenever you tighten a contract. Do not leave old permissive tests in place if they now encode the wrong behavior.
- If a compatibility surface remains, keep its phase label and removal condition explicit.
- Do not declare a layer finished if it still relies on warnings to confess missing execution.

## Deliverables expected from Claude
- Code changes
- Updated tests
- Updated docs/comments where semantics changed
- Short change log mapping issue IDs to code changes
- Honest note on any issue IDs not completed and why

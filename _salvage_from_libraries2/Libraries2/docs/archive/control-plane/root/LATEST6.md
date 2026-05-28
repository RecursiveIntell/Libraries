# LATEST6.md — Code-Only Reference (Post-Conformance Pass)

> **Type:** current-state snapshot
> **Generated:** 2026-03-07
> **Source:** Direct source code reads across entire workspace after conformance pass.

---

## Changes relative to LATEST5.md

### stack-ids
- **Added**: `RelationId` (edge between entities)
- **Added**: `ImportBatchId` (bridge-produced batch identifier)
- **Corrected**: `AttemptId` doc now says "logical retry family" (not "each retry gets new AttemptId")
- **Corrected**: `TrialId` doc now says "concrete execution within retry family"
- **Total public ID types**: 12 (was 10)

### semantic-memory
- **Added phase label**: `projection_import` module marked "Phase status: compatibility / migration-only"
- **Added phase label**: `ImportEnvelope` struct marked compat-only with removal condition
- **Added phase label**: `import_envelope()` method marked compat-only, points to `import_projection_batch()`
- **Added phase label**: `TraceId` in types.rs marked compat-only, points to `stack_ids::TraceCtx`
- **Added comment**: re-export block for V10 types marked as legacy with guidance
- `import_projection_batch()` documented as canonical path (was already in code)

### LLM-Pipeline
- **Added dependency**: `stack-ids`
- **Added interop**: `TraceId::to_trace_ctx()` and `TraceId::from_trace_ctx()`
- **Added phase label**: `TraceId` type and module marked "Phase status: compatibility / migration-only"

### agent-graph
- **Added dependency**: `stack-ids`
- **Added interop**: `GraphConfig::trace_ctx()` and `GraphConfig::with_trace_ctx()`
- **Added phase label**: `trace_id: Option<String>` field and `GraphEvent` enum trace/attempt fields marked compat

### job-queue
- **Added dependency**: `stack-ids`
- **Added interop**: `QueueJob::with_trace_ctx()` and `QueueJob::trace_ctx()`
- **Added phase label**: `trace_id: Option<String>` field marked compat, events module header marked compat

### AI-Batch-Queue
- **Added dependency**: `stack-ids`

### Tauri-Queue
- **Added dependency**: `stack-ids`
- **Added phase label**: `include_trace_id` config field marked compat

### forge-memory-bridge
- **No code changes** (already conforms; Case A confirmed)

---

## stack-ids Shared Primitive Inventory (v0.1.0)

| Primitive | Status | Notes |
|-----------|--------|-------|
| `EnvelopeId` | Implemented | Assigned by exporting authority |
| `ClaimId` | Implemented | Stable across claim versions |
| `ClaimVersionId` | Implemented | New per mutation |
| `EntityId` | Implemented | Opaque string wrapper |
| `EpisodeId` | Implemented | Assigned by episode creator |
| `AttemptId` | Implemented | Logical retry family (corrected docs) |
| `TrialId` | Implemented | Concrete execution in retry family (corrected docs) |
| `ArtifactId` | Implemented | Stored artifact identifier |
| `ProjectionId` | Implemented | Derived view identifier |
| `RelationId` | **Added now** | Edge between entities |
| `RelationVersionId` | Implemented | New per relation mutation |
| `ImportBatchId` | **Added now** | Bridge-produced batch identifier |
| `ScopeKey` | Implemented | Canonical partition key |
| `Scope` | Implemented | Full scope definition |
| `TraceCtx` | Implemented | W3C-compatible trace context |
| `BaggageEntry` | Implemented | Bounded baggage item |
| `ContentDigest` | Implemented | BLAKE3 content digest |
| `DigestBuilder` | Implemented | Incremental digest builder |
| `PhaseStatus` | Implemented | Current / Compatibility / PhaseGated |

---

## Supporting Crate stack-ids Adoption Status

| Crate | Depends on stack-ids | Interop methods | Phase labels | Local trace type |
|-------|---------------------|-----------------|--------------|-----------------|
| LLM-Pipeline | Yes | `TraceId::to/from_trace_ctx()` | Yes | `TraceId` (compat) |
| agent-graph | Yes | `GraphConfig::trace_ctx()` / `with_trace_ctx()` | Yes | `String` (compat) |
| job-queue | Yes | `QueueJob::with_trace_ctx()` / `trace_ctx()` | Yes | `Option<String>` (compat) |
| AI-Batch-Queue | Yes | None yet | Partial | None |
| Tauri-Queue | Yes | None (pass-through) | Partial | Inherited from job-queue |

---

## Compatibility Surfaces Still Active

| Surface | Location | Phase | Removal Condition |
|---------|----------|-------|-------------------|
| `ImportEnvelope` | `semantic-memory::projection_import` | compatibility | All callers migrate to bridge pipeline |
| `import_envelope()` | `MemoryStore` method | compatibility | All callers migrate to `import_projection_batch()` |
| `TraceId` | `semantic-memory::types` | compatibility | All internal usage migrates to `TraceCtx` |
| `TraceId` | `llm-pipeline::trace` | compatibility | All callers migrate to `TraceCtx` |
| `trace_id: String` | `agent-graph::GraphConfig` | compatibility | Replaced with `TraceCtx` field |
| `trace_id: String` | `agent-graph::GraphEvent` | compatibility | Replaced with `TraceCtx` field |
| `attempt: u32` | `agent-graph::GraphEvent::NodeStart` | compatibility | Replaced with `AttemptId` |
| `trace_id: Option<String>` | `job-queue::QueueJob` | compatibility | Replaced with `TraceCtx` field |
| `attempt_count: u32` | `job-queue::QueueJobDetails` | compatibility | Replaced with `AttemptId` |
| `trace_id: Option<String>` | `job-queue::events::*` | compatibility | Replaced with `TraceCtx` |
| `LegacyImportEnvelopeV1` | `forge-memory-bridge::legacy` | compatibility | All callers migrate to `ExportEnvelopeV1` |
| `from_legacy_trace_id()` | `stack-ids::trace` | compatibility | All crates use `TraceCtx` natively |
| `to_legacy_trace_id()` | `stack-ids::trace` | compatibility | All crates use `TraceCtx` natively |

---

## Forge -> Memory Seam Classification

**Classification: compat-only and acceptable this phase.**

The surviving Forge -> memory seam consists of:
1. `forge-memory-bridge::legacy::LegacyImportEnvelopeV1` — one-cycle compat wrapper
2. `semantic-memory::projection_import::ImportEnvelope` — V10 legacy import type
3. `MemoryStore::import_envelope()` — V10 legacy import method

All three are:
- Phase-labeled as "compatibility / migration-only"
- Not presented as normal-path guidance
- Have explicit removal conditions (all callers migrate to bridge pipeline)
- The canonical path (`ExportEnvelopeV1` → `transform_envelope()` → `ProjectionImportBatchV1` → `import_projection_batch()`) is documented as the preferred path

**Removal condition**: All callers migrate to the bridge pipeline. The `import_log` table is retained read-only for audit.

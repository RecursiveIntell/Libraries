# MIGRATION_NOTES_V4.md

> **Purpose:** Migration guide for upgrading to the V4 canonical stack architecture. Covers rollout order, compatibility paths, namespace migration, backfill rules, and deprecations.
>
> **Type:** canonical / normative
> **Last updated:** 2026-03-07

---

## 1. Rollout Order

Crates must be upgraded in dependency order. Upgrading out of order will cause compilation failures due to shared type dependencies.

### Required sequence

```text
1. stack-ids
   (shared ID, scope, trace, and digest primitives -- no upstream deps)

2. forge-memory-bridge
   (depends on stack-ids; defines ExportEnvelopeV1, ProjectionImportBatchV1,
    LegacyImportEnvelopeV1, and the transformation pipeline)

3. semantic-memory (V11)
   (depends on stack-ids; consumes ProjectionImportBatchV1 via non-public
    integration boundary; V10 import_log preserved alongside V11
    projection_import_log)

4. knowledge-runtime
   (depends on stack-ids + semantic-memory; query pipeline, entity registry,
    projection tracker)

5. Supporting crates (in any order after the above):
   - llm-pipeline
   - agent-graph
   - job-queue
   - tauri-queue
   - ai-batch-queue
   - ollama-vision-rs
   - comfyui-rs
```

### Why this order matters

- `stack-ids` defines all shared ID newtypes (`EnvelopeId`, `ClaimId`, `ClaimVersionId`, `ScopeKey`, `TraceCtx`, `ContentDigest`, etc.). Every downstream crate depends on these types.
- `forge-memory-bridge` defines the envelope and batch schemas that `semantic-memory` consumes. It must be available before memory can accept the new import format.
- `semantic-memory` V11 adds the `projection_import_log` table and the non-public integration boundary for `ProjectionImportBatchV1`.
- `knowledge-runtime` depends on both `stack-ids` types and `semantic-memory` adapter capabilities.
- Supporting crates adopt `stack-ids` types (e.g., `TraceCtx` instead of local `TraceId`) but have no ordering dependency on each other.

---

## 2. Compatibility Path

### Old import_envelope() still works

The existing `semantic-memory::projection_import::ImportEnvelope` type and the `import_envelope()` method remain functional. They are marked as **compatibility / migration-only** and will be removed after one migration cycle.

Callers using the old path do not need to change immediately. The old path:
1. Validates the `ImportEnvelope` as before.
2. Checks idempotency via the V10 `import_log` table.
3. Writes records atomically.
4. Returns an `ImportReceipt`.

### LegacyImportEnvelopeV1 converter

For callers that have a legacy-format envelope but want to use the new pipeline, `forge-memory-bridge::legacy` provides:

- `upgrade_legacy_envelope(&LegacyImportEnvelopeV1) -> Result<ExportEnvelopeV1>` -- converts the old format to the canonical export envelope, including:
  - `namespace` mapped to `ScopeKey` via `ScopeKey::from_legacy_namespace()`
  - `trace_id` mapped to `TraceCtx` via `TraceCtx::from_legacy_trace_id()`
  - `Fact` records converted to `ExportRecord::Claim` with `projection_family: "legacy_import"` and `subject_entity_id: "_legacy_unresolved"`
  - `Episode` records converted to `ExportRecord::Episode`
  - Content digest recomputed for the new format

- `transform_legacy_envelope(&LegacyImportEnvelopeV1) -> Result<ProjectionImportBatchV1>` -- convenience function that combines upgrade + transform in one call.

### Timeline

| Phase | What happens |
|-------|-------------|
| Now | Both old and new paths are functional. Old path writes to `import_log`. New path writes to `projection_import_log`. |
| Migration cycle | Callers migrate to `ExportEnvelopeV1` -> bridge -> `ProjectionImportBatchV1`. Legacy path remains available. |
| Post-migration | `ImportEnvelope`, `LegacyImportEnvelopeV1`, and `import_envelope()` are removed. `import_log` table is retained read-only for audit. |

---

## 3. Namespace -> ScopeKey Mapping

### The rule

Legacy code uses `namespace: String` as the partition key. The canonical partition key is now `ScopeKey`.

The deterministic mapping is:

```text
"foo" -> ScopeKey {
    namespace: "foo",
    domain: None,
    workspace_id: None,
    repo_id: None,
}
```

### Canonical functions

All conversion must go through these functions in `stack-ids::scope`:

```rust
// Forward: namespace -> ScopeKey
let sk = ScopeKey::from_legacy_namespace("my-namespace");

// Reverse: ScopeKey -> namespace (valid only for namespace-only scopes)
let ns: &str = sk.to_legacy_namespace();

// Check: is this scope namespace-only?
assert!(sk.is_namespace_only());
```

### Where to use them

- **Bridge**: `upgrade_legacy_envelope()` uses `ScopeKey::from_legacy_namespace()` to convert the legacy `namespace` field.
- **Importer**: When accepting legacy envelopes, convert namespace before routing.
- **Tests**: Use `ScopeKey::from_legacy_namespace()` when constructing test scopes from string namespaces.
- **Runtime**: The `SemanticMemoryAdapter` extracts `sk.to_legacy_namespace()` when passing scope to the upstream `semantic-memory` search API (which still accepts namespace strings).

### What NOT to do

Do not invent ad-hoc namespace-to-scope conversions. The following is forbidden:

```rust
// WRONG: ad-hoc conversion
let sk = ScopeKey { namespace: ns.to_string(), domain: None, workspace_id: None, repo_id: None };

// CORRECT: use the canonical function
let sk = ScopeKey::from_legacy_namespace(ns);
```

Both produce the same result today, but the canonical function is the single point of truth if the mapping ever changes.

---

## 4. Backfill

### Legacy data remains queryable

Per MASTER_SUPPORTING_DELTA section 12.1: older imported `Fact | Episode` data must remain queryable after migration.

Guarantees:

- **V10 `import_log` preserved**: The `import_log` table from V10 is not dropped or altered. It remains readable for audit queries (`query_import_log()`, `last_import_at()`).
- **V11 `projection_import_log` added alongside**: New imports via the bridge pipeline write to the new `projection_import_log` table. The two tables coexist.
- **Document/fact/episode rows untouched**: Migration does not rewrite existing document, fact, or episode rows in `semantic-memory`'s SQLite database. Existing embeddings, HNSW indices, and search results are unaffected.
- **Provenance survives**: Legacy import records retain their original `envelope_id`, `schema_version`, `content_digest`, `source_authority`, `namespace`, and `trace_id` in the `import_log` table.

### What happens to legacy imports during the migration window

- The old `import_envelope()` path continues to write to `import_log` as before.
- The new bridge path writes to `projection_import_log` with richer metadata (scope_key, claim_version_id, relation_version_id, etc.).
- Queries against `semantic-memory` return results from both old and new imports transparently -- the document/fact/episode tables are shared.

### After migration

- `import_log` is retained as a read-only audit table.
- New code uses `projection_import_log` exclusively.
- `import_envelope()` is removed from the public API.

---

## 5. Deprecations

### Forbidden in new code

| Term / Type | Status | Replacement |
|-------------|--------|-------------|
| Bare `ImportEnvelope` | **Forbidden in new code** | `ExportEnvelopeV1` (export side) or `ProjectionImportBatchV1` (import side) |
| `import_envelope()` on `MemoryStore` | **Compatibility only** | Bridge pipeline: `transform_envelope()` -> memory integration boundary |
| `TraceId` (crate-local copy) | **Deprecated** | `stack_ids::TraceCtx` |
| `namespace: String` as partition key | **Legacy compat** | `ScopeKey` via `ScopeKey::from_legacy_namespace()` |

### One migration cycle only

The following exist for backward compatibility during one migration cycle and will be removed afterward:

| Type | Location | Purpose | Removal condition |
|------|----------|---------|-------------------|
| `LegacyImportEnvelopeV1` | `forge-memory-bridge::legacy` | Upgrade old-format envelopes | All callers migrated to `ExportEnvelopeV1` |
| `upgrade_legacy_envelope()` | `forge-memory-bridge::legacy` | Convert legacy to canonical | All callers migrated |
| `transform_legacy_envelope()` | `forge-memory-bridge::legacy` | Convert + transform in one step | All callers migrated |
| `ImportEnvelope` | `semantic-memory::projection_import` | V10 import format | All callers migrated to bridge pipeline |
| `TraceCtx::from_legacy_trace_id()` | `stack-ids::trace` | Convert old `TraceId(String)` to `TraceCtx` | All crates using `TraceCtx` natively |
| `TraceCtx::to_legacy_trace_id()` | `stack-ids::trace` | Extract string for legacy interop | All crates using `TraceCtx` natively |

### Phase labeling

All compatibility code is marked with `## Phase status: compatibility / migration-only` in doc comments. Grep for this label to find all compatibility surfaces:

```bash
grep -r "Phase status: compatibility" --include="*.rs" .
```

---

## 6. Release-Gate Test Obligations for Backfill, Recovery, and Migration

The guarantees in sections 4 and 5 must be backed by explicit release-blocking or clearly tracked test obligations. The following categories are release-blocking.

### Import and consistency tests (release-blocking)

| Test obligation | Status | Notes |
|----------------|--------|-------|
| Out-of-order envelope arrival (older valid-time envelope after newer) | Required | Verify import succeeds and provenance is correct |
| Duplicate-but-not-identical envelopes (same `envelope_id`, different content) | Required | Verify rejection with `ImportDuplicate` or explicit conflict handling |
| Rollback on mid-import failure (partial record set) | Required | Verify atomicity: no partial visibility |
| Late-arriving older valid-time envelope | Required | Verify correct handling without silent data loss |
| Import-lag warning propagation to runtime | Required | Verify `ProjectionImportStale` surfaces in `QueryTrace` |

### Storage and recovery tests (release-blocking)

| Test obligation | Status | Notes |
|----------------|--------|-------|
| WAL crash during projection import | Required | Verify SQLite atomicity on restart |
| HNSW rebuild after projection-storage migration | Required | Verify rebuild from SQLite produces correct index |
| Restart during bridge dual-path migration window | Required | Verify both `import_log` and `projection_import_log` remain consistent |

### Identity and entity tests (release-blocking)

| Test obligation | Status | Notes |
|----------------|--------|-------|
| Alias unmerge after downstream projections exist | Required | Verify projections are invalidated or recomputed |
| Human-confirmed merge reversal via explicit migration/repair flow | Required | Verify `is_human_confirmed_final` protection |
| Competing canonical IDs under new evidence | Required | Verify deterministic winner selection |

### Retry and trace tests (release-blocking)

| Test obligation | Status | Notes |
|----------------|--------|-------|
| Nested retry misconfiguration rejected | Required | Verify that multiple retry owners for one path produce a diagnostic error |
| Same logical attempt with multiple retry owners rejected | Required | Verify violation is caught at configuration time or returns error |
| Replay trace linked but not parented | Required | Verify span links (not parent-child) across replay hops |
| Queue retry storm isolation | Required | Verify each re-enqueue produces separate `AttemptId` with own `TrialId` chain |

### Ranking tests (release-blocking)

| Test obligation | Status | Notes |
|----------------|--------|-------|
| Deterministic ordering under identical semantic base with overlay changes | Required | Verify stable tie-breaking |
| Unsupported causal leg cannot silently outrank supported evidence | Required | Verify warning surfaces if causal mode is ungated |

### Migration law (preserved)

These guarantees remain in force:
- `import_log` preserved as read-only audit table after migration.
- `projection_import_log` coexists with `import_log` during migration window.
- Legacy `Fact | Episode` rows remain queryable after migration.
- One-cycle compatibility path lifetime: compatibility surfaces are removed only when all callers have migrated.
- Read-only retention of `import_log` after migration.
- Explicit removal conditions for compatibility-only surfaces (listed in section 5).

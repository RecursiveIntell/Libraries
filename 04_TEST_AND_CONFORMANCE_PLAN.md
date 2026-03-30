# 04. Test and Conformance Plan

## Gate Hierarchy

Tests are organized into three tiers. All tiers must pass for a conforming finish.

**Tier 1: Cargo gates**
```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Tier 2: Static gates (scripts/)**
All `scripts/check_*.py` and `scripts/check_*.sh` pass against the shipped pack with zero crashes and zero failures.

**Tier 3: Issue-specific acceptance tests**
Each P0 and P1 issue has named acceptance tests listed below.

---

## Phase 1 Acceptance Tests

### GOV-003 — governance feature on by default

**Test:** `cargo check -p forge-pilot`
**Assert:** Compiles with all 7 governance crates. No `--features governance` flag needed.

### GOV-001 — observe_governance() returns real observations

**Test file:** `forge-pilot/tests/governance_observation_tests.rs`

```
Test: governance_observation_reflects_artifacts
  Setup: Create a MemoryStore. Insert a governance artifact (e.g., an IncidentCase
         with continuity_incident_active = true) into semantic-memory.
  Call:  observe_governance(&memory_store, &knowledge_runtime)
  Assert:
    - result.continuity_incident_active == true
    - result != GovernanceObservation::default()

Test: governance_observation_empty_when_no_artifacts
  Setup: Create an empty MemoryStore.
  Call:  observe_governance(&memory_store, &knowledge_runtime)
  Assert:
    - result == GovernanceObservation::default()
    - Function does not error (fail-open)

Test: governance_observation_detects_authority_delegation
  Setup: Insert an authority delegation artifact with valid chain.
  Call:  observe_governance(...)
  Assert:
    - result.authority_delegation_valid == true
```

### GOV-002 — governance receipt populated and gates execution

**Test file:** `forge-pilot/tests/governance_gating_tests.rs`

```
Test: governance_blocks_execution_on_active_incident
  Setup: LoopRunner with governance artifacts indicating active incident.
  Call:  runner.run()
  Assert:
    - Loop iteration report has governance_receipt != None
    - governance_receipt.gate_result == Blocked
    - No action was executed in that iteration

Test: governance_advisory_on_pending_amendment
  Setup: Governance artifacts with constitutional_amendment_pending = true.
  Call:  runner.run()
  Assert:
    - governance_receipt.gate_result == AdvisoryOnly
    - advisory_only_steps == true in loop totals

Test: governance_allows_when_clean
  Setup: Governance artifacts with all-clear state.
  Call:  runner.run()
  Assert:
    - governance_receipt.gate_result == Allow
    - Normal execution proceeds
```

### ID-001 — no invented episode IDs

**Test file:** `forge-memory-bridge/tests/episode_identity_regression.rs`

```
Test: legacy_import_without_episode_id_returns_error
  Setup: Construct a legacy import record with episode_id = None.
  Call:  transform function that previously had unwrap_or_else
  Assert:
    - Returns Err(BridgeError::MissingEpisodeIdentity { .. })
    - No EpisodeId was generated

Test: canonical_import_with_episode_id_succeeds
  Setup: Construct import record with valid episode_id.
  Call:  transform function
  Assert:
    - Returns Ok(...)
    - episode_id in output matches input exactly

Test: export_import_roundtrip_identity_stable
  Setup: Create episode with known ID. Export. Bridge transform. Import.
  Assert:
    - Episode ID after roundtrip == original episode ID
    - No UUID was generated during the pipeline
```

### TMP-001 — temporal provenance carries coordinates

**Test file:** `knowledge-runtime/tests/temporal_provenance_tests.rs` (or extend `cross_crate_proof.rs`)

```
Test: temporal_query_provenance_includes_coordinates
  Setup: KnowledgeRuntime with test data.
  Call:  query_temporal_with_trace(query, scope, None,
           valid_at=Some("2025-06-15T00:00:00Z"),
           recorded_at_or_before=Some("2025-07-01T00:00:00Z"))
  Assert:
    - provenance.valid_as_of == Some("2025-06-15T00:00:00Z")
    - provenance.recorded_as_of == Some("2025-07-01T00:00:00Z")
    - provenance.temporal_mode is set (not empty)

Test: temporal_query_provenance_shows_downgrade
  Setup: Query with temporal parameters that force a downgrade/fallback.
  Assert:
    - provenance.temporal_mode == "downgraded" or "fallback"
    - Warning also present in provenance.warnings

Test: non_temporal_query_provenance_has_no_coordinates
  Setup: Regular query (no temporal parameters).
  Assert:
    - provenance.valid_as_of == None
    - provenance.recorded_as_of == None
    - provenance.temporal_mode == "none" or equivalent
```

### SAFE-001 — no production panics

**Test:** Run `scripts/check_no_prod_panics.sh`
**Assert:** Zero non-test hits. Exit code 0.

**Verification:** `grep -n 'expect\|unwrap()' semantic-memory/src/embedder.rs` returns only lines inside `#[cfg(test)]` blocks.

---

## Phase 2 Acceptance Tests

### GATE-001 — no public type shadows

**Test:** Run `scripts/check_public_type_drift.py`
**Assert:** Exit code 0. No `ValidationResult` duplicates.

**Verification:** `grep -rn "pub type ValidationResult" --include="*.rs" | grep -v target` returns zero results. Each governance crate has a crate-specific name.

### DOC-001 — doc coverage green

**Test:** Run `scripts/check_public_api_docs.py`
**Assert:** All doc-certified crates at 100%. Governance surface decision table exists.

### DOC-002 — doc-truth gate green

**Test:** Run `scripts/check_doc_truth.sh`
**Assert:** Exit code 0.

### SURF-001 — repo-surface gate green

**Test:** Run `scripts/check_repo_surface.sh`
**Assert:** Exit code 0. AGENTS.md, docs/README.md, and release receipt exist.

### CHECK-001 — no checker crashes

**Test:** Run every `scripts/check_*.py` and `scripts/check_*.sh` against the shipped pack.
**Assert:** No Python tracebacks. No shell errors. Every failure is a policy-level message.

### TEST-001 — llm-tool-runtime test coverage

**Test:** `cargo test -p llm-tool-runtime`
**Assert:** At least 20 tests pass. Coverage includes dispatch, receipts, error classes, provider fallback.

### OPS-001, OPS-002, PACK-001

**Test:** Run respective gate scripts.
**Assert:** Exit code 0.

### SURF-002 — lane manifest is authoritative

**Test:** Verify `scripts/lane_manifest.json` exists and is consumed by at least:
- `print_supported_lane.py`
- `check_public_type_drift.py`
- `check_public_api_docs.py`
- `check_no_prod_panics.sh`

**Assert:** No hardcoded crate list in any gate script that diverges from the manifest.

---

## Phase 3 Acceptance Tests

### TYPE-001 — timestamp validation

```
Test: effect_window_builder_rejects_invalid_timestamp
  Call: EffectWindowV1::builder(..., earliest_start="not-a-date", ...).build()
  Assert: Returns Err(...)

Test: effect_window_builder_accepts_valid_iso8601
  Call: EffectWindowV1::builder(..., earliest_start="2025-06-15T00:00:00Z", ...).build()
  Assert: Returns Ok(...)
```

### TYPE-002 — typed repair/verification artifacts

```
Test: repair_record_rejects_invalid_disposition
  Assert: Schema rejects free-text where typed enum is now required.

Test: verification_plan_typed_blockers
  Assert: VerificationPlanArtifact blocker field uses typed enum, not String.
```

### TYPE-003 — views use typed enums

```
Test: effect_runtime_view_uses_typed_reversibility
  Assert: EffectRuntimeViewV1.reversibility_class is a typed enum, not String.
  Assert: Serializes to same wire format as before.
```

### EXEC-001 — execution context backpointers

```
Test: execution_context_roundtrip_preserves_lineage
  Setup: Create ToolReceipt with rich lineage.
  Convert to ExecutionContextV1.
  Assert: All critical fields either inlined or backpointed and reachable.
```

### TEST-002 — property tests

**Test:** `cargo test -p effect-runtime` and `cargo test -p assurance-runtime`
**Assert:** Proptest passes for builder → validate pipeline.

---

## Final Gate Checklist

Run this sequence. Every line must pass.

```bash
# Tier 1
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# Tier 2 (representative, run all check_* scripts)
python3 scripts/check_public_type_drift.py
python3 scripts/check_public_api_docs.py
python3 scripts/check_commit_permit_paths.py
bash scripts/check_no_prod_panics.sh
bash scripts/check_repo_surface.sh
bash scripts/check_doc_truth.sh
bash scripts/check_pack_truth.sh
bash scripts/check_mirror_discipline.sh
bash scripts/check_hotspot_budgets.sh

# Tier 3 (issue-specific — verified by cargo test)
cargo test -p forge-pilot -- governance
cargo test -p forge-memory-bridge -- episode_identity
cargo test -p knowledge-runtime -- temporal_provenance
cargo test -p llm-tool-runtime
cargo test -p effect-runtime -- proptest
cargo test -p assurance-runtime -- proptest
```

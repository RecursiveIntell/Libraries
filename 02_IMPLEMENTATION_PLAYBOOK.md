# 02. Implementation Playbook

## Timeline

**Today:** March 30, 2026
**Deadline:** April 10, 2026
**Working days available:** ~9
**Estimated total effort:** 5-7 focused days with AI agents

## Execution Phases

### Phase 1 — Constitutional Core (Days 1-3)

Stop the constitutional defects. These are the issues that make the system's governance claims fraudulent.

**Slice A: Governance Loop Integration**
Issues: GOV-003 → GOV-001 → GOV-002

Execution order:
1. **GOV-003** — Change `default = ["governance"]` in `forge-pilot/Cargo.toml`. Run `cargo check -p forge-pilot` to verify all governance crates compile. (15 minutes)
2. **GOV-001** — Rewrite `observe_governance()` to accept `&MemoryStore` and `&KnowledgeRuntime`, query for governance views, populate `GovernanceObservation` from results. Update call site in `observe.rs` to pass the store/runtime. (2-4 hours)
3. **GOV-002** — In `loop_runner.rs`, after observation, call `gate_execution()` on the governance observation. Call `build_governance_receipt()`. Populate `governance_receipt` field. Wire `Blocked`/`AdvisoryOnly` results to loop control flow. (2-3 hours)

**Gate:** `cargo test -p forge-pilot --features governance` passes. Governance observation returns non-default values when governance artifacts exist.

**Slice B: Identity + Temporal Provenance**
Issues: ID-001, TMP-001

Execution order:
1. **ID-001** — In `forge-memory-bridge/src/transform.rs`, replace both `unwrap_or_else(EpisodeId::generate)` calls with error returns. Add `MissingEpisodeIdentity` variant to `BridgeError`. Add regression test. (1-2 hours)
2. **TMP-001** — Add `valid_as_of: Option<String>`, `recorded_as_of: Option<String>`, `temporal_mode: String` to `RuntimeQueryProvenanceV1`. Populate in `runtime_query_provenance()` from query parameters. Add cross-crate proof tests. (2-3 hours)

**Gate:** Bridge regression test proves no invented episode IDs. Temporal provenance carries coordinates.

**Slice C: Panic Policy**
Issues: SAFE-001

1. **SAFE-001** — In `semantic-memory/src/embedder.rs`, mark `pub fn new()` as `#[cfg(test)]` or remove it. All production callers use `try_new()`. (30 minutes)

**Gate:** `check_no_prod_panics.sh` passes with zero non-test hits.

### Phase 2 — Gate Honesty (Days 3-5)

Make every gate tell the truth.

**Slice C: Gate + Doc + Surface Repairs**
Issues: GOV-004, GATE-001, DOC-001, DOC-002, SURF-001, CHECK-001, TEST-001, OPS-001, OPS-002, PACK-001, SURF-002

Execution order:
1. **SURF-002** — Create `scripts/lane_manifest.json` as the single source of truth for lane membership. (1 hour)
2. **GOV-004** — Add `cargo check --workspace` to `scripts/release_gate_set.py`. Align all release-facing docs to describe the same proof surface. All scripts consume `lane_manifest.json`. (2 hours)
3. **GATE-001** — Rename `ValidationResult` in all 7 governance crate error modules to crate-specific names (`EffectValidationResult`, etc.). Update all internal call sites. Update allowlist. (1-2 hours)
4. **DOC-001** — Add `///` doc comments to the ~5 missing public functions across the 5 crates. Create governance surface decision table. (1-2 hours)
5. **DOC-002** — Update `check_doc_truth.sh` to use current pack's narrative, or generate required strings from manifests. (1 hour)
6. **SURF-001** — Create `AGENTS.md`, `release/closeout_receipt_v1.json`, `docs/README.md`. Update `check_repo_surface.sh` to scope to shipped pack. (1-2 hours)
7. **CHECK-001** — Make `check_commit_permit_paths.py` existence-aware. Create `scripts/manifest/scope_manifest.json`. (1 hour)
8. **OPS-001** — Restore sync script or remove from gate. (30 minutes)
9. **OPS-002** — Create `docs/module_budget_exceptions.md` or narrow gate scope. (30 minutes)
10. **PACK-001** — Update `check_pack_truth.sh` to match actual artifact set. (30 minutes)
11. **TEST-001** — Add 15-20 tests to `llm-tool-runtime`. (2-3 hours)

**Gate:** All `scripts/check_*.py` and `scripts/check_*.sh` pass. `cargo test --workspace` passes.

### Phase 3 — Artifact Quality (Days 5-7)

Improve the artifacts themselves.

**Slice D: Type Safety + Evidence + Testing**
Issues: EXEC-001, TYPE-001, TYPE-002, TYPE-003, SEM-001, TEST-002, TEST-003, CONC-001, API-001, API-002

Execution order (priority within slice):
1. **TYPE-001** — Add timestamp validation to `EffectWindowV1` builder (parse with `chrono`). Extend to other critical timestamps. (2-3 hours)
2. **TYPE-002** — Promote `RepairRecordV1` and `VerificationPlanArtifact` string fields to typed enums. (2-3 hours)
3. **TYPE-003** — Replace String fields in `knowledge-runtime/src/views.rs` with typed enums from owner crates. (1-2 hours)
4. **EXEC-001** — Add backpointer fields to `ExecutionContextV1` or enrich it. Round-trip tests. (2-3 hours)
5. **TEST-002** — Add proptest coverage for effect-runtime and assurance-runtime builders. (1-2 hours)
6. **SEM-001** — Add semantic oracle golden fixtures to kernel-conformance. (2-3 hours)
7. **CONC-001** — Wrap HNSW operations in `spawn_blocking`. (1-2 hours)
8. **TEST-003** — Add criterion benchmarks for semantic-memory search. (1-2 hours)
9. **API-001** — Refactor EffectIntentV1Builder to method-chaining. (1-2 hours)
10. **API-002** — Evaluate serde(flatten) migration. If breaking, document and defer. (1 hour)

**Gate:** `cargo test --workspace` passes. Schema tests reject invalid values. Proptest builders pass.

## Dependency Graph

```
GOV-003 (enable feature)
  └─> GOV-001 (wire observe_governance)
        └─> GOV-002 (wire governance receipt + gating)

ID-001 (standalone)
TMP-001 (standalone)
SAFE-001 (standalone)

GOV-004 ──> SURF-002 (lane manifest first)
GATE-001 (standalone)
DOC-001 (standalone)
DOC-002 (standalone)
SURF-001 (standalone)
CHECK-001 (standalone)
TEST-001 (standalone)

Phase 3 issues are all standalone
```

## Rules

1. **Do not change semantics by "fixing the docs."** If the gate and the code disagree, fix the code.
2. **Do not weaken gates to get green.** If a gate is red, make the underlying surface pass, not the gate easier.
3. **Do not invent compatibility IDs and call them canonical.** (ID-001)
4. **Do not flatten rich receipts without backpointers.** (EXEC-001)
5. **Prefer generated manifests over copied crate/path lists.** (SURF-002)
6. **Every fix must land with a proof artifact:** test, fixture, or gate update.
7. **Run `cargo test --workspace` after every slice.** Do not batch untested changes.
8. **Commit messages must reference issue IDs.** E.g., `fix(GOV-001): wire observe_governance to semantic-memory projections`

## Success Criteria

The pack is done when:
- [ ] `cargo check --workspace` green
- [ ] `cargo test --workspace` green
- [ ] `cargo clippy --workspace -- -D warnings` green
- [ ] All `scripts/check_*.py` and `scripts/check_*.sh` green against the shipped pack
- [ ] `observe_governance()` returns non-default values when governance artifacts exist
- [ ] `governance_receipt` populated in loop iteration reports
- [ ] No bridge path invents episode IDs
- [ ] Temporal provenance carries coordinates
- [ ] No `expect()`/`unwrap()` in supported-lane non-test code
- [ ] Zero public type shadows across governance crates

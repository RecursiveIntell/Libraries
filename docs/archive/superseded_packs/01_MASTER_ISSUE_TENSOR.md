# 01. Master Issue Tensor

27 issues. 7 critical. 11 high. 9 medium.
Sources: Claude 10-angle audit (C), GPT hostile audit (G), or both (C+G).

## Priority Key

- **P0 (critical):** Blocks CLARA submission or constitutes a constitutional-trust defect
- **P1 (high):** Breaks a gate, weakens an evidence claim, or creates a credibility gap
- **P2 (medium):** Reduces quality, maintainability, or future extensibility

## Quick Reference

| ID | P | Category | Title | Source |
|---|---|---|---|---|
| GOV-001 | P0 | governance/loop | observe_governance() is a stub returning Default::default() | C+G |
| GOV-002 | P0 | governance/loop | governance_receipt hardcoded to None in loop iterations | C |
| GOV-003 | P0 | governance/feature | governance feature off by default — CLARA requires it | C |
| ID-001 | P0 | identity/bridge | Legacy bridge import can invent episode IDs | G |
| TMP-001 | P0 | temporal/provenance | Runtime temporal provenance omits actual temporal coordinates | G |
| GOV-004 | P0 | governance/gates | Release gate and support-profile claims out of sync | G |
| SAFE-001 | P0 | safety/panic | Supported-lane panic audit fails on embedder.rs expect() | G |
| GATE-001 | P1 | gate/contracts | Public-type-drift gate red: ValidationResult duplicated x7 | C+G |
| DOC-001 | P1 | documentation | Public API doc-truth gate is red on 5 crates | G |
| EXEC-001 | P1 | execution/evidence | Canonical execution context thinner than runtime receipts | G |
| TYPE-001 | P1 | type-safety | 132 timestamp fields typed as String, not DateTime | C |
| TYPE-002 | P1 | type-safety | Governance artifacts stringly typed in repair/verification | C+G |
| TYPE-003 | P1 | type-safety | knowledge-runtime views re-serialize typed enums to String | C |
| TEST-001 | P1 | testing | llm-tool-runtime: 5 tests for 4K lines of code | C |
| SURF-001 | P1 | repo-surface | Repo-surface truth gate red: missing front-door artifacts | G |
| DOC-002 | P1 | documentation | Doc-truth gate misaligned with active docs | G |
| CHECK-001 | P1 | gate/tooling | Commit-permit checker crashes on excluded paths | G |
| SURF-002 | P2 | governance/surface | Support-lane and doc-certified lane not generated from one source | G |
| OPS-001 | P2 | operations | Mirror-discipline gate missing sync script | G |
| OPS-002 | P2 | operations | Hotspot-budget gate references missing files | G |
| PACK-001 | P2 | packaging | Pack-truth gate expects CSV not present | G |
| SEM-001 | P2 | semantic | Reference-interpreter obligation not obvious in code surface | G |
| TEST-002 | P2 | testing | Zero property-based tests on governance artifact builders | C |
| TEST-003 | P2 | testing | Zero benchmarks anywhere in workspace | C |
| CONC-001 | P2 | concurrency | std::sync::RwLock on HNSW index used in async context | C |
| API-001 | P2 | api-design | 15-arg positional builders defeat builder pattern purpose | C |
| API-002 | P2 | api-design | Double serde(flatten) on effect artifacts creates fragile schemas | C |

---

## P0 — Critical Issues

### GOV-001 — observe_governance() is a stub returning Default::default()

**Priority:** P0
**Source:** Claude audit (Angles 1, 5), GPT audit (Lens 1)
**Category:** governance/loop

**Summary:** `forge-pilot/src/governance_gate.rs` defines `observe_governance()` which returns `GovernanceObservation::default()` — all booleans false, all Options None. The function never reads governance artifacts from semantic-memory projections. The architecture is correctly wired but operationally vacant.

**Evidence:**
- `forge-pilot/src/governance_gate.rs:97-103` — function body is `GovernanceObservation::default()`
- `forge-pilot/src/observe.rs:352-353` — observation pipeline calls `observe_governance()` under `#[cfg(feature = "governance")]`
- Profile-runtime adapters (`profile-runtime/src/adapters.rs`, ~600 lines) already project every governance profile type into `ObligationContributionV1` — the composition pipeline exists but the loop never invokes it

**Impact:** DARPA CLARA evaluators will see zero governance trace in loop output. The system cannot demonstrate that governance artifacts influence execution decisions.

**Fix:** Wire `observe_governance()` to query governance artifacts from `semantic-memory` projections. The function should:
1. Query the knowledge-runtime for governance views (`EffectRuntimeViewV1`, `DelegationRuntimeViewV1`, `DeployabilityRuntimeViewV1`, `ContinuityRuntimeViewV1`)
2. Populate `GovernanceObservation` fields from the query results
3. Accept a `&KnowledgeRuntime` or `&MemoryStore` parameter (currently takes no arguments)

**Acceptance criteria:**
- `observe_governance()` accepts a store/runtime reference and queries actual governance state
- When governance artifacts are present in semantic-memory, the returned `GovernanceObservation` reflects them
- When no governance artifacts exist, the function still returns the default (fail-open behavior preserved)
- A test proves that inserting a governance artifact changes the observation output

**Touch set:**
- `forge-pilot/src/governance_gate.rs`
- `forge-pilot/src/observe.rs`
- `forge-pilot/src/loop_runner.rs`
- `forge-pilot/tests/` (new governance observation test)

---

### GOV-002 — governance_receipt hardcoded to None in loop iterations

**Priority:** P0
**Source:** Claude audit (Angle 5)
**Category:** governance/loop

**Summary:** `LoopIterationReport` has a `governance_receipt: Option<GovernanceReceiptV1>` field gated behind `#[cfg(feature = "governance")]`. In both places where `LoopIterationReport` is constructed in `loop_runner.rs` (lines 817-818 and 860-861), it is set to `governance_receipt: None`.

**Evidence:**
- `forge-pilot/src/loop_runner.rs:817-818` — `governance_receipt: None`
- `forge-pilot/src/loop_runner.rs:860-861` — `governance_receipt: None`
- `forge-pilot/src/governance_gate.rs:130-149` — `build_governance_receipt()` exists but is never called from the loop

**Impact:** Even with the governance feature enabled, loop output carries no governance evidence. `build_governance_receipt()` is dead code.

**Fix:** After calling `observe_governance()`, call `gate_execution()` on the result, then `build_governance_receipt()`. Store the receipt in the iteration report. If `gate_execution()` returns `Blocked`, halt execution for that iteration. If `AdvisoryOnly`, set `advisory_only = true`.

**Acceptance criteria:**
- Loop iterations with governance enabled populate `governance_receipt` with actual observation data
- `gate_execution()` return value influences loop behavior (Blocked → skip, AdvisoryOnly → advisory mode)
- `build_governance_receipt()` is called in the loop and its output is stored in the report
- Tests prove governance gating: a blocked observation prevents execution, an advisory observation sets advisory mode

**Touch set:**
- `forge-pilot/src/loop_runner.rs`
- `forge-pilot/src/governance_gate.rs`
- `forge-pilot/tests/` (new governance gating tests)

---

### GOV-003 — governance feature off by default

**Priority:** P0
**Source:** Claude audit (Angle 5)
**Category:** governance/feature

**Summary:** `forge-pilot/Cargo.toml` declares `default = []`. The `governance` feature that brings in the 7 governance crates is not in the default feature set. A plain `cargo build` or `cargo test` does not compile or test the governance integration.

**Evidence:**
- `forge-pilot/Cargo.toml:2-3` — `[features] default = []`

**Impact:** CI and default builds silently skip all governance code. A CLARA evaluator running `cargo test` will see zero governance tests execute.

**Fix:** Add `governance` to the default features. This ensures all governance code compiles and tests by default.

**Acceptance criteria:**
- `default = ["governance"]` in `forge-pilot/Cargo.toml`
- `cargo test -p forge-pilot` exercises governance observation, gating, and receipt generation
- `cargo check --workspace` compiles all governance crates

**Touch set:**
- `forge-pilot/Cargo.toml`

---

### ID-001 — Legacy bridge import can invent episode IDs

**Priority:** P0
**Source:** GPT audit (Lens 2), confirmed by Claude
**Category:** identity/bridge

**Summary:** Canonical bundle-bearing export already rejects missing `episode_id`, but legacy import paths still synthesize episode IDs with `unwrap_or_else(EpisodeId::generate)`.

**Evidence:**
- `forge-memory-bridge/src/transform.rs:499` — `let episode_id = ep.episode_id.clone().unwrap_or_else(EpisodeId::generate);`
- `forge-memory-bridge/src/transform.rs:644` — `episode_id: ep.episode_id.clone().unwrap_or_else(EpisodeId::generate),`

**Impact:** Creates identity collapse, split lineage, and replay mismatch. A generated ID is indistinguishable from a canonical one, so the system cannot tell whether an episode's identity was assigned by its authority or invented by the bridge.

**Fix:** Replace `unwrap_or_else(EpisodeId::generate)` with a typed compatibility/quarantine path. Legacy records with missing episode identity should produce a `BridgeError::MissingEpisodeIdentity` or be tagged with a `LegacyImportDisposition::QuarantinedIdentity` marker.

**Acceptance criteria:**
- No bridge import path generates `EpisodeId` for canonical projection records
- Legacy records with missing episode identity produce typed compatibility/import-failure artifacts
- Replay tests prove stable identity across export → bridge → import
- Regression test: importing a legacy bundle without `episode_id` does not silently invent one

**Touch set:**
- `forge-memory-bridge/src/transform.rs` (2 call sites)
- `forge-memory-bridge/src/legacy.rs`
- `forge-memory-bridge/src/error.rs` (new error variant)
- `forge-memory-bridge/tests/` (new regression test)

---

### TMP-001 — Runtime temporal provenance omits actual temporal coordinates

**Priority:** P0
**Source:** GPT audit (Lens 3), confirmed by Claude
**Category:** temporal/provenance

**Summary:** `query_temporal_with_trace` accepts `valid_at` and `recorded_at_or_before` parameters, but `RuntimeQueryProvenanceV1` does not record those coordinates. The provenance artifact cannot testify which temporal slice was answered.

**Evidence:**
- `knowledge-runtime/src/runtime/core.rs:361` — `pub async fn query_temporal_with_trace(&self, query, scope, trace_ctx, valid_at, recorded_at_or_before)`
- `knowledge-runtime/src/obs/trace.rs:238-265` — `RuntimeQueryProvenanceV1` struct has no `valid_as_of` or `recorded_as_of` fields

**Impact:** Temporal queries can run with explicit bitemporal semantics while the emitted provenance artifact cannot fully testify which temporal slice was answered. This undermines the stack's claim that execution is evidence.

**Fix:** Add to `RuntimeQueryProvenanceV1`:
- `valid_as_of: Option<String>` (ISO 8601)
- `recorded_as_of: Option<String>` (ISO 8601)
- `temporal_mode: String` (exact / widened / downgraded / fallback)

Populate these fields in `runtime_query_provenance()` from the query parameters and any downgrade/fallback that occurred.

**Acceptance criteria:**
- Provenance includes the exact temporal coordinates used
- Temporal downgrade/fallback is visible in canonical provenance, not only in warnings
- Cross-crate proof tests assert provenance fidelity for temporal queries

**Touch set:**
- `knowledge-runtime/src/obs/trace.rs`
- `knowledge-runtime/src/runtime/core.rs`
- `knowledge-runtime/tests/cross_crate_proof.rs`
- `contract-schema-gen/src/lib.rs`

---

### GOV-004 — Release gate and support-profile claims out of sync

**Priority:** P0
**Source:** GPT audit (Lens 1)
**Category:** governance/gates

**Summary:** SUPPORT_PROFILE.md claims governance crates are build-checked by `cargo check --workspace`, but the release-gate command set does not include a workspace-wide cargo check and the release-lane target only runs fmt/clippy/tests over the supported lane.

**Evidence:**
- `SUPPORT_PROFILE.md` — claims build-checked status for governance crates
- `scripts/release_gate_set.py` — does not include `cargo check --workspace`
- `Makefile` — release target scopes to supported lane only

**Impact:** A green gate can be interpreted as stronger evidence than it actually is. This is a constitutional-trust bug in a repo that markets truthful artifacts and governed release claims.

**Fix:** Add explicit `cargo check --workspace` to the gate path. Align SUPPORT_PROFILE.md, README, PACK_README, and RELEASE_CHECKLIST to describe the same proof surface. Generate the release-proof vocabulary from one manifest.

**Acceptance criteria:**
- Workspace-wide cargo check exists in the gate path
- All release-facing docs describe the same proof surface
- A single generated manifest owns the release-proof vocabulary

**Touch set:**
- `SUPPORT_PROFILE.md`
- `scripts/release_gate_set.py`
- `Makefile`
- `README.md`
- `PACK_README.md`
- `RELEASE_CHECKLIST.md`

---

### SAFE-001 — Supported-lane panic audit fails on embedder.rs expect()

**Priority:** P0
**Source:** GPT audit (Lens 8), confirmed by Claude
**Category:** safety/panic

**Summary:** `semantic-memory/src/embedder.rs:84` has a deprecated `new()` constructor that calls `try_new(...).expect("Failed to build reqwest client")`. This is a production panic path in the supported lane.

**Evidence:**
- `semantic-memory/src/embedder.rs:84` — `Self::try_new(config).expect("Failed to build reqwest client")`

**Impact:** Violates the stated no-unwrap-in-production-code policy. Breaks the panic audit gate.

**Fix:** Mark the deprecated constructor `#[cfg(test)]` only, or remove it entirely and update all callers to use `try_new()`.

**Acceptance criteria:**
- Supported-lane panic audit passes with zero non-test hits
- No deprecated API with `expect()` remains in release-facing code paths

**Touch set:**
- `semantic-memory/src/embedder.rs`
- `scripts/check_no_prod_panics.sh`
- `scripts/prod_panic_allowlist.json`

---

## P1 — High Issues

### GATE-001 — Public-type-drift gate red: ValidationResult duplicated x7

**Priority:** P1
**Source:** Claude audit (Angle 3), GPT audit (Lens 7)
**Category:** gate/contracts

**Summary:** Seven crates each define `pub type ValidationResult = Result<(), XxxValidationError>`. These are distinct types that shadow each other in the public API surface, causing the public-type-drift checker to fail.

**Evidence:**
- `effect-runtime/src/error.rs:28`, `constitutional-memory/src/error.rs:21`, `mechanism-runtime/src/error.rs:21`, `assurance-runtime/src/error.rs:21`, `attestation-exchange/src/error.rs:21`, `authority-delegation/src/error.rs:21`, `continuity-runtime/src/error.rs:27`

**Fix:** Rename each to a crate-specific name: `EffectValidationResult`, `AssuranceValidationResult`, etc. Or create a shared `GovernanceValidationResult<E>` generic type in `stack-ids`.

**Acceptance criteria:**
- `check_public_type_drift.py` passes with no semantic duplicates
- Gate crate list sourced from one manifest

**Touch set:**
- `*/src/error.rs` for all 7 governance crates
- `scripts/check_public_type_drift.py`
- `scripts/public_type_drift_allowlist.json`

---

### DOC-001 — Public API doc-truth gate is red on 5 crates

**Priority:** P1
**Source:** GPT audit (Lens 6)
**Category:** documentation

**Summary:** Doc-coverage gate fails: forge-pilot 62/63, profile-runtime 43/44, effect-runtime 36/37, verification-policy 49/50, semantic-memory-forge 34/35. Also expects a governance-surface decision table that is absent.

**Fix:** Add `///` doc comments to the ~5 missing public functions. Create or restore governance surface decision table.

**Acceptance criteria:**
- All doc-certified crates pass 100% function-doc coverage
- Governance surface decision table exists and includes every expected crate

**Touch set:**
- `forge-pilot/src/error.rs`
- `profile-runtime/src/compose.rs`
- `effect-runtime/src/error.rs`
- `verification-policy/src/permit.rs`
- `semantic-memory-forge/src/envelope.rs`
- `docs/closeout_v21_v24/governance_surface_decision_table.md`

---

### EXEC-001 — Canonical execution context thinner than runtime receipts

**Priority:** P1
**Source:** GPT audit (Lens 4)
**Category:** execution/evidence

**Summary:** `ExecutionContextV1` captures only a subset of execution lineage while `ToolReceipt` carries richer semantics: family linkage, replay parents, budget context, retry ownership, provider call IDs.

**Fix:** Either enrich `ExecutionContextV1` with backpointer fields or require strong backpointers from thin context into richer receipt artifacts.

**Acceptance criteria:**
- No canonical execution artifact silently drops retry/family/replay/provider lineage
- Execution-context schema documents what is inlined vs backpointed
- Round-trip tests verify lineage reachability

**Touch set:**
- `semantic-memory-forge/src/v9.rs`
- `llm-tool-runtime/src/contracts.rs`
- `forge-pilot/src/receipts.rs`

---

### TYPE-001 — 132 timestamp fields typed as String, not DateTime

**Priority:** P1
**Source:** Claude audit (Angle 3)
**Category:** type-safety

**Summary:** 132 date/time fields across the workspace are typed as `String` (e.g., `expires_at: String`, `generated_at: String`, `triggered_at: String`) while only 99 uses of `chrono::DateTime` exist. Validators only check `!field.is_empty()`, not temporal validity.

**Impact:** No compile-time guarantee of valid timestamps. Comparison/ordering impossible without parse-at-every-callsite. A governance system that can't tell if a certificate is expired because the date field might contain "banana" is not credible.

**Fix:** For the highest-value governance crate timestamps (effect windows, certifications, incident timelines), change `String` to `chrono::DateTime<Utc>`. For fields that must remain wire-compatible, add `parse_timestamp()` validation in the builder `.build()` method.

**Acceptance criteria:**
- Critical timestamp fields (effect windows, certification expiry, incident timelines) use `DateTime<Utc>` or have validated parsing in builders
- Builder `.build()` methods reject unparseable timestamps
- At minimum, effect-runtime `EffectWindowV1` timestamps are typed or validated

**Touch set:**
- `effect-runtime/src/effect.rs`
- `assurance-runtime/src/assurance.rs`
- `assurance-runtime/src/certification.rs`
- `continuity-runtime/src/incident.rs`
- `continuity-runtime/src/slo.rs`

---

### TYPE-002 — Governance artifacts stringly typed in repair/verification surfaces

**Priority:** P1
**Source:** Claude audit (Angle 3), GPT audit (TYPE-001)
**Category:** type-safety

**Summary:** `RepairRecordV1`, `VerificationPlanArtifact`, and related types in `forge-pilot/src/types.rs` use broad `String` and `Vec<String>` for blast radius, reversibility, action class, evidence classes, and blocker descriptions.

**Fix:** Promote core semantics to enums from existing typed vocabulary: `ReversibilityClassV1` from effect-runtime, `BlastRadiusCeilingV1` from effect-runtime, etc.

**Acceptance criteria:**
- `RepairRecordV1` uses typed enums for disposition semantics
- `VerificationPlanArtifact` distinguishes typed blockers from human notes
- Schema tests reject invalid control-plane values

**Touch set:**
- `forge-pilot/src/types.rs`
- `forge-pilot/src/receipts.rs`
- `contract-schema-gen/src/lib.rs`

---

### TYPE-003 — knowledge-runtime views re-serialize typed enums to String

**Priority:** P1
**Source:** Claude audit (Angle 3)
**Category:** type-safety

**Summary:** `EffectRuntimeViewV1` has `reversibility_class: String`, `run_mode: String`, `publication_status: String`. These same concepts already have proper enums in `effect-runtime` (`ReversibilityClassV1`, `RunModeV1`, `PublicationStatusV1`). The view layer re-serializes typed enums back into stringly-typed fields.

**Fix:** Import and use the typed enums from their owner crates in the view structs.

**Acceptance criteria:**
- View structs use typed enums, not Strings, for fields that have defined enum types
- Views remain `Serialize + Deserialize` compatible (enum serde matches existing wire format)

**Touch set:**
- `knowledge-runtime/src/views.rs`
- `knowledge-runtime/Cargo.toml` (add deps if needed)

---

### TEST-001 — llm-tool-runtime: 5 tests for 4K lines

**Priority:** P1
**Source:** Claude audit (Angle 4)
**Category:** testing

**Summary:** `llm-tool-runtime` defines the tool dispatch contract for the entire LLM interaction surface (3,966 lines). It has only 5 tests total: 2 in-module and 3 in test files.

**Fix:** Add tests for: tool registration/deregistration, dispatch routing, receipt generation, error classification, retry ownership semantics, provider fallback.

**Acceptance criteria:**
- At least 20 tests covering the core dispatch and receipt paths
- `ToolReceipt` round-trip serialization tested
- `ToolErrorClass` classification tested for each variant
- Provider fallback path tested

**Touch set:**
- `llm-tool-runtime/tests/core_tests.rs` (expand)
- `llm-tool-runtime/tests/` (new test files)

---

### SURF-001 — Repo-surface truth gate red: missing front-door artifacts

**Priority:** P1
**Source:** GPT audit
**Category:** repo-surface

**Summary:** `check_repo_surface.sh` expects release receipts, AGENTS.md, docs/README.md, archive manifests, and multiple runbooks that are absent from the archive.

**Fix:** Either restore missing files or narrow the repo-surface gate to the shipped pack scope.

**Acceptance criteria:**
- Repo-surface gate passes against the shipped pack
- Front-door references are present and non-stale

**Touch set:**
- `scripts/check_repo_surface.sh`
- `AGENTS.md` (create)
- `release/closeout_receipt_v1.json` (create)
- `docs/README.md` (create)

---

### DOC-002 — Doc-truth gate misaligned with active docs

**Priority:** P1
**Source:** GPT audit
**Category:** documentation

**Summary:** `check_doc_truth.sh` expects specific reconciliation text and command phrasing that the current pack no longer preserves.

**Fix:** Choose which is authoritative (current docs or old checker). Generate required strings from one source.

**Acceptance criteria:**
- Doc-truth gate passes
- Required narrative snippets are either generated or retired from the checker

**Touch set:**
- `scripts/check_doc_truth.sh`
- `MASTER_ISSUE_MATRIX.md`
- `PACK_README.md`
- `CONFORMANCE_GATES.md`
- `RELEASE_CHECKLIST.md`

---

### CHECK-001 — Commit-permit checker crashes on excluded paths

**Priority:** P1
**Source:** GPT audit
**Category:** gate/tooling

**Summary:** `check_commit_permit_paths.py` reads `LLM-Pipeline/src/tool_loop.rs`, but `LLM-Pipeline` is excluded from the workspace and absent from the archive, causing `FileNotFoundError`.

**Fix:** Make path checks existence-aware. If a target path is optional/excluded, skip with explicit reason or fail with policy message. Never throw raw traceback.

**Acceptance criteria:**
- Checker never crashes on missing optional/excluded paths
- Every checked path sourced from a manifest
- Failure mode is deterministic and policy-semantic

**Touch set:**
- `scripts/check_commit_permit_paths.py`
- `scripts/manifest/` (create scope manifest)

---

## P2 — Medium Issues

### SURF-002 — Support-lane and doc-certified lane not generated from one source

**Priority:** P2
**Source:** GPT audit
**Category:** governance/surface

**Summary:** SUPPORT_PROFILE.md, public-doc checks, and gate scripts each carry overlapping but not identical views of the active release-facing surface.

**Fix:** Create one `lane_manifest.json` consumed by all scripts and docs.

**Touch set:**
- `scripts/lane_manifest.json` (create)
- `SUPPORT_PROFILE.md`
- `scripts/print_supported_lane.py`
- `scripts/check_public_api_docs.py`
- `scripts/check_no_prod_panics.sh`
- `scripts/check_public_type_drift.py`

---

### OPS-001 — Mirror-discipline gate missing sync script

**Priority:** P2
**Source:** GPT audit

**Fix:** Restore `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh` or remove from gate.

**Touch set:**
- `scripts/check_mirror_discipline.sh`
- `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh`

---

### OPS-002 — Hotspot-budget gate references missing files

**Priority:** P2
**Source:** GPT audit

**Fix:** Manifest-drive the checker or constrain to shipped pack scope.

**Touch set:**
- `scripts/check_hotspot_budgets.sh`
- `docs/module_budget_exceptions.md`

---

### PACK-001 — Pack-truth gate expects CSV not present

**Priority:** P2
**Source:** GPT audit

**Fix:** Emit CSV as part of pack generation or remove from hard requirement list.

**Touch set:**
- `scripts/check_pack_truth.sh`
- `PACK_MANIFEST.json`

---

### SEM-001 — Reference-interpreter obligation not obvious in code surface

**Priority:** P2
**Source:** GPT audit

**Fix:** Create small, explicit semantic-oracle module with golden fixtures for highest-risk seams.

**Touch set:**
- `kernel-conformance/` (add semantic oracle fixtures)
- `knowledge-runtime/` (add differential tests)

---

### TEST-002 — Zero property-based tests on governance artifact builders

**Priority:** P2
**Source:** Claude audit (Angle 4)

**Fix:** Add `proptest!` coverage for builder → validate pipeline in at least effect-runtime and assurance-runtime.

**Touch set:**
- `effect-runtime/tests/` (new proptest file)
- `assurance-runtime/tests/` (new proptest file)
- `effect-runtime/Cargo.toml` (add proptest dev-dep)

---

### TEST-003 — Zero benchmarks anywhere in workspace

**Priority:** P2
**Source:** Claude audit (Angle 4)

**Fix:** Add `criterion` benchmarks for semantic-memory search and knowledge-runtime query pipeline.

**Touch set:**
- `semantic-memory/benches/` (create)
- `knowledge-runtime/benches/` (create)
- `Cargo.toml` (add criterion workspace dep)

---

### CONC-001 — std::sync::RwLock on HNSW index in async context

**Priority:** P2
**Source:** Claude audit (Angle 7)

**Summary:** `semantic-memory` uses `std::sync::RwLock<HnswIndex>` in an async context. If future code holds HNSW read locks across `.await` points, the OS thread blocks and the tokio runtime starves.

**Fix:** Wrap HNSW operations in `spawn_blocking` or migrate to `tokio::sync::RwLock`. Given current single-threaded usage, `spawn_blocking` wrapper is the safer minimal change.

**Touch set:**
- `semantic-memory/src/lib.rs` (HNSW access points)
- `semantic-memory/src/hnsw.rs`

---

### API-001 — 15-arg positional builders defeat builder pattern purpose

**Priority:** P2
**Source:** Claude audit (Angle 6)

**Fix:** Refactor builders to use method chaining (`.with_effect_class(...)`) instead of positional `::new()` constructors. Start with `EffectIntentV1Builder` (15 args).

**Touch set:**
- `effect-runtime/src/effect.rs`
- All call sites constructing these builders

---

### API-002 — Double serde(flatten) on effect artifacts creates fragile schemas

**Priority:** P2
**Source:** Claude audit (Angle 6)

**Summary:** `EffectPreflightReportV1` and `EffectCommitDecisionV1` each have two `#[serde(flatten)]` attributes, which produces non-obvious JSON shapes and makes OpenAPI/JSON Schema generation unreliable.

**Fix:** Nest the flattened fields into named sub-objects (`citation: V25ConstitutionCitation` and `obligation_refs: V25ObligationRefs` become regular fields instead of flattened).

**Touch set:**
- `effect-runtime/src/effect.rs`
- `effect-runtime/src/v25.rs`
- Wire-format migration considerations (breaking change — may defer)

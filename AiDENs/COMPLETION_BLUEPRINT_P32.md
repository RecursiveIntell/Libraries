# AiDENs Complete Completion Blueprint

**Date:** 2026-05-29
**From:** P31B (candidate) → P32 (schema compatibility + wiring)
**Based on:** Hostile audit results, sibling crate research, P32 audit plan, failing gate analysis

---

## Part 1: Gate Fixes (Must-Happen-First)

These 9 failing gates block certification. Fix in this order because some are prerequisites for others.

### 1.1 SHADOW_SEMANTICS_AUDIT.md (blocks cargo test, C-01)

**Fix:** Copy from archive to repo root
```bash
cp docs/root-markdown-archive/P31A_archive/SHADOW_SEMANTICS_AUDIT.md SHADOW_SEMANTICS_AUDIT.md
```
**Complexity:** TRIVIAL — one file copy

### 1.2 STATUS.md crate inventory (blocks no_scaffold_promoted, C-02)

**Fix:** Add a crate inventory table to STATUS.md. The gate script `assert_no_scaffold_promoted.sh` greps for `| \`crate_name\` | status |` format.

Add this table with statuses:
- `implemented` for most crates (they compile and have tests)
- `partial` for `aidens-delegation-kit` (0 tests), `boundary-compiler-core` (new, narrow scope)
- `scaffold-only` for `aidens-profile-daemon`, `aidens-profile-desktop`, `aidens-profile-memory`, `aidens-profile-research`
- Also remove or rephrase the `scaffold/` directory reference in README.md line 48

**Complexity:** SIMPLE — markdown table addition

### 1.3 P32 audit plan classification (blocks codex_artifact_classification, C-03)

**Fix:** Add two entries to `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`:
```json
{"path": "docs/codex-runs/P32_AUDIT_PLAN.md", "classification": "durable-plan", "run": "P32", "active": true},
{"path": "HOSTILE_AUDIT_P32_2026-05-29.md", "classification": "run-evidence", "run": "P32", "active": true}
```
**Complexity:** TRIVIAL — two JSON entries

### 1.4 phase_injections directory (blocks phase_gate_integrity)

**Fix:** Create `phase_injections/` directory with 6 P26 gate files. Copy from `docs/root-markdown-archive/P31A_archive/phase_injections/` where available. Create `P26_GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md` with required terms ("stop", "blocking human-in-the-loop gate", "do not proceed").

**Complexity:** SIMPLE — create directory + 6 small markdown files

### 1.5 P29 matrices and manifests (blocks phase19_high_risk_quarantine + super_pass_docs_evidence_closure)

**Fix:**
- Copy `docs/root-markdown-archive/P31A_archive/matrices/P29_MASTER_ISSUE_MATRIX.csv` → `matrices/P29_MASTER_ISSUE_MATRIX.csv`
- Copy/create `P29_STATUS_EVIDENCE_MANIFEST.json` at root (from archive if exists, otherwise create with `audit_bug_status_classification` and `high_risk_layer_quarantine` objects)
- Create `06_CLAUDE_AUDIT_INTEGRATION.md` with CLAUDE-F-015 (fixed) and CLAUDE-F-016 (quarantined) rows
- Create `matrices/SUPER_PASS_BACKLOG_1020.csv` (or copy from archive)

**Complexity:** SIMPLE — file copies from archive

### 1.6 Package generation (blocks package_self_replay + package_validation)

**Fix:** Run `python3 z.py --root . --profile aidens --mode codex-context --strict` to generate the P31B/P32 package. Then re-run validation.

Note: The `extracted_replay_certified=false` environmental blocker (PermissionError in temp dir) may still prevent full replay certification, but the package dir will exist.

**Complexity:** MODERATE — operational step, may encounter permission issues

### 1.7 Duplicate receipt ID (blocks check_examples)

**Fix:** Investigate `crates/aidens-cli/target/aidens-receipts/aidens-next-mock/canonical-receipts.ndjson` — the ID `agency-policy-report:f5cf6aa277bfbdec` appears twice. This is either:
- A deterministic hash collision in receipt ID generation (same input → same hash → two entries with same ID)
- A test fixture issue where the same agency policy report is written twice

Check `crates/aidens-agency-kit/src/lib.rs` for the `agency-policy-report` receipt ID generation. If deterministic, add a sequence counter or timestamp to disambiguate.

**Complexity:** MODERATE — requires source code investigation and fix

---

## Part 2: Repository Hygiene (Must-Happen-First-Adjacent)

### 2.1 Clean audit debris from repo root

~35 MB of stale z.py artifacts (10 zip/manifest/findings/report/excluded sets), old archives (`AiDENs 4/26.zip`, `libraries-source-clean-*.zip`, etc.), finish pack zips. All untracked.

**Fix:**
1. Add to `.gitignore`:
   ```
   AiDENs-aidens-*.zip
   AiDENs-aidens-*.manifest.json
   AiDENs-aidens-*.findings.json
   AiDENs-aidens-*.excluded.json
   AiDENs-aidens-*.report.md
   AiDENs-aidens-*.codex-archive.json
   libraries-source-clean-*.zip
   aidens_hostile_audit_finish_pack.zip
   aidens_p31b_hermes_finish_pack.zip
   target/
   ```
2. Remove the old zip archives (`AiDENs 4/26.zip`, `aidens.zip`, etc.) or move to `docs/archives/`
3. Keep only the latest z.py package set

### 2.2 Commit all P31B changes, create clean branch

**Fix:**
```bash
git add -A
git checkout -b p32-schema-compatibility
git commit -m "P31B: verify all 18 gates pass, fix remaining gate blockers"
```
The current branch name `p31a-recovery` is misleading since content is P31B-verified.

### 2.3 Add other missing gitignore entries

- `HOSTILE_AUDIT_P32_2026-05-29.md` (or classify it properly in artifact JSON)
- `commands_run.log` at root
- `handoffs/` directory (empty or transient)
- `docs/source-packages/archive/` build artifacts

---

## Part 3: AGENTS.md Doctrine Compliance (Ongoing Quality)

### 3.1 Production unwrap→Result migration (H-01, 287 unwraps)

Priority order by count:
1. **aidens-tool-kit** (103): These are in runtime/control/tool paths. Many are `serde_json::Value` access patterns that should use typed access with proper error variants.
2. **aidens-queue-kit** (57): Queue operations that should propagate errors rather than panicking.
3. **aidens-receipts** (39): Receipt material paths — unwraps here can silently destroy evidence. Highest doctrine concern.
4. **aidens-provider-kit** (24): Provider routing should degrade gracefully, not panic.
5. **aidens-daemon-kit** (18): Daemon lifecycle must not panic.

**Approach:** Systematic per-crate pass. Replace `.unwrap()` with `?` or `.ok_or(ErrorKind::...)?`. Add specific error variants to existing error types.

### 3.2 serde_json::Value in production paths (H-02, 28 files)

**Legitimate uses (keep):**
- `boundary-compiler-core` (3 uses) — boundary input parsing domain, already produces strict receipts
- `aidens-cli` (114+ uses) — CLI display/formatting, not a typed boundary
- `aidens-testkit` (81 uses) — test fixtures
- `aidens-contracts/schema_catalog.rs` (18 uses) — schema generation machinery

**Doctrine-violating uses (fix):**
- `aidens-receipts/src/lib.rs` (18 uses) — receipt construction must use typed structs
- `aidens-runner/src/lib.rs` + `provider_tool.rs` (19 uses) — runtime tool dispatch must use typed paths
- `aidens-tool-kit/src/lib.rs` (7 uses) + `canonical_stack.rs` — tool operations
- `aidens-provider-kit/src/lib.rs` (9 uses) — provider routing

**Approach:** Create typed boundary structs for receipt and tool-dispatch paths first, then migrate uses. The 49 `.unwrap_or_default()` calls in `aidens-agency-kit` are the most dangerous pattern (erasing parse failures).

### 3.3 Monolithic files (H-03)

Split in priority order:
1. **aidens-cli/src/lib.rs** (4,996 lines) → Extract `agent.rs`, `package.rs`, `doctor.rs`, `profile.rs`, `support.rs`, `receipt_display.rs` as proper modules
2. **aidens-tool-kit/src/lib.rs** (3,396 lines) → Extract `canonical_stack.rs`, `tool_registry.rs`, `permit_gate.rs` as proper modules

### 3.4 Duplicate receipt ID (H-01 variant)

Investigate `agency-policy-report:f5cf6aa277bfbdec` duplication. If deterministic, the receipt ID generation needs either:
- A sequence counter component
- Inclusion of a timestamp or unique context element
- Better yet, use `DigestBuilder` from stack-ids for material ID generation per AGENTS.md rule 11

---

## Part 4: P32 Implementation (Schema Compatibility + Reference Fixtures + Wired Path)

### 4.1 ConformanceRunReceiptV1 type

**Where:** `crates/aidens-contracts/src/schema_catalog.rs` or new `crates/aidens-contracts/src/conformance.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceRunReceiptV1 {
    pub run_id: String,
    pub fixture_count: u32,
    pub fixture_results: Vec<ConformanceFixtureResultV1>,
    pub timestamp: DateTime<Utc>,
    pub environment: ConformanceEnvironmentV1,
    pub overall_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceFixtureResultV1 {
    pub fixture_path: String,
    pub outcome: ConformanceOutcomeV1,
    pub digest: String,
    pub degradation_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ConformanceOutcomeV1 { Pass, Fail, Degraded { reasons: Vec<String> }, Skipped }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceEnvironmentV1 {
    pub rust_version: String,
    pub target_triple: String,
    pub timestamp: DateTime<Utc>,
}
```

Register in `ArtifactFamilyRegistryV1` as `conformance-run-receipt`.

### 4.2 Reference fixture corpus for boundary compiler

**Where:** `crates/boundary-compiler-core/tests/fixtures/` or `crates/aidens-testkit/src/fixtures/`

Create JSON fixture files with input + expected output pairs:
- Happy path: valid boundary JSON → clean acceptance
- Duplicate keys: JSON with duplicate keys → rejection with `DuplicateKeyFindingV1`
- Unknown fields: JSON with unknown fields → quarantine with `TreatmentIntegrityReceiptV1`
- Coercion attempt: type coercion → rejection
- Resource ceiling breach: oversized JSON → `ResourceCeilingsV1` rejection
- Schema validation: valid/invalid schemas
- Version skew: future-version boundary input

Each fixture must be self-contained and registered in `GoldenFixtureManifestV1`.

### 4.3 Wire boundary-compiler-core into aidens-boundary-kit

**Current state:** `boundary-compiler-core` is standalone (no workspace crate depends on it). `aidens-boundary-kit` has its own `compile_json_boundary()` that doesn't use it.

**Approach:**
1. Add `boundary-compiler-core` as a dependency of `aidens-boundary-kit` in `Cargo.toml`
2. In `aidens-boundary-kit/src/lib.rs`, delegate strict-parse path to `boundary_compiler_core::compile_json_boundary()`
3. Map `boundary_compiler_core` types to `aidens_contracts` types (adapter pattern)
4. Integration test: tool dispatch path through boundary compiler → verify `BoundaryCompileReceiptV1` appears in receipt chain

### 4.4 Schema generation for boundary-compiler-core types

1. Add `schemars` dependency to `boundary-compiler-core/Cargo.toml`
2. Add `#[derive(JsonSchema)]` to all public types
3. Create a schema generation test that produces `GeneratedSchemaManifestV1` for all boundary-compiler types
4. Validate schemas are self-consistent (no circular `$ref`)

### 4.5 Type alias replacement (DigestHex, JsonPointerLikePath)

Replace local aliases in `boundary-compiler-core`:
- `DigestHex` → use `stack_ids::ContentDigest` or `DisplayDigestV1` from `aidens-contracts`
- `JsonPointerLikePath` → use proper pointer type from `stack-ids`

This adds `stack-ids` or `aidens-contracts` as a dependency of `boundary-compiler-core`. Currently it's standalone with only `sha2`, `serde`, `serde_json`.

---

## Part 5: Sibling Crate Wiring (Ready-to-Consume Capabilities)

These are capabilities that exist in Libraries siblings but are NOT wired into AiDENs. Each is a candidate for future integration but NOT blocking for P32 certification.

### 5.1 High-Impact, Low-Risk (Next Sprint)

| Capability | Source Crate | What It Enables | Wiring Path |
|---|---|---|---|
| Knowledge-runtime view types | knowledge-runtime | Constitutional/policy state surfacing | `aidens-memory-kit` → add `CanonicalKnowledgeAdapter` method |
| Verification-control governance cases | verification-control | Effect review, delegation review, release gate flows | `aidens-governance-kit` → add case creation methods |
| Assurance runtime release readiness | assurance-runtime | Deployment gate decisions | `aidens-governance-kit` → add `ReleaseReadinessDecisionV1` |
| Memory integrity verification | semantic-memory | Receipt-bearing integrity checks | `aidens-memory-kit` → wire `verify_integrity()` |
| Search receipt replay | semantic-memory | Deterministic search verification | `aidens-memory-kit` → wire `replay_search_receipt()` |

### 5.2 Medium-Impact (Future Sprints)

| Capability | Source Crate | What It Enables |
|---|---|---|
| Graph traversal | semantic-memory | Multi-hop knowledge retrieval |
| Inference advisory/risk gate | knowledge-runtime | Advisory governance signals |
| Rollback/rollout decisions | verification-adjudication | Deployment decision flows |
| Boundary-compiler JCS | boundary-compiler | Canonical JSON for digests |
| Authority-delegation deeper types | authority-delegation | Break-glass, dual control |

### 5.3 Foundational (Long-Term)

| Capability | Source Crate | What It Enables |
|---|---|---|
| Vector codecs (TurboQuant, q8) | semantic-memory | Quantized embedding search |
| Mechanism/theory artifacts | mechanism-runtime | Theory versioning |
| Incident/pager operational IDs | stack-ids | Operational escalation |
| Verification-policy execution permits | verification-policy | Commit/permit tokens |
| Federated settlement | federated-settlement | Multi-party dispute resolution |

---

## Part 6: Execution Order

### Phase 0 — Gate Fixes (1-2 hours)
1. Copy SHADOW_SEMANTICS_AUDIT.md to root → fixes cargo test
2. Add crate inventory table to STATUS.md → fixes no_scaffold_promoted
3. Add P32 files to CODEX_ARTIFACT_CLASSIFICATION.json → fixes codex_artifact_classification
4. Create phase_injections/ with 6 gate files → fixes phase_gate_integrity
5. Restore P29 matrices/manifests from archive → fixes phase19 + super_pass_docs
6. Fix README.md scaffold line

### Phase 1 — Tree Hygiene (30 min)
7. `.gitignore` audit debris patterns
8. Clean root of stale zips/manifests
9. Commit all P31B changes on `p32-schema-compatibility` branch
10. Re-run `verify_current.sh` — should have 27+ PASS now

### Phase 2 — Package + Receipt Fix (2-4 hours)
11. Run z.py to generate P32 package → fixes package_validation + package_self_replay (if env permits)
12. Investigate + fix duplicate receipt ID → fixes check_examples

### Phase 3 — P32 Core (2-3 days)
13. Create ConformanceRunReceiptV1 in aidens-contracts
14. Create boundary compiler fixture corpus
15. Wire boundary-compiler-core into aidens-boundary-kit
16. Add schemars to boundary-compiler-core, generate schemas
17. Replace DigestHex/JsonPointerLikePath aliases with canonical types

### Phase 4 — Doctrine Compliance (1-2 weeks, parallel)
18. Begin unwrap→Result migration (top 3 crates: aidens-tool-kit, aidens-queue-kit, aidens-receipts)
19. Audit serde_json::Value usage — classify and fix doctrine violations
20. Split aidens-cli/lib.rs into modules
21. Add tests to aidens-delegation-kit

### Phase 5 — Certification
22. Re-run all 36+ verification gates
23. Run cargo check/test/clippy on clean tree
24. Update CURRENT_RUN.json to P32 candidate
25. Cascade certification status through all protected docs
26. Generate final package and verify replay
27. Produce P32 final hostile audit handoff report

---

## Verification Checklist

- [ ] `SHADOW_SEMANTICS_AUDIT.md` exists at repo root
- [ ] STATUS.md contains crate inventory table with all 34 crates
- [ ] CODEX_ARTIFACT_CLASSIFICATION.json includes P32 entries
- [ ] phase_injections/ directory has 6 gate files with required terms
- [ ] P29 matrices exist at expected paths
- [ ] README.md scaffold line removed/rephrased
- [ ] `.gitignore` covers audit debris
- [ ] Root is clean of stale z.py artifacts
- [ ] All P31B changes committed on clean branch
- [ ] `cargo test --workspace --locked` passes (0 failures)
- [ ] `verify_current.sh` passes all gates
- [ ] ConformanceRunReceiptV1 type defined and registered
- [ ] Boundary compiler fixture corpus covers happy-path + edge cases
- [ ] boundary-compiler-core wired into aidens-boundary-kit
- [ ] Schema generation works for boundary-compiler types
- [ ] Conformance run emits ConformanceRunReceiptV1 with evidence
- [ ] One real structured I/O path wired through boundary compiler
- [ ] 287 production unwraps reduced below 100 (or annotated with justification)
- [ ] aidens-cli/lib.rs below 3000 lines
- [ ] aidens-tool-kit/lib.rs below 2500 lines
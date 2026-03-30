# Master Issue Matrix — V27 Governance Integration Pack

**Generated:** 2026-03-25
**Source:** Hostile audit of libraries-source-clean-20260325.zip

Severity: P0 = blocks CLARA submission, P1 = degrades credibility under audit, P2 = technical debt

---

## P0 — Submission blockers

### GOV-001: Zero governance crate integration into forge-pilot OODA loop

**Severity:** P0
**Crates:** forge-pilot, all governance crates
**Evidence:** `grep -r "assurance_runtime\|effect_runtime\|authority_delegation\|continuity_runtime\|mechanism_runtime\|constitutional_memory\|attestation_exchange" forge-pilot/src` returns 0 hits.
**Problem:** The 7 governance schema crates define typed artifact families (effects, assurance cases, delegation chains, continuity reviews, attestation exchanges, constitutional amendments, mechanism fitness). None of these are consumed, produced, or checked anywhere in the forge-pilot loop. The OODA loop operates entirely through verification-control, verification-policy, and kernel-* without governance surface awareness.
**Required fix:** Wire governance artifact checks into the forge-pilot observe/act/receipts path. At minimum, the loop must be able to: (1) check whether an effect intent's preflight passes before acting, (2) evaluate assurance-case readiness before promotion, (3) respect authority-delegation capability gates, (4) surface continuity-runtime incident state in observation disposition.
**Owner:** forge-pilot/src/observe.rs, forge-pilot/src/act.rs, forge-pilot/src/receipts.rs, forge-pilot/src/loop_runner.rs
**Test:** kernel-conformance integration test that runs the loop with governance constraints active and verifies governance artifacts appear in the loop report.

### GOV-002: Missing constraint-compiler and profile-runtime from workspace

**Severity:** P0
**Evidence:** `ls constraint-compiler/src` and `ls profile-runtime/src` both fail. Cargo.toml still lists them as members. forge-pilot, kernel-execution, kernel-oracles, kernel-conformance, knowledge-runtime all depend on constraint-compiler. contract-schema-gen depends on profile-runtime.
**Problem:** Two of the most substantive governance crates were dropped from the zip. constraint-compiler contains the hypergraph compiler with real graph algorithms. profile-runtime contains the fold-class composition engine. Without these, the workspace cannot build.
**Required fix:** Restore both crates from the working repository. Verify `cargo check --workspace` passes.
**Owner:** workspace root
**Test:** `cargo check --workspace` succeeds.

### GOV-003: SAM.gov registration incomplete

**Severity:** P0
**Evidence:** User-reported.
**Problem:** Cannot receive federal funding or submit to DARPA CLARA without valid UEI and active SAM registration. Registration can take 2-4 weeks.
**Required fix:** Complete SAM.gov registration before April 10, 2026.
**Owner:** Josh (non-code task)
**Test:** Active SAM registration with valid UEI.

---

## P1 — Credibility issues under audit

### ERR-001: 200+ unwrap() calls in production code across certified lane

**Severity:** P1
**Crates:** semantic-memory-forge (59), kernel-conformance (38), knowledge-runtime (29), stack-ids (18), semantic-memory (17), llm-tool-runtime (10), forge-memory-bridge (7), contract-schema-gen (6), kernel-oracles (4), recursive-kernel-core (2)
**Problem:** A governance runtime claiming determinism guarantees should not panic on recoverable errors. An auditor evaluating for regulated deployment will flag this immediately.
**Hotspot files:**
- `semantic-memory-forge/src/v11.rs` — 59 unwraps
- `kernel-conformance/src/lib.rs` — 29 unwraps
- `knowledge-runtime/src/entity/registry.rs` — 18 unwraps
- `semantic-memory/src/projection_derivation.rs` — 14 unwraps
- `stack-ids/src/trace.rs` — 10 unwraps
- `stack-ids/src/digest.rs` — 8 unwraps
**Required fix:** Replace unwrap() with proper Result propagation in all production (non-test, non-example) code. Test and example code may retain unwrap().
**Owner:** Per-crate owners
**Test:** `bash scripts/check_no_prod_panics.sh` passes (script must be created or updated to enforce zero unwrap/expect in src/ excluding test modules).

### ERR-002: Governance crates use &'static str errors instead of typed errors

**Severity:** P1
**Crates:** assurance-runtime, attestation-exchange, authority-delegation, constitutional-memory, mechanism-runtime
**Evidence:** All validate() methods return `Result<(), &'static str>`. Only effect-runtime and continuity-runtime have typed error enums.
**Problem:** Untyped string errors cannot be matched programmatically. A consumer calling `validate()` cannot distinguish between "missing required field" and "invalid state transition" without string parsing.
**Required fix:** Each governance crate should define a typed validation error enum following effect-runtime's `EffectRuntimeValidationError` pattern. Migrate all `Result<(), &'static str>` returns to the typed error.
**Owner:** Per-crate src/error.rs (new file for most)
**Test:** Each crate compiles with the new error type and existing tests pass.

### TST-001: llm-tool-runtime has only 3 tests

**Severity:** P1
**Crate:** llm-tool-runtime
**Evidence:** `grep -r "#\[test\]" llm-tool-runtime --include="*.rs" | wc -l` = 3
**Problem:** llm-tool-runtime's `ToolReceipt` and `ToolRetryOwner` types flow directly into verification-control and are consumed by the entire verification chain. 3 tests for 3,159 lines of code that feeds the core verification pipeline is a critical coverage gap.
**Required fix:** Add tests for: ToolReceipt construction and serialization, ToolRetryOwner lifecycle, registry lookup, provider dispatch, starter tool execution, and error propagation.
**Minimum:** 15 additional tests covering the public API surface.
**Owner:** llm-tool-runtime/tests/
**Test:** `cargo test -p llm-tool-runtime` runs ≥18 tests.

### TST-002: verification-calibration has only 2 tests

**Severity:** P1
**Crate:** verification-calibration (120 lines, 2 tests)
**Problem:** CalibrationSnapshot can force advisory-only mode in the loop. Two tests for a component that gates all execution decisions is insufficient.
**Required fix:** Add tests for: comparability version drift detection, calibration caveat propagation, abstention threshold enforcement, forces_advisory_only flag behavior.
**Minimum:** 8 additional tests.
**Owner:** verification-calibration/tests/
**Test:** `cargo test -p verification-calibration` runs ≥10 tests.

### TST-003: recursive-kernel-core has only 3 tests

**Severity:** P1
**Crate:** recursive-kernel-core (256 lines, 3 tests)
**Problem:** Defines the operator contract model used by kernel-execution and kernel-oracles. OperatorMetadata::validate() is the entry gate for the entire kernel pipeline.
**Required fix:** Add tests for: all OperatorContract field combinations, KernelRun authority_class, Syndrome serialization, ResidualArtifact and WitnessArtifact roundtrips, CertificateArtifact with/without oracle slice, KernelRefutationResult all outcome variants.
**Minimum:** 10 additional tests.
**Owner:** recursive-kernel-core/src/lib.rs (inline tests) or tests/
**Test:** `cargo test -p recursive-kernel-core` runs ≥13 tests.

### DOC-001: Governance crates lack integration documentation

**Severity:** P1
**Crates:** All 7 governance crates
**Problem:** No governance crate documents how its artifacts connect to the forge-pilot loop, the verification pipeline, or the benchmark. The crate-level doc comments accurately describe what each crate publishes, but a reviewer cannot determine how they compose.
**Required fix:** Each governance crate's lib.rs doc comment must include a "Integration Points" section specifying: (1) which forge-pilot phase consumes the artifacts, (2) which verification-control case types are affected, (3) which Stack Arena scenario classes exercise the artifacts.
**Owner:** Per-crate src/lib.rs
**Test:** `python3 scripts/check_public_api_docs.py` passes for all governance crates.

---

## P2 — Technical debt

### DEBT-001: forge-pilot loop_runner.rs is 987 lines

**Severity:** P2
**File:** forge-pilot/src/loop_runner.rs
**Problem:** The core OODA loop is a single 987-line method. Adding governance integration (GOV-001) will make it larger. This file has been flagged across multiple review cycles.
**Required fix:** Extract governance-specific logic into a separate `governance_gate.rs` module rather than inlining it into the existing loop body.
**Owner:** forge-pilot/src/governance_gate.rs (new)
**Test:** Existing loop tests continue to pass after extraction.

### DEBT-002: expect() calls in verification-policy and verification-adjudication

**Severity:** P2
**Crates:** verification-policy (7 expect calls), verification-adjudication (2 expect calls)
**Problem:** These are in the critical verification path that gates execution decisions.
**Required fix:** Replace with proper Result propagation.
**Owner:** verification-policy/src/lib.rs, verification-adjudication/src/lib.rs
**Test:** Existing tests pass after replacement.

### DEBT-003: kernel-conformance uses 38 unwraps + 13 expects

**Severity:** P2
**Crate:** kernel-conformance
**Problem:** Integration test code that an auditor will read. Even in test harness code, excessive panicking makes the codebase look unfinished.
**Required fix:** Replace at least the 29 unwraps in lib.rs with result propagation or explicit test assertions (assert!/assert_eq! are acceptable).
**Owner:** kernel-conformance/src/lib.rs
**Test:** `cargo test -p kernel-conformance` still passes.

### DEBT-004: contract-schema-gen should include governance crate schemas

**Severity:** P2
**Crate:** contract-schema-gen
**Problem:** If governance crate artifacts are supposed to be part of the public schema contract, contract-schema-gen must generate their JSON schemas. Currently it depends on profile-runtime and discovery-portfolio (both missing from zip) but it's unclear if governance crate schemas are included.
**Required fix:** Verify contract-schema-gen generates schemas for all governance artifact families. Add any missing ones.
**Owner:** contract-schema-gen/src/lib.rs
**Test:** `cargo run -p contract-schema-gen -- schemas.generated` produces schemas for all artifact families including governance.

### DEBT-005: Stack Arena stack_governed lane marked unsupported for 5/8 scenarios

**Severity:** P2
**Evidence:** Stack Arena report shows stack_governed as "unsupported" for temporal_supersession, contradiction_visibility, widening_disclosure, execution_contamination, replay_drift_honesty.
**Problem:** The governed lane only exercises 3 scenarios (verification_plan_yield, repair_record_yield, paired_patch_attribution, policy_frontier). The remaining 5 universal scenarios should eventually have governed-lane counterparts that demonstrate governance artifact production.
**Required fix:** Add stack_governed lane implementations for at minimum temporal_supersession and contradiction_visibility, since these are the scenarios where governance constraints are most relevant.
**Owner:** benchmark/stack-arena scenarios
**Test:** Stack Arena run completes with ≥5 stack_governed passing scenarios.

---

## Summary

| Priority | Count | Category |
|----------|-------|----------|
| P0 | 3 | Submission blockers |
| P1 | 6 | Credibility under audit |
| P2 | 5 | Technical debt |
| **Total** | **14** | |

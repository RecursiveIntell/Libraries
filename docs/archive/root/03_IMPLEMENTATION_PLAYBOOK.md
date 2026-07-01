# Implementation Playbook — V29

## Execution model

Work in phase order. Within each phase, issues may be parallelized unless a dependency is noted. Commit after each issue. Run `cargo check --workspace` after every issue. Run `cargo test --workspace` after completing each phase.

## Phase 1 — Submission Blockers (estimate: 1–2 hours with AI assist)

### Dependency graph
```
TRUTH-001 ──→ DOC-002 (README rewrite depends on knowing the canonical snapshot)
GATE-001 (independent)
```

### TRUTH-001: Unify snapshot dates
1. Decide canonical snapshot name: `libraries-source-clean-20260330.zip`
2. Rewrite `README.md` — see DOC-002 (combined fix)
3. Rewrite `SOURCE_BASIS.md` — reference canonical snapshot
4. Rewrite `STATUS_DASHBOARD.md` — update active lane to `2026-03-30`
5. Rewrite `PACK_MANIFEST.json` — update `generated_at`, `repo_snapshot`, `pack_name`
6. Commit: `fix(TRUTH-001): unify snapshot references to 20260330`

### GATE-001: Fix permit type regex
1. Open `scripts/check_commit_permit_paths.py`
2. Replace `ExecutionPermit` with `ToolExecutionPermit` in the first regex
3. Verify the `execute_plan` signature check in act.rs uses `ExecutionPermit` (from verification-policy) — this is correct as-is
4. Run: `python3 scripts/check_commit_permit_paths.py`
5. Commit: `fix(GATE-001): update permit type regex to ToolExecutionPermit`

### DOC-002: Rewrite README.md
1. Replace entire README.md with a project-level description:
   - What RecursiveIntell is
   - What the stack does (OODA governance orchestrator)
   - Crate architecture (3-tier: Tier 1 constraint-compiler/kernel-oracles, Tier 2 semantic-memory/forge-engine/knowledge-runtime, Tier 3 stack-ids/llm-tool-runtime/profile-runtime)
   - Build instructions (`cargo build --workspace`, `cargo test --workspace`, `make gate`)
   - Link to canonical spec
2. Commit: `fix(DOC-002): rewrite README as project description`

## Phase 2 — Credibility Risks (estimate: 3–5 hours with AI assist)

### Dependency graph
```
TRUTH-002 (independent — archive cleanup)
TRUTH-003 (independent — generate missing artifact)
GATE-002 (independent — fix budget script)
WIRE-001 (independent — bulk serde annotation)
DOC-001 (independent — doc comment pass, longest single task)
```

### TRUTH-002: Archive superseded docs
1. Create `docs/archive/superseded_packs/`
2. Move all numbered-prefix docs (01_ through 17_) except current pack docs to archive
3. Move `CANONICAL_STACK_SPEC_V6.md`, `CANONICAL_STACK_SPEC_V7*.md` to archive (keep only V25 and V26)
4. Move superseded `HOSTILE_AUDIT_REPORT.md`, `IMPLEMENTATION_PLAYBOOK.md`, etc. to archive
5. Create `docs/archive/SUPERSESSION_INDEX.md` listing every archived file and its supersession date
6. Commit: `fix(TRUTH-002): archive 60+ superseded control documents`

### TRUTH-003: Generate archive manifest
1. Create `docs/archive/root_closeout_history/manifest.json` from current state
2. Or: update `STATUS_EVIDENCE_MANIFEST.json` to remove the stale reference
3. Run: `python3 scripts/check_root_archive_manifest.py`
4. Commit: `fix(TRUTH-003): restore archive manifest artifact`

### GATE-002: Fix hotspot budget duplicates
1. Open `scripts/check_hotspot_budgets.sh`
2. Remove duplicate entries — each file gets ONE budget
3. Create/update `docs/module_budget_exceptions.md` with justifications for files above 1000 lines
4. Run: `bash scripts/check_hotspot_budgets.sh`
5. Commit: `fix(GATE-002): deduplicate hotspot budget entries`

### WIRE-001: Add rename_all to 56 enums
1. For each file listed in the tensor, add `#[serde(rename_all = "snake_case")]` to the identified enums
2. Work crate-by-crate: forge-pilot → knowledge-runtime → semantic-memory → living-memory → verification-* → attestation-exchange → continuity-runtime → remote-oracle-admission → semantic-memory-forge
3. Run: `cargo check --workspace` after each crate
4. Run: `cargo test --workspace` after all crates
5. Commit: `fix(WIRE-001): add rename_all to 56 serializable enums`

### DOC-001: Doc comment coverage pass
1. Add `///` doc comments to all pub struct, pub enum, pub trait in supported-lane crates
2. Add module-level `//!` docs to all lib.rs files that lack them
3. Priority order: forge-pilot, effect-runtime, forge-memory-bridge, verification-control, living-memory, knowledge-runtime
4. Commit: `fix(DOC-001): raise doc comment coverage to >80%`

## Phase 3 — Convention & Hygiene (estimate: 2–3 hours with AI assist)

All issues are independent.

### TRUTH-004: Exclude target-* from archive
1. Add `target-*` to `.gitignore`
2. Update `zip.py` to exclude `target-*` directories
3. Commit: `fix(TRUTH-004): exclude target-* from archive`

### GATE-003: Archive stale versioned scripts
1. Create `scripts/archive/`
2. Move `check_v9_*`, `check_v10_*`, `check_v11_*`, `check_v15_*`, `check_v21_v24_*`, `run_v16_v20_*` to archive
3. Verify `scripts/release_gate_set.py` does not reference any archived scripts
4. Commit: `fix(GATE-003): archive 12 stale versioned gate scripts`

### WIRE-002: Audit .ok() calls
1. Review each of the 25 `.ok()` calls listed in the tensor
2. For SQLite write operations: replace with `.map_err(|e| tracing::warn!(...)).ok()` or propagate
3. For quantization fallback: add `// INTENTIONAL: quantization is optional enhancement` comment
4. Commit: `fix(WIRE-002): document or replace error-swallowing patterns`

### CONV-001: HashMap audit
1. Add `// CONVENTION EXCEPTION: O(1) lookup required for HNSW index` to semantic-memory/src/hnsw.rs
2. Convert HashMap to BTreeMap in knowledge-runtime/src/entity/registry.rs and lifecycle.rs
3. Convert HashMap to BTreeMap in discovery-portfolio/src/lib.rs
4. Document remaining HashMap in semantic-memory/src/search.rs (performance-critical path)
5. Commit: `fix(CONV-001): convert or document HashMap usage`

### GOV-001: Document governance observation scope
1. Add comprehensive module docs to governance_gate.rs listing:
   - What is observed (6 predicates)
   - What is NOT yet observed (attestation, detailed mechanism state, detailed assurance state)
   - Why the current scope is sufficient for CLARA V1
2. Commit: `fix(GOV-001): document governance observation scope`

### PERF-001: Generate performance baseline
1. Run `bash scripts/collect_canonical_perf_baseline.sh`
2. Commit output as `evidence/perf_baseline_20260330.json`
3. Reference from STATUS_DASHBOARD.md
4. Commit: `fix(PERF-001): generate canonical performance baseline`

### SAFE-001: Verify panic checker
1. Run `bash scripts/check_no_prod_panics.sh`
2. If false positives emerge, add to `scripts/prod_panic_allowlist.json` with justification
3. Commit: `fix(SAFE-001): verify panic checker accuracy`

## Phase 4 — Post-Submission

### GOV-002: Document attestation gap
1. Add section to SCOPE_NOTES.md acknowledging attestation-exchange integration gap
2. Commit: `fix(GOV-002): document attestation-exchange forward declaration`

## Final gate

After all phases:
```bash
make gate
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All must pass green before archive generation.

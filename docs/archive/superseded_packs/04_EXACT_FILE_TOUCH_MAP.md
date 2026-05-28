# Exact File Touch Map — V29

Every file that must be created, modified, or moved to close the 16 issues.

## Phase 1

### TRUTH-001 + DOC-002 (combined — README rewrite covers both)
- **MODIFY** `README.md` — full rewrite as project README
- **MODIFY** `SOURCE_BASIS.md` — update snapshot reference to 20260330
- **MODIFY** `STATUS_DASHBOARD.md` — update active lane date
- **MODIFY** `PACK_MANIFEST.json` — update generated_at, repo_snapshot, pack_name

### GATE-001
- **MODIFY** `scripts/check_commit_permit_paths.py` — line 38: `ExecutionPermit` → `ToolExecutionPermit`

## Phase 2

### TRUTH-002
- **CREATE** `docs/archive/superseded_packs/` (directory)
- **CREATE** `docs/archive/SUPERSESSION_INDEX.md`
- **MOVE** `01_EXECUTIVE_SUMMARY.md` → `docs/archive/superseded_packs/`
- **MOVE** `01_MASTER_ISSUE_MATRIX.json` → `docs/archive/superseded_packs/`
- **MOVE** `01_MASTER_ISSUE_MATRIX.md` → `docs/archive/superseded_packs/`
- **MOVE** `01_MASTER_ISSUE_TENSOR.json` → `docs/archive/superseded_packs/`
- **MOVE** `01_MASTER_ISSUE_TENSOR.md` → `docs/archive/superseded_packs/`
- **MOVE** `01_SOURCE_BASIS_AND_RECONCILIATION.md` → `docs/archive/superseded_packs/`
- **MOVE** `02_MASTER_ISSUE_MATRIX.md` → `docs/archive/superseded_packs/`
- **MOVE** `02_SOURCE_BASIS.md` → `docs/archive/superseded_packs/`
- **MOVE** all `03_*` through `17_*` → `docs/archive/superseded_packs/`
- **MOVE** `CANONICAL_STACK_SPEC_V6.md` → `docs/archive/superseded_packs/`
- **MOVE** `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md` → `docs/archive/superseded_packs/`
- **MOVE** `HOSTILE_AUDIT_REPORT.md` → `docs/archive/superseded_packs/`
- **MOVE** `IMPLEMENTATION_PLAYBOOK.md` → `docs/archive/superseded_packs/`
- **MOVE** `CLAUDE_AUDIT_RECONCILIATION.md` → `docs/archive/superseded_packs/`
- **MOVE** `CLAUDE_CODE_PROMPT.md` → `docs/archive/superseded_packs/`
- **MOVE** `PHASED_EXECUTION_PLAN.md` → `docs/archive/superseded_packs/`
- **MOVE** `24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md` → `docs/archive/superseded_packs/`
- **MOVE** `MASTER_ISSUE_MATRIX.json` → `docs/archive/superseded_packs/`
- **MOVE** `MASTER_ISSUE_MATRIX.md` → `docs/archive/superseded_packs/`
- **MOVE** `SCAN_SUMMARY.json` → `docs/archive/superseded_packs/`
- **MOVE** `TASK_GRAPH.json` → `docs/archive/superseded_packs/`
- **MOVE** `VALIDATION.txt` → `docs/archive/superseded_packs/`
- **MOVE** `04_CLAUDE_RECONCILIATION.md` → `docs/archive/superseded_packs/`

### TRUTH-003
- **CREATE** `docs/archive/root_closeout_history/manifest.json`
- **MODIFY** `STATUS_EVIDENCE_MANIFEST.json` (if regenerating reference)

### GATE-002
- **MODIFY** `scripts/check_hotspot_budgets.sh` — deduplicate entries
- **CREATE or MODIFY** `docs/module_budget_exceptions.md`

### WIRE-001 (28 files across 13 crates)
- **MODIFY** `forge-pilot/src/act.rs` — ActionFamily
- **MODIFY** `forge-pilot/src/loop_runner_report.rs` — HaltReason
- **MODIFY** `forge-pilot/src/targets.rs` — TargetKind, TargetPriority
- **MODIFY** `knowledge-runtime/src/entity/registry.rs` — MatchQuality
- **MODIFY** `knowledge-runtime/src/obs/trace.rs` — QueryWarning
- **MODIFY** `knowledge-runtime/src/query/classify.rs` — QueryMode
- **MODIFY** `knowledge-runtime/src/query/merge.rs` — ScoreNormalization
- **MODIFY** `knowledge-runtime/src/query/route.rs` — RetrievalStrategy
- **MODIFY** `knowledge-runtime/src/temporal/claims.rs` — TemporalContradictionStatus
- **MODIFY** `semantic-memory/src/types.rs` — SearchSource, GraphEdgeType
- **MODIFY** `semantic-memory-forge/src/bundle.rs` — Refut, RefutationResult
- **MODIFY** `semantic-memory-forge/src/envelope.rs` — ExportAuthority, ExportEnvelopeError
- **MODIFY** `semantic-memory-forge/src/estimator.rs` — EstimatorKind
- **MODIFY** `living-memory/living-memory/src/experiment.rs` — ExperimentMode, EffectKind, TrialSide, CacheMode
- **MODIFY** `living-memory/living-memory/src/baseline.rs` — BaselineSourceKind
- **MODIFY** `living-memory/living-memory/src/failure.rs` — FailureClass
- **MODIFY** `living-memory/living-memory/src/scoring.rs` — ComparabilityClass, ObjectiveKind
- **MODIFY** `living-memory/living-memory/src/lab/evidence.rs` — 13 enums
- **MODIFY** `living-memory/living-memory/src/lab/evidence_analysis.rs` — 6 enums
- **MODIFY** `verification-adjudication/src/lib.rs` — RefutationClassV1, +3 more
- **MODIFY** `verification-control/src/lib.rs` — EffectBlockReasonV1, ReleaseGateFinalStateV1
- **MODIFY** `verification-policy/src/lib.rs` — DelegationRoleCombinationV1
- **MODIFY** `verification-policy/src/v14.rs` — DisclosureRevealClassV1, RefuterAllowanceV1
- **MODIFY** `verification-policy/src/profile_p1_privacy.rs` — 2 enums
- **MODIFY** `attestation-exchange/src/lib.rs` — 4 enums
- **MODIFY** `attestation-exchange/src/profile_p6_vendor.rs` — 5 enums
- **MODIFY** `continuity-runtime/src/vocab.rs` — IncidentSeverityV1
- **MODIFY** `remote-oracle-admission/src/lib.rs` — 2 enums

### DOC-001 (broad — all supported-lane crate pub types)
- **MODIFY** every `src/lib.rs` and source file with undocumented pub types across 17 supported-lane crates
- Priority files (lowest current coverage):
  - `forge-memory-bridge/src/batch.rs`, `transform.rs`, `lib.rs`
  - `effect-runtime/src/effect.rs`, `observation.rs`, `compensation.rs`, `v25.rs`
  - `forge-pilot/src/governance_gate.rs`, `observe.rs`, `orient.rs`, `act.rs`, `config.rs`, `targets.rs`, `types.rs`
  - `verification-control/src/lib.rs`, `v14.rs`

## Phase 3

### TRUTH-004
- **MODIFY** `.gitignore` — add `target-*`
- **MODIFY** `zip.py` — exclude `target-*`

### GATE-003
- **CREATE** `scripts/archive/` (directory)
- **MOVE** `scripts/check_v9_closeout_pack.sh` → `scripts/archive/`
- **MOVE** `scripts/check_v10_pack_truth.sh` → `scripts/archive/`
- **MOVE** `scripts/check_v11_release_readiness.sh` → `scripts/archive/`
- **MOVE** `scripts/check_v15_pack_truth.sh` → `scripts/archive/`
- **MOVE** `scripts/check_v21_v24_final_pack_truth.sh` → `scripts/archive/`
- **MOVE** `scripts/run_v16_v20_closeout_checks.sh` → `scripts/archive/`
- **MOVE** `scripts/check_post_v24_profile_repo_truth.sh` → `scripts/archive/`
- **VERIFY** `scripts/release_gate_set.py` references no archived scripts

### WIRE-002
- **MODIFY** `semantic-memory/src/db.rs` — lines 765, 1052
- **MODIFY** `semantic-memory/src/episodes.rs` — lines 166, 639, 697, 775, 838
- **MODIFY** `semantic-memory/src/knowledge.rs` — lines 691, 765, 814
- **MODIFY** `semantic-memory/src/lib.rs` — lines 1237, 1289, 1342, 1394
- **MODIFY** `semantic-memory/src/documents.rs` — line 274
- **MODIFY** `semantic-memory/src/graph.rs` — line 684
- **MODIFY** `semantic-memory/src/conversation.rs` — line 637
- **MODIFY** `semantic-memory/src/projection_import.rs` — lines 421, 431

### CONV-001
- **MODIFY** `semantic-memory/src/hnsw.rs` — add CONVENTION EXCEPTION comments
- **MODIFY** `semantic-memory/src/search.rs` — add CONVENTION EXCEPTION comments
- **MODIFY** `semantic-memory/src/graph.rs` — evaluate conversion
- **MODIFY** `knowledge-runtime/src/entity/registry.rs` — convert to BTreeMap
- **MODIFY** `knowledge-runtime/src/projection/lifecycle.rs` — convert to BTreeMap
- **MODIFY** `discovery-portfolio/src/lib.rs` — convert to BTreeMap

### GOV-001
- **MODIFY** `forge-pilot/src/governance_gate.rs` — expand module docs

### PERF-001
- **CREATE** `evidence/perf_baseline_20260330.json`
- **MODIFY** `STATUS_DASHBOARD.md` — reference baseline

### SAFE-001
- **MODIFY** `scripts/check_no_prod_panics.sh` (if needed)
- **MODIFY** `scripts/prod_panic_allowlist.json` (if needed)

## Phase 4

### GOV-002
- **MODIFY** `SCOPE_NOTES.md`

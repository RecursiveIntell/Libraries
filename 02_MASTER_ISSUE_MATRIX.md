# Master Issue Matrix — V29

**Source basis:** `libraries-source-clean-20260330.zip`
**Auditors:** Claude (10-angle), GPT (10-lens)
**Total issues:** 16 (3 P0, 5 P1, 7 P2, 1 P3)

## Phase 1 — Submission Blockers (P0)

These must be resolved before any DARPA submission artifact is generated.

| ID | Title | Files | Acceptance |
|----|-------|-------|------------|
| TRUTH-001 | Repo tells four different snapshot dates | README.md, PACK_MANIFEST.json, SOURCE_BASIS.md, STATUS_DASHBOARD.md | All active docs reference same snapshot |
| GATE-001 | check_commit_permit_paths.py expects wrong permit type | scripts/check_commit_permit_paths.py | Script passes |
| DOC-002 | README.md is a stale pack description, not a project README | README.md | README describes the project |

## Phase 2 — Credibility Risks (P1)

These materially weaken submission credibility if left unaddressed.

| ID | Title | Files | Acceptance |
|----|-------|-------|------------|
| TRUTH-002 | 78 root-level control documents | docs/archive/, README.md | Root has <20 active docs |
| TRUTH-003 | Missing archive manifest artifact | docs/archive/root_closeout_history/manifest.json | check_root_archive_manifest.py passes |
| GATE-002 | check_hotspot_budgets.sh has contradictory limits | scripts/check_hotspot_budgets.sh, docs/module_budget_exceptions.md | Script passes, no duplicates |
| WIRE-001 | 56 serializable enums missing rename_all | 28 files across 13 crates | 0 enums without rename_all |
| DOC-001 | Doc comment coverage below review threshold | All lib.rs, all undocumented pub types | >80% coverage on supported-lane crates |

## Phase 3 — Convention & Hygiene (P2)

These should be addressed but do not block submission.

| ID | Title | Files | Acceptance |
|----|-------|-------|------------|
| TRUTH-004 | 6 target-* directories pollute archive | .gitignore, zip.py | No target-* in clean archive |
| GATE-003 | 12 stale versioned gate scripts | scripts/archive/ | No check_v{N}_* in active scripts/ |
| WIRE-002 | .ok() error-swallowing in semantic-memory | 8 files in semantic-memory | All .ok() documented or replaced |
| CONV-001 | 33 HashMap refs violate BTreeMap convention | 6 files across 3 crates | All HashMap documented or converted |
| GOV-001 | Governance observation pipeline is thin | forge-pilot/src/governance_gate.rs | Module docs state observation scope |
| PERF-001 | No canonical performance evidence artifact | evidence/perf_baseline_20260330.json | Dated baseline exists |
| SAFE-001 | Panic checker edge cases | scripts/check_no_prod_panics.sh | Script passes cleanly |

## Phase 4 — Post-Submission (P3)

| ID | Title | Files | Acceptance |
|----|-------|-------|------------|
| GOV-002 | attestation-exchange is unconsumed | SCOPE_NOTES.md | Explicitly acknowledged |

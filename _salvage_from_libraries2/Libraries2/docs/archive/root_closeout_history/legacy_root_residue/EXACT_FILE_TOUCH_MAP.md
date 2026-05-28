# Exact file touch map

Every file each fix is permitted to touch, and only those files.
No fix may touch files outside its listed set without an explicit amendment to this document.

## FIX-001 — Replace stale SCAN_SUMMARY.json

```
SCAN_SUMMARY.json                              MODIFY (full replacement)
```

## FIX-002 — Resolve DEMO-001/BENCH-001 contradiction

```
MASTER_ISSUE_MATRIX.md                         MODIFY (status fields for DEMO-001, BENCH-001)
03_MASTER_ISSUE_MATRIX.md                      MODIFY (same changes if it mirrors the above)
STATUS_DASHBOARD.md                            MODIFY (status entries for DEMO-001, BENCH-001)
```

## FIX-003 — Make forge-bench honest

```
docs/benchmarks/score_sheet.json               MODIFY (add assessment_mode, limitation fields)
contracts/fixtures/bench/forge_bench_casebook.json  MODIFY (add assessment_mode, limitation fields)
docs/benchmarks/README.md                      MODIFY (add Assessment modes section)
CLAUDE_AUDIT_RECONCILIATION.md                 MODIFY (add correction note)
```

## FIX-004 — Fix archive manifest physically_archived_groups

```
docs/archive/root_closeout_history/manifest.json   MODIFY (populate physically_archived_groups)
```

## FIX-005 — Regenerate closeout receipt

```
release/closeout_receipt_v1.json               MODIFY (regenerated output from generate script)
scripts/generate_closeout_receipt.py           MODIFY IF NEEDED (only if it doesn't read archive manifest)
```

## FIX-006 — Reconcile SUPPORT_PROFILE crate count

```
SUPPORT_PROFILE.md                             MODIFY (restore to 17-crate list)
SCOPE_NOTES.md                                 CREATE (documents adjacent crates)
release/closeout_receipt_v1.json               MODIFY (regenerated after profile correction)
```

## FIX-007 — Reconcile ghost numbered root files

```
docs/archive/root_closeout_history/manifest.json       MODIFY (add files to active pack or superseded)
docs/archive/root_closeout_history/legacy_root_residue/  RECEIVE new files (moves from root)
[specific root files identified as superseded]         MOVE to archive dir
```

Files to archive from root (move, do not delete):
- `00_START_HERE.md`
- `02_SOURCE_BASIS.md`
- `02_HOSTILE_AUDIT_RECONCILED.md`
- `04_IMPLEMENTATION_SEQUENCE.md`
- `08_EXACT_FILE_TOUCH_MAP.md`
- `09_CRATE_BOUNDARY_MAP.md`
- `CONFORMANCE_GATES.md`
- `IMPLEMENTATION_PLAYBOOK.md`
- `PHASED_EXECUTION_PLAN.md`
- `RISKS_AND_FORBIDDEN_SHORTCUTS.md`

Files to add to active_root_closeout_pack in manifest:
- `07_GATES_AND_ACCEPTANCE.md`
- `09_RISK_REGISTER.md`
- `11_BENCHMARK_PLAN.md`
- `PACK_README.md`
- `MASTER_ISSUE_MATRIX.md`
- `MASTER_ISSUE_MATRIX.json`
- `RELEASE_CHECKLIST.md`
- `CONFORMANCE_GATES.md` (if kept active, otherwise archive)
- `AGENTS.md`
- `PROMPT.md`

## FIX-008 — Live forge-bench execution

```
docs/benchmarks/run_forge_bench.py             MODIFY (add --mode execution, execute temporal_correctness)
docs/benchmarks/score_sheet.json               MODIFY (add executed case alongside fixture-asserted ones)
docs/benchmarks/README.md                      MODIFY (add execution mode documentation)
```

Do NOT modify:
- `contracts/fixtures/bench/forge_bench_casebook.json` (already modified in FIX-003)
- Any fixture files under `contracts/fixtures/v21/`, `v22/`, `v23/`

## FIX-009 — Fix prompts/system.md local path

```
prompts/system.md                              MODIFY (one line change on line 50)
```

## FIX-010 — Complete physical root reduction

```
docs/archive/root_closeout_history/legacy_root_residue/  RECEIVE remaining archived files
docs/archive/root_closeout_history/manifest.json   MODIFY (update archived_count)
release/closeout_receipt_v1.json               MODIFY (final regeneration)
```

---

## Files that must not be touched in this pass

```
Any file under contracts/fixtures/v21/
Any file under contracts/fixtures/v22/
Any file under contracts/fixtures/v23/
Any file under contracts/fixtures/v24/
Any file under contracts/fixtures/p1/ through p7/
Any .rs source file (except docs/benchmarks/run_forge_bench.py which is Python)
Any Cargo.toml
Any schema file under schemas/
scripts/check_*.sh and scripts/check_*.py (read only — do not modify gates)
verification-control/tests/e2e_effect_authority_assurance_release.rs
release/closeout_receipt_v1.json (except as output of generate script in FIX-005/FIX-006/FIX-010)
```

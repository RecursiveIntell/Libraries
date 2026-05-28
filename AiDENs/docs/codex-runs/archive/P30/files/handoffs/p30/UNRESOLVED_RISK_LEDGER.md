# P30 Unresolved Risk Ledger

| Risk | Owner | Evidence | Next target |
|---|---|---|---|
| Full command process-tree termination is not implemented. | P31 command sandbox owner | `target/p30/audit/cargo_test_workspace_all_targets.log`; P30-ABSORB-0017 in `ISSUE_ABSORPTION_REPORT.csv` | Add process-group/job-object termination and grandchild survival regression. |
| Warning-class dynamic JSON/panic/lint debt remains. | P31 pattern-debt owner | `target/p30/audit/p30_guard.log` shows `findings=1824 hard=0` | Retire warning classes crate by crate; keep hard guard at zero. |
| Historical missing gate artifacts and root-doc package hygiene are not fully restored. | P31 gate/package owner | `ISSUE_ABSORPTION_REPORT.csv`; `target/p30/audit/parent_make_gate.log` | Restore or supersede historical gate artifacts and parent pack-truth docs. |
| v11B graph/region/subtraction/causal runtime remains executable seed coverage, not full runtime conformance. | P31/P32 v11B owner | `target/p30/audit/scripts_verify.log` v11B seed checks | Expand reference interpreter and production-path coverage before stronger claim. |
| Material IDs are not fully migrated to owner-proven material digests. | P31 identity owner | `rg display_only_unstable_id` still finds display/default constructors | Replace material-path display IDs with `generated_artifact_id_from_material` or canonical stack IDs. |

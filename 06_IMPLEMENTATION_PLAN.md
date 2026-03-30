
# 06_IMPLEMENTATION_PLAN

## Order of operations

### Phase 0 — make the front door honest
1. Close `PACK-001`: generate and add `04_MASTER_ISSUE_MATRIX.csv`.
2. Close `PACK-002`: fix the archive manifest count mismatch.
3. Close `TRUTH-001`: rewrite `STATUS_DASHBOARD.md`, `STATUS_EVIDENCE_MANIFEST.json`, and regenerate `release/closeout_receipt_v1.json` from current truth.
4. Close `GATE-001`: make the Makefile, proof ledger, and receipt derive from one identical gate list.
5. Close `SAFE-001`: either move inline test modules out of `src/`, or patch the panic audit to ignore them.

### Phase 1 — restore CI and the shipped production-closure lane
6. Close `CI-001`: add `.github/workflows/ci.yml`.
7. Close `V25-001`: restore or retire the broken v25 pack surfaces.
8. Close `V25-002`: finish the effect/policy/control v25 production-closure markers.
9. Close `TYPE-001`: centralize `V25ConstitutionCitation` and widen the drift check.

### Phase 2 — make the repo externally credible
10. Close `NAME-001`: rename or deepen the thin governance/runtime shells.
11. Close `DOC-001`: widen doc coverage or explicitly demote shell crates from the credibility lane.
12. Close `MOD-001`: split oversized modules into reviewable submodules.
13. Close `LLM-001`: implement or delete `llm-refinement`.
14. Close `EXTRACT-001`: replace or sharply bound the line-based Rust symbol extractor.
15. Close `ROOT-001`: collapse the duplicated root pack surfaces into one active authority lane.

## Closure rules

- Do not patch the dashboard before patching the underlying repo truth.
- Do not claim a gate is green unless the current repo can reproduce it.
- Do not widen the supported lane while closing the truth lane.
- Do not smuggle horizon/spec work into the finish pass.
- Do not delete history; archive or demote it.

## Suggested closure checkpoints

### Checkpoint A
- `check_pack_truth.sh` passes
- `check_root_archive_manifest.py` passes
- dashboard/receipt/evidence manifest are rewritten from current truth

### Checkpoint B
- panic audit is either clean or explicitly removed from the claimed gate set
- CI exists
- v25 repo-truth scripts are no longer broken on missing files

### Checkpoint C
- v25 production closure passes
- duplicate constitution citation type removed
- one active pack remains at the root

# P25 Hard Audit

## Verdict

The prior-run baseline materially improved AiDENs. The package is clean and current enough for a surgical release-hygiene pass.

P25 should not chase major capability expansion. It should fix process/control weaknesses that could cause future Codex runs to drift.

## Package state

- Current run: prior run baseline
- Included files: 1209
- Root Markdown files: 29
- Phase injection files: 9
- Large Rust file candidates: 90
- Findings file: 0 errors, 0 warnings
- Codex archive sidecar: `active_stale_after = []`

## P25 critical findings

### P25-AUD-001 — Root workspace Markdown context noise

The archive root contains root-level Markdown that appears to be historical run, audit, prompt, matrix, and planning residue. These files can pollute Codex context and confuse current-run truth.

Root Markdown classification snapshot:

| path | classification | reason |
| --- | --- | --- |
| 06_RISK_REGISTER.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| AGENTS.md | protected-functional | explicit protected root doc |
| AUDIT_2026-04-01.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| CLAUDE.md | protected-functional | explicit protected root doc |
| COMBINED_AUDIT_2026-04-01.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| CONFORMANCE_GATES.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| CONTRACT_AND_TEMPORAL_TRUTH_HARDENING.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| CRATE_HARDENING_MATRIX.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| EXECUTION_EVIDENCE_AND_REFERENCE_INTERPRETER_PLAN.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| HOSTILE_AUDIT_SYNTHESIS_V5.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| KERNEL_AND_REGION_RUNTIME_PLAN.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| LIBRARIES_MASTER_MATRIX_V8.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| LIBRARIES_PROMPT.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| LIB_MASTER_ISSUE_MATRIX.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| LIB_PROMPT.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| MASTER_ISSUE_MATRIX.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| MASTER_TENSOR.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| PROMPT.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| README.md | protected-functional | explicit protected root doc |
| RISK_REGISTER.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| SCOPE_NOTES.md | ambiguous-stop | root markdown not protected but no archive pattern match; stop for operator classification |
| SNAPSHOT_2026-04-11.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| SOURCE_BASIS.md | protected-functional | explicit protected root doc |
| STATUS_DASHBOARD.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| SUPPORT_PROFILE.md | protected-functional | explicit protected root doc |
| TEST_AND_CONFORMANCE_PLAN.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| V9_IMPLEMENTATION_PLAYBOOK.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| claude_hard_audit_2026-03-30.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |
| gpt54_hard_audit_2026-03-30.md | candidate-archive | root-level run/audit/spec/prompt/planning residue pattern |


### P25-AUD-002 — Phase injection stale IDs

Current phase-injection files contain stale references such as prior-run IDs or stale `target/pXX`/`handoffs/pXX` paths.

| path | stale_tokens | has_stop_wait |
| --- | --- | --- |
| AiDENs/phase_injections/GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md | P25 | True |
| AiDENs/phase_injections/PHASE_00_TO_01_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_01_TO_02_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_02_TO_03_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_03_TO_04_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_04_TO_05_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_05_TO_06_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_06_TO_07_REVALIDATION.md | handoffs/p25;target/p25 | True |
| AiDENs/phase_injections/PHASE_07_TO_08_REVALIDATION.md | handoffs/p25;target/p25 | True |


This directly matches the user-observed failure: Codex did not stop properly for injections and instead treated phase docs as advisory.

### P25-AUD-003 — z.py scope risk

`z.py` has become useful as operator-local packaging/certifier tooling. It must not become a runtime or semantic adapter. P25 may touch z.py only for root Markdown archiving.

### P25-AUD-004 — Large-file maintainability risk

Largest Rust file candidates:

| path | bytes | lines |
| --- | --- | --- |
| AiDENs/crates/aidens-contracts/src/lib.rs | 371136 | 10410 |
| AiDENs/crates/aidens-cli/src/lib.rs | 235631 | 6698 |
| knowledge-runtime/tests/cross_crate_proof.rs | 133452 | 3610 |
| AiDENs/crates/aidens-tool-kit/src/lib.rs | 90164 | 2454 |
| AiDENs/crates/aidens-runner/src/lib.rs | 75397 | 1910 |
| verification-control/src/lib.rs | 70140 | 1915 |
| AiDENs/crates/aidens-agency-kit/src/lib.rs | 67998 | 1815 |
| living-memory/living-memory/src/lab/evidence.rs | 66113 | 1627 |
| semantic-memory/src/lib.rs | 62756 | 1619 |
| semantic-memory/src/projection_lane.rs | 61912 | 1471 |
| semantic-memory/tests/import_ugly_cases.rs | 61103 | 1783 |
| semantic-memory/tests/import_boundary_tests.rs | 60779 | 1641 |
| semantic-memory-forge/src/envelope.rs | 56863 | 1434 |
| kernel-conformance/src/lib.rs | 56767 | 1394 |
| profile-runtime/src/adapters.rs | 54472 | 1792 |


P25 should not refactor them unless required by a gate. It should create a containment plan.

### P25-AUD-005 — Support-claim discipline

Prior-run support was supported-local and fixture-backed. P25 preserves that truth. Cloud, broad autonomy, federation, and V10+ runtime geometry remain deferred.

## Required P25 stance

Close hygiene and evidence seams. Do not chase the horizon.

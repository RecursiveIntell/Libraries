# Codex Hardening Super Pass Master Prompt

You are operating on AiDENs after the P29 clean source package and multiple hard audits. Your job is to perform a **hardening super pass** that fixes the merged backlog and any additional major blockers you discover.

## Source-basis rule

The operator-created bundle finished successfully and is clean. Do **not** waste time treating the skipped post-bundle operator gate as a product failure. However, because you will modify the tree, you MUST regenerate package sidecars and run extracted-package self-replay at the end before claiming any support label.

## Primary materials to read first

1. `README.md`
2. `02_SOURCE_BASIS_AND_AUDIT_NORMALIZATION.md`
3. `matrices/SUPER_PASS_BACKLOG_1020.csv`
4. `03_PRIORITIZED_BUILD_ORDER.md`
5. `04_PHASE_PLAN.md`
6. `05_ACCEPTANCE_GATES_AND_COMMAND_BAR.md`
7. `06_CLAUDE_AUDIT_INTEGRATION.md`
8. `07_FORBIDDEN_FINAL_STATES_AND_LABEL_POLICY.md`
9. The copied source audits under `source_audits/`

## Non-negotiable rules

- Do not create shadow truth owners inside AiDENs. AiDENs owns local orchestration/operator/display/runtime surfaces only.
- Do not claim `v11B-complete`, `v11C-complete`, `production-cloud-ready`, `broad-autonomy-ready`, or `canonical-truth-owner`.
- Do not mark marker-string assertions as sufficient for hard gates. Replace marker tests with semantic fixtures wherever possible.
- Do not allow user-visible completion before durable material-operation receipts are written.
- Do not allow `Local` provider route to mean `mock` without explicit route/degradation disclosure.
- Do not allow material boundary repair to silently alter treatment-critical fields.
- Do not degrade silently. Every degradation, fallback, widening, repair, quarantine, or waiver must be receipt-bearing and queryable.
- Do not delete or hide audit rows. Classify them.
- Do not broaden scope to cloud/broad autonomy. Keep v11A supported-local honest; seed only one minimal v11B slice.

## Required strategy

Solve the backlog by epics, not row-by-row. For every phase:

1. Identify all backlog rows in scope.
2. Fix source code where appropriate.
3. Add hostile/semantic tests that would fail before the fix.
4. Update the issue matrix status for all rows touched.
5. Produce a phase report with files changed, tests run, rows closed/quarantined/deferred, and unresolved risk.
6. Run the phase gate before moving on.

## Required high-level order

1. Receipt/log durability and no done without receipts.
2. Sandbox/security hostile corpus.
3. Tool exposure and permit parity.
4. Transactional patch engine.
5. Command execution receipts.
6. Provider honesty.
7. Queue/daemon concurrency.
8. Boundary compiler/schema/repair.
9. Bitemporal/proof/view reference corpus.
10. Minimal v11B region.
11. Schema governance.
12. Artifact lifecycle/operator effects.
13. Module decomposition/source ownership.
14. Replace marker tests with semantic fixtures.
15. Docs/evidence/known limitations closure.
16. Config/env/secrets/redaction.
17. App/scaffold/profile readiness.
18. Search/pool/HNSW hardening from Claude audit.
19. Unaudited high-risk layers audit/quarantine.
20. Final package/replay/release bar.

## Expected final deliverables in repo

- Passing `cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`, and doc/test command bar where available.
- Updated issue matrix with no raw `open` rows for the supported scope.
- Hard hostile fixture suites for sandbox, patch, boundary, receipts, queue, provider, temporal/proof/view, and v11B minimal region.
- Final known limitations register.
- Final auditor handoff.
- Exact package sidecars and extracted-package replay receipt.
- Honest final labels only if gates pass.

## Completion standard

The pass is complete only if the acceptance gates in `05_ACCEPTANCE_GATES_AND_COMMAND_BAR.md` pass. If anything remains unproven, label it `quarantined`, `deferred`, or `unsupported`, and include it in the known limitations register. Receipts or it did not happen.

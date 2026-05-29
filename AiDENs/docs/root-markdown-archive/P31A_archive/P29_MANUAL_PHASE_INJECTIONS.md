# P29 Manual Phase Injections

Use only these gates. Do not stop after every phase.

## Injection 1 — after Phase 03

```text
STOP. Manual gate after Phase 03.

Revalidate:

1. CURRENT_RUN.md says P29.
2. P29_STATUS_EVIDENCE_MANIFEST.json or template says P29.
3. scripts/p29_verify.sh exists or is scheduled before forward progress.
4. scripts/verify_current.sh will delegate to p29_verify.sh.
5. No P29 docs, handoffs, or scripts are classified as stale.
6. P28 failure postmortem is written.
7. Manifest path validation is implemented or scheduled.
8. The Claude audit matrix contains all 200 BUG IDs or a documented parser exception.

Write handoffs/p29/PHASE_03_MANUAL_GATE.md with PASS/FAIL for each item.
If any item fails, repair before Phase 04.
```

## Injection 2 — after Phase 07

```text
STOP. Manual gate after Phase 07.

Revalidate:

1. HNSW critical issues BUG-001 through BUG-010 are fixed or quarantined.
2. SQLite/migration issues BUG-011 through BUG-020 and BUG-076 through BUG-085 are fixed or quarantined.
3. Search/ranking/dedup issues BUG-021 through BUG-030 and BUG-053 through BUG-059 are fixed or quarantined.
4. New tests cover lock ordering, migration atomicity, and dedup/recency behavior.
5. No v11A/v11B claim has been advanced prematurely.

Write handoffs/p29/PHASE_07_MANUAL_GATE.md.
If any item fails, repair before Phase 08.
```

## Injection 3 — after Phase 11

```text
STOP. Manual gate after Phase 11.

Revalidate:

1. AiDENs contract bugs BUG-066 through BUG-071 are fixed or quarantined.
2. Tool receipt start/completion timing is correct.
3. Execution context fingerprint is not just the aidens-contracts crate version.
4. Artifact lifecycle transitions are legal and receipt-backed.
5. Proof/debt/waiver semantics exist and waiver is not treated as proof.
6. Boundary compiler profile exists and duplicate/malformed structured input tests exist.
7. Receipt chain validation exists.

Write handoffs/p29/PHASE_11_MANUAL_GATE.md.
If any item fails, repair before Phase 12.
```

## Injection 4 — after Phase 15

```text
STOP. Manual gate after Phase 15.

Revalidate v11A local release candidate:

1. Supported-local agent path has artifact envelope, execution context, operator contract, input/output manifest, receipt, proof/degradation state.
2. Completion is blocked/degraded if receipts or proof state are missing.
3. Semantic/view disclosure is user-visible.
4. Package/evidence repair gates remain green.
5. No full v11B or v11C claim exists.

Write handoffs/p29/PHASE_15_MANUAL_GATE.md.
If any item fails, repair before v11B seed work.
```

## Injection 5 — after Phase 19

```text
STOP. Manual gate after Phase 19.

Revalidate v11B seed only:

1. Right-graph misuse tests exist.
2. RegionContractV1 seed exists.
3. BoundaryMessage/BoundaryReceipt seed exists.
4. Residual/syndrome/convergence seed exists.
5. Lawful subtraction seed exists.
6. All v11B surfaces are labeled executable seed, not complete.
7. v11C remains reserved-only.

Write handoffs/p29/PHASE_19_MANUAL_GATE.md.
If any item fails, repair before docs/status convergence.
```

## Injection 6 — before final package generation

```text
STOP BEFORE FINAL PACKAGE GENERATION.

Do not package until:

1. cargo fmt/check/test/clippy/doc pass.
2. bash scripts/p29_verify.sh passes.
3. scripts/p29_verify.sh is active and included.
4. scripts/verify_current.sh works in source tree.
5. An extracted package replay has been run.
6. P29_STATUS_EVIDENCE_MANIFEST.json references only included or explicitly external/degraded files.
7. No P29 files are archived as stale.
8. Final support labels are allowed labels only.
9. Known limitations register exists.
10. Final auditor handoff exists.

If any item fails, do not generate final package.
```

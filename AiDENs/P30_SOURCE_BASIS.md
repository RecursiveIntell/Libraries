# P30 Source Basis

## Package basis

- AiDENs source package: `AiDENs-aidens-next-codex-context-20260508.zip`
- Package report: `AiDENs-aidens-next-codex-context-20260508.report.md`
- Manifest: `AiDENs-aidens-next-codex-context-20260508.manifest.json`
- Findings sidecar: `AiDENs-aidens-next-codex-context-20260508.findings.json`
- Excluded sidecar: `AiDENs-aidens-next-codex-context-20260508.excluded.json`
- Prior hostile audit: `AIDENS_HOSTILE_AUDIT_20260508.md`
- Prior hostile issue matrix: `AIDENS_HOSTILE_AUDIT_ISSUES_20260508.csv/json`

## Facts to preserve

- The certifier reported strict packaging mode and zero findings, but this is not build, semantic, or conformance certification.
- The package contains 1,611 included files and 141 excluded files.
- The package contains 600 Markdown files and 515 Rust files.
- Root Markdown archival was disabled and 134 root Markdown files remained ambiguous.
- Archive hash is a zip-byte hash, not canonical content identity.
- Codex archive current run is P29; P30 must create clear supersession records.

## Spec basis

- v9: episode identity, execution evidence, bridge invariants, repair records, verification-plan artifacts.
- v11A: constitutional artifact runtime core; material work as receipt-bearing artifact transitions.
- v11B: right-graph law, region contracts, convergence governance, repair, causal/interventional execution, lawful subtraction.
- v11C: reserve future admission/federation/self-hosting surfaces without smuggling them into current authority.
- End-state spec: typed artifact machine; evidence before inference; execution is evidence; lawful subtraction; no shadow constitutions.

## Audit basis

P30 absorbs 554 hostile audit issues:

- P0: 15
- P1: 292
- P2: 247

Highest-volume categories:

- REPLAY-IDENTITY: 83
- SILENT-DEGRADATION: 80
- PANIC-SURFACE: 80
- DYNAMIC-JSON: 80
- NONDETERMINISM: 60
- LINT-SUPPRESSION: 50
- DETERMINISM: 40
- OBSERVABILITY: 30
- GATE-DRIFT: 7
- CODE-SHAPE: 7
- SCHEDULING: 6
- PARSER-BOUNDARY: 5

## Source path rule

Run from the archive root layout, not from a copied standalone `AiDENs/` folder, unless P30 implements and proves standalone path remapping. The nested workspace intentionally depends on sibling crates.

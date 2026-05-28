# AiDENs Status - P29 Evidence Repair + v11A Local Release Candidate

Record date: `2026-05-07`

This is the active P29 release-truth ledger for AiDENs. P29 starts from useful P28 implementation work but treats the P28 final release claim as contaminated by evidence/package failures until the P29 verifier, manifest, package, and extracted self-replay gates pass.

## Current Run

| Field | Value |
|---|---|
| Current run | P29 Evidence Repair + v11A Local Release Candidate + v11B Executable Seed |
| Prior run | P28 v11A Constitutional Material-Operation Kernel |
| Current status | Phase 20 docs/status convergence; blocked from release claim until Phase 21 command bar, strict package, and extracted package self-replay pass |
| Declared path | supported-local operator/agent/coding-agent path with receipts, execution context, manifests, proof/debt/degradation state, and boundary compiler records |
| Final package status | pending under `target/p29/package/`; no final package claim yet |

## P29 Phase Ledger

| Phase | Evidence | Result |
|---|---|---|
| 00 | `handoffs/p29/PHASE_00_REPORT.md` | source basis and run identity lock |
| 01 | `handoffs/p29/PHASE_01_REPORT.md` | P28 evidence/package failure absorption |
| 02 | `handoffs/p29/PHASE_02_REPORT.md` | package/archive classifier repair |
| 03 | `handoffs/p29/PHASE_03_REPORT.md` | verifier and manifest repair |
| 04 | `handoffs/p29/PHASE_04_REPORT.md` | audit triage/quarantine |
| 05-07 | `handoffs/p29/PHASE_05_REPORT.md` through `handoffs/p29/PHASE_07_REPORT.md`; `handoffs/p29/PHASE_07_MANUAL_GATE.md` | HNSW, SQLite/migration, search/ranking/dedup repairs or quarantines |
| 08-11 | `handoffs/p29/PHASE_08_REPORT.md` through `handoffs/p29/PHASE_11_REPORT.md`; `handoffs/p29/PHASE_11_MANUAL_GATE.md` | vector/chunker/query/contract boundary repairs or quarantines |
| 12-15 | `handoffs/p29/PHASE_12_REPORT.md` through `handoffs/p29/PHASE_15_REPORT.md`; `handoffs/p29/PHASE_15_MANUAL_GATE.md` | v11A declared supported-local evidence present, pending final package |
| 16-19 | `handoffs/p29/PHASE_16_REPORT.md` through `handoffs/p29/PHASE_19_REPORT.md`; `handoffs/p29/PHASE_19_MANUAL_GATE.md` | v11B executable seed present, no completion claim |
| 20 | `handoffs/p29/PHASE_20_REPORT.md` | docs/status/support convergence |
| 21 | `handoffs/p29/PHASE_21_REPORT.md` | pending final command bar, package, and extracted self-replay |

## Current Support Posture

P29 may claim only `in-progress` / `candidate-pending-final-package` until the final command bar and extracted package replay pass. Allowed final labels are limited to:

- `p29-package-repaired`
- `p29-supported-local-plus`
- `v11A-local-release-candidate`
- `v11B-executable-seed`
- `v11C-reserved-only`

P29 must not claim v11B completion, v11C completion, broad autonomy readiness, production-cloud readiness, or canonical ownership of memory, governance, kernel, provider/tool, schema, federation, or ID truth.

Current candidate support posture:

| Surface | Candidate state | Blocking condition |
|---|---|---|
| Package/evidence repair | verifier and manifest evidence present | final strict package and extracted package replay |
| v11A declared supported-local path | local release-candidate evidence present for `run-coding-agent` | final command bar and package replay |
| v11B graph/region/subtraction surfaces | executable seed only | future canonical-owner activation; no P29 completion claim |
| v11C | reserved only | outside P29 scope |

## P28 Carry-Forward

P28 implementation work remains useful candidate implementation evidence. P29 does not inherit the P28 release claim until it repairs:

- archive/current-run identity drift;
- active P29 artifact classification;
- verifier wrapper delegation;
- manifest path resolution;
- package self-replay from an extracted zip.

## Non-Claims

AiDENs is not production-cloud-ready, broadly autonomous, v11B complete, v11C complete, or a replacement for canonical memory/governance/kernel/runtime crates.

## Current Hardening Super-Pass Overlay

The active hardening super-pass uses `matrices/SUPER_PASS_BACKLOG_1020.csv` and `handoffs/super-pass/` as the current closure evidence. The operator-created clean source bundle is accepted as source basis; it is not a product-conformance or release-package claim after this tree was modified.

Current super-pass status as of Phase 15: docs/evidence closure in progress, no final package label. Final package sidecars, audit-log hash refresh, and extracted-package self-replay are still required before any package/replay support label.

Sidecar identity rule: final package evidence must distinguish zip-byte hashes from canonical content or manifest identities. A matching zip hash alone does not prove canonical replay semantics.

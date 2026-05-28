# Source Basis - P29

Record date: `2026-05-07`

This file records the active P29 source basis.

## Current Run

| Field | Value |
|---|---|
| Current run | P29 Evidence Repair + v11A Local Release Candidate + v11B Executable Seed |
| Prior run | P28 v11A Constitutional Material-Operation Kernel |
| Workspace root | `AiDENs/` |
| Expected parent workspace | `/home/sikmindz/Coding/Libraries` |
| Rust edition | 2021 |
| Minimum Rust version | 1.76 |
| Final strict gate | pending |

## Inputs

- P29 packet files: `P29_OPERATOR_PASTE_FIRST.md`, `P29_MASTER_PACKET.md`, `P29_PHASE_PLAN.md`, `P29_ACCEPTANCE_GATES.md`, `P29_CLAUDE_AUDIT_ABSORPTION.md`.
- P29 issue matrix: `matrices/P29_MASTER_ISSUE_MATRIX.csv`.
- P28 status/evidence/package sidecars, treated as failure evidence and candidate implementation evidence, not a clean release basis.
- v11A/v11B/v11C specs under `docs/codex-runs/Specs/`.
- Active status/support docs: `STATUS.md`, `SUPPORT_PROFILE.md`, `SOURCE_BASIS.md`.

## Canonical Sibling Ownership

AiDENs depends on sibling crates through path dependencies under the parent Libraries workspace. If those siblings are absent, cargo/package replay must classify the result as environment-blocked, not clean.

| Surface | Canonical owner |
|---|---|
| IDs, digests, trace IDs | `stack-ids` |
| Raw evidence/export packages | `semantic-memory-forge` |
| Bridge transforms | `forge-memory-bridge` |
| Queryable projected memory | `semantic-memory` |
| Runtime views/widening disclosure | `knowledge-runtime` |
| Tool/provider runtime contracts | `llm-tool-runtime` |
| Verification/control/policy/adjudication | `verification-*` crates |
| Kernel operators/witnesses/syndromes/residuals/oracles | `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance` |
| Schema generation | `contract-schema-gen` |
| Federation/admission/mechanism authority | `attestation-exchange`, `remote-oracle-admission`, `federated-settlement`, `mechanism-runtime` |

## Replay Modes

| Mode | Classification | Meaning |
|---|---|---|
| Local parent workspace present | `sibling_workspace_present` | Cargo checks may run against sibling path dependencies. |
| Local parent workspace absent or incomplete | `sibling_workspace_missing` | Cargo/package replay is environment-blocked and must not be called clean. |
| Strict P29 final verifier | `exact_check` | Only available after the final P29 command bar passes. |
| Skipped cargo or degraded replay | `degraded_exact_check` | Static/package checks may run, but cargo-backed replay proof is absent. |

## Active Truth Docs

- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `P29_*`
- `matrices/P29_*`
- `prompts/phases/P29_*`
- `handoffs/p29/*`
- `docs/p29/*`

Historical P24/P25/P26/P27/P28 materials are evidence, not active support claims, unless explicitly cited as prior-run evidence.

## Current Hardening Super-Pass Overlay

The operator-created clean source package is the source basis for this pass. The skipped optional post-bundle operator gate is evidence hygiene, not a source/product defect. Because this pass changes the tree, final package sidecars and extracted-package self-replay must be regenerated from the final tree.

Supported-scope distinction:

| Scope | Current classification |
|---|---|
| v11A supported-local path | active hardening scope, candidate only until final gates pass |
| v11B region/subtraction | minimal executable seed only |
| v11C/federation/self-hosting | reserved/quarantined |
| Cloud/provider-native loops | deferred |
| Broad autonomy | deferred |

Active super-pass truth documents are `matrices/SUPER_PASS_BACKLOG_1020.csv`, `handoffs/super-pass/`, `docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md`, `docs/super-pass/SUPPORT_TRACEABILITY.md`, and the final package/replay receipts once generated.

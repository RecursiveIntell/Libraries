# Source Basis — P31A

Record date: `2026-05-29`

This file records the active P31A source basis.

## Current Run

| Field | Value |
|---|---|
| Current run | P31A Recovery |
| Prior run | P30 Codex Super Pass |
| Workspace root | `AiDENs/` |
| Expected parent workspace | `/home/sikmindz/Coding/Libraries` |
| Rust edition | 2021 |
| Minimum Rust version | 1.76 |
| Final strict gate | pending — P31A Phase 09 final hostile audit |

## Inputs

- P31A hostile audit finish pack: `aidens_hostile_audit_finish_pack.zip`
- P31A sidecars: `AiDENs-aidens-next-codex-context-20260529T054601Z.*`
- P30 status/evidence/package sidecars, treated as failure evidence and candidate implementation evidence, not a clean release basis.
- Active status/support docs: `STATUS.md`, `SUPPORT_PROFILE.md`, `SOURCE_BASIS.md`, `README.md`, `AGENTS.md`.

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
| Strict P31A final verifier | `exact_check` | Only available after the final P31A command bar passes. |
| Skipped cargo or degraded replay | `degraded_exact_check` | Static/package checks may run, but cargo-backed replay proof is absent. |

## Active Truth Docs

- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `P31A_*` (if any)
- `matrices/P31A_*` (if any)
- `docs/codex-runs/CURRENT_RUN.json`
- `docs/codex-runs/CURRENT_RUN.md`

Materials from codex runs prior to P31A are evidence, not active support claims, unless explicitly cited as prior-run evidence.

**Ledger reference:** `docs/codex-runs/CURRENT_RUN.json`  
**Support label:** `p31a-certified-release-truth-repair`  
**Status:** `certified`

## Supported-scope distinction

| Scope | Current classification |
|---|---|
| v11A supported-local path | candidate only until P31A gates pass |
| v11B region/subtraction | deferred |
| v11C/federation/self-hosting | reserved/quarantined |
| Cloud/provider-native loops | deferred |
| Broad autonomy | deferred |

## Active Hardening

The active hostile audit finish pack is `aidens_hostile_audit_finish_pack.zip`. Its evidence and plan docs are task material, not source truth.

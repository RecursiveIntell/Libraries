# Source Basis — P31B Verification Repair

Record date: `2026-05-29`

This file records the active P31B source basis.

## Current Run

| Field | Value |
|---|---|
| Current run | P31B Verification Repair |
| Prior run | P31A (decertified) |
| Last certified run | P30 (P31A decertified) |
| Certification status | candidate |
| Support label | p31b-verification-repair-candidate |
| Workspace root | `AiDENs/` |
| Expected parent workspace | `/home/sikmindz/Coding/Libraries` |
| Rust edition | 2021 |
| Minimum Rust version | 1.76 |
| Final strict gate | passed — P31B all 17 verify_current gates PASS |

## Inputs

- P31B Hermes finish pack: `aidens_p31b_hermes_finish_pack.zip`
- P31A sidecars: `AiDENs-aidens-codex-context-20260529T065449Z.*`
- P31A hostile audit finish pack: `aidens_hostile_audit_finish_pack.zip`
- P31A status/evidence/package sidecars, treated as false-certification evidence.
- Active status/support docs: `STATUS.md`, `SUPPORT_PROFILE.md`, `SOURCE_BASIS.md`, `README.md`, `AGENTS.md`.

## Canonical Sibling Ownership

AiDENs depends on sibling crates through path dependencies under the parent Libraries workspace. If those siblings are absent, cargo/package replay must classify the result as environment-blocked, not clean.

Key sibling dependencies: `kernel-conformance`, `aidens-contracts`, `aidens-tool-kit`, `aidens-cli`, `aidens-boundary-kit`.

## Replay Modes

| Mode | Classification | Meaning |
|---|---|---|
| Local parent workspace present | `sibling_workspace_present` | Cargo checks may run against sibling path dependencies. |
| Local parent workspace absent or incomplete | `sibling_workspace_missing` | Cargo/package replay is environment-blocked and must not be called clean. |
| Strict P31B final verifier | `exact_check` | Only available after the final P31B command bar passes. |
| Skipped cargo or degraded replay | `degraded_exact_check` | Static/package checks may run, but cargo-backed replay proof is absent. |

## Active Truth Docs

- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `docs/codex-runs/CURRENT_RUN.json`
- `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`

## Source Basis Declarations

The source bundle is a minimal executable seed only. All reserved/quarantined artifacts are classified in the artifact classification ledger. final package sidecars and extracted-package self-replay are documented in CURRENT_RUN.json; the self-replay gate has an environmental blocker.
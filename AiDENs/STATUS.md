# AiDENs Status — P31A Recovery

Record date: `2026-05-29`
Ledger: `docs/codex-runs/CURRENT_RUN.json`
Support label: `p31a-blocked-release-truth-repair`
Certification status: `blocked`

This is the active P31A release-truth recovery ledger. P31A repairs the run identity drift, root Markdown contamination, verification gate misalignment, and static safety findings discovered during the 2026-05-29 hostile audit. It does not add runtime features or claim boundary compiler integration.

## Current Run

| Field | Value |
|---|---|
| Current run | P31A Recovery |
| Prior run | P30 Codex Super Pass |
| Last certified run | P30 |
| Current status | Phase 00 preflight complete; Phase 01 release-truth ledger closure in progress |
| Declared path | supported-local operator/agent/coding-agent path with receipts, execution context, manifests, proof/debt/degradation state |
| Final package status | pending — no package claim until Phase 08 passes |

## P31A Phase Ledger

| Phase | Evidence | Result |
|---|---|---|
| 00 | `docs/codex-runs/P31A_RECOVERY/preflight_report.md` | evidence lock and repo state freeze |
| 01 | `docs/codex-runs/P31A_RECOVERY/phase_01_report.md` | release-truth ledger closure |
| 02 | `docs/codex-runs/P31A_RECOVERY/phase_02_report.md` | root Markdown and Codex artifact classification |
| 03 | `docs/codex-runs/P31A_RECOVERY/phase_03_report.md` | verification and support gate repair |
| 04 | `docs/codex-runs/P31A_RECOVERY/phase_04_report.md` | static hard-blocker repair |
| 05 | `docs/codex-runs/P31A_RECOVERY/phase_05_report.md` | boundary compiler ownership decision |
| 06 | `docs/codex-runs/P31A_RECOVERY/phase_06_report.md` | one real boundary/receipt vertical slice |
| 07 | `docs/codex-runs/P31A_RECOVERY/phase_07_report.md` | build/test command bar |
| 08 | `docs/codex-runs/P31A_RECOVERY/phase_08_report.md` | strict package and extracted replay |
| 09 | `docs/codex-runs/P31A_RECOVERY/phase_09_report.md` | final hostile audit and handoff |

## Current Support Posture

P31A may claim only `in-progress` / `release-truth-repair` until the final command bar and extracted package replay pass. Allowed labels are limited to:

- `p31a-release-truth-repair`
- `p31a-supported-local-plus`

P31A must not claim completion of v11A scope, completion of v11B scope, completion of v11C scope, broad autonomy readiness, readiness for production cloud deployment, or canonical ownership of memory, governance, kernel, provider/tool, schema, federation, or ID truth.

## Non-Claims

AiDENs is not ready for production cloud deployment, broadly autonomous, completion of v11B scope, completion of v11C scope, or a replacement for canonical memory/governance/kernel/runtime crates.

## Previous Run Carry-Forward

P30 implementation work remains useful candidate evidence. P31A does not inherit the P30 release claim until it repairs:

- archive/current-run identity drift;
- active artifact classification;
- verifier wrapper delegation;
- manifest path resolution;
- package self-replay from an extracted zip;
- `p30_guard` hard and broad findings.

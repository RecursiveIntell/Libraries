# High-Risk Layer Quarantine

Record date: `2026-05-07`

This ledger is the Phase 19 support boundary for high-risk sibling and control layers. It is a quarantine record, not an audit pass for those layers. AiDENs remains an operator/orchestration/display/runtime surface and does not become the canonical owner for these domains.

| Layer | Local evidence inspected | Phase 19 status | Support effect |
|---|---|---|---|
| `forge-pilot` | Sibling crate present at `../forge-pilot` with orchestration, repo-chat, bootstrap, provider, and receipt modules. | quarantined | Not part of AiDENs supported-local completion claims until separately audited and activated by an explicit gate. |
| `effect-runtime` | Sibling crate present at `../effect-runtime` with effect, observation, compensation, and v25 modules. | quarantined | Effects cannot widen AiDENs support labels beyond receipt-backed local orchestration. |
| verification pipeline | Sibling crates present at `../verification-policy`, `../verification-control`, `../verification-calibration`, and `../verification-adjudication`. | quarantined | Verification policy/control/adjudication cannot be treated as an AiDENs-owned correctness oracle. |
| federation | Sibling crates present at `../federated-settlement`, `../remote-oracle-admission`, and `../mechanism-runtime`. | quarantined | Federation, remote oracle admission, and mechanism runtime remain outside supported-local scope. |
| attestation | Sibling crate present at `../attestation-exchange`. | quarantined | Attestation exchange cannot satisfy AiDENs package/replay gates without its own audit evidence. |
| `authority-delegation` | Sibling crate present at `../authority-delegation` with capability, emergency, and separation-of-duty modules. | quarantined | Delegated authority cannot be promoted into broad autonomy or cloud authority claims. |
| `recursive-kernel-core` | Sibling crate present at `../recursive-kernel-core`. | quarantined | Kernel correctness remains a canonical sibling concern and is not claimed by AiDENs. |

## Guardrail

The supported AiDENs scope may reference these layers only as external/canonical owners or future integration surfaces. A supported-local AiDENs run must not imply audited correctness, federation readiness, attestation authority, recursive-kernel validity, or delegated autonomy from these quarantined layers.

Any later widening requires all of the following:

- a layer-specific audit report;
- semantic or hostile tests for the claimed behavior;
- receipt-backed activation evidence;
- an updated issue matrix row;
- known-limitations closure or replacement with a stronger evidence record.

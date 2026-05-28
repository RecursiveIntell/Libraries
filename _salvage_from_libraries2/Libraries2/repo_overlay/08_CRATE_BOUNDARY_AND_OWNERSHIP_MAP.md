# Crate boundary and ownership map — post-v24 profiles

| Profile | Primary owner | Secondary consumers | Notes |
|---|---|---|---|
| P1 | `verification-policy` | `attestation-exchange`, `constitutional-memory`, `knowledge-runtime`, `continuity-runtime` | Policy-owned because privacy, redaction, retention, and audit extraction are admissibility constraints before they are transport details. |
| P2 | `verification-policy` | `remote-oracle-admission`, `federated-settlement`, `knowledge-runtime`, `attestation-exchange` | Policy-owned because residency and tenancy are cross-cutting boundary constraints consumed by exchange, replay, and federation. |
| P3 | `authority-delegation` | `verification-policy`, `verification-control`, `effect-runtime`, `continuity-runtime` | Authority-owned because role/approval/recusal semantics extend delegated-authority law directly. |
| P4 | `assurance-runtime` | `verification-control`, `verification-adjudication`, `continuity-runtime`, `constitutional-memory` | Assurance-owned because regime mapping and recertification bind directly to release readiness and certification artifacts. |
| P5 | `assurance-runtime` | `continuity-runtime`, `mechanism-runtime`, `verification-control`, `effect-runtime` | Assurance-owned with continuity consumers because hazard doctrine drives both release gating and incident playbooks. |
| P6 | `attestation-exchange` | `assurance-runtime`, `remote-oracle-admission`, `verification-policy`, `continuity-runtime` | Exchange-owned because vendor adapters are principally about translation, trust roots, revocation, and external evidence admission. |
| P7 | `continuity-runtime` | `verification-policy`, `authority-delegation`, `knowledge-runtime`, `llm-tool-runtime` | Continuity-owned because taxonomy, severity, clocks, and routes are incident-time operational law. |

## No-workspace-expansion rule

This pass does **not** add a new owner crate.
The profile suite lands through already-present crates plus `contract-schema-gen`, `stack-ids`, and root docs.

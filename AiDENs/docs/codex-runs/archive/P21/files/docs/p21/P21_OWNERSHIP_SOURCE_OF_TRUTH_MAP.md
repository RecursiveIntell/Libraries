# P21 Ownership / Source-of-Truth Map

## Hard rule

AiDENs owns orchestration, plans, profiles, product-facing configs, CLI UX, receipt routing, agency policy application, and generated-agent scaffolds.

AiDENs does **not** own canonical truth semantics for memory, evidence, kernel inference, verification, repair, federation, mechanism search, or primitive identity.

## Canonical map

| Domain | Canonical source | AiDENs allowed behavior |
|---|---|---|
| primitive IDs/digests/traces | `../stack-ids` from `libraries` | import/re-export/use |
| raw evidence/export truth | `../semantic-memory-forge` | configure/delegate |
| bridge/import backpointers | `../forge-memory-bridge` | configure/delegate |
| queryable memory projection | `../semantic-memory` | configure/delegate |
| runtime view/widening | `../knowledge-runtime` | configure/delegate/report |
| tool runtime contracts/receipts | `../llm-tool-runtime` | configure/delegate/report |
| kernel recursive inference | `../recursive-kernel-core`, `../constraint-compiler`, `../kernel-execution`, `../kernel-oracles`, `../kernel-conformance` | configure/delegate/report |
| verification/control/adjudication | `../verification-*` | configure/delegate/report |
| attested exchange/federation | `../attestation-exchange`, `../remote-oracle-admission`, `../federated-settlement` | defer/configure/report only unless wired |
| mechanism search | `../mechanism-runtime` | defer/configure/report only unless wired |
| agent profiles/plans/templates | AiDENs | own |
| test-agent/operator UX | AiDENs | own |
| agency/influence policy at AiDENs output boundary | AiDENs | own v0.2 policy surface; emit receipts; do not pretend universal ethics proof |

## Recall / Recall-Coding role

Recall and Recall-Coding are source examples, not canonical owners of AiDENs semantics. Use them to extract:

- app wiring patterns;
- daemon/session/IPC pitfalls;
- tool routing lessons;
- developer UX expectations;
- profile defaults;
- approval/safe-mode lessons.

Do not copy Recall-specific state models or UI assumptions into AiDENs core.

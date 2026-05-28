# P20.2 Ownership and Source-of-Truth Map

| Concern | Canonical owner | AiDENs role |
|---|---|---|
| IDs / digests / trace primitives | `stack-ids` | consume/re-export where useful |
| raw evidence / export truth | `semantic-memory-forge` | delegate |
| bridge/import semantics | `forge-memory-bridge` | delegate |
| queryable memory projections | `semantic-memory` | delegate |
| runtime view/retrieval/degradation | `knowledge-runtime` | delegate |
| kernel inference / oracle slices | kernel crates | delegate |
| verification/control/adjudication | `verification-*` crates | delegate |
| provider/tool receipts | `llm-tool-runtime` + AiDENs receipt surface | orchestrate, do not invent hidden truth |
| runner/profile/agent construction | AiDENs | own wiring/profile/config/runner logic |
| agency/influence gate for AiDENs outputs | AiDENs agency layer, future canonical candidate | own v0.1 policy gate; mark heuristic |
| test reference models | `aidens-testkit` | pure reference only |
| production integration tests | `aidens-integration-tests` | exercise full AiDENs paths |

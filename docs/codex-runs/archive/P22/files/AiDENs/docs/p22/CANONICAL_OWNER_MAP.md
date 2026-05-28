# Canonical Owner Map — AiDENs Contract Ownership Collapse

## Principle

AiDENs may direct, wire, render, summarize, schedule, and report. AiDENs must not define canonical stack truth.

## Canonical owner matrix

| Concept family | Canonical owner crate | AiDENs allowed role | Forbidden AiDENs role |
|---|---|---|---|
| Opaque IDs, content digests, trace IDs | `stack-ids` | Re-export/use IDs; display summaries | Local canonical digest, deterministic artifact ID, content-address law |
| Forge evidence/export truth | `semantic-memory-forge` | Route/export/report with backpointers | Local evidence/verification truth objects |
| Forge-to-memory transforms | `forge-memory-bridge` | Call bridge, display transform report | Bridge policy engine or local import reinterpretation |
| Queryable projected memory | `semantic-memory` | Query and display memory results | Local memory truth store |
| Runtime view/degradation/widening | `knowledge-runtime` | Request/report runtime decisions | Local widening/degradation semantics |
| Tool descriptors/calls/results/receipts | `llm-tool-runtime` | Provide tool implementations and display receipts | Local canonical tool-call universe |
| Verification/control records | `verification-control`, `verification-policy`, `verification-adjudication`, `verification-calibration` | Invoke/report/backpointer | Local promotion/repair/adjudication truth |
| Recursive kernel contracts | `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance` | Compile/execute/report with witnesses | Local inference semantics |
| Attestation exchange | `attestation-exchange` | Re-export and display | Local `AttestationEnvelopeV1` |
| Federated settlement | `federated-settlement` | Re-export and display | Local `SharedDispositionV1` or `SettlementCaseV1` |
| Mechanism/theory artifacts | `mechanism-runtime` | Re-export and display | Local `TheoryVersionV1`, `TheoryRefuterSuiteV1`, `HypothesisLibraryV1` |
| Remote oracle admission | `remote-oracle-admission` | Call/report admission | Local remote oracle trust/admission law |
| Schema generation | `contract-schema-gen` | Run or consume generated schemas | Local canonical schema registry |
| Closed-loop pilot | `forge-pilot` | Invoke/report pilot receipts | Local control-loop truth |
| Recall/Recall-Coding wiring | `~/Coding/Recall`, `~/Coding/Recall-Coding` | Reference patterns only | Copy app-local truth as canonical AiDENs law |
| Supplemental crates | `~/Coding/Libraries2` | Reference only if canonical owner missing | Replace canonical `~/Coding/Libraries` ownership |

## Current exact P0 duplicate public types

| Type | AiDENs local source | Canonical owner |
|---|---|---|
| `AttestationEnvelopeV1` | `crates/aidens-contracts/src/lib.rs` | `attestation-exchange` |
| `SharedDispositionV1` | `crates/aidens-contracts/src/lib.rs` | `federated-settlement` |
| `SettlementCaseV1` | `crates/aidens-contracts/src/lib.rs` | `federated-settlement` |
| `TheoryRefuterSuiteV1` | `crates/aidens-contracts/src/lib.rs` | `mechanism-runtime` |
| `TheoryVersionV1` | `crates/aidens-contracts/src/lib.rs` | `mechanism-runtime` |
| `HypothesisLibraryV1` | `crates/aidens-contracts/src/lib.rs` | `mechanism-runtime` |

## Ownership decision rule

When a local AiDENs type appears to overlap a canonical concept:

1. Search canonical crates first.
2. If exact canonical type exists: delete local type or convert to `pub use`.
3. If canonical type does not exist but concept belongs to canonical law: quarantine and stop.
4. If concept is product/report/display-only: keep but rename/document as non-authoritative and add backpointers.
5. If ambiguity remains: halt; do not create a compatibility shim.

# AiDENs — Complete Completion Status

**Status**: P00-P19 all passes implemented. 502 tests, 0 failures. 18 verification gates pass.

## Pass completion summary

| Pass | Title | Status |
|---|---|---|
| P00 | Source lock, fake-ready freeze | DONE |
| P01 | API honesty | DONE |
| P02 | Provider runtime truth | DONE |
| P03 | Turn executor | DONE |
| P04 | Capability gate, permits | DONE |
| P05 | Durable receipts | DONE |
| P06 | Boundary compiler | DONE |
| P07 | Schema registry | DONE |
| P08 | Reference interpreters | DONE |
| P09 | Episode memory | DONE (semantic-memory wired) |
| P10 | Coding tools | DONE |
| P11 | Queue, schedule, daemon | DONE |
| P12 | Verification/repair/governance | DONE (VerificationPlanV1, ClaimEvidenceBundleV1, GovernanceDecisionV1) |
| P13 | Multi-view runtime | DONE (ViewDisclosureV1, RuntimeQueryProvenanceV1) |
| P14 | Product surface | DONE (CLI commands) |
| P15 | Regional decoder | DONE (RegionContractV1, ResidualEnvelopeV1, SyndromeEnvelopeV1, RegionConvergenceReportV1) |
| P16 | Lawful subtraction | DONE (SubtractionPlanV1, SupportCoreV1, RemovalFrontierV1, InvariantBudgetV1, CompactionReceiptV1) |
| P17 | Attested federation | DONE (AdmissionDecisionV1, FederationAdapter) |
| P18 | Mechanism/theory | DONE (MechanismBundleV1, RefuterSuiteV1, MechanismAdapter) |
| P19 | Final release audit | DONE (all gates pass) |

## Crate Inventory

| Crate | Status |
|---|---|
| `aidens` | implemented |
| `aidens-agency-kit` | implemented |
| `aidens-app-kit` | implemented |
| `aidens-arbiter-kit` | implemented |
| `aidens-boundary-kit` | implemented |
| `aidens-budget-kit` | implemented |
| `aidens-capability-kit` | implemented |
| `aidens-cli` | implemented |
| `aidens-config` | implemented |
| `aidens-contracts` | implemented |
| `aidens-daemon-kit` | implemented |
| `aidens-delegation-kit` | implemented (federation wired) |
| `aidens-governance-kit` | implemented (verification wired) |
| `aidens-integration-tests` | implemented |
| `aidens-kernel-kit` | implemented (regional decoder + lawful subtraction + mechanism) |
| `aidens-memory-kit` | implemented (semantic-memory adapter wired) |
| `aidens-permit-kit` | implemented |
| `aidens-plan-kit` | implemented |
| `aidens-profile-coding` | implemented |
| `aidens-profile-daemon` | scaffold-only (honest) |
| `aidens-profile-desktop` | scaffold-only (honest) |
| `aidens-profile-memory` | scaffold-only (honest) |
| `aidens-profile-research` | scaffold-only (honest) |
| `aidens-provider-kit` | implemented |
| `aidens-queue-kit` | implemented |
| `aidens-receipts` | implemented |
| `aidens-repair-kit` | implemented |
| `aidens-runner` | implemented |
| `aidens-schedule-kit` | implemented |
| `aidens-security-kit` | implemented |
| `aidens-testkit` | implemented |
| `aidens-tool-kit` | implemented |
| `aidens-wake-kit` | implemented |
| `boundary-compiler-core` | implemented |

## Key integrations

- semantic-memory: wired via SemanticMemoryAdapter (verify_integrity, replay_search_receipt, graph_traversal, compressed_search)
- claim-ledger: types available via aidens-contracts
- TurboQuant: compressed search via semantic-memory-integration feature

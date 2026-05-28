# P20 Ownership and Source-of-Truth Map

## Canonical source-of-truth matrix

| Concept | Owner | AiDENs rule |
|---|---|---|
| Stack IDs, digests, trace IDs | `stack-ids` | Use/re-export; do not locally redefine |
| Episode identity | `semantic-memory` / canonical episode contracts | Do not invent local episode identity law |
| Raw evidence/export packages | `semantic-memory-forge` | Delegate; AiDENs may report only |
| Bridge import/export transform | `forge-memory-bridge` | Delegate; preserve digests/backpointers |
| Queryable projection truth | `semantic-memory` | Delegate; do not store alternate truth |
| Runtime view/widening semantics | `knowledge-runtime` | Delegate; disclose degradation |
| Kernel operators, witnesses, syndromes, residuals, oracles | `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance` | Delegate; no local algorithmic replacement |
| Verification policy/control/adjudication/calibration | `verification-*` crates | Delegate; no local control law replacement |
| Tool/provider runtime contracts | `llm-tool-runtime` | Delegate where possible; AiDENs adapter receipts allowed |
| Closed-loop orchestration | `forge-pilot`; AiDENs runner consumer-only | Do not self-promote pilot output into truth |
| Contract/schema generation | `contract-schema-gen` | Use canonical generation; no handwritten replacement if canonical exists |
| Agency/influence governance | `aidens-agency-kit` v0.1 boundary layer | Gate AiDENs UI/runner influence; later may be promoted to canonical crate |

## Local AiDENs type naming rule

Allowed local names should communicate non-authoritative use:

- `Aidens...ReportV1`
- `Aidens...DisplayV1`
- `Aidens...AdapterReceiptV1`
- `Aidens...ConfigV1`
- `Aidens...CliOutputV1`

Forbidden/unsafe local names unless re-exported from canonical owner:

- `CanonicalEvidenceBundleV1`
- `CanonicalEpisodeBundleV1`
- `ClaimTruthV1`
- `BitemporalTruthV1`
- `KernelWitnessV1`
- `VerificationDecisionV1`
- `RepairLawV1`

# SourceTruthAmbiguityRecord

## ID

`STA-20260513-001`

## Date Recorded

`2026-05-13T00:00:00Z`

## Ambiguous Concept

- IDs: SCR-specific decision receipt ID ownership is unresolved.
- Artifacts: SCR local fixture artifacts are clear, but upstream artifact truth
  remains domain-owned.
- Evidence: SCR evidence refs must remain opaque; canonical evidence owner is
  not established for P0A.
- Provenance: SCR provenance basis refs must remain opaque; canonical provenance
  owner depends on the source domain.
- Receipts: SCR `ControlDecisionReceiptV1` ownership and relationship to
  `verification-control::ControlReceipt` are unresolved.
- Policies: SCR local reference policy ownership is clear only for fixture
  evaluation; workspace policy truth remains with existing policy crates.
- Schemas: local SCR schema generation path is clear, but integration into the
  containing workspace generator is unresolved.
- Time/bitemporal fields: local `valid_time_basis` and `recorded_time`
  representation is unresolved.
- Errors: local SCR errors are acceptable only for SCR parsing, validation, and
  evaluation failures; cross-domain error ownership remains domain-local.

## Observed Candidates

| Candidate | Path/crate | Evidence | Risk |
|---|---|---|---|
| `stack-ids` | `/home/sikmindz/Coding/Libraries/stack-ids` | Crate docs state it is the single source of truth for cross-crate identity types and owns `ContentDigest`. | Creating local global ID newtypes would shadow existing identity law. |
| `verification-control::ControlReceipt` | `/home/sikmindz/Coding/Libraries/verification-control` | Existing control-plane receipt type with IDs, trace context, time fields, citation context, and replay links. | Forcing SCR receipts into this type may lose SCR-specific decision basis fields. |
| Domain receipt families | `effect-runtime`, `authority-delegation`, `attestation-exchange` | Existing effect, authority, and transparency receipt types. | Reusing a domain receipt for SCR decisions may imply execution/provenance truth SCR does not own. |
| `contract-schema-gen` | `/home/sikmindz/Coding/Libraries/contract-schema-gen` | Existing generated schema owner and drift checker for the containing workspace. | A separate generator may duplicate conventions unless explicitly scoped to SCR. |
| `verification-policy` | `/home/sikmindz/Coding/Libraries/verification-policy` | Existing policy snapshot, policy decision, and execution permit surfaces. | SCR policy canonicalization could be mistaken for workspace policy truth if not scoped. |
| `knowledge-runtime` bitemporal fields | `/home/sikmindz/Coding/Libraries/knowledge-runtime` | Existing `valid_as_of` and `recorded_as_of` query provenance fields. | Importing query semantics into SCR would violate P0A non-goals. |

## Why Ownership Is Unclear

SCR-P0A requires a replayable decision receipt with fields not present as a
single exact match in the observed receipt candidates. The containing workspace
already has control-plane and domain receipt types, but SCR decision receipts
must record evaluator-specific rule checks, axes, derived pressures, rejected
actions, reason codes, policy hash, and input hash.

The target bundle also requires `valid_time_basis` and `recorded_time`, while
observed crates use a mix of RFC3339 strings, generated timestamps, and
bitemporal query coordinates. No existing owner was observed that directly
defines SCR evaluation time basis.

## Blocked Work

Do not implement mutation or integration work that claims canonical ownership
over:

- upstream artifact truth;
- upstream evidence truth;
- upstream provenance truth;
- workspace-wide receipt semantics;
- workspace-wide time/bitemporal semantics.

## Allowed Local Placeholder, If Any

Allowed only as adapter-scoped, explicitly non-canonical Phase 1 types:

- opaque actor, permit, subject, environment, evidence, and artifact refs;
- local `ControlDecisionReceiptV1` if documented as a SCR evaluation receipt,
  not a workspace receipt replacement;
- local RFC3339 string validation for `recorded_time`;
- local `valid_time_basis` model scoped only to SCR fixture evaluation.

## Required Resolution

Before public integration with the containing workspace, the operator must
choose one of:

- register SCR-specific IDs and schemas through existing workspace owner crates;
- keep SCR-P0A as a standalone reference workspace with adapter traits;
- define an explicit conversion from SCR decision receipts to
  `verification-control::ControlReceipt` without losing SCR receipt law fields.

## Operator Decision

Pending automated phase gate remediation after phase completion.

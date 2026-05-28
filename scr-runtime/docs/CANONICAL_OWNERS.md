# Canonical Owners

SCR-P0A is not a source of truth for upstream IDs, artifacts, evidence,
provenance, schemas, repository state, or execution state. Where an owner is
unclear, P0A must use adapter-only local references until the operator resolves
ownership.

| Concept | Canonical owner or adapter plan |
|---|---|
| IDs | `stack-ids` is the observed cross-crate owner for opaque IDs and content digests. SCR-P0A should reuse existing IDs where there is an exact semantic match. If a SCR-specific ID is needed, register it through `stack-ids` or keep it local and explicitly non-canonical until approved. |
| Artifacts | Domain split. Existing owners include `effect-runtime`, `authority-delegation`, `attestation-exchange`, `verification-control`, and `semantic-memory-forge`. SCR-P0A may refer to artifacts only through opaque adapter refs and must not assert artifact truth. |
| Evidence references | Adapter plan: opaque evidence refs supplied in `ControlEvaluationInputV1`. Existing candidate: `semantic-memory-forge::ExportEvidenceRef`; related planning owner: `assurance-runtime`. SCR-P0A must not fetch evidence or declare evidence canonical. |
| Provenance references | Adapter plan: carry provenance basis refs only. Existing candidates include `attestation-exchange` for attestation/provenance envelopes and `knowledge-runtime::RuntimeQueryProvenanceV1` for query provenance. SCR-P0A must not become a provenance verifier. |
| Receipts | UNKNOWN for SCR decision receipts. Existing candidate owners include `verification-control::ControlReceipt` and domain receipt families. Phase 1 must use adapter-only semantics until the operator accepts a local `ControlDecisionReceiptV1` scope or a `ControlReceipt` adapter. |
| Policies | `verification-policy` owns existing policy/permit surfaces. SCR-P0A owns only its local reference policy model and canonical policy hash for deterministic fixture evaluation. It must not replace workspace policy truth. |
| Schemas | Existing workspace pattern owner is `contract-schema-gen`; SCR-P0A must generate local schemas from Rust types into `schemas/generated/` and fail on drift. |
| Time/bitemporal fields | UNKNOWN for SCR inputs. Existing patterns use RFC3339 strings, `chrono` validation, and bitemporal query fields in `knowledge-runtime`. Phase 1 must define local evaluation time basis without claiming global time ownership. |
| Errors | Domain crates own domain errors. SCR-P0A may own only local parsing, schema validation, policy canonicalization, and evaluator errors, preferably with stable `kind()` discriminants. |

## Ambiguity Handling

The unresolved owners for SCR-specific receipts and time basis are recorded in
`docs/SourceTruthAmbiguityRecord.md`.

Until resolved, SCR-P0A implementation must:

- use opaque refs for upstream artifacts, evidence, provenance, actors,
  subjects, environments, and permits;
- emit `SourceTruthAmbiguityRecord` when asked to mutate or integrate against an
  unclear owner;
- keep all P0A logic deterministic, fixture-driven, and local to proposed-action
  control evaluation;
- avoid runtime calls to upstream memory, retrieval, tool, Recall, or AiDENs
  systems.

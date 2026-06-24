# P28 v11A Contract Targets

## Artifact and lifecycle

```rust
ArtifactEnvelopeV1 {
  artifact_ref,
  family,
  schema_version,
  content_digest,
  issuer,
  created_recorded_time,
  namespace,
  authority_class,
  lifecycle_state,
  bitemporal_applicability,
  signature_or_attestation_refs,
}
```

```rust
ArtifactManifestV1 {
  manifest_id,
  inputs,
  outputs,
  missing_or_opaque_refs,
  canonicalization_profile,
  schema_identities,
  created_recorded_time,
}
```

```rust
ArtifactTransitionReceiptV1 {
  receipt_id,
  artifact_ref,
  previous_state,
  new_state,
  triggering_operator,
  actor,
  execution_context_ref,
  proof_refs,
  degradation_refs,
  recorded_time,
}
```

## Operator/effects

Effects must include at least:

- reads truth
- projects truth
- proposes inference
- emits receipt
- widens view
- repairs state
- subtracts structure
- changes promotion
- changes schema
- crosses trust boundary
- affects future execution
- affects user agency

## Execution evidence

`ExecutionContextEnvelopeV1` and `ToolCallReceiptV1` must use digests/refs rather than duplicating large payloads. Raw payloads may be retained in bounded/redacted stores with digest pointers.

## Proof economy

`ProofDebtLedgerV1` must restrict allowed uses. `ProofWaiverReceiptV1` is queryable governance state and cannot be interpreted as proof.

## Boundary compiler

Boundary compiler profiles must distinguish:

- strict parse
- reject duplicate keys
- schema validate
- canonicalize
- allow repair only with receipt
- treatment integrity required for material evidence/patch/import paths

## Compatibility

Existing P27 structs may remain as legacy/display/admitted facades, but they must not be advertised as canonical v11 receipts unless they satisfy v11 required fields.

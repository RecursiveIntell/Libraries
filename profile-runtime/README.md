# profile-runtime

Canonical effective constitution and profile composition runtime for the local-first AI systems stack.

## Usage

```rust
use profile_runtime::{
    ApplicabilityContext, EffectiveConstitution, CompositionReceipt, ProfileSet,
};
```

## What this crate owns

- applicability context selection
- profile-set normalization
- fold and conflict rule sets
- effective constitution and compiled obligations
- explicit conflicts, exceptions, and policy impact diffs

This crate does not own family-specific profile content. It consumes typed
surfaces from existing owner crates and compiles them into one replayable,
context-specific constitutional answer.

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`ApplicabilityContextId`, `ProfileSetId`, `EffectiveConstitutionId`, `CompositionReceiptId`, etc.)
- `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `continuity-runtime` -- profile content surfaces
- `verification-policy` -- policy profile types

**Depended on by:**
- `contract-schema-gen`

## stack-ids integration

Uses `ApplicabilityContextId`, `ProfileSetId`, `EffectiveConstitutionId`,
`CompositionReceiptId`, `CompositionRuleSetId`, and
`ProfileExceptionBundleId` from `stack-ids` for composition identity and
constitutional citation.

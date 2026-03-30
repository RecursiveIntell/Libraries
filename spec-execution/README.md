# spec-execution

Typed surface crate for v20 spec bundles, normative ASTs, generated artifacts, proof obligations, and veto/challenge surfaces.

## Usage

```rust
use spec_execution::{SpecBundleV1, NormativeAstV1, ProofObligationSetV1};
```

## Owns

- spec bundles
- normative ASTs
- generated schema/interpreter/corpus/migration bundles
- proof obligation sets and proof evaluation receipts
- self-hosting build receipts
- human veto and meta challenge bundles

## Does not own

- silent auto-amendment
- authoritative constitutional admission
- hidden override over human governance

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`SpecBundleId`, `NormativeAstId`, `ProofObligationSetId`, `ContentDigest`, etc.)

**Depended on by:**
- `kernel-conformance`
- `contract-schema-gen`

## stack-ids integration

Uses `SpecBundleId`, `NormativeAstId`, `GeneratedSchemaBundleId`,
`GeneratedInterpreterBundleId`, `GeneratedConformanceCorpusId`,
`GeneratedMigrationPlanId`, `ProofObligationSetId`,
`ProofEvaluationReceiptId`, `HumanVetoBundleId`, `MetaChallengeBundleId`,
`SelfHostingBuildReceiptId`, `ContentDigest`, and `SurfaceStatus` from
`stack-ids`.

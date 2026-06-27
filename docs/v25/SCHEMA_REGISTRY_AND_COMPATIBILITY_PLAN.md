# Schema registry and compatibility plan — v25

## Canonical owner mapping

| Artifact family | Owner crate | Canonical schema path | Canonical example path | Compatibility class |
|---|---|---|---|---|
| `ApplicabilityContextV1` | `profile-runtime` | `schemas/applicability-context-v1.schema.json` | `examples/applicability-context-v1.example.json` | additive-compatible |
| `ProfileSetV1` | `profile-runtime` | `schemas/profile-set-v1.schema.json` | `examples/profile-set-v1.example.json` | behavioral-compatible-but-proof-requiring |
| `CompositionRuleSetV1` | `profile-runtime` | `schemas/composition-rule-set-v1.schema.json` | `examples/composition-rule-set-v1.example.json` | breaking-with-migration |
| `CompositionReceiptV1` | `profile-runtime` | `schemas/composition-receipt-v1.schema.json` | `examples/composition-receipt-v1.example.json` | additive-compatible |
| `EffectiveConstitutionV1` | `profile-runtime` | `schemas/effective-constitution-v1.schema.json` | `examples/effective-constitution-v1.example.json` | behavioral-compatible-but-proof-requiring |
| `CompiledObligationSetV1` | `profile-runtime` | `schemas/compiled-obligation-set-v1.schema.json` | `examples/compiled-obligation-set-v1.example.json` | behavioral-compatible-but-proof-requiring |
| `CompositionConflictSetV1` | `profile-runtime` | `schemas/composition-conflict-set-v1.schema.json` | `examples/composition-conflict-set-v1.example.json` | additive-compatible |
| `ProfileExceptionBundleV1` | `profile-runtime` | `schemas/profile-exception-bundle-v1.schema.json` | `examples/profile-exception-bundle-v1.example.json` | behavioral-compatible-but-proof-requiring |
| `PolicyImpactDiffV1` | `profile-runtime` | `schemas/policy-impact-diff-v1.schema.json` | `examples/policy-impact-diff-v1.example.json` | additive-compatible |

## Supporting profile-family surfaces already consumed by v25

| Surface | Current owner |
|---|---|
| `EffectPolicyProfileV1` | `verification-policy` |
| `DelegationPolicyProfileV1` | `authority-delegation` |
| `ReleasePolicyProfileV1` | `verification-policy` |
| `ContinuityPolicyProfileV1` | `verification-policy` |
| `ResidencyPolicyProfileV1` / `TenantBoundaryProfileV1` | `verification-policy` |
| vendor translation and revocation families | `attestation-exchange` |
| regulated deployment and hazard families | `assurance-runtime` |
| incident taxonomy and routing families | `continuity-runtime` |

## Compatibility rules

1. Any change to `CompositionRuleSetV1` semantics is **constitutional work**, not ordinary shape-only schema evolution.
2. Any change that alters fold class, conflict behavior, or exception admissibility requires:
   - migration owner,
   - compatibility window,
   - updated fixture corpus,
   - and new proof artifacts.
3. Any addition of new obligation families is additive only if:
   - consumers may safely ignore them, **and**
   - their absence cannot silently widen permissiveness.
4. Any field that changes how a blocked path becomes admissible is proof-requiring by default.

## Registry truth rule

`contracts/schemas/v25/manifest.json` is the canonical manifest for the v25 family set.
The repo-truth and JSON-surface checks treat it as the authoritative list for the v25 wave.

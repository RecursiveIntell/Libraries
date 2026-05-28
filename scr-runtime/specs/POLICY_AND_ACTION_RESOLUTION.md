# Policy and Action Resolution

## Human policy source

Policies may be edited as TOML.

## Canonical policy artifact

Canonical policy hash is computed from:

```text
TOML -> parsed model -> normalized canonical JSON -> hash
```

Receipts must store the canonical hash.

## Action resolver

Hard rules and minimum action floors must outrank score thresholds.

Conflict resolution is deterministic and must be documented in `docs/ACTION_RESOLUTION.md`.

## Minimum action floors

```text
source_truth_drift                 >= RequireVerification
false_completion_missing_tests     >= GenerateRepairPacket
unknown_owner_for_mutation         >= RequireOwnerResolution
destructive_missing_rollback       >= BlockRelease
invalid_schema                     >= QuarantineArtifact
FEUT contamination in production   >= QuarantineArtifact
```

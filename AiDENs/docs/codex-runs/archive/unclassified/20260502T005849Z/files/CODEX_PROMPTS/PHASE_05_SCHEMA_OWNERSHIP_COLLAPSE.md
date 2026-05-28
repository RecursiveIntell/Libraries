# PHASE 05 — Schema Ownership Collapse

## Objective

Stop AiDENs from acting as schema authority for canonical stack artifact families.

## Required actions

1. Inspect:
   - `ArtifactFamilyRegistryV1`
   - `ArtifactFamilyRegistrationV1`
   - `GeneratedSchemaManifestV1`
   - `GeneratedSchemaDocumentV1`
   - `generated_schema_documents`
   - `generated_schema_manifest`
2. Remove canonical family schema generation from AiDENs:
   - attestation
   - settlement/federation
   - mechanism/theory/hypothesis
   - memory/evidence/episode/claim
   - kernel/region/syndrome/residual
   - verification/repair/adjudication
   - digest/content-addressing
3. Route canonical schema work through owner crates / `contract-schema-gen`.
4. Keep only AiDENs-local report/display schema docs, explicitly labeled non-authoritative.
5. Update docs explaining schema authority.

## Required gate

```bash
python3 scripts/assert_schema_generation_scope.py
```

## Acceptance

- AiDENs does not generate canonical artifact family schemas.
- Local schema manifest is limited to AiDENs-local DTOs.
- Canonical schema authority is documented.
- Ambiguous schemas are quarantined.

## Stop

Stop after this phase and wait for `GUARDRAIL_05_TO_06`.

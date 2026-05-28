# Codex Prompt — P07 Schema generation, artifact registry, compatibility, and migration law

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P07_SCHEMA_GENERATION_CONTRACT_REGISTRY_AND_MIGRATION_LAW.md`.

Implement P07 only. Do not start later passes.

## Goal

Make all wire-visible artifacts type-owned, schema-generated, versioned, and compatibility-checked.

## Primary crates

- `aidens-contracts`
- `aidens-cli`
- `workspace root`
- `schemas`

## Required artifacts

- `ArtifactFamilyRegistryV1`
- `GeneratedSchemaManifestV1`
- `SchemaCompatibilityReportV1`
- `MigrationPlanV1`
- `BackfillReceiptV1`

## Acceptance gates

- cargo run -p aidens-cli -- schemas generate creates deterministic schema files.
- schemas check fails on unregistered artifact family or incompatible breaking change without major bump.
- All old fixtures remain readable after migration path tests.

## Forbidden shortcuts

- Do not hand-maintain schemas that drift from Rust types.
- Do not change interpretation of a V1 artifact without new version.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.

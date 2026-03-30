# contract-schema-gen

Canonical JSON schema generator for the supported release lane.

## Usage

```rust
use contract_schema_gen::{generate_all_schemas, verify_committed_schemas};
```

## Purpose

This crate is the schema source of truth for the canonical contract surfaces:

- `semantic-memory-forge::ExportEnvelopeV3`
- `forge-memory-bridge::ProjectionImportBatchV3`
- `constraint-compiler::CompileOutput`
- `kernel-execution::ExecutionReport`
- `kernel-oracles::OracleAssessment`

The generated artifacts are committed under `../schemas/`, and the supported
gate verifies them with:

```bash
bash scripts/check_schema_compat.sh
```

## Ecosystem

**Depends on:**
- `semantic-memory-forge`, `forge-memory-bridge`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`
- `semantic-memory`, `forge-pilot`, `knowledge-runtime`, `mechanism-runtime`
- `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `constitutional-memory`, `continuity-runtime`
- `discovery-portfolio`, `effect-runtime`, `federated-settlement`
- `remote-oracle-admission`, `profile-runtime`, `spec-execution`
- `verification-adjudication`, `verification-calibration`, `verification-control`, `verification-policy`

**Depended on by:** (leaf crate -- no downstream consumers)

## stack-ids integration

This crate does not use `stack-ids` directly. It generates JSON schemas from
types defined in sibling crates, many of which embed stack-ids identity types
in their schema output.

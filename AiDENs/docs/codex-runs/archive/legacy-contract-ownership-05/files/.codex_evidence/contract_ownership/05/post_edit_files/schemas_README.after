# Schemas

This directory contains AiDENs-local, non-authoritative display/report DTO schemas only.

Canonical stack artifact family schemas are owned by canonical owner crates and `~/Coding/Libraries/contract-schema-gen`, not by AiDENs. AiDENs schema generation must not emit canonical schemas for attestation, federation, mechanism, memory/evidence, kernel/region, verification/repair, or digest/content-addressing law.

AiDENs-local schema reports can be regenerated with:

```bash
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
```

Generated AiDENs-local schemas live under `schemas/<artifact-family>/vN.schema.json`.
`generated_schema_manifest_v1.json` records the generated local set and non-authoritative display schema digests.

The remaining `*.sketch.json` files are historical design sketches only; they are not the compatibility gate.

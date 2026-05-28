# Phase 2 — Schema and Boundary Contract Hardening

## Objective

Make wire-visible schemas at least as strict as Rust validation and prevent silent schema widening.

## Required actions

1. Add `#[serde(deny_unknown_fields)]` to every wire-visible struct unless explicitly documented as an exception.
2. Ensure `schema_version` fields are represented as constants in generated schemas or post-processed generated schemas.
3. Ensure `ScoreBps` and `WeightBps` generated schemas include min `0` and max `10000`. If derive cannot express this correctly, implement manual `JsonSchema` or a schema postprocessor with tests.
4. Ensure `ExternalArtifactRef` and `Other(String)` variants reject empty strings and enforce bounded lengths if practical.
5. Decide and document time semantics:
   - If still opaque strings, name them as opaque basis fields and do not claim bitemporal compliance.
   - If RFC3339 validation is implemented, add tests and schema format declarations.
6. Add `scripts/validate_strict_schemas.py`.
7. Update `scripts/generate_schemas.sh` so generation + strict validation is one gate.
8. Add negative tests/fixtures for:
   - unknown fields;
   - wrong schema version;
   - `ScoreBps > 10000`;
   - empty refs;
   - invalid `Other("")` if relevant;
   - unexpected nested fields.

## Acceptance gate

```bash
cargo test --workspace --all-targets
bash scripts/generate_schemas.sh
python3 scripts/validate_strict_schemas.py
git diff --exit-code schemas/generated
```

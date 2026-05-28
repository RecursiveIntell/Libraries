# Phase 02 — Kernel contract and schema parity

## Goal

Make Rust contracts and generated JSON schemas match.

## Tasks

1. Add non-empty schema constraints for every Rust non-empty field:
   - `input_id`
   - `schema_version`
   - `ref_kind`
   - `ref_value`
   - `owner_hint`
   - `valid_time_basis`
   - `recorded_time`
   - reason codes and all string ids.
2. Decide recorded-time semantics:
   - implement RFC3339 validation and schema `format: date-time`, or
   - rename/document as opaque recorded-time basis.
3. Add typed control signal model:
   - `ControlSignalV1`
   - `SignalSourceV1`
   - signal ids from explicit field only.
4. Add authority/evidence/owner/rollback basis input/result types, or adapter-declared equivalents.
5. Add schema parity tests:
   - empty strings fail schema,
   - unknown fields fail schema,
   - invalid score/weight fails schema,
   - Rust and schema agree on representative negative cases.
6. Add `scripts/validate_schema_rust_parity.py` or extend `validate_strict_schemas.py`.

## Acceptance gate

- JSON schema rejects payloads Rust rejects for all required invariants.
- No `serde(default)` masks required semantic fields unless documented as compatibility.
- `cargo test` includes schema parity tests or CLI-level schema tests.

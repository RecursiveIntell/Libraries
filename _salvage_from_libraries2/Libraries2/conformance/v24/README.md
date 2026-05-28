# v24 conformance

This directory tracks the final-pass conformance target for v24 — Operational Continuity, Incident Command, and Recovery Replay.

## Required checks
- schema/example publication is complete,
- fixture bundles exist and remain parseable,
- one happy path exists,
- one degraded or blocked path exists,
- one replay/revocation/exception path exists where relevant,
- advisory or non-admitted states remain explicit.

## Canonical owner
- `continuity-runtime`

## Fixture directory
- `contracts/fixtures/v24/`

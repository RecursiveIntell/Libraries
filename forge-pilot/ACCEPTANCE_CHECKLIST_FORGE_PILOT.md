# Acceptance checklist for forge-pilot

## Surface and build

- [ ] `forge-pilot` exists as a new crate.
- [ ] Default features compile without `LLM-Pipeline`.
- [ ] Root workspace / Makefile surface is truthful.

## Observation

- [ ] Runtime advisory surfaces are consumed.
- [ ] Projection import logs are queried through public APIs.
- [ ] `kernel_payload_json` is parsed only when present and valid.
- [ ] `compile_batch()` + `schedule_execution()` are reused, not rewritten.
- [ ] Missing kernel payload degrades explicitly.

## Targeting and decision

- [ ] Stable target keys exist.
- [ ] Scoring is deterministic.
- [ ] Retry decay exists.
- [ ] Thin export becomes a visible target or degradation, not silent invention.
- [ ] Halt threshold works.

## Act and export

- [ ] Oracle plans execute through `kernel-oracles`.
- [ ] Patch plans execute through `PairedExperimentRunner`.
- [ ] A local bundle builder exists.
- [ ] Export uses `forge_engine::export_bundle()`.
- [ ] Bridge uses `transform_envelope_v3()`.
- [ ] Import uses `MemoryStore::import_projection_batch()`.

## Loop behavior

- [ ] Max iterations enforced.
- [ ] Time budget enforced.
- [ ] Cooldown enforced.
- [ ] Halt reasons reported honestly.
- [ ] Advisory-only actions are labeled and excluded from closure metrics.

## Testing

- [ ] Observation fixture tests pass.
- [ ] Scoring tests pass.
- [ ] Oracle plan tests pass.
- [ ] Loop roundtrip tests pass.
- [ ] Degradation tests pass.

## Optional LLM

- [ ] LLM refinement is feature-gated.
- [ ] LLM cannot choose targets or mutate authority state.

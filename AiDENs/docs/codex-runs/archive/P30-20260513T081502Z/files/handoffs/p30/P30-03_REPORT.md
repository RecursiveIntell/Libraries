# P30-03 Report

## Scope

Phase slice: replay identity, deterministic material IDs, and exposure/attempt identity law.

Issue IDs addressed from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- `P30-ABSORB-0008`: global process-local artifact counter must not be presented as material artifact identity.
- `P30-ABSORB-0009`: legacy non-replay `generated_artifact_id` API must not remain public.
- `P30-ABSORB-0014`: tool exposure IDs must be deterministic and material-derived.
- `P30-ABSORB-0033`: CLI fallback attempt-family ID must not use process-local display identity.

## Changed Files

- `crates/aidens-contracts/src/lib.rs`
  - Corrected stale identity comment from old `generated_artifact_id` wording to `display_only_unstable_id`.
- `crates/aidens-contracts/src/tests.rs`
  - Added `p30_legacy_process_local_generated_artifact_id_api_is_absent`.
- `crates/aidens-cli/src/agent.rs`
  - Replaced fallback `display_only_unstable_id("attempt-family")` with `generated_artifact_id_from_material("attempt-family", &run_id)`.
- `crates/aidens-cli/src/lib.rs`
  - Imported `generated_artifact_id_from_material` for the agent module.
- `crates/aidens-cli/src/tests.rs`
  - Added static regression coverage for material-derived CLI fallback and absence of legacy generated-artifact API references.

Observed existing implementation evidence:

- `crates/aidens-contracts/src/lib.rs:103` now names the process-local counter `DISPLAY_ONLY_UNSTABLE_ID_COUNTER`, not `GENERATED_ARTIFACT_COUNTER`.
- `crates/aidens-contracts/src/lib.rs:109` provides deterministic `generated_artifact_id_from_material`.
- `crates/aidens-contracts/src/lib.rs:117` explicitly names process-local IDs `display_only_unstable_id`.
- `crates/aidens-tool-kit/src/lib.rs:374` already uses `generated_artifact_id_from_material("tool-exposure", &exposure_material)`.
- `crates/aidens-tool-kit/tests/p30_tool_hardening.rs` already proves tool exposure IDs are stable and not the constant `tool-exposure`.

## Tests Added Or Updated

- `p30_legacy_process_local_generated_artifact_id_api_is_absent`
  - Proves `pub fn generated_artifact_id(` is absent from contracts and the replacement APIs are present.
- `p30_agent_run_fallback_attempt_family_id_is_material_derived`
  - Proves the CLI fallback uses `generated_artifact_id_from_material("attempt-family", &run_id)`.
- `p30_cli_source_does_not_reference_legacy_generated_artifact_id_api`
  - Proves CLI source does not reference the removed legacy API.

Existing relevant test retained:

- `p30_tool_exposure_id_is_content_derived`

## Commands Run

- `cargo test --manifest-path Cargo.toml -p aidens-contracts p30_ -- --nocapture`
  - Result: pass, 1 targeted P30 contract test passed.
- `cargo test --manifest-path Cargo.toml -p aidens-cli p30_ -- --nocapture`
  - Result: pass, 2 targeted P30 CLI tests passed.
- `cargo check --manifest-path Cargo.toml -p aidens-contracts -p aidens-cli --all-targets --locked`
  - Result: pass.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo .`
  - Result: exit 0, `findings=1838 hard=0`.
- Static search:
  - `pub fn generated_artifact_id(`: no production matches.
  - `GENERATED_ARTIFACT_COUNTER`: no production matches.
  - `ArtifactId::new("tool-exposure")`: no production matches in `aidens-tool-kit`.
  - `Uuid::new_v4(`: no matches in inspected AiDENs identity surfaces.

## Unresolved Risks And Quarantines

- P30-03 has 183 matrix rows. This pass fixed the safe P0/P1 edge and added proof around already-absorbed P0 fixes, but did not convert every `display_only_unstable_id` constructor across contracts.
- `AidensRunContextV1::new` still uses process-local display IDs for `run_id`, `trace_id`, `attempt_family_id`, and `attempt_id`. Replacing these requires a constructor/API design that separates replay material identity from unique live-run identity and should not be done by guessing material inputs.
- Many P1/P2 `Utc::now()` findings remain in runtime/artifact constructors. They require injected-clock/API changes and are recorded as remaining replay debt.
- `p30_guard.py` still reports broad warning debt but no hard failures.

## Invariant Revalidation Checklist

- Legacy public `generated_artifact_id` API is absent.
- Old `GENERATED_ARTIFACT_COUNTER` symbol is absent.
- Deterministic material ID helper remains available.
- Tool exposure ID is material-derived, not a constant.
- CLI fallback attempt-family ID is material-derived from `run_id`.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-03 can proceed for the P0 identity blockers and the CLI fallback repair covered here. The broad replay-identity sweep remains explicit debt and must constrain final release claims.

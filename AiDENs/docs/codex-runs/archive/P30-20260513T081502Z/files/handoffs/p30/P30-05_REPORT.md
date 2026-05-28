# P30-05 Report

## Scope

Phase slice: verification semantics, proof debt, degradation honesty, and no advisory promotion.

Matrix inventory from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- 82 total P30-05 rows.
- Priority split: 1 P0, 40 P1, 41 P2.
- Category split: 1 `VERIFICATION-SEMANTICS`, 80 `SILENT-DEGRADATION`, 1 `TOOL-DEDUP`.
- ID ranges: `P30-ABSORB-0015`, `P30-ABSORB-0113` through `P30-ABSORB-0145`, `P30-ABSORB-0301` through `P30-ABSORB-0307`, and `P30-ABSORB-0441` through `P30-ABSORB-0481`.

Issue IDs addressed:

- `P30-ABSORB-0015`: final-output runner control records now persist the selected verification attempt state as `advisory_only` in control receipt details, and the durable receipt test asserts both the advisory state and the advisory-only marker.

Issue IDs quarantined as remaining debt:

- `P30-ABSORB-0113` through `P30-ABSORB-0145`: broad P1 silent-degradation findings across agency, boundary, CLI, app-kit, and related surfaces.
- `P30-ABSORB-0301` through `P30-ABSORB-0307`: additional silent-degradation findings outside this final-output control path.
- `P30-ABSORB-0441` through `P30-ABSORB-0480`: P2 silent-degradation or fallback-semantic findings.
- `P30-ABSORB-0481`: runner tool-call dedup key may suppress legitimate repeated identical calls; not changed in this slice.

## Changed Files

- `crates/aidens-runner/src/lib.rs`
  - `turn_control_records` now includes `verification_attempt_state` in the canonical control receipt details it emits.
- `crates/aidens-runner/src/tests.rs`
  - `canonical_log_reopens_runner_report_after_process_boundary` now asserts the final-output `verification-control/control-receipt` records `details.verification_attempt_state == "advisory_only"` and top-level `advisory_only == true`.

Observed code evidence:

- `crates/aidens-runner/src/lib.rs:1531`: receipt details include `verification_attempt_state`.
- `crates/aidens-runner/src/lib.rs:1548`: final/control plan remains `CheckMethod::AdvisoryOnly`.
- `crates/aidens-runner/src/lib.rs:1188`: final-output path passes `VerificationAttemptState::AdvisoryOnly`.
- `crates/aidens-runner/src/tests.rs:144`: durable receipt assertion checks `details.verification_attempt_state`.
- `crates/aidens-runner/src/tests.rs:149`: durable receipt assertion checks top-level `advisory_only`.

## Tests Added Or Updated

Updated:

- `canonical_log_reopens_runner_report_after_process_boundary`

The regression target is durable evidence, not only in-memory state: the test reads the persisted canonical control receipt emitted for `final-output-produced`.

## Commands Run

- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `cargo test --manifest-path Cargo.toml -p aidens-runner canonical_log_reopens_runner_report_after_process_boundary -- --nocapture`
  - First run after adding the initial assertion failed because the receipt body has no top-level `attempt` object. This was a test-shape error.
  - Final result after recording and asserting `details.verification_attempt_state`: pass. Output: 1 unit test passed; 0 failed; integration test binaries were filtered with 0 tests run.
- `cargo check --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo . | tail -n 8`
  - Result: exit 0, `findings=1841 hard=0`.
  - Note: findings increased from the previous `1838` because this slice added one dynamic JSON detail field in an existing `serde_json::json!` receipt detail object. Hard findings remain zero.

Environment note:

- An earlier targeted test attempt failed before execution because Cargo could not create a temp directory under `target/debug/...`; `target` had disappeared. Creating `target/debug/deps` and `target/debug/build` allowed compilation to proceed. No source change was made for this environment issue.

## Unresolved Risks And Quarantines

- The canonical `verification-control::ControlReceipt` schema exposes `advisory_only` but does not expose the verification attempt state as a typed top-level field. AiDENs now records the state in its runner-owned receipt details; a stronger cross-crate fix would require the canonical owner to type this field in the control receipt schema.
- The 80 silent-degradation rows remain open. This phase did not audit or rewrite every `unwrap_or_default` / `unwrap_or_else` site in the listed crates.
- `P30-ABSORB-0481` remains open. The current tool-call dedup key risk was not changed or tested here.
- `p30_guard` still reports broad warnings; this phase only establishes `hard=0`, not absence of all risk.

## Invariant Revalidation Checklist

- Advisory-only final-output control is not represented as a succeeded verification attempt in the runner final-output path.
- Durable receipt evidence now records the runner-selected attempt state for final-output control.
- The final-output receipt remains marked `advisory_only`.
- No silent-degradation matrix rows outside `P30-ABSORB-0015` are claimed fixed.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-05 can proceed only for `P30-ABSORB-0015`. The remaining P30-05 P1/P2 rows are explicit quarantined debt and must constrain any final release or compliance claim.

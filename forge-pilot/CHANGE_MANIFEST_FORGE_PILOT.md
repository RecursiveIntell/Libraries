# Change manifest for forge-pilot

## 1. Required repo changes

### 1.1 Root workspace

| Path | Change | Why |
|---|---|---|
| `Cargo.toml` | add `forge-pilot` to workspace members | bring the crate under normal build/test control |
| `Makefile` | add `test-forge-pilot` target; optionally add to broader CI target after green | truthful surface for execution |

### 1.2 New crate files

| Path | Change |
|---|---|
| `forge-pilot/Cargo.toml` | new crate manifest; default features must not require `LLM-Pipeline` |
| `forge-pilot/src/lib.rs` | public crate surface and re-exports |
| `forge-pilot/src/config.rs` | loop config and feature flags |
| `forge-pilot/src/error.rs` | typed error surface |
| `forge-pilot/src/observe.rs` | runtime + memory + kernel-payload reconstruction |
| `forge-pilot/src/targets.rs` | target kinds, candidate structs, stable keys |
| `forge-pilot/src/orient.rs` | extraction, dedupe, scoring |
| `forge-pilot/src/decide.rs` | deterministic selection and optional LLM refinement |
| `forge-pilot/src/act.rs` | oracle execution and paired patch execution |
| `forge-pilot/src/bundle_builder.rs` | local builder for `forge_engine::EvidenceBundle` |
| `forge-pilot/src/export.rs` | canonical export/import roundtrip glue |
| `forge-pilot/src/history.rs` | in-memory retry/exhaustion tracking |
| `forge-pilot/src/loop_runner.rs` | bounded loop orchestration |
| `forge-pilot/src/cli.rs` | optional command-line surface |

### 1.3 New tests

| Path | Change |
|---|---|
| `forge-pilot/tests/observation_fixture_tests.rs` | prove observation reconstruction from public surfaces |
| `forge-pilot/tests/scoring_tests.rs` | prove scoring / retry decay / tie-breaks |
| `forge-pilot/tests/oracle_plan_tests.rs` | prove oracle-plan execution family |
| `forge-pilot/tests/loop_roundtrip_tests.rs` | prove end-to-end loop closure on fixtures |
| `forge-pilot/tests/degradation_tests.rs` | prove thin-export and missing-kernel-payload honesty |

## 2. Optional but acceptable updates after the crate is green

| Path | Change | When |
|---|---|---|
| `README.md` | mention `forge-pilot` only after tests are real | post-green |
| `AGENTS.md` | add a short bullet mentioning the new crate | post-green |
| root docs / pack docs | add truthful status note | post-green |

## 3. Files that should remain untouched unless a bug is proven

- `semantic-memory-forge/src/envelope.rs`
- `forge-memory-bridge/src/transform.rs`
- `kernel-execution/src/lib.rs`
- `kernel-oracles/src/lib.rs`
- `recursive-kernel-core/src/lib.rs`

`forge-pilot` is supposed to consume these, not redesign them.

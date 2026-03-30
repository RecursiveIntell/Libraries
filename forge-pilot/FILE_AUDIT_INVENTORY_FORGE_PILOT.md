# File audit inventory for forge-pilot

## 1. Existing files this pack assumes and consumes

### Root control-plane / architecture files

- `Cargo.toml`
- `AGENTS.md`
- `CANONICAL_STACK_SPEC_V6.md`
- `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`

### Current-code integration points

- `recursive-kernel-core/src/lib.rs`
- `constraint-compiler/src/lib.rs`
- `kernel-execution/src/lib.rs`
- `kernel-oracles/src/lib.rs`
- `kernel-conformance/src/lib.rs`
- `knowledge-runtime/src/runtime.rs`
- `knowledge-runtime/src/adapters/semantic_memory.rs`
- `semantic-memory/src/lib.rs`
- `semantic-memory/src/types.rs`
- `living-memory/living-memory/src/experiment.rs`
- `living-memory/living-memory/src/export.rs`
- `living-memory/living-memory/src/lab/evidence.rs`

## 2. New files to add

- `forge-pilot/Cargo.toml`
- `forge-pilot/src/lib.rs`
- `forge-pilot/src/config.rs`
- `forge-pilot/src/error.rs`
- `forge-pilot/src/observe.rs`
- `forge-pilot/src/targets.rs`
- `forge-pilot/src/orient.rs`
- `forge-pilot/src/decide.rs`
- `forge-pilot/src/act.rs`
- `forge-pilot/src/bundle_builder.rs`
- `forge-pilot/src/export.rs`
- `forge-pilot/src/history.rs`
- `forge-pilot/src/loop_runner.rs`
- `forge-pilot/src/cli.rs`
- `forge-pilot/tests/observation_fixture_tests.rs`
- `forge-pilot/tests/scoring_tests.rs`
- `forge-pilot/tests/oracle_plan_tests.rs`
- `forge-pilot/tests/loop_roundtrip_tests.rs`
- `forge-pilot/tests/degradation_tests.rs`

## 3. Optional small upstream changes

- `Makefile`
- `README.md`
- root `AGENTS.md`

## 4. Files explicitly out of scope unless a bug is proven

- `forge-memory-bridge/src/transform.rs`
- `semantic-memory-forge/src/envelope.rs`
- `kernel-execution/src/lib.rs`
- `kernel-oracles/src/lib.rs`
- `constraint-compiler/src/lib.rs`

If those files need major surgery to land `forge-pilot`, the pilot design is probably wrong.

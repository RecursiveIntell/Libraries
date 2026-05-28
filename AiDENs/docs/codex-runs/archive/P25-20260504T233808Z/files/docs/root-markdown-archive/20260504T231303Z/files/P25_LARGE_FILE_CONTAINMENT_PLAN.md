# P25 Large-File Containment Plan

## Purpose

P25 identifies large-file risk and creates a low-risk future containment backlog without forcing broad refactors in this pass.

## Risk baseline

Candidate threshold used for planning:

- Files over 1,500 lines or high-risk mixed domains are backlog candidates.
- Existing file boundaries must remain stable unless a future acceptance gate requires a split.

## Current large file candidates (as measured in this workspace)

| path | bytes | lines |
| --- | --- | --- |
| `crates/aidens-contracts/src/lib.rs` | 371136 | 10410 |
| `crates/aidens-cli/src/lib.rs` | 235631 | 6698 |
| `knowledge-runtime/tests/cross_crate_proof.rs` | 133452 | 3610 |
| `crates/aidens-tool-kit/src/lib.rs` | 90164 | 2454 |
| `crates/aidens-runner/src/lib.rs` | 75397 | 1910 |
| `verification-control/src/lib.rs` | 70140 | 1915 |
| `crates/aidens-agency-kit/src/lib.rs` | 67998 | 1815 |
| `living-memory/living-memory/src/lab/evidence.rs` | 66113 | 1627 |
| `semantic-memory/src/lib.rs` | 62756 | 1619 |
| `semantic-memory/src/projection_lane.rs` | 61912 | 1471 |
| `semantic-memory/tests/import_ugly_cases.rs` | 61103 | 1783 |
| `semantic-memory/tests/import_boundary_tests.rs` | 60779 | 1641 |
| `semantic-memory-forge/src/envelope.rs` | 56863 | 1434 |
| `kernel-conformance/src/lib.rs` | 56767 | 1394 |
| `profile-runtime/src/adapters.rs` | 54472 | 1792 |

## Planned future splits (deferred to future pass)

### `crates/aidens-contracts/src/lib.rs`

- Split into type families:
  - run bundle / receipts,
  - support profile / capability labels,
  - provider routes / provider contracts,
  - package/report types,
  - schema-export helpers.
- Candidate checks in the next pass:
  - Public API compatibility snapshot for moved modules.
  - `cargo check --workspace`.
  - Existing contract-boundary tests.

### `crates/aidens-cli/src/lib.rs`

- Split command surface by command domain:
  - run bundle commands (`run-test-agent`, `run-coding-agent`, `inspect-run`),
  - permit command surface,
  - package/verifier command surface,
  - format/output rendering helpers.
- Candidate checks in the next pass:
  - CLI smoke matrix for command parsing.
  - `scripts/p25_verify.sh` command-run parity tests.
  - fixture replay output stability.

### `crates/aidens-tool-kit/src/lib.rs`

- Split by tool domain:
  - repository read/list/status adapters,
  - patch proposal and patch apply/dry-run,
  - permit/receipt formatting,
  - fixture utilities.
- Candidate checks in the next pass:
  - patch proposal/apply integration tests,
  - permit-usage evidence tests,
  - failure-mode parity tests for blocked write operations.

### Additional candidates

- `crates/aidens-runner/src/lib.rs`, `verification-control/src/lib.rs`, and `knowledge-runtime/tests/cross_crate_proof.rs` are large and cross-domain; evaluate only if evidence or acceptance lanes demand.
- Keep as-is for P25 to avoid widening scope.

## Containment rule for this pass

- No large-file refactor is allowed in P25 unless directly required by an active acceptance gate.
- All splits remain backlog only and do not block current pass closure.

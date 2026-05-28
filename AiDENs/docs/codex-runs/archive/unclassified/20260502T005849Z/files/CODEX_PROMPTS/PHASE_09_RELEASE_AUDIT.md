# Phase 09 — Release audit

## Objective

Run full gates, docs/cargo parity, no-compat ledger, and package readiness.

## Mandatory read list

- `AGENTS.md`
- `SOURCE_BASIS.md`
- `CANONICAL_OWNER_MAP.md`
- `SHADOW_SEMANTICS_AUDIT.md`
- `ACCEPTANCE_GATES.md`
- `TESTKIT_TARGETS.md`
- `GOLDEN_VERTICAL_SLICE_SPEC.md` when relevant

## Source references

- AiDENs workspace and real stack deps: `~/Coding/Libraries/AiDENs/Cargo.toml:L1-L35` and `~/Coding/Libraries/AiDENs/Cargo.toml:L58-L72`.
- Stack IDs authority: `~/Coding/Libraries/stack-ids/src/lib.rs:L1-L25`.
- Major duplicate local contracts: `~/Coding/Libraries/AiDENs/crates/aidens-contracts/src/lib.rs:L14-L23 and L39-L79`, `~/Coding/Libraries/AiDENs/crates/aidens-contracts/src/lib.rs:L155-L180`, `~/Coding/Libraries/AiDENs/crates/aidens-contracts/src/lib.rs:L1849-L2010`.
- Canonical Forge/Bridge/Memory/Runtime chain: `~/Coding/Libraries/semantic-memory-forge/src/lib.rs:L3-L28 and L39-L56 and L79-L82`, `~/Coding/Libraries/forge-memory-bridge/src/transform.rs:L123-L188`, `~/Coding/Libraries/semantic-memory/Cargo.toml:L20-L33 and ~/Coding/Libraries/semantic-memory/src/lib.rs:L159-L327`, `~/Coding/Libraries/knowledge-runtime/src/lib.rs:L1-L27 and L49-L72 and L111-L140`.

## Allowed changes

Edit docs/tests/scripts only unless gate fixes required.

## Forbidden changes

Must not claim finished without commands passing.

Always forbidden: using `~/Coding/Libraries2/stack-ids`; adding local canonical truth types; starting blocked later-phase work.

## Tasks

Release audit: run all gates, ensure docs/cargo parity, no compatibility surfaces, no shadow truth, and honest limitations.

Detailed steps:

1. Inspect the cited files in the real repo and verify line references still match.
2. Apply only changes needed for this phase.
3. Prefer deleting/collapsing duplicate semantics over wrapping them.
4. Add/update phase tests in `crates/aidens-testkit`.
5. Run the required scripts and tests.
6. Stop and report if a canonical API is missing instead of inventing one.

## Required scripts

- `scripts/assert_stack_paths.sh`
- `scripts/assert_no_shadow_truth.sh`
- `scripts/assert_docs_match_cargo.sh`
- `scripts/assert_adapter_delegation.sh`
- `scripts/assert_compat_is_finite.sh`

## Required tests

- `release_truth_audit`

## Acceptance gates

```bash
./scripts/run_codex_phases.sh verify 09
```

If cargo is available, run phase tests directly, for example:

```bash
cargo test -p aidens-testkit release_truth_audit
```

## Final report format

```text
PHASE COMPLETED: 09 — Release audit
FILES CHANGED:
TESTS ADDED/UPDATED:
GATES RUN:
GATES PASSED:
GATES FAILED:
SOURCE REFERENCES USED:
KNOWN LIMITATIONS:
NEXT PHASE UNBLOCKED: yes/no + reason
```

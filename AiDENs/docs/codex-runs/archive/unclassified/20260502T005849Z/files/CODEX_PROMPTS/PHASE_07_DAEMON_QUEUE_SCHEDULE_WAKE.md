# Phase 07 — Daemon/queue/schedule/wake

## Objective

Only after phases 03 and 06 pass, reintroduce queue/daemon as app lifecycle only.

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

Edit daemon/queue/schedule/wake or adapter to supplemental Libraries2 job-queue if Libraries has no queue owner.

## Forbidden changes

Forbidden before golden vertical slice and governance gates.

Always forbidden: using `~/Coding/Libraries2/stack-ids`; adding local canonical truth types; starting blocked later-phase work.

## Tasks

Only if phases 03 and 06 pass, reintroduce daemon/queue/schedule/wake as lifecycle-only surfaces. Evaluate `libraries2/job-queue` only if useful and never stack-ids.

Detailed steps:

1. Inspect the cited files in the real repo and verify line references still match.
2. Apply only changes needed for this phase.
3. Prefer deleting/collapsing duplicate semantics over wrapping them.
4. Add/update phase tests in `crates/aidens-testkit`.
5. Run the required scripts and tests.
6. Stop and report if a canonical API is missing instead of inventing one.

## Required scripts

- `scripts/assert_stack_paths.sh`

## Required tests

- `daemon_namespace_isolated`
- `schedule_no_duplicate_storm`
- `restart_does_not_reenqueue_completed_jobs`

## Acceptance gates

```bash
./scripts/run_codex_phases.sh verify 07
```

If cargo is available, run phase tests directly, for example:

```bash
cargo test -p aidens-testkit daemon_namespace_isolated
```

## Final report format

```text
PHASE COMPLETED: 07 — Daemon/queue/schedule/wake
FILES CHANGED:
TESTS ADDED/UPDATED:
GATES RUN:
GATES PASSED:
GATES FAILED:
SOURCE REFERENCES USED:
KNOWN LIMITATIONS:
NEXT PHASE UNBLOCKED: yes/no + reason
```

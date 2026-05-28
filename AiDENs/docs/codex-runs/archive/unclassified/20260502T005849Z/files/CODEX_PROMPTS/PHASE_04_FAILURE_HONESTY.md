# Phase 04 — Failure honesty

## Objective

Make malformed tools, denied tools, provider failure, budget exhaustion, and fallback produce canonical receipts/degradation.

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

Edit runner/provider/tool/receipts/governance adapters.

## Forbidden changes

Must not silently swallow failures.

Always forbidden: using `~/Coding/Libraries2/stack-ids`; adding local canonical truth types; starting blocked later-phase work.

## Tasks

Add failure honesty: malformed tool calls, denial/approval, provider unavailable, budget exhaustion produce explicit canonical receipts/degradation.

Detailed steps:

1. Inspect the cited files in the real repo and verify line references still match.
2. Apply only changes needed for this phase.
3. Prefer deleting/collapsing duplicate semantics over wrapping them.
4. Add/update phase tests in `crates/aidens-testkit`.
5. Run the required scripts and tests.
6. Stop and report if a canonical API is missing instead of inventing one.

## Required scripts

- `scripts/assert_adapter_delegation.sh`

## Required tests

- `malformed_tool_call_degrades`
- `denied_tool_requires_approval`
- `budget_exhaustion_receipt`
- `provider_route_unavailable`

## Acceptance gates

```bash
./scripts/run_codex_phases.sh verify 04
```

If cargo is available, run phase tests directly, for example:

```bash
cargo test -p aidens-testkit malformed_tool_call_degrades
```

## Final report format

```text
PHASE COMPLETED: 04 — Failure honesty
FILES CHANGED:
TESTS ADDED/UPDATED:
GATES RUN:
GATES PASSED:
GATES FAILED:
SOURCE REFERENCES USED:
KNOWN LIMITATIONS:
NEXT PHASE UNBLOCKED: yes/no + reason
```

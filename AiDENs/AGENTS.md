# AGENTS.md — AiDENs P30 hardening doctrine

## Mission

You are implementing the P30 Codex super-pass for AiDENs. Your job is to harden AiDENs against the 2026-05-08 hostile audit and move the codebase closer to v11A/v11B compliance without creating shadow semantics.

## Core law

1. Provenance-first design is a hard constraint.
2. Correctness outranks speed, momentum, aesthetics, and completion theater.
3. No silent approximation, no semantic widening, no invented compatibility semantics.
4. AiDENs directs, wires, scopes, exposes, validates, and coordinates. AiDENs must not become the owner of domain truth owned by sibling crates.
5. Every material operation must be represented as a typed, receipt-bearing artifact transition where possible.
6. Runtime/tool/control layers must not become hidden truth stores.
7. Execution is evidence: tool calls, retries, queue hops, provider routes, deadlines, cancellations, fallback paths, degraded paths, replay attempts, and budget exhaustion must be receipt-bearing or explicitly non-durable/degraded.
8. Valid time and recorded/transaction time must remain distinct. Never collapse them for convenience.
9. Append-plus-supersession only. No silent destructive rewrite of truth-bearing state.
10. Repairs, boundary repairs, parse repairs, rollback, schema repair, and compatibility handling must emit explicit repair/degradation provenance.
11. Material IDs must be deterministic and replay-safe. Process-local counters, random UUIDs, and branch-order IDs are forbidden for material receipts/artifacts/manifests/operator invocations.
12. No user-visible “done,” “verified,” “succeeded,” “ready,” or “v11A compliant” state unless the required receipts/checks exist.

## Source-of-truth map

| Surface | Canonical owner | AiDENs role | Forbidden AiDENs behavior |
|---|---|---|---|
| Stable IDs, digests, trace primitives | `stack-ids` and contract owner crates | consume/wire only | invent new material identity law |
| Semantic memory/projection truth | `semantic-memory`, `semantic-memory-forge`, `forge-memory-bridge` | coordinate import/query paths | create duplicate memory truth layer |
| Evidence/export truth | `semantic-memory-forge`, `living-memory`, bridge crates | consume/wire/package | reinterpret evidence meaning |
| Tool contracts/receipts | `llm-tool-runtime`, AiDENs receipt/tool kits as adapters | expose safely and record receipts | drop tool evidence or repair silently |
| Verification policy/control | `verification-*`, `assurance-runtime` | route/check/report | represent advisory observation as verified success |
| Kernel/oracle/conformance | `recursive-kernel-*`, `constraint-compiler`, `kernel-*` | orchestrate and expose | invent local oracle semantics |
| Artifact contracts | `aidens-contracts` only where AiDENs-owned; sibling crates where stack-owned | define orchestration contracts | duplicate canonical stack contracts without owner map |
| Package/source certification | `z.py`, `zip.py`, certifier sidecars | run/report/consume | claim semantic/build correctness from packaging-only evidence |

## Hard fail patterns

Codex must stop or quarantine if it encounters any of these:

- `unwrap_or_default()` used to erase read/serialization/parse failures in material paths.
- `filter_map` used to drop malformed executable tool-call entries without rejected-call receipts.
- permissive JSON repair feeding executable tool calls without strict degradation/approval gates.
- rollback errors ignored with `let _ = ...`.
- wildcard permits, host ambient PATH, or unfrozen toolchain execution for command tools.
- material IDs generated from process-local counters, random UUIDs, wall-clock-only data, branch order, or constant strings.
- advisory checks reported as `Succeeded` verification attempts.
- failure paths returning empty evidence where durable failure receipts are required.
- missing gate scripts/schemas referenced by docs without supersession evidence.
- root Markdown ambiguity allowed to steer current run instructions.
- `serde_json::Value` or dynamic JSON used where a typed boundary contract is required.
- `panic!`, `unwrap`, `expect`, `todo!`, `unimplemented!`, broad `allow(...)`, or lint suppression in runtime/control/tool/evidence paths unless explicitly justified and tested.

## Required evidence per phase

Every phase must emit:

- changed file summary;
- issue IDs addressed from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`;
- tests added/updated;
- commands run with outputs captured;
- unresolved risks and quarantines;
- invariant revalidation checklist;
- statement of whether the phase can proceed.

## Final claim discipline

Do not claim v11A/v11B compliance unless the release/conformance gates prove it. Acceptable claims are narrower, e.g. “P30 repaired the parser fallback P0s and added tests,” or “P30 introduced v11A seed contracts but full v11A release remains pending.”

You are entering a new P20 execution phase.

Before doing any work, revalidate all invariants. Do not assume anything still holds from prior phases.

HARD REQUIREMENTS:

1. Provenance-first design is a hard constraint.
2. AiDENs must not invent semantics owned by canonical stack crates.
3. No shadow truth: no local memory, evidence, episode, kernel, repair, verification, temporal, or control truth.
4. No silent semantic widening, fallback, compatibility reinterpretation, or degraded behavior without explicit receipts and docs.
5. Execution is evidence: tool calls, retries, provider routes, budgets, failures, queue hops, approvals, and degraded paths must be receipt-bearing where implemented.
6. Contract-first boundaries: Rust types/versioned envelopes are source of truth; JSON/schema/patch repair must be bounded and provenance-bearing.
7. Graph separation: storage, retrieval, inference, repair, and control/receipt graphs must not be collapsed.
8. Agency/influence governance is required for high-impact, personalized, repeated, scheduled, or tool-mediated advice.
9. Docs must not claim support without code path + test/proof.
10. If ownership is unclear, stop and report/quarantine. Do not invent a substitute.

VALIDATION TASKS BEFORE PROCEEDING:

- State which phase you are entering.
- State which invariants are at risk in this phase.
- Identify any likely files/modules that could violate ownership.
- Confirm what you will not touch.
- Confirm the pass/fail gate for this phase.

FAILURE CONDITION:

If any invariant is violated and you continue anyway, that is an architectural violation. Halt, repair, quarantine, rollback, or mark P20 failed.

# P32-SCR-RUNTIME-SUPERPASS scope

## Immediate target

Complete the SCR-P0A reference runtime enough that a hostile auditor can verify:

1. The Rust workspace builds and tests.
2. SCR evaluates proposed action + requested effect materially.
3. Authority, evidence, rollback, containment, source-owner, and policy basis are explicit and receipt-bearing.
4. Opaque refs are not scanned or reinterpreted as control facts.
5. Schema contracts are not weaker than Rust validation.
6. Receipts preserve full candidate arbitration and losing-candidate reasons.
7. Final docs/receipts prove what was done.

## Not target unless already easy and bounded

- Full ClaimLedger integration.
- Full AgentSecurity/MCP admission daemon.
- Full stack-wide v11 runtime.
- External owner-crate path dependency integration if source ownership remains ambiguous.
- Public release claims.

## Required final docs

Codex must produce or update:

```text
docs/P32_COMPLETION_REPORT.md
docs/P32_COMMAND_RECEIPTS.md
docs/P32_CHANGED_FILES.md
docs/P32_UNRESOLVED_RISKS.md
docs/P32_HOSTILE_AUDITOR_HANDOFF.md
docs/P32_POLICY_CHANGE_RECEIPT.md
docs/P32_ROLLBACK_PLAN.md
docs/SCR_CANONICAL_JSON_V1.md
docs/SCR_ADAPTER_SEAMS.md
docs/SCR_ACTION_SEMANTICS.md
docs/SCHEMA_RUST_PARITY.md
docs/EVALUATOR_BUILD_DIGEST.md
```

If any are not applicable, Codex must create the file and state why it is not applicable.

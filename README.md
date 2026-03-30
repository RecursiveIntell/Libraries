# RecursiveIntell

A 30-crate Rust workspace implementing an OODA governance orchestrator for recursive intelligence systems.

## What it does

RecursiveIntell provides a full **Observe → Orient → Decide → Act** loop with integrated verification, calibration, and adjudication. The system enforces governance constraints at every stage: observations are checked against constitutional memory, decisions are gated by execution permits, and actions produce auditable effect receipts.

The stack generates 211 JSON schemas from the Rust type system via `contract-schema-gen`, ensuring wire-format contracts stay synchronized with code.

## Crate architecture

```
┌─────────────────────────────────────────────────────────┐
│  Tier 1 — Core Intelligence                             │
│  constraint-compiler · kernel-oracles                   │
├─────────────────────────────────────────────────────────┤
│  Tier 2 — Orchestration                                 │
│  semantic-memory · forge-engine (living-memory)          │
│  knowledge-runtime · forge-pilot                        │
├─────────────────────────────────────────────────────────┤
│  Tier 3 — Support & Bridge                              │
│  stack-ids · llm-tool-runtime · profile-runtime          │
│  forge-memory-bridge · semantic-memory-forge             │
├─────────────────────────────────────────────────────────┤
│  Governance                                             │
│  assurance-runtime · attestation-exchange                │
│  authority-delegation · constitutional-memory            │
│  continuity-runtime · effect-runtime · mechanism-runtime │
├─────────────────────────────────────────────────────────┤
│  Verification Pipeline                                  │
│  verification-control · verification-policy              │
│  verification-calibration · verification-adjudication    │
└─────────────────────────────────────────────────────────┘
```

## Build

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Gate verification

```bash
make gate
```

This runs the full release gate set including permit path checks, hotspot budgets, panic safety, doc coverage, and schema compatibility.

## Canonical specification

See [`CANONICAL_STACK_SPEC_V26_ADVISORY_CONSTITUTIONAL_SEARCH_MINIMAL_EXCEPTION_SYNTHESIS_AND_POLICY_COUNTERFACTUAL_RUNTIME.md`](CANONICAL_STACK_SPEC_V26_ADVISORY_CONSTITUTIONAL_SEARCH_MINIMAL_EXCEPTION_SYNTHESIS_AND_POLICY_COUNTERFACTUAL_RUNTIME.md) for the current constitutional specification.

## Snapshot

Source basis: `libraries-source-clean-20260330.zip`

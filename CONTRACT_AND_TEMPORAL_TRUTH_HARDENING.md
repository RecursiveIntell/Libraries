# Contract and Temporal Truth Hardening

## Objective
Turn the stack's truth surfaces into **answerable-by-construction** surfaces:
- versioned,
- type-owned,
- replayable,
- bitemporal,
- and explicitly compatible across evolution.

## Must-ship constitutional moves
1. Freeze one canonical Episode / Claim / Evidence package family.
2. Require execution context on all risk-bearing / promotable artifacts.
3. Publish all wire-visible schemas through `contract-schema-gen`.
4. Declare explicit compatibility mode and migration owner for every artifact family.
5. Add executable reference semantics for:
   - valid-time / recorded-time query behavior,
   - widening / degradation semantics,
   - bridge atomicity,
   - repair invariants.

## Concrete artifact families
- `EpisodeBundleV1`
- `ExecutionContextV1`
- `VerificationPlanV1`
- `RepairRecordV1`
- `ImportFailureRecordV1`
- `RuntimeQueryProvenanceV1`
- `ControlReceiptV1`

## Hard rules
- No silent destructive rewrite.
- No runtime truth mutation.
- No hidden compatibility breaks.
- No replay semantics that exist only in prose.

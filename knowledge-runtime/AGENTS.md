# AGENTS.md — knowledge-runtime

Read the root control plane before changing this crate:

1. `../CANONICAL_STACK_SPEC_V6.md`
2. `../CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`
3. `../PACK_README.md`
4. `../SOURCE_BASIS.md`
5. `../STATUS_DASHBOARD.md`
6. `../MASTER_ISSUE_CHANGE_MATRIX.md`
7. `../CONFORMANCE_GATES.md`
8. `../PHASED_EXECUTION_PLAN.md`
9. `../IMPLEMENTATION_PLAYBOOK.md`
10. `../RISKS_AND_FORBIDDEN_SHORTCUTS.md`

## Scope

`knowledge-runtime` owns planning, routing, retrieval composition, merge behavior, and explicit
degradation reporting over memory-visible projections.

## What this crate must do in the finish-line pass

1. preserve projection-backed retrieval as the real supported retrieval substrate,
2. execute supported temporal routes honestly across `valid_as_of` and `recorded_as_of`,
3. push scope filters down where supported and report explicit widening where not,
4. implement the explicit identity ladder:
   - exact ID
   - alias
   - bounded evidence-backed candidates
   - semantic fallback
5. keep audit/explain evidence dereference separate from normal ranking,
6. keep overlays derivable and non-authoritative.

## Do not do

- do not persist authoritative truth here
- do not query raw Forge receipts during normal retrieval
- do not silently widen scope or identity
- do not teach hybrid fact search as a substitute for projection-backed temporal behavior
- do not make audit/evidence reads part of default ranking behavior

## Primary files

- `src/adapters/semantic_memory.rs`
- `src/runtime.rs`
- `src/query/classify.rs`
- `src/entity/registry.rs`
- `src/evidence/support.rs`
- `tests/cross_crate_proof.rs`
- `tests/ugly_case_tests.rs`

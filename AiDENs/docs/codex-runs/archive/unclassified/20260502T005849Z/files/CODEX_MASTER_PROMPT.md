# Codex Master Prompt — AiDENs Contract Ownership Collapse v2

You are working in:

```text
~/Coding/Libraries/AiDENs
```

Canonical sibling libraries are in:

```text
~/Coding/Libraries
```

Supplemental reference code may exist in:

```text
~/Coding/Libraries2
~/Coding/Recall
~/Coding/Recall-Coding
```

Use `Libraries2`, `Recall`, and `Recall-Coding` only as reference. Do not import from them when a canonical owner exists in `~/Coding/Libraries`.

## You have no prior context

Assume you know nothing from previous conversations. All required context is in this bundle.

## Mission

Perform the AiDENs Contract Ownership Collapse Run v2.

The target is not feature expansion. The target is to prove that `crates/aidens-contracts` no longer owns canonical stack concepts.

By the end:

- exact public duplicate canonical types are deleted or converted to explicit `pub use` re-exports;
- missing canonical owner crates are wired into `Cargo.toml`;
- local digest/canonicalization law is removed or demoted to non-authoritative display-only helpers;
- schema generation for canonical artifact families is owned by `contract-schema-gen`, not AiDENs;
- tool, repair, runtime-view, kernel, federation, mechanism, and schema surfaces are canonical objects or display/report wrappers with backpointers;
- generated ownership gates exist and fail on future duplicate canonical public type definitions;
- docs reflect the actual 2026-04-28 source basis and dependencies;
- rollback/quarantine/evidence records are written.

## Mandatory read order

1. `AGENTS.md`
2. `CODEX_PHASE_MANIFEST.yaml`
3. `docs/FINAL_STATE_SPEC.md`
4. `docs/CANONICAL_OWNER_MAP.md`
5. `docs/SHADOW_OWNERSHIP_ISSUE_MATRIX.md`
6. `docs/ROLLBACK_AND_QUARANTINE_PLAN.md`
7. `docs/EVIDENCE_REPORTING_REQUIREMENTS.md`
8. `docs/HUMAN_GUARDRAIL_PROMPTS.md`
9. `CODEX_PROMPTS/PHASE_00_PREFLIGHT_SOURCE_BASIS.md`
10. then each phase prompt in order

## Absolute prohibitions

- Do not split `aidens-contracts` in this run.
- Do not add new features.
- Do not implement local stand-ins for missing canonical APIs.
- Do not use `Libraries2/stack-ids`.
- Do not preserve local duplicate types through compatibility layers.
- Do not downgrade canonical artifacts into JSON blobs.
- Do not make AiDENs the schema authority for canonical stack families.
- Do not proceed after a failed phase gate.

## Required phases

Run phases in `CODEX_PHASE_MANIFEST.yaml` order.

After each phase, stop and wait for the matching human guardrail prompt. Do not proceed automatically.

## Required gates

At minimum, run:

```bash
bash scripts/phase_verify_contract_ownership.sh 00
bash scripts/phase_verify_contract_ownership.sh 01
bash scripts/phase_verify_contract_ownership.sh 02
bash scripts/phase_verify_contract_ownership.sh 03
bash scripts/phase_verify_contract_ownership.sh 04
bash scripts/phase_verify_contract_ownership.sh 05
bash scripts/phase_verify_contract_ownership.sh 06
bash scripts/phase_verify_contract_ownership.sh 07
bash scripts/phase_verify_contract_ownership.sh final
```

If `cargo` is available, also run:

```bash
cargo check --workspace
cargo test --workspace
```

If full workspace tests are too expensive, run targeted tests first and report what was skipped with exact rationale.

## Completion bar

The run is complete only when:

1. generated duplicate-type gate passes;
2. digest-law gate passes;
3. schema-scope gate passes;
4. tool-runtime delegation gate passes;
5. stale-doc/source-basis gate passes;
6. no crate split occurred;
7. no compatibility ledger rows exist;
8. wrapper/backpointer gate passes or all exceptions are quarantined;
9. existing AiDENs gates still pass or have documented legitimate updates;
10. `aidens-contracts` owns no canonical stack truth semantics.

If any gate fails, halt and repair. Do not continue by refactoring around the failure.

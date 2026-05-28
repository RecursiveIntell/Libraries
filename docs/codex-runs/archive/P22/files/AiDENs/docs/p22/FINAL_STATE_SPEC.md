# Final State Specification

The run is complete only when the repo satisfies this target state.

## Required final repository state

`crates/aidens-contracts`:

- contains no local public definitions whose type names exactly match canonical public type names from canonical crates;
- uses explicit `pub use` for canonical artifacts where AiDENs needs to surface them;
- defines only AiDENs-local app/report/display/orchestration DTOs;
- does not define canonical digest/content-addressing law;
- does not define canonical schema generation for stack artifact families;
- does not own tool-call, repair, runtime-view, memory, kernel, federation, mechanism, or verification truth.

Root / docs:

- source-basis docs reflect 2026-04-28 archive basis;
- `docs/contract-ownership/` exists and contains final proof artifacts;
- quarantine ledger exists even if empty;
- compatibility ledger has no rows beyond header.

Scripts:

- generated duplicate-type gate exists and passes;
- digest law gate exists and passes;
- schema scope gate exists and passes;
- tool delegation gate exists and passes;
- final phase verification script passes.

## Forbidden leftovers

- `pub struct AttestationEnvelopeV1` in `aidens-contracts`;
- `pub struct SharedDispositionV1` in `aidens-contracts`;
- `pub struct SettlementCaseV1` in `aidens-contracts`;
- `pub struct TheoryRefuterSuiteV1` in `aidens-contracts`;
- `pub struct TheoryVersionV1` in `aidens-contracts`;
- `pub struct HypothesisLibraryV1` in `aidens-contracts`;
- exported `stable_json_digest`;
- exported `stable_text_digest`;
- exported `deterministic_artifact_id`;
- canonical family schema generation from AiDENs;
- compatibility shims preserving removed local semantics;
- new crates created from `aidens-contracts`.

## Explicit non-goals

- no crate split;
- no feature additions;
- no UI work;
- no provider expansion;
- no daemon/scheduler work;
- no kernel algorithm expansion;
- no Recall/Recall-Coding copying except reference patterns.

## Defer to later run

After ownership is clean, a future run may split `aidens-contracts` into smaller crates. That future split must preserve the ownership gates introduced here.

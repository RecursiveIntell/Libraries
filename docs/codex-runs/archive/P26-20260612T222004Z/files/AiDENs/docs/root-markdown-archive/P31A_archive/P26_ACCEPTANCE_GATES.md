# P26 Acceptance Gates

## A. AgentSpecV1 gate

Pass iff:

- `AgentSpecV1` exists with schema and fixtures.
- Validation rejects missing support label, missing evidence policy, unsupported tool policy, unsupported provider policy, and invalid permit policy.
- Schema generation/check passes.
- No canonical sibling semantics are copied into AiDENs.

## B. PlanActVerifyLoopV1 gate

Pass iff:

- A bounded local loop runs from an AgentSpec fixture.
- All actions emit receipts.
- Budget/turn/deadline stop rules are enforced.
- Unsupported paths abstain.
- Run is inspectable and replayable.

## C. Memory grounding gate

Pass iff:

- A local agent can use memory seam evidence as context.
- Export/import/query evidence is attached to run evidence.
- No AiDENs-local memory truth store is created.
- View/widening/degradation is disclosed where applicable.

## D. CodingAgentV1 gate

Pass iff:

- The agent can read/list/search a sandbox repo.
- It can propose a patch.
- It cannot apply a patch without a valid permit.
- It can apply a patch with a scoped permit.
- It can run checks under permit.
- Failed checks create evidence and do not report fake success.

## E. Evidence/RunBundleV3 gate

Pass iff:

- Run bundle captures agent spec digest, trace/attempt/trial IDs, tool receipts, permit receipts, memory evidence, verification receipts, support labels, and replay recipe.
- `inspect-run` validates digest and evidence completeness.
- V2 compatibility or migration is tested.

## F. Repair/abstention gate

Pass iff:

- Missing permit, invalid structured output, failed verification, unsafe patch, unsupported cloud provider, and exhausted budget all emit explicit abstention/repair evidence.
- None of those cases reports success.

## G. Package/replay gate

Pass iff:

- `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo doc --workspace --no-deps` pass in the active workspace.
- Strict package validation passes with zero findings.
- Package self-replay passes or emits a precise non-silent quarantine with reproduction steps and operator-approved deferral.

## H. Support-claim gate

Pass iff:

- `STATUS.md` and `SUPPORT_PROFILE.md` match actual evidence.
- Advanced local agent support is marked supported-local only if the verifier proves it.
- Cloud/autonomy/V10 remain deferred.

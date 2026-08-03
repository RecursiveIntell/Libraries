# Libraries Council Remediation Plan

Date: 2026-07-16
Status: implemented and verified; fresh release recording deferred pending a clean tree

## Objective

Resolve every actionable finding from
`LIBRARIES_COUNCIL_HOSTILE_AUDIT_2026-07-16.md` without treating a dirty tree,
old receipts, or a best-effort checkpoint write as a release-ready state.

## Ownership decisions

- A configured `agent-graph::CheckpointStore` is mandatory: a successful
  graph result requires creation, attempt, snapshot, and terminal persistence
  to have succeeded. No configured-store error may fall back to a UUID or be
  silently discarded. No-store and legacy `CheckpointSaver` behavior remain
  compatible.
- `llm-pipeline` receipt digests are SHA-256 bytes encoded as 64 lowercase hex
  characters. Existing 16-character `DefaultHasher` values were never valid
  SHA-256 receipts and are not comparable to the corrected values.
- Release evidence is written only by the explicit recorder after a clean,
  green source change. The verifier remains read-only and will become stricter;
  this run must not manufacture a pass while unrelated dirty paths exist.
- Legacy string storage keys stay compatible during identity migration.
  Canonical graph-run identity gains a typed `stack-ids` owner; retry lineage
  continues to use `AttemptId` and `TrialId`.

## Ordered implementation

### 1. Restore the build and receipt digest contract — complete

- `llm-pipeline/src/pipeline.rs`: constrain streaming parse output as `T`, use
  `sha2::Sha256`, and add known-vector coverage.
- `llm-pipeline/Cargo.toml` and root `Cargo.lock`: add the workspace `sha2`
  dependency.
- Format only the owned `llm-pipeline` changes before package checks.

Completed: explicit streaming parse typing, real SHA-256 receipts with a known
vector, and strict package clippy all pass.

### 2. Make checkpoint persistence fail closed — complete

- Add typed checkpoint-store operation failures in `agent-graph`.
- Propagate store creation through all four executor entry points; retain
  no-store UUID generation.
- Propagate attempt, snapshot, and terminal-store failures; a configured store
  can never yield a successful graph result after a failed write.
- Add fault-injecting checkpoint-store contract tests for every write phase and
  preserve existing no-store/legacy behavior.

Completed: every configured-store persistence operation now returns a typed
failure; fault-injection coverage verifies creation, attempt, snapshot, and
terminal failures cannot yield success.

### 3. Harden evidence verification, but defer evidence recording — complete

- Strengthen the verifier to require clean all-pass, complete, source-bound
  command evidence; test malformed, dirty, stale, reordered, and failed cases.
- Do not run the recording writer until the full workspace is green and all
  unrelated dirty artifacts are reconciled. At that point record evidence once
  and verify it read-only.

Completed: the verifier rejects dirty trees, partial/reordered gate sets, and
non-passing/malformed receipts. Fresh recording remains intentionally blocked
until the full shared worktree is clean.

### 4. Repair the ownership inventory generator — complete

- Make CSV output LF-only in the owning Python generator.
- Regenerate only after confirming the existing generated CSV deltas are
  derivative of the current source, and validate duplicate/type assertions.

Completed: all CSV writers explicitly emit LF. An isolated generator run
confirmed LF-only output; the pre-existing generated artifacts were not
overwritten.

### 5. Migrate canonical identities in two compatibility-preserving waves — queue creation complete

1. Add `GraphRunId` and `GraphCheckpointAttemptId` to `stack-ids`; migrate
   `agent-graph` run/attempt creation to canonical values while retaining
   existing string storage and wire compatibility.
2. Add `QueueJobId` and `BatchJobId` to `stack-ids`; migrate `job-queue` and
   `ai-batch-queue` generators to canonical values while retaining their
   established string storage keys. `AttemptId` and `TrialId` remain the
   distinct retry-lineage identities.

Gate: stack-ID, graph, and queue tests plus a source guard restricting new raw
UUID construction to documented storage/correlation boundaries.

## Verification and remaining release action

- `cargo fmt --all -- --check` passes.
- `cargo test --workspace --all-targets --no-fail-fast` passes.
- `cargo clippy -p llm-pipeline --all-targets -- -D warnings` passes.
- Python evidence-contract tests and bytecode checks pass.
- The read-only release verifier correctly reports that the shared worktree is
  dirty, so no release-current claim or evidence record was manufactured.

Once unrelated worktree changes are reconciled, record evidence once from the
clean, green source and then verify its evidence-only descendant read-only.

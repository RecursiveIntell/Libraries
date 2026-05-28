# 05 — Acceptance Gates and Assertions

The next pass should replace marker assertions with semantic gates. Each gate below must produce a receipt/log artifact and be runnable on a clean extracted package.

## Gate — Security & sandbox boundary

- **Relevant spec gate:** v11A Boundary/Execution Evidence
- **Assertion:** Add hostile sandbox tests and deny/quarantine receipts for .git/.env/symlink/hardlink/unicode/TOCTOU cases.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Tool exposure & permit policy

- **Relevant spec gate:** v11A Material Operation / Permit
- **Assertion:** Add tool exposure parity harness: declared vs registered vs executable vs exposed vs provider schema vs permit-used.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Patch transactionality & treatment integrity

- **Relevant spec gate:** v11A Boundary Compiler / Treatment Integrity
- **Assertion:** Replace or constrain patch_apply; require transaction receipt, before/after digests, rollback/quarantine, and post-write verification.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Command execution & environment control

- **Relevant spec gate:** v11A Execution Evidence
- **Assertion:** Require structured argv, process-group kill, output caps, environment/toolchain fingerprints, and replay mismatch taxonomy.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Provider route & local/mocking honesty

- **Relevant spec gate:** v11A Execution Evidence / Support Profile
- **Assertion:** Make local/mock/Ollama/provider routes honest; prevent local=>mock fallback without explicit degradation and support-label change.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Receipts, event logs, durability & replay

- **Relevant spec gate:** v11A Execution Evidence / Artifact Lifecycle
- **Assertion:** Introduce file locking/single-writer logs, atomic writes, fsync, hash-chain verification, corruption quarantine, and no done without receipts.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Queue, scheduler, daemon & concurrency

- **Relevant spec gate:** v11A Execution Evidence / v11B Region Scheduling
- **Assertion:** Make queue append/lease/complete atomic with leases, locks, hop receipts, idempotency proofs, and safe-mode quarantine.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Boundary compiler, JSON, schema & repair

- **Relevant spec gate:** v11A Boundary Compiler Gate
- **Assertion:** Adopt strict boundary compiler profiles, full schema support or unsupported-key rejection, durable repair receipts, and fuzz/property tests.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Bitemporal, proof, view & semantic state

- **Relevant spec gate:** v11A Bitemporal / Proof Economy / View Disclosure
- **Assertion:** Build bitemporal/proof/view reference fixture corpus and production-path differential tests.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — v11B graph, region, convergence, subtraction & causal runtime

- **Relevant spec gate:** v11B Regional Runtime Gates
- **Assertion:** Implement one minimal v11B region with right-graph misuse tests, convergence failure, syndrome, repair, support-core, oracle diff.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Schema, contract governance & generated artifacts

- **Relevant spec gate:** v11A Schema/Contract Gate
- **Assertion:** Gate generated schema changes with meta-validation, compatibility diffing, canonical digest identity, and migration receipts.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Artifact lifecycle & operator effect system

- **Relevant spec gate:** v11A Artifact Lifecycle / Operator Effects
- **Assertion:** Enforce OperatorContract effect sets and artifact lifecycle transition receipts for all material paths.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Code modularity, crate ownership & maintainability

- **Relevant spec gate:** Release Maintainability / Source Ownership
- **Assertion:** Split gravity-well modules by source-of-truth boundary and forbid new semantics in facades.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Testing quality, marker assertions & hostile fixtures

- **Relevant spec gate:** v11+ Conformance Gates
- **Assertion:** Replace marker-only assertions with semantic, negative, extracted-package, concurrency, and fuzz fixtures.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Docs, evidence, source-basis & run hygiene

- **Relevant spec gate:** Release Evidence / Source Basis
- **Assertion:** Make docs/support labels/matrix statuses gate-derived and classify active/seeded/reserved surfaces precisely.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — Config, environment, secrets & redaction

- **Relevant spec gate:** v11A Execution Evidence / Security
- **Assertion:** Add config receipt, redaction, environment fingerprint, default-policy, and migration fixtures.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.

## Gate — App/scaffold/profile readiness

- **Relevant spec gate:** Application Builder Readiness
- **Assertion:** Make scaffolds receipt-first and profile-tier honest; generated apps include conformance smoke tests.
- **Failure handling:** halt, quarantine, or emit explicit proof debt. Do not silently widen semantics.
- **Minimum evidence:** command output, fixture refs, changed-file summary, before/after receipts where applicable.


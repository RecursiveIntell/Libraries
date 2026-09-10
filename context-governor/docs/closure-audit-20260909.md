# Context Governor closure: hostile audit and execution record

Date: 2026-09-09. Status: implementation in progress, NOT certified or activated.
Source baseline: Libraries main `b63a341ea0e947979fac40302050804d52ae2bdb`.
Ares companion baseline must be freshly resolved and recorded before edits.

## Authority and allowed scope

The operator requested a hostile audit followed by implementation, incremental commits and pushes to isolated PRs. No merge, deployment, installation on an operator host, live-store migration, receipt re-signing, key rotation, force-push, or mutation of another agent's branches is authorized. In particular, do not modify `hermes-ares-recon`.

Libraries/context-governor owns receipt formats, authentication, lineage, canonical error semantics and derived V3 storage. Ares owns adapter projection and host lifecycle. Existing V1/V2 receipt bytes remain unchanged. SQLite and V3 remain rebuildable projections, never independent authority. Do not create a new database, runtime, or generic cryptographic framework to repair these paths.

## Hostile audit of the earlier plan

1. **P1 V3 correctness / activation blocker:** random encryption nonces are generated for every reference while the blob path is keyed by plaintext digest and existing files are silently skipped. The defect can occur in a single fresh migration with shared ancestor evidence, not just during resume. Add a failing multi-generation witness before repair.
2. **P1 publication integrity:** `exists()` followed by success is not verified reuse, and check-then-rename is not no-clobber publication. Require verified immutable objects and explicit collision/incomplete-output failures. Prove interruption and concurrency semantics rather than naming a function atomic.
3. **P1 trust boundary:** hashing content using digests supplied by the same untrusted manifest does not authenticate receipt/source membership. Derive expected manifests from authenticated V2 evidence and confine reads to digest-derived paths. Reject symlinks and unsupported schemas. Bound decompression and input sizes.
4. **P1 contract / secret handling:** migration options derive Debug despite containing a key; serialization exclusion is not logging protection. Public migration outcomes need unambiguous V2 coverage versus full-corpus coverage. Missing source roots and invalid options must not yield vacuous success.
5. **P1 CLI contract:** structured errors already exist in Rust but are lost through Display output; Ares substring matching can misclassify them. Add a negotiated versioned wire envelope, retaining existing human CLI behavior only through an explicit mode. Never infer certified behavior from untyped stderr. Do not leak arbitrary input/paths/secrets through diagnostics.
6. **P1 lifecycle:** process-tree enumeration is not containment. The previous proposed psutil walk does not prove fork/escape safety. Preserve the existing platform admission boundary and inspect governed descriptor support before claiming Windows certification. POSIX parent exit does not prove group death; all waits/drains must be bounded. Process death is also not proof that a mutation did not commit.
7. **P1 validation:** `hardening-and-release` does not explicitly include standalone context-governor in its workspace matrix. A green root workflow is not a certificate for this crate. Add direct locked crate checks, executable baseline regression witnesses, and exact-head receipts.
8. **P2 scope/exactness:** serialized typed Message equivalence is not original JSON-byte equivalence. Preserve original receipt digests separately; do not claim byte-exact historical JSON from normalized Message serialization. Provider-token exactness remains an egress-owner gate, not a reason to broaden this storage repair.
9. **P1 rollout:** the previous plan's instruction to land on main and update the reconciliation branch conflicts with the current operator authorization. This work stays in independent draft PRs. Cross-repository compatibility and adoption are explicit later gates.
10. **Proof blocker, not an observed source defect:** full private corpus certification requires the exact corpus and permitted historical keys. Synthetic tests and hosted CI must not be substituted for this evidence. Never upload private receipts/keys to public GitHub or public Actions artifacts.

Severity applies to the affected optional/certified surface. These findings do not establish live V2 data corruption or justify disabling unrelated runtime capabilities.

## Revised execution sequence

A. Snapshot current owners, instructions, branch state, dependencies and relevant tests. Persist the audit before code changes.
B. Add direct Context Governor CI and failing encrypted-reuse/coverage witnesses. Record baseline failure, not only candidate pass.
C. Repair V3 within its current owner: strict format/config, immutable verified reuse, source-bound validation, bounded IO, explicit resume/coverage and redacted secret handling. Retain V2 authority and existing legacy read semantics unless explicitly versioned.
D. Add a negotiated failure-envelope mode in the existing CLI; expose capabilities. Test each relevant error class without dependence on Display wording. Keep recovery decisions outside the error wire.
E. Repair the Ares adapter in a separate PR: capability admission, strict envelope validation, final budget field validation and bounded timeout fencing. Keep mutation-timeout reconciliation with existing pending/activation owners; never blindly replay a possibly committed effect.
F. Run targeted real-path tests and complete crate tests/Clippy/fmt/ROI/release generation witness. Run final-head Ares checks. Add regression tests for audit findings rather than suppressing gates.
G. Certify the private corpus on a read-only copy only if available; otherwise deliver the executable verifier and report that gate NOT RUN. Produce a final source/commands/test/rollback receipt. Remove temporary transport helpers.

## Required invariants and acceptance gates

- Shared encrypted evidence reads correctly across generations and repeated references.
- Same-source/same-config resume verifies existing state; wrong key/config/source, malformed objects, missing/tampered artifacts cannot report completion.
- Paths cannot escape projection scope. Invalid digest/schema/duplicate fields and excessive decompression are rejected.
- V1-only, mixed and empty/missing input are accounted distinctly; pending receipts are not committed evidence.
- V3 manifests never authenticate themselves; V2 source membership is checked.
- Every async timeout is bounded and cannot imply rollback or successful cancellation of an already committed mutation.
- Certified failures never depend on English error text; protocol mismatches fail before protected mutation.
- No changes outside the owner-approved paths, no weakened checks, no global dependency upgrade, no live mutation.

## Rollback and final state

Before adoption, close/revert the isolated PRs. Candidate V3 output may be quarantined and rebuilt from immutable V2; do not repair V3 by editing or re-signing source evidence. Preserve a failed projection for inspection rather than silently deleting it. Certified adapter/binary adoption must be coordinated; mismatches are explicit failures, not compatibility fallback.

Completion requires a final receipt with exact commit SHAs, actual test results and skips, baseline counterexamples, source/corpus hashes, migrations performed (expected: none live), residual proof debt, and auditor rerun commands. This initial record makes no claim that code or tests have been completed.

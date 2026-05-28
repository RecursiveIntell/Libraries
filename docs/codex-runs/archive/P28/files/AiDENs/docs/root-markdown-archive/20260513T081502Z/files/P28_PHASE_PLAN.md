# P28 Phase Plan — v11A Constitutional Material-Operation Kernel

## Phase 00 — Source basis, scope lock, and hard stop rules

**Goal:** prevent P28 from becoming an unbounded cleanup bucket.

Tasks:

1. Record source basis and P27 package/evidence baseline.
2. Import Claude audit into bug absorption matrix.
3. Add P28 status/evidence manifest template.
4. Write non-goals into support profile target.
5. Create phase report directory and templates.

Exit gate:

- `P28_SOURCE_BASIS.md` complete.
- `P28_BUG_ABSORPTION_MATRIX.*` complete.
- No code changes yet except doc scaffolding.

## Phase 01 — Immediate P0 bugfix lane

**Goal:** close correctness/security bugs before adding v11 scaffolding.

Required fixes/tests:

- deterministic or documented identity behavior for queue leases and generated artifacts
- `safe_relative` fails closed on symlinks/outside root
- profile expansion returns `Result`, no production panic
- patch write creates no dirty dirs on failed write
- symlink escape regression for `resolve_target_sandboxed_path`
- known limitations empty register logic
- waiver/blocked traceability semantics
- convergence/degraded validation
- history preservation semantics
- aggregate semantic status reflects degraded subchecks

Exit gate:

- targeted regression tests for every P0.
- no release blocker remains unquarantined.

## Phase 02 — Artifact identity and lifecycle kernel

**Goal:** define v11A envelope/lifecycle law in code without stealing sibling authority.

Implement or admit:

- `ArtifactEnvelopeV1`
- `ArtifactManifestV1`
- `ArtifactLifecycleStateV1`
- `ArtifactTransitionReceiptV1`
- deterministic/content-addressed ID helper for replay-sensitive artifacts
- stable schema/digest identity helper

Rules:

- runtime artifacts are execution-authoritative only unless imported/promoted through canonical path.
- display summaries are not canonical receipts unless they satisfy the canonical contract.

Exit gate:

- lifecycle tests cover create → validate → admit/project/propose → verify/refute/quarantine/promote/supersede/retire.

## Phase 03 — ExecutionContextEnvelope and receipt immutability

**Goal:** make execution conditions artifact semantics, not logs.

Implement or admit:

- `ExecutionContextEnvelopeV1`
- `ExecutionContextRefV1`
- `ToolCallReceiptV1`
- `OperatorInvocationReceiptV1`
- immutable `CanonicalEventLog` sequence + previous digest chain
- attempt/content-addressed run bundle store

Minimum required fields:

- execution id
- trace id
- span/operation id
- attempt family id
- retry family id
- queue/message lineage
- provider/tool route
- environment/runtime fingerprint
- start/end recorded time
- deadline/budget allocation and consumption
- timeout/cancellation/truncation/degradation state
- replay handle or non-replayability reason
- redaction/disclosure state

Exit gate:

- no material done state without receipts.
- run bundle overwrite regression fails.

## Phase 04 — Material operation registry and effect system

**Goal:** no material operation gets to just do work.

Implement or admit:

- `OperatorContractV1`
- `OperatorEffectV1`
- `MaterialOperationRegistryV1`
- `OperationConformanceReportV1`

Minimum registered operators:

- agent spec validate/doctor/new
- runner turn execution
- provider route
- repo read/list/stat/search
- patch propose/apply
- command/check run
- package generation/validation/self-replay
- memory-grounding adapter call
- report/final done transition

Exit gate:

- effect safety tests block undeclared effects.
- all registered operators emit invocation receipts.

## Phase 05 — Boundary compiler profile and structured-output hardening

**Goal:** every structured boundary becomes a compiler front end.

Implement or admit:

- `BoundaryCompilerProfileV1`
- `BoundaryCompileReceiptV1`
- `BoundaryRepairReceiptV1`
- `TreatmentIntegrityReceiptV1`
- schema meta-validation hook
- canonical JSON/digest helper

Fixes mapped:

- duplicate-key rejection stays hard.
- `ArtifactKindV1` schema must enumerate valid variants or use declared open-string profile.
- schema catalog avoids hardcoded singleton report IDs for run-specific reports.

Exit gate:

- adversarial JSON fixtures pass.
- parser repair cannot change treatment silently.

## Phase 06 — Proof economy, waiver law, and release readiness

**Goal:** proof gaps are explicit and enforceable.

Implement or admit:

- `ProofProfileV1`
- `ProofObligationV1`
- `ProofDebtLedgerV1`
- `ProofWaiverReceiptV1`
- `PromotionEligibilityReportV1`

Fixes mapped:

- waiver does not satisfy blocked traceability row unless state is `Waived`.
- degraded surface blocks release unless explicit lawful waiver.
- fabricated approval receipt IDs do not satisfy compaction approval.

Exit gate:

- proof waiver != proof regression.
- degraded release readiness regression.

## Phase 07 — SemanticState, view disclosure, and degradation law

**Goal:** claim-like outputs carry semantic product, not just payload/confidence.

Implement or admit:

- `SemanticStateV1`
- `ViewDisclosureV1`
- `DegradationRecordV1`
- exact/degraded/support/proof carriers
- retrieval/query report upgrades

Fixes mapped:

- provider degraded reason is `None` when no reason codes exist.
- retrieval timeless fallback semantics validated.
- output text extraction has bounded structured summary, not arbitrary full JSON blob.

Exit gate:

- degraded answers cannot masquerade as exact.
- view widening is inspectable on CLI/audit surfaces.

## Phase 08 — Tool, patch, sandbox, and command hardening

**Goal:** make supported-local tools evidence-grade enough for v11A declared path.

Required changes:

- revalidate open/read/write paths after canonicalization where practical
- block/fail safe on symlinked parents for create/write
- rollback or avoid dirty directory creation on failed patch writes
- component-based `.git`/`target` skip logic
- full-list digest or total count for truncated listings
- fail, do not empty-digest, on `file_stat` read/hash failure
- command allowlist tests and receipts
- timeout output marked partial/truncated
- busy-loop replaced or justified with low-cost wait strategy

Exit gate:

- hostile symlink and patch fixtures pass.
- no security gate remains untested.

## Phase 09 — Bitemporal reference fixtures and differential check

**Goal:** create the reference-conformance seam before expanding memory behavior.

Implement:

- tiny bitemporal fixture/reference interpreter for declared local memory/query path
- valid-time only, recorded-time only, combined `as_of(valid, recorded)` fixtures
- retroactive correction and supersession fixtures
- stale projection fixture
- degraded disclosure fixture

Exit gate:

- production declared path is differentially checked against reference behavior.

## Phase 10 — Package/replay truth and z.py hardening

**Goal:** make the package report self-verifying and semantically honest.

Required changes:

- `safe_relative` fails closed and never falls back unsafely
- non-UTF-8/binary detection is stricter
- archive SHA is labeled zip-byte hash
- content manifest digest is separately computed if needed
- P28 manifest references actual final package sidecars
- skip-cargo/degraded replay downgrades aggregate semantic status

Exit gate:

- package self-replay full pass.
- degraded replay is honestly classified.

## Phase 11 — Large-file containment and module split

**Goal:** prevent v11A core from being buried in megafiles.

Targets:

- split `aidens-contracts/src/lib.rs` into modules
- split `aidens-runner/src/lib.rs` into execution, receipts, provider/tool, finalization, replay modules
- keep `aidens-cli` a display/adapter layer, not semantic owner

Exit gate:

- no loss of public API without compatibility record.
- module docs clarify owner plane.

## Phase 12 — v11B/v11C reserved containment

**Goal:** reserve future surfaces without activating them prematurely.

Tasks:

- label v11B region/subtraction artifacts as draft/advisory only
- add v11C activation-level enum/stubs
- external admission defaults to quarantine
- agency-risk classification preserved where already enabled
- learned/advisory systems cannot waive proof or promote truth

Exit gate:

- no v11B/v11C active claim.
- no incompatible horizon artifact families.

## Phase 13 — Adversarial conformance suite

**Goal:** test failure modes deliberately.

Fixtures must include:

- duplicate JSON keys
- schema mismatch
- parser repair changing treatment
- symlink escape
- patch writes failing after dir creation
- timeout partial output
- retry/degraded aggregate status
- stale projection answering current query
- proof waiver treated as proof
- degraded release surface
- storage graph used as inference graph, reserved test only
- subtraction removing support core, reserved test only
- personalized advice without disclosure, reserved test only

Exit gate:

- every fixture has expected failure/pass semantics.

## Phase 14 — Docs, support profile, and operator quickstart convergence

**Goal:** keep support claims aligned with code.

Update:

- `SUPPORT_PROFILE.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- operator quickstart
- P28 support traceability
- known limitations register

Exit gate:

- docs claim no more than evidence supports.

## Phase 15 — Final audit, verifier, package, replay

**Goal:** produce final package and handoff.

Tasks:

- run full command set
- fill `P28_STATUS_EVIDENCE_MANIFEST.json`
- write `docs/p28/P28_FINAL_AUDIT_REPORT.md`
- write `handoffs/p28/FINAL_AUDITOR_HANDOFF.md`
- package strict
- package self-replay full
- package self-replay degraded if needed with honest aggregate status

Exit gate:

- all acceptance gates satisfied or release-blocking exceptions recorded.

# RecursiveIntell Semantic Law V1

**Status:** proposed cross-repository contract  
**Scope:** RecursiveIntell-owned active repositories and RecursiveIntell-authored integration boundaries  
**Normative language:** MUST / MUST NOT / SHOULD / MAY are intentional.

This document normalizes semantic vocabulary already present across the stack. It does not grant authority, activate runtime behavior, migrate durable state, or retroactively change legacy artifact meaning.

## 1. Prime directive

A semantic concept has one canonical owner for one explicitly named domain.

A consumer may reference, transform, cache, project, transport, display, or verify an owner's artifacts only within its declared contract. A consumer MUST NOT silently become a second semantic owner.

When ownership is unresolved, the state is **reconciliation required**. Ambiguity is not permission to choose whichever copy is convenient.

## 2. Scope every strong word

The following words MUST be qualified by domain in cross-repository contracts and public claims:

- canonical
- owner
- authority
- truth
- verified
- success
- receipt
- replay
- rollback
- recovery

Examples:

- good: "canonical semantic-memory SQLite state"
- good: "execution authority for this effect"
- good: "locally verified on commit X under test profile Y"
- bad: "the source of truth" with no domain
- bad: "verified" with no predicate/evidence boundary

Use **record**, **assertion**, **claim**, or **observation** instead of unqualified "truth" when objective truth is not what the artifact establishes.

## 3. Ownership vocabulary

### Canonical semantic owner

The component that defines the meaning, invariants, mutation rules, and versioning of an artifact family.

There MUST be exactly one canonical semantic owner for a named artifact family.

### Canonical source

The one editable source-code location from which releases or mirrors for a package/artifact family derive.

A package MAY have multiple published/mirrored repositories, but only one canonical source at a time.

### State owner

The component that serializes and commits mutations for a specific durable state family.

A canonical semantic owner and a state owner may be the same component, but the terms are not interchangeable.

### Execution owner

The component permitted to apply a particular external or effectful operation.

Execution ownership does not imply semantic ownership of the input records.

### Resource owner

Human, device, profile, account, filesystem, or service ownership. Do not shorten this to "owner" at a cross-repository boundary.

### Replica

A copy derived from canonical state. A replica MUST declare its source identity/generation and MUST NOT accept independent semantic mutations that would create dual truth.

### Projection

A derived view. A projection is non-authorizing and non-canonical by default. It MUST identify its source basis and version/exactness/lossiness where material.

### Cache / sidecar / index

Disposable or rebuildable derived acceleration state. It MUST NOT become canonical merely because it is faster or persisted.

## 4. Identity and digest law

Identity and content identity are different classes.

### Opaque object identity

A typed ID identifies an object/record/run/attempt. UUIDv4 is allowed when the canonical owner permits it.

Opaque IDs MUST NOT be used as evidence that two payloads are content-identical.

### Lineage identity

A lineage/family ID remains stable across versions of the same logical lineage.

### Version identity

A version ID changes when the material semantic version changes.

### Content identity

A content digest is derived from bytes/content and is not a random object ID.

Every cross-repository digest contract MUST identify its algorithm, canonicalization or exact preimage, semantic domain, and schema/profile version. These fields are part of the contract, not properties inferred from a hex string. No digest algorithm is universal across domains.

One **opt-in V2 profile** for a domain that explicitly adopts structured JSON content identity may combine strict JSON admission (including duplicate-key rejection), RFC 8785 JCS canonical bytes, an explicit schema identifier/version and domain separator, and BLAKE3-256. This is not a retroactive rule for other digest families or a mandate to adopt that profile. The owning protocol defines its bytes and algorithm; a consuming adapter MUST NOT recanonicalize or rehash them into stronger evidence.

**boundary-compiler** owns its structured-JSON canonicalization contract. **stack-ids** supplies shared typed identity/BLAKE3 primitives where appropriate; neither owns every domain's digest meaning. Same-named ContentDigest implementations remain **reconciliation required** only where their actual wire domains overlap.

Existing SHA-256, HMAC-SHA256, BLAKE3, and other receipt families remain valid under their declared V1 contracts. Algorithms MUST NOT be silently rewritten for aesthetic consistency.

A raw 64-character hex string MUST NOT be treated as a cross-stack digest contract without algorithm/preimage/domain/version semantics.

## 5. Boundary and schema law

Cross-repository typed ingress MUST:

- reject malformed encodings;
- reject duplicate object keys where JSON is the wire form;
- validate schema/version before semantic use;
- reject unknown semantic widening unless the owning schema explicitly allows it;
- preserve caller/owner identifiers instead of regenerating them during transport.

Canonicalization proves deterministic bytes, not domain validity, authorization, truth, or trust.

## 6. Authority law

**Authority** means the ability to admit, deny, authorize, or fence a state/effect transition.

Identity, provenance, evidence, signatures, relevance scores, model outputs, and trust labels do not automatically confer authority.

Prefer qualified terms:

- assertion authority
- action/effect authority
- admission authority
- delegation authority
- operator authority
- execution authority

### Capability

A capability describes a maximum allowed class/surface. It is not necessarily active authority for a concrete effect.

### Grant

A grant is a durable/scoped delegation basis from which current authority may be derived. A grant is not a consumed effect permit.

### Approval

An approval is a decision/witness satisfying a policy condition. It MUST NOT be treated as a timeless execution permit.

### Permit

A permit is bounded current authorization for a concrete operation/effect.

A material permit SHOULD bind at least:

- effect/request identity;
- actor/principal;
- target/scope;
- policy version/digest;
- current generation/incarnation where applicable;
- validity window;
- budget/use count;
- approval basis.

Retired or superseded generations MUST NOT be reopened merely to reuse old permits.

### Lease / custody

A lease is temporary custody bound to a holder and validity condition. It does not transfer canonical ownership of the underlying semantic family.

### Authentication is not authorization

A valid signature/MAC proves only the authentication/integrity statement defined by that protocol. Policy must separately decide admission/authority.

## 7. Receipt law

A **receipt** is an immutable typed record that an issuer recorded an operation, observation, decision, transition, or measurement under a declared schema.

A receipt proves only the predicates declared by its schema and verification method.

A receipt does **not**, by existence alone, prove:

- objective truth;
- independent external completion;
- task success;
- authorization;
- currentness;
- production readiness;
- benchmark generality.

Receipt families SHOULD be explicitly named when ambiguity matters, for example:

- admission receipt
- execution receipt
- transition receipt
- observation receipt
- control/policy receipt
- benchmark receipt
- export receipt
- compensation receipt

Integrity mechanisms are distinct:

- **content digest** — unkeyed content identity/integrity;
- **MAC** — keyed integrity/authenticity under shared secret;
- **signature** — asymmetric issuer authentication;
- **hash chain** — ordering/tamper evidence when anchored to a trusted head.

Do not call these interchangeable "proof".

## 8. Evidence, claims, and retrieval

### Observation

What a named component observed at a named time. Observation does not grant authority.

### Evidence

An artifact relevant to evaluating a claim. Presence/retrieval of evidence does not itself establish support.

### Claim

A proposition evaluated against evidence. Claim support/refutation/unknown semantics belong to the canonical claim/evidence owner.

A memory "fact" is a stored fact-record/assertion unless separately verified. Retrieval score is not truth confidence.

### Candidate

A proposed/retrieved/approximate item that has not crossed its promotion/admission gate.

Candidates MUST NOT self-promote into canonical or verified state.

### Verification

"Verified" MUST name or bind:

- the predicate/check that passed;
- exact artifact/source revision;
- relevant environment/population/scope;
- verifier or procedure;
- time/evidence reference where material.

A test pass is evidence for the tested predicate only.

## 9. Outcome law

Use explicit outcome classes across effect boundaries:

- **reported outcome** — a participant says what happened;
- **owner-observed outcome** — the relevant owner read back/observed state;
- **externally confirmed outcome** — an independent external system confirmed state;
- **ambiguous/indeterminate outcome** — available evidence cannot safely decide.

Do not collapse these into an unqualified success.

A reported-success receipt MUST NOT automatically settle an externally effectful operation when independent state may differ.

## 10. Time law

Time dimensions are distinct:

- **valid_time** — when the record/assertion is declared effective in the domain;
- **observed_time** — when an observer measured/encountered it;
- **recorded_time / recorded_at** — when the canonical state owner durably recorded it;
- **issued_at** — when an issuer created a receipt/grant/etc.;
- **expires_at** — end of validity.

valid_time MUST NOT be documented as "when objectively true" unless the domain actually guarantees that stronger claim.

## 11. Version / revision / generation / epoch / incarnation

These terms MUST NOT be used interchangeably in new cross-repository contracts.

- **schema version** — wire/contract compatibility lineage;
- **revision** — monotonic change to configuration/specification within an identity;
- **generation** — monotonic authority/head lineage; retirement fences prior generations;
- **epoch** — invalidation/accounting namespace for derived state such as cache, retrieval, export, or replication;
- **incarnation** — identity of one concrete store/daemon/enrollment lifetime.

New generic fields named only version or epoch SHOULD be avoided when the intended class is one of the above.

## 12. Mutation law

Material semantic state SHOULD be append-plus-supersession.

Do not destructively rewrite history merely to present a new current view.

Destructive erasure/forgetting is allowed only when the owning domain defines it explicitly and preserves whatever tombstone, closure, receipt, or proof obligations that domain requires.

Derived state may be rebuilt rather than superseded when its owner explicitly classifies it as rebuildable.

## 13. Retry and idempotency law

- **operation/idempotency identity** — stable identity of the requested operation;
- **AttemptId** — one logical retry family within one retry-owner boundary;
- **TrialId** — one concrete execution try within that attempt.

Same operation identity + same canonical request MAY reconcile/read back/replay according to the owner protocol.

Same operation identity + different canonical request MUST be rejected as a collision.

Changing retry owner, explicit re-enqueue, or a semantically new request requires a new attempt according to the owning protocol.

## 14. Replay / resume / recovery / reconciliation / rollback / compensation

These are different operations.

### Recorded replay

Re-emits recorded artifacts/results. It does not re-call a provider or re-execute effects.

### Deterministic re-execution

Re-runs a declared deterministic, side-effect-free or separately authorized computation from recorded inputs. It MUST be named as re-execution, not silently called replay.

### Resume

Continues an incomplete run from a checkpoint.

### Checkpoint

A durable capture sufficient for a specifically declared resume path. A checkpoint is not generic execution authority.

### Recovery

Restores local operability/consistency after failure. Recovery MUST NOT infer unknown external-effect outcomes.

### Reconciliation

Resolves ambiguity by owner readback, exact identity comparison, and/or declared evidence. Reconciliation grants no authority by itself and MUST NOT imply automatic effect retry.

### Rollback

Restores an owned reversible local state to a declared prior target. It does not mean history disappeared.

### Compensation

A new effect intended to counter a prior effect. Compensation is not rollback and must have its own authority and receipt.

## 15. Projection / promotion law

A projection, model output, specialist report, retrieved memory, approximate candidate, benchmark result, or advisory decision is non-authorizing unless a canonical owner explicitly admits/promotes it.

Promotion MUST be an explicit owner transition with:

- source identity;
- policy/criteria;
- target artifact family;
- disposition;
- durable evidence/receipt when material.

No advisory artifact may self-promote.

## 16. Attestation and trust

An attestation is an issuer-authenticated statement about an artifact/state.

Authenticity of the attestation does not establish that its statement is true.

Trust-root/admission policy is separate from signature verification.

External/legacy artifacts that lack a supported verifier MUST remain external/quarantined rather than being silently upgraded.

## 17. Source presence, activation, and qualification

These are distinct:

- source present;
- compiled/built;
- tests passed;
- installed;
- configured;
- activated;
- runtime observed;
- externally/independently verified;
- release/production qualified.

No earlier state implies a later state.

## 18. Compatibility law

Old artifact meaning is immutable.

When semantics change:

- add an explicit versioned contract;
- add an explicit bridge/migration if supported;
- preserve legacy verification/readback when required;
- do not make an old field name silently mean something stronger.

"Compatibility" MUST NOT mean semantic widening.

## 19. Mirror/source topology law

A duplicated active package MUST have one declared canonical source.

Other copies MUST be one of:

- generated mirror;
- pinned release mirror;
- vendored snapshot;
- archived/frozen historical copy.

Two independently editable copies with the same package/artifact identity are forbidden.

A mirror MUST NOT be newer than its declared canonical source unless it is being promoted through an explicit source-ownership transition.

Same package name + same version + materially different source is a semantic drift failure.

## 20. Cross-repository language rules

For new APIs/docs:

- prefer canonical_<domain>_state over truth;
- prefer qualified *_owner;
- prefer qualified *_authority;
- use reported_*, observed_*, verified_* for outcomes/evidence;
- use recorded_replay vs reexecute;
- use projection for rebuildable derived state;
- never use "receipt proves success" without the exact predicate;
- never use "source of truth" without the domain and canonical owner.

## 21. Proposed domain-qualified concept owners

These are proposed domain-qualified owner assignments based on inspected source
and the operator-selected claim/memory directions. They are not evidence that
every adapter, release mirror, or pending PR already conforms. Source-location
conflicts remain separate in OWNER_REGISTRY_V1.yaml.

| Concept family | Canonical semantic owner |
|---|---|
| Cross-crate typed IDs and identity vocabulary | stack-ids |
| Strict JSON boundary admission / RFC 8785 JCS | boundary-compiler |
| Bitemporal record/query semantics | bitemporal-runtime |
| Claim/support judgment, admission, contradiction, proof-debt ledger events | claim-ledger |
| Causal/effect verification EvidenceBundle and Forge export records | semantic-memory-forge |
| Forge -> memory projection transformation | forge-memory-bridge |
| Canonical memory storage/retrieval/governed mutation | semantic-memory |
| Delegated authority artifacts | authority-delegation |
| Effect lifecycle artifacts | effect-runtime |
| Verification control cases/plans/receipts | verification-control |
| Verification policy/approval/execution permits | verification-policy |
| Continuity/incident/recovery artifacts | continuity-runtime |
| Effective profile/constitution composition | profile-runtime |
| Attestation/trust-root/transparency artifacts | attestation-exchange |
| Assurance/release-readiness artifacts | assurance-runtime |
| Supported contract schema generation | contract-schema-gen |
| Core graph execution semantics | ri-agent-graph |
| Graph MCP transport/persistence/operator surface | agent-graph-mcp |
| Provider-backed LLM payload pipeline | llm-pipeline |
| Multi-device memory routing/replication control | mnemes |
| TurboQuant codec/approximate sidecar semantics | turbo-quant |
| FibQuant codec semantics | fib-quant |
| Shared KV-pool semantics | proveKV |
| Native RecProv receipt provenance protocol | RecProv |
| Recursive Agent execution-kernel/runtime semantics | recursive-agent |
| Ares-specific managed runtime/continuity orchestration | Ares |

The table does not resolve which duplicate repository path currently owns a duplicated package's editable source. That is intentionally separated in OWNER_REGISTRY_V1.yaml.

## 22. Enforcement target

After P0/P1 owner and duplicate classification, an early blocking no-new-drift
gate SHOULD use existing ownership tooling and `contract-schema-gen` for its
registered wire schemas. Snapshot existing debt with path, owner, failure mode,
reason and removal condition. Do not make a second schema registry or a
universal semantics crate. Behavioral owner tests still decide conformance.

The gate should reject NEW:

1. undeclared duplicate package sources;
2. forbidden shared-semantic drift within a proven directional release mirror;
3. cross-boundary digest profiles lacking algorithm, preimage, domain or version;
4. new unqualified authority/owner/replay semantics at public boundaries;
5. projection/candidate writes that bypass owner admission;
6. same idempotency key with divergent canonical payload;
7. unversioned semantic widening;
8. proven mirrors diverging outside their declared packaging differences.

The first adoption step is documentation + owner registration only. Runtime migrations must be separate, reviewable changes with their own validation and rollback.

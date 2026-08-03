# Mnemes Synchronization — Evidence-Led Completion Plan

**Status:** controller-reconciled design; not an implementation authorization  
**Date:** 2026-07-28  
**Scope:** `semantic-memory`, `semantic-memory-mcp`, and the canonical Mnemes control-plane repository once its live checkout is identified.  
**Primary outcome:** device-owned semantic-memory primaries can replicate typed, replayable mutations to admitted server replicas without silent conflict overwrite, inferred delivery, or uncontrolled live-store mutation.

---

## 1. Verdict

The current work is a **local replication primitive**, not Mnemes synchronization.

It is now build-compatible and source-certified, but it lacks the contracts required to call it cross-device sync:

- no authenticated peer transport;
- no registered identity binding for CLI-supplied device/store IDs;
- no canonical wire envelope or protocol negotiation;
- no durable receiver apply ledger or sender acknowledgement watermark;
- no payload-digest conflict detection for duplicate sequence numbers;
- no canonical replay dispatcher covering the semantic mutation inventory;
- no shadow bootstrap, cutover discipline, or sync-worker lifecycle.

The correct next move is **not** to add a daemon or HTTP endpoint. First close the canonical mutation/replay contract and prove one disposable device-primary → server-replica vertical slice.

---

## 2. Evidence basis and claim boundary

### 2.1 Source-verified facts

| Fact | Evidence | Consequence |
|---|---|---|
| `MAX_SCHEMA_VERSION` is `37`. | `semantic-memory/src/db.rs:934`; source certification passed. | The restored baseline preserves the current schema ceiling. New persistence requires a forward-only migration owned by the canonical schema owner. |
| V37 creates `mutation_journal(home_device_id, store_id, sequence, operation_kind, payload, created_at)` with unique `(home_device_id, store_id, sequence)`. | `semantic-memory/src/journal.rs:29-42`. | There is an outbound ordered payload stream primitive, not a complete replication protocol. |
| Local mutation and journal append can share a SQLite transaction. | `mutate_and_journal`, `journal.rs:76-108`. | This is the right local atomicity foundation, but only for covered mutation paths. |
| Export detects sequence gaps. | `export_contiguous`, `journal.rs:110-173`. | The sender can fail closed on a non-contiguous stream. |
| Replay checks only `(home_device_id, store_id, sequence)` and returns `AlreadyApplied` without comparing payload. | `journal.rs:190-233`. | A same-sequence/different-payload attack or divergence is silently accepted today. This is a P0 semantic defect for sync admission. |
| Actual production journal use was inspected on fact insertion paths; broad coverage is unproven. | `semantic-memory/src/knowledge.rs` inspection and search. | Messages, documents/chunks, graph changes, episodes, imports, redactions, supersession, and deletion require a complete write-path inventory before continuous sync. |
| MCP accepts `--mnemes-device-id`, `--mnemes-store-id`, `--mnemes-required` and calls `MemoryStore::configure_replication`. | `semantic-memory-mcp/src/main.rs:273-283`. | These flags now configure local journaling identity only. They do not create a Mnemes transport or establish peer trust. |
| Existing replication E2E tests are raw in-memory SQLite primitive tests with a caller-provided replay closure. | `semantic-memory/tests/journal_replication_e2e.rs:1-148`. | They do not prove public `MemoryStore` mutation → canonical payload → network/transport → fresh replica → durable ack. |

### 2.2 Locally reproduced certification

The restored source passed:

```text
cargo fmt -p semantic-memory -- --check
cargo test -p semantic-memory --lib             # 117 passed, 3 intentionally ignored
cargo test -p semantic-memory --tests            # passed
cargo clippy -p semantic-memory --all-targets --all-features -- -D warnings
cargo check -p semantic-memory-mcp --features full
```

The MCP full build emits one non-blocking pre-existing unused-variable warning in `semantic-memory-mcp/src/server.rs`; semantic-memory strict Clippy is clean.

### 2.3 Council receipt and reconciliation

A three-lane Agent Graph council completed:

```text
graph:    mnemes-sync-design-council
graph id: mnemes-sync-design-council
version:  sha256:2b6640ef6b33bb0cb502365275230c40f87f6b5e9a80ab7b2a3af81e1d00e868
run:      run-19fab2298b5-2
runtime:  47.765 seconds, 7 graph nodes
receipt:  integrity-only; volatile persistence (no graph integrity key)
```

Raw candidate: `2026-07-28-mnemes-sync-council-candidate.md`.

The protocol and rollout lanes supplied useful candidate ideas. The consistency/security lane was empty, and the synthesis was truncated. The council therefore **does not constitute a complete plan or verification evidence**.

| Council recommendation | Controller decision |
|---|---|
| Loopback transport first; network transport later. | **Adopt.** It enables deterministic contract tests before external effects. |
| mTLS/derived key identities/TOFU. | **Deferred decision.** First inspect the actual Mnemes device registry and existing authenticated service. Do not create a second identity system. |
| Batch envelopes, per-entry responses, watermarks. | **Adopt with a stricter invariant:** only the longest contiguous authenticated-and-applied prefix advances the watermark. |
| Shadow replica then reconciliation then explicit cutover. | **Adopt.** The server remains a replica; it never becomes a silent second writer. |
| Server-authoritative sync. | **Reject wording.** Server replication authority is not canonical content authority. Each home device remains the normal writer for its own shard generation. |

---

## 3. Non-negotiable invariants

1. **Canonical owner separation.**
   - `semantic-memory` owns typed canonical mutations, replayable mutation construction, replay semantics, and transaction boundaries.
   - Mnemes owns device registry, grants, peer admission, transport, sync orchestration, operational receipts, and rebuildable routing/control-plane projections.
   - `semantic-memory-mcp` exposes configuration/status and controlled operator actions; it must not invent a parallel sync semantic layer.

2. **One normal writer per shard generation.** The home device primary owns normal writes for `(home_device_id, store_id, generation)`. A server replica is a replay target, never an unrestricted writable mirror.

3. **No payload invention.** Replication sends owner-produced canonical bytes plus a verified digest. It must not reconstruct mutations from affected IDs, projections, textual descriptions, or digest-only metadata.

4. **Atomic replica application.** Canonical semantic replay, receipt/apply-ledger write, and contiguous-watermark update occur through the *same SQLite transaction handle*. If any step fails, none may commit.

5. **No silent conflicts.** Reusing an admitted source sequence with a different payload digest is a typed conflict. The entry is quarantined with retained evidence; the contiguous watermark cannot pass it.

6. **No inferred delivery.** Sender progress advances only on a durable receiver acknowledgement bound to the source identity, stream generation, accepted contiguous sequence, batch digest, and receiver receipt.

7. **Fail closed.** Unknown protocol version, unknown mutation kind, malformed payload, payload digest mismatch, untrusted/revoked peer, store-scope violation, missing durable state, or a detected gap blocks sync without local data mutation.

8. **Local-first safety.** Network failure never blocks local mutation unless an explicit policy says so. It leaves a local outbox backlog and an observable degraded state.

9. **SQLite backup discipline.** Never copy a live WAL SQLite file. Bootstrap/cutover uses SQLite online backup or an admitted export envelope, then integrity/schema/count/digest validation and a scratch restore drill.

10. **Authority stays explicit.** Sync arrival is not claim truth, assertion authority, or action authority. It only preserves source-owned semantic mutations and provenance.

---

## 4. Target architecture

```text
Home device (canonical normal writer)
  semantic-memory primary
    canonical mutation API
       └── ReplayableMutationV1 + mutation_journal outbox
  mnemes sync agent
    sender state / retry / backoff / transport client

Authenticated admitted transport
  version negotiation + peer identity + signed/encrypted envelopes

Mnemes service
  device registry + grants + key/epoch policy + sync receipt registry
  per-device replica store
    canonical semantic-memory replay dispatcher
    replication_apply_ledger + sync_peer_watermark
```

### 4.1 Canonical mutation payload contract — `semantic-memory`

Create one public, versioned, canonical contract rather than exposing a generic replay closure:

```rust
pub enum ReplayableMutationV1 {
    AddFact { /* canonical, complete fields */ },
    // Add variants only after their owned write paths are made canonical and tested.
}

pub struct MutationOutboxEntryV1 {
    pub stream: StreamIdentityV1,
    pub sequence: u64,
    pub operation: ReplayableMutationV1,
    pub payload_digest: [u8; 32],
    pub schema_version: u16,
}

pub trait ReplicaApplyTx {
    fn apply_replayable_mutation(
        &mut self,
        entry: &MutationOutboxEntryV1,
    ) -> Result<CanonicalApplyOutcome, MemoryError>;
}
```

Required rules:

- canonical encoding is deterministic and versioned;
- digest is computed over exact canonical payload bytes and is stored with the local outbox entry;
- unknown variants and versions reject before any semantic write;
- callers cannot supply a custom replay closure in the production path;
- each mutation variant owns a stable idempotency key and payload schema;
- mutations without lossless replay representation are **not syncable** and must be explicitly blocked rather than approximated.

### 4.2 Receiver durable state

Do **not** overload `mutation_journal` for all receiver bookkeeping. Add forward-only persistence only after the schema owner approves a migration:

```text
replication_apply_ledger
  source_device_id
  source_store_id
  source_generation
  sequence
  payload_digest
  envelope_digest
  outcome: applied | duplicate_same_payload | quarantined | rejected
  canonical_receipt_ref
  applied_at (receiver stamped)
  PRIMARY KEY (source_device_id, source_store_id, source_generation, sequence)

sync_peer_watermark
  source_device_id
  source_store_id
  source_generation
  next_expected_sequence
  highest_contiguous_applied
  last_ack_digest
  updated_at (receiver stamped)
  PRIMARY KEY (source_device_id, source_store_id, source_generation)

replication_quarantine
  source identity + sequence + received digest + existing digest
  reason code + durable evidence reference + first_seen_at
  operator disposition: unresolved | reject | repair_by_new_generation
```

A duplicate source sequence is legal only when its stored digest equals the incoming digest. A mismatch is never `AlreadyApplied`.

### 4.3 Wire contract

Start with a loopback implementation of this exact contract. Select the eventual authenticated transport only after Mnemes registry/auth discovery.

```text
SyncHelloV1
  protocol_version
  supported_payload_versions
  claimed device/store/generation
  nonce

SyncBatchV1
  protocol_version
  batch_id
  source device/store/generation
  first_sequence / last_sequence
  prior_stream_head_digest
  entries: ordered OutboxEntryV1[]
  canonical_batch_digest
  credential-bound signature or authenticated channel binding

SyncAckV1
  protocol_version
  source identity/generation
  batch_id
  accepted_through_sequence       # contiguous prefix only
  disposition for every non-accepted entry
  receiver_apply_receipt_digest
  receiver timestamp
```

Required admission order:

1. authenticate transport and obtain admitted peer principal;
2. bind principal to claimed source device and authorized store/generation;
3. negotiate exact protocol and payload versions;
4. enforce size/count/order bounds;
5. verify batch and entry digests/signature/channel binding;
6. compare `first_sequence` with durable `next_expected_sequence`;
7. apply only the contiguous valid prefix in one transaction;
8. persist apply ledger and watermark in that transaction;
9. issue an acknowledgement bound to durable state;
10. quarantine/reject anything after the first non-admissible sequence.

### 4.4 Identity and authorization decision gate

Do **not** change `--mnemes-device-id` into a new DID format until Phase 0 determines the live Mnemes identity contract.

The implementation must select one admitted option:

- bind sync to existing Mnemes `DeviceId`/device credential registry; or
- establish a new device credential/key lifecycle with a migration and revocation plan.

Either option must prove:

- device identity cannot be asserted by a free-form CLI string alone;
- a peer can access only granted source/store/generation tuples;
- revocation/quarantine blocks new sync and terminates/rejects active sessions;
- keys/certificates are never written to receipts/logs;
- server-stamped times and registry epoch are included in admitted receipts.

---

## 5. Phased delivery plan

### Phase 0 — Ownership and runtime preflight

**Goal:** establish the real owner/repository/runtime contracts before creating a new protocol.

**Work:**

1. Locate the canonical Mnemes repository, its active branch, governing instructions, service entrypoint, DB paths, and registry schema.
2. Capture source HEADs, dirty state, current binaries, active service command lines, listeners, profile/tool inventory, and source/runtime compatibility.
3. Inventory every `semantic-memory` canonical mutation path and its actual transaction owner:
   - facts;
   - documents/chunks;
   - conversations/messages;
   - episodes;
   - graph edges and invalidation;
   - claim/projection imports;
   - supersession, deletion/redaction/forgetting;
   - maintenance-created semantic changes, if any.
4. Classify each path: `journal-covered`, `replayable-but-uncovered`, `not-yet-replayable`, or `must-never-sync`.
5. Choose the one first supported mutation type: **AddFact only** unless the inventory proves another path has the same canonical/replay quality.

**Exit gate:** source-backed ownership matrix, a no-shadow-truth decision record, and a RED test proving the same-sequence/different-payload silent acceptance defect.

**Rollback:** none; read-only discovery.

### Phase 1 — Canonical outbox and strict local replay

**Goal:** turn local journal mechanics into a truthful canonical contract for one mutation type.

**Work:**

1. Define `ReplayableMutationV1` and deterministic canonical encoder/decoder in `semantic-memory`.
2. Add payload digest/version to local outbox entries through an additive migration.
3. Ensure public `add_fact` creates semantic state and exact outbox entry in one transaction.
4. Replace production custom replay closures with the canonical replay dispatcher taking the active transaction handle.
5. Implement a receiver apply ledger and strict duplicate check:
   - same source sequence + same digest ⇒ idempotent duplicate;
   - same source sequence + different digest ⇒ typed quarantine conflict;
   - missing/forward sequence ⇒ gap error; no out-of-order advancement.
6. Add schema and API version refusal paths.

**Exit gate:** public API test proves:

```text
MemoryStore::add_fact
→ durable outbox entry with exact digest
→ export
→ fresh semantic-memory replica canonical replay
→ apply ledger + watermark in same transaction
→ reopen replica and compare fact/provenance
```

The test must also prove rollback on decoder failure, semantic-write failure, ledger-write failure, and commit failure.

**Rollback:** feature disabled by default; no live configuration or service changes.

### Phase 2 — Loopback sync vertical slice

**Goal:** prove protocol behavior before external transport.

**Work:**

1. Create a `SyncTransport` trait and `LoopbackTransport` test adapter only.
2. Implement `SyncHelloV1`, `SyncBatchV1`, and `SyncAckV1` as deterministic encoded data contracts.
3. Persist a sender-side delivery state that advances only from a verified durable ack.
4. Enforce single sync runner per source stream; bounded retry and explicit degraded state.
5. Build a process-boundary harness that launches sender and receiver processes against separate disposable stores.

**Exit gate:** crash and restart cases prove no duplicate semantic result, no skipped sequence, and no inferred acknowledgement.

**Rollback:** kill disposable worker, remove test stores. The local primary remains unchanged.

### Phase 3 — Peer admission and authenticated transport

**Goal:** connect the proven protocol to the actual Mnemes credential/registry owner.

**Work:**

1. Implement the identity decision made in Phase 0; do not duplicate device registry semantics.
2. Add a bounded transport endpoint with protocol negotiation, peer/store/generation scope enforcement, message limits, timeouts, and rate limits.
3. Bind batch receipt evidence to the admitted peer principal, registry epoch, stream identity, and protocol version.
4. Make unknown/revoked peers, stale registration epochs, unsupported versions, and unauthenticated requests fail before decoding/applying payloads.
5. Add observability: local backlog, contiguous sender/receiver watermarks, quarantine count, last successful sync receipt, last refusal code, and degraded reason.

**Exit gate:** authenticated two-process sync passes against a shadow replica; unauthorized, revoked, malformed, and version-mismatched cases fail closed.

**Rollback:** endpoint remains loopback/disabled by config; revoke the test credential and discard shadow replica.

### Phase 4 — Snapshot-plus-tail bootstrap and shadow reconciliation

**Goal:** initialize a replica safely without live SQLite copying.

**Work:**

1. Create a sealed SQLite online backup/export envelope from the home primary.
2. Record schema/version, canonical owner version, integrity check, content-free manifest, stream sequence, and snapshot digest.
3. Restore into an isolated shadow replica and apply only tail entries after the snapshot sequence.
4. Compare normalized canonical data and witnessed-query outputs, not just row counts.
5. Preserve every mismatch in a reconciliation ledger; do not auto-resolve same-ID/different-content conflicts.

**Exit gate:** scratch restore drill and shadow-vs-primary parity suite pass with exact documented exclusions; outstanding conflict count is zero or explicitly quarantined with no watermark progression.

**Rollback:** discard the shadow replica and retained staged artifacts; do not touch primary.

### Phase 5 — Explicit admission of continuous replica sync

**Goal:** enable a supervised replica worker for one admitted device/store pair.

**Preconditions:** user/operator approval is required before this phase because it creates live external effects.

**Required approval artifact:**

```text
SyncEnableApprovalV1
  target device/store/generation
  approved transport endpoint and credential epoch
  source backup manifest + verified restore drill receipt
  exact binary/source versions
  shadow reconciliation receipt
  expiry
  named operator
```

**Work after approval:**

1. Start one least-privilege user/system service for the pair; no broad global process.
2. Use both bounded periodic catch-up and connectivity-transition wake-up only after their shared durable sender state is proven safe under overlap.
3. Preserve receipts and safe backoff; never silently retry a quarantined conflict.
4. Keep server replicas non-authoritative for normal writes.

**Exit gate:** a supervised restart/offline/reconnect drill shows exact backlog behavior, durable acknowledgements, and no loss/duplication across crash boundaries.

**Rollback:** disable the service, revoke endpoint credentials if needed, preserve logs/receipts, and restore only from the separately verified backup under explicit approval.

### Phase 6 — Expand mutation coverage one type at a time

**Goal:** add only inventory-approved canonical mutation variants.

Each new mutation class repeats Phases 1–4 focused gates. Do not bulk-enable message/document/claim/deletion paths because they share a table.

**Special restrictions:**

- redaction/forgetting/deletion requires an explicit governance/authority policy before sync;
- derived indexes, embeddings, FTS/HNSW/usearch artifacts, caches, and routing projections are rebuildable and do not become canonical replicated truth by default;
- claim adjudication, assertion authority, and action authority are never derived from replication arrival.

---

## 6. Required test matrix

### Contract and unit tests

| Case | Required result |
|---|---|
| Canonical payload encoding determinism | Identical logical mutation produces identical bytes/digest. |
| Unknown mutation or payload version | Typed refusal before write. |
| Oversize/malformed envelope | Typed refusal before allocation/apply beyond limits. |
| Same `(source, stream, sequence)`, same digest | `duplicate_same_payload`; no replay side effect. |
| Same sequence, different digest | durable quarantine conflict; no overwrite. |
| Gap or out-of-order entry | no apply and no watermark advance. |
| Batch with valid prefix then invalid entry | apply only contiguous valid prefix; ack stops at its terminal sequence. |
| Replay semantic mutation failure | no apply-ledger/watermark commit. |
| Apply ledger write failure | no semantic mutation commit. |
| Watermark write failure | no semantic mutation commit. |
| Revoked peer | no decode/replay; observable refusal. |
| Scope mismatch | no decode/replay; observable refusal. |

### Public semantic-memory vertical-slice tests

- Public mutation API rather than direct SQL.
- Fresh replica opened through the same canonical store API after sync.
- Canonical content, source/provenance fields, applicable temporal fields, and receipt references compared after reopen.
- Existing direct journal primitives remain covered but are labelled **mechanics tests**, not end-to-end sync proof.

### Process-boundary and failure tests

- Sender crash before transmit, after transmit/before ack, after ack/before sender persistence.
- Receiver crash before semantic commit, after semantic commit attempt/before ledger commit, after durable commit/before ack transmission.
- Retry after every above state.
- Network duplicate, reordering, timeout, partial body, corrupted bytes, replayed old batch, and stale credential epoch.
- Restart with a persisted state DB and confirm terminal receipts/watermarks are reconstructed truthfully.
- Concurrent sync-agent launch for the same stream; exactly one obtains the lease.
- SQLite WAL backup/restore bootstrap; never `cp` a live database.
- Shadow parity: normalized witnessed-query IDs/order/receipt state, not count-only parity.

### Release/runtime checks

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release --bins
```

Then run the exact release binary with disposable stores and verify its executable path/version, listener, authenticated admission, actual data effect, durable receipt, shutdown, and restart behavior.

---

## 7. Explicit non-goals

- No CRDT/LWW merge policy for governed semantic truth.
- No generic bidirectional writable replication in the first release.
- No silent use of a server replica as the canonical source.
- No raw SQLite file sync, rsync, or database copy as a sync mechanism.
- No continuous service before loopback, shadow, backup, and restart gates pass.
- No backfilling unsupported mutation types by inventing replay payloads.
- No copying Hermes behavior/config/secrets as part of semantic-memory sync.
- No claims of resumable/replayable graph orchestration from the volatile council run.

---

## 8. Decision log

| Decision | Default | Why / next evidence needed |
|---|---|---|
| First supported mutation | `AddFact` only | It is the only inspected journal-backed production path. Expand only from inventory evidence. |
| Receiver bookkeeping | separate apply ledger and watermark tables | Outbound journal order and receiver admission state have different ownership and retention semantics. |
| Duplicate semantics | digest equality required | Sequence-only idempotency silently accepts divergent payloads. |
| Watermark semantics | contiguous accepted prefix only | Per-entry result reporting must never create a logical skip. |
| Transport | loopback first; then actual Mnemes-authenticated endpoint | Prevents a protocol designed around an unverified or duplicate auth stack. |
| Identity | reuse active Mnemes registry if suitable | Do not create a shadow identity authority. |
| Encoding | deterministic, explicitly versioned bytes | Exact choice (canonical CBOR/protobuf/etc.) is deferred until ecosystem/dependency audit. |
| Bootstrap | online SQLite backup + tail | Avoids unsafe live database copying and supports reproducible parity checks. |
| Server role | replica, not normal writer | Preserves device-owned local-first authority. |
| Continuous worker | explicit approval after shadow cutover | This is an operational mutation/availability decision, not a code default. |

---

## 9. First implementation slice when authorized

The smallest useful implementation is **Phase 1 only**:

1. Add a failing test showing source-sequence/payload-digest divergence is not silently accepted.
2. Define `ReplayableMutationV1` for `AddFact` only.
3. Make public `add_fact` atomically emit canonical payload bytes and a digest.
4. Add a strict replica apply ledger and transaction-bound canonical dispatcher.
5. Prove public mutation → outbox → fresh replica → reopening/query equivalence, including duplicate/mismatch/failure cases.

Do not start HTTP, mTLS, a daemon, multi-device sync, or a schema-wide mutation migration until that slice is green.

---

## 10. Handoff and artifacts

- **Controller plan:** this file.
- **Council candidate, incomplete by design:** `2026-07-28-mnemes-sync-council-candidate.md`.
- **Graph run:** `run-19fab2298b5-2`.
- **Semantic-memory destructive-diff backup:** `/home/sikmindz/.hermes/backups/semantic-memory/20260728T231356Z/`.
- **Current source status:** schema-37 baseline restored, compatibility additions compile/test clean; no commit, service deployment, database migration, or runtime replacement performed.

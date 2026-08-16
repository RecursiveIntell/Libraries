# ContextCompactionReceiptV2 recursive provenance contract

Status: implemented source contract; live activation requires the Ares adapter's
governed descriptor and host-commit-fence integration.

## Ownership and authority

`ContextCompactionReceiptV2` is the only recursive-provenance owner. The
Hermes adapter supplies a session ID, the current transcript, and an optional
explicit legacy parent locator; it does not construct generations, copy source
hashes, select ancestry, or decide retention eligibility. File and SQLite
indexes remain rebuildable projections of issued receipts.

Conversation/source bytes are authoritative evidence of what was observed.
Every deterministic or LLM summary is a projection. A projection can identify
source evidence but can never create a source identity or become authoritative
evidence merely by being summarized again.

## Version and compatibility

- A V1 JSON object remains `ContextCompactionReceiptV1`. Its fields and
  single-receipt exact expansion semantics do not change.
- A V1 receipt has no recursive parent, generation, supersession, or complete
  source manifest. Those facts must never be inferred and written into V1.
- A fresh V2 lineage starts at generation 1 with no parent or superseded
  receipt.
- An explicitly selected, verified V1 receipt may be the parent of a V2
  generation-2 receipt. The bridge carries only V1 exact-fallback leaves that
  V1 already proves. It does not fabricate identities for V1 content that was
  never placed in its exact store.
- Automatic restart recovery considers V2 lineage tips only. It never chooses
  one of several historical V1 receipts as a parent. An ambiguous V2 tip fails
  closed.

## Receipt identity

`receipt_id` is an opaque, unique, immutable store locator. It is not a content
hash. `receipt_identity_blake3` and `receipt_identity_sha256` bind the canonical
V2 receipt fields with the two identity fields cleared. They cover local
compaction hashes, parent and supersession fields, generation, transitive
source references, local source IDs, durability, and lineage digests.

Every certified V2 response has two governed detached HMACs. `hmac`
authenticates the complete receipt and projection. `evidence_hmac`
authenticates the immutable provenance and exact-source evidence while
excluding compacted messages and their rebuildable projection fields. V2
requires a full 64-hex SHA-256 signing-key ID; historical V1 verification alone
also accepts its original 8-hex key fingerprint and non-empty legacy key
lengths. Legacy compatibility cannot be used to sign or admit V2.

The compacted projection is separately bound by the transcript BLAKE3 and
SHA-256 fields already present in the receipt. This separation is deliberate:
a corrupt summary must be detected by a full load, but it must not prevent
recovery of separately verified source evidence.

`lineage_blake3` and `lineage_sha256` are deterministic replay proofs over the
parent provenance identity, generation, original input-transcript identities,
and the ordered source-identity manifest. They exclude the child receipt UUID,
creation time, and the new compacted-projection hash (the projection embeds its
random receipt locator and is independently bound by the receipt). Replaying
the same lineage construction against the same parent must reproduce these
digests even though the newly issued receipt ID and time differ. Independently
issued roots intentionally produce distinct descendant lineages because a
child binds the identity of its particular issued parent; replay equality is
defined for the same verified parent and identical input, not semantic dedupe
across separate receipt histories.

## Parent, generation, and supersession

- A V2 receipt has zero or one `parent_receipt`.
- A root has `generation == 1`, no parent, and no `supersedes_receipt_id`.
- A child has `generation == parent_generation + 1` and
  `supersedes_receipt_id == parent_receipt.receipt_id`.
- A parent reference binds parent schema, ID, effective generation, receipt
  provenance identity, lineage identity when available, and parent compacted
  transcript hashes.
- Parent and child session IDs must match.
- The parent compacted transcript must be an object-for-object exact prefix of
  the child's input transcript, including role, content, ID, name, and
  metadata. Only the suffix can introduce new source evidence. Ares owns any
  narrowly proven host-store normalization or rehydration before invoking
  Rust; the core never repairs a mismatched parent prefix.
- One issued child may supersede a tip. Competing children, duplicate receipt
  IDs, multiple active tips, and cycles are rejected.

Supersession selects the current projection; it does not revoke, mutate, or
erase the historical parent. A superseded receipt remains directly readable
and is required for replay and fallback while any retained descendant refers
to it.

## Source evidence and transitive coverage

Every newly observed, non-compaction-projection message receives a deterministic
`source_id` derived by Rust from session, generation, source index, role, and
the exact message/content hashes. The originating receipt stores the exact
message once in `source_evidence`. Descendants copy only immutable
`OriginalSourceRefV2` records, not source bytes.

The child's `covered_original_sources` must be exactly:

1. the verified parent's transitive source references (or the verified V1
   exact-fallback leaves for an explicit V1 bridge), plus
2. references derived from the child's new transcript suffix.

The set is sorted and duplicate-free. The local source IDs must match the
local `source_evidence` records exactly. Compaction-summary messages already
marked as projections are not promoted to original evidence.

No summary text is parsed for hashes, IDs, parentage, or authority. Certified
finalization receives the original signed `compact-v2` candidate separately
from replacement compacted messages. Rust first authenticates the unchanged
candidate under governed descriptor authority, changes only projection fields
and their derived receipt fields, then re-signs both authentication scopes.
Structural hashes alone are never sufficient because an attacker can recompute
them over forged provenance.

## Exact fallback

Expansion from a V2 receipt accepts a source ID or a unique legacy exact-item
ID. It:

1. loads the requested receipt and every parent to the root;
2. verifies unique IDs, acyclic generation order, parent identities,
   supersession, deterministic lineage digests, transitive manifests, and all
   source hashes;
3. selects exactly one manifest target;
4. reads exact bytes from the target's originating V2 `source_evidence` record
   or verified V1 `exact_store`; and
5. returns those bytes (subject only to the caller's explicit display
   truncation), never reconstructed summary prose.

A full receipt load rejects projection tampering. Certified exact expansion may
bypass a damaged projection only after `evidence_hmac`, provenance/source
structure, and per-source hashes all verify. This preserves exact recovery
without treating a corrupt summary as trusted input. Older V2 receipts without
`evidence_hmac` require their full HMAC to verify.

Missing parents, cycles, duplicate targets, hash disagreement, missing source
records, or corrupt provenance fail closed. Missing derived indexes do not
matter; they may be rebuilt from receipts.

## Restart, replay, and retention

Restart recovery scans authoritative receipt files/rows, validates their V2
edges, and selects the single unsuperseded V2 tip for the session. No adapter
memory is required. Zero tips starts a new generation-1 lineage. More than one
tip is an error. V2 load, parent selection, search, expansion, and retention
fail closed unless the store was constructed from Ares-held governed key and
snapshot descriptors. Keyless mode is only a V1 inspection compatibility lane.

Issued V2 receipts are append-only and use a host-commit fence:

1. `prepare-v2` validates signer/ring admission, ancestry, provenance, sources,
   and projection, applies durability fields, re-signs, and atomically writes
   `.pending/<receipt-id>.json`.
2. The host commits the normalized transcript to its conversation database.
3. `activate-v2` hashes the supplied committed governor projection and atomically
   publishes the pending receipt only when count, BLAKE3, and SHA-256 match.
4. `discard-v2` removes an authenticated pending receipt after an aborted host
   commit. `pending-v2` enumerates authenticated pending projections for crash
   reconciliation.

Pending receipts never participate in tip selection, search, expansion, or
retention. Activation rechecks the active parent under the publication lock, so
competing pending children cannot both become active. Every signer/ring check is
complete before a pending or active rename; unsigned and wrong-key failures
leave no published receipt, and a valid retry can use the same receipt ID.

Ordinary count-based retention may remove an unreferenced leaf, but it must not
remove any V1 or V2 receipt referenced by a retained descendant. Required
ancestors are skipped and reported. Supersession never makes ancestry eligible
by itself. Removing an entire closed lineage requires a separate explicit
deletion/tombstone contract and is outside this change.

## Hostile migration and lineage matrix

| Case | Construction | Required result |
|---|---|---|
| V1 compatibility | Load an existing V1 fixture/store file | Reads as V1; local exact expansion is byte/hash exact; no parent is invented |
| V2 generation 1 | Fresh transcript, no V2 tip | Generation 1, no parent/supersession, local source manifest exact |
| V2 generation 2 | Compact generation-1 projection again | One parent, generation 2, transitive manifest equals parent plus new suffix |
| Generations 4 and 8 | Repeatedly compact prior projection | One acyclic chain; stable original source IDs remain covered |
| Restart between generations | Drop all in-memory engine state | Store selects the same unique V2 tip and creates the next generation |
| Restart after final generation | Reopen store | Full chain validates and exact expansion still succeeds |
| Mandatory omitted marker | Omit unique marker in generation 1, compact again, restart, expand from generation 2 | Exact original marker is recovered through verified parent lineage |
| Ancestor provenance tamper | Change parent/source/lineage field or source bytes | Chain/expansion fails closed |
| Newest summary tamper | Change only newest compacted summary bytes | Full load detects corruption; exact expansion still recovers verified source bytes |
| Recomputed provenance forgery | Rebuild internally consistent hashes/source IDs under an attacker key | Finalize and store reject governed-authority mismatch before publishing |
| Host commit failure | Prepare succeeds but SessionDB commit aborts | Receipt stays pending/inert and can be authenticated then discarded |
| Crash after host commit | Pending receipt remains after restart | Host compares committed normalized projection to authenticated pending expectation, then activates exact match only |
| Missing parent | Remove/copy store without referenced parent | Load/expansion and next-generation construction fail closed |
| Missing original source | Remove local source record at origin | Expansion fails closed |
| Hash mismatch | Change source content or either bound hash | Expansion fails closed |
| Superseded recovery | Expand via newest and directly via an ancestor | Both return the same exact source while history is retained |
| Retention pressure | Prune below a still-referenced ancestor | Required ancestry is retained and reported protected |
| Replay | Reconstruct twice from identical parent/input | Lineage digests and ordered source IDs match |
| Duplicate child | Save two children of the same tip | Second save fails closed |
| Cycle/duplicate parent graph | Tamper edge or duplicate IDs | Validation fails closed |
| Explicit V1 bridge | Select one verified V1 parent | V2 generation 2 carries only proven V1 exact leaves; V1 stays unchanged |
| Ambiguous legacy receipts | Multiple V1 receipts, no explicit parent | New V2 root or typed ambiguity policy; never guessed parentage |
| Index loss | Delete only derived index in disposable store | Rebuild succeeds; lineage and exact expansion are unchanged |

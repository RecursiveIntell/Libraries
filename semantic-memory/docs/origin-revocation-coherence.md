# Origin revocation coherence

`MemoryAuthority::revoke_origin` now commits a newly inserted revocation and one
retrieval-epoch increment in the same owner SQLite transaction. It reuses the
compare-and-set path used by append, supersede and redact. The shared helper
rejects epochs outside SQLite's signed INTEGER range rather than wrapping.
There is no schema migration or new authority store.

Previously the revocation row changed authorization but not the epoch or lineage
heads hashed by `current_state`. A witnessed read could authorize a row, observe
a later committed revocation, and still pass its final state-equality check.
The first revocation now changes both retrieval epoch and snapshot identity.

## Preserved contracts

- The write-time origin label and lineage history remain immutable
- Exact key/fact/reference replay does not insert or advance the epoch
- A changed fact or reference under the same key still rejects with
  `AuthorityIdempotencyConflict`; principal is not newly added to that key
- Invalid capability, missing origin and invalid input do not advance state
- Overflow, failed epoch CAS/update, or later transaction errors roll back the
  insertion and epoch together
- A separately opened owner observes the committed state; concurrent deferred
  SQLite transaction upgrades may report BUSY/LOCKED, and an exact caller retry
  remains idempotent

## Reproducible gates

Use disposable stores and `MockEmbedder`, without credentials or provider calls:

```sh
cargo test --locked -p semantic-memory --no-default-features \
  --features testing,brute-force --test origin_authority --test governed_witness_v2
cargo test --locked -p semantic-memory --no-default-features \
  --features testing,brute-force --test authority_transactions
cargo clippy --locked -p semantic-memory --no-default-features \
  --features testing,brute-force --test origin_authority --test governed_witness_v2 -- -D warnings
cargo fmt -p semantic-memory -- --check
```

The epoch/replay regression fails on base `02ee5dd96a3f5fb403bdd2b955d57a1456b929ea`
with actual epoch 2 versus expected 3. Tests cover reopen persistence, denied
readback, unchanged labels, exact/conflicting replay, overflow, SQL-trigger
rollback and independent owner handles. A testing-only per-handle notification
barrier orders revocation after allow decisions and witness construction but
before the final coherence read. Both V1 and V2 must reject with
`AuthoritySnapshotChanged`; timeouts bound failure, not race ordering.

This fence detects changes during witnessed retrieval. It is not authenticated
transport, a store-incarnation identity, a final-use admission guard, a live-store
canary, or a claim that every raw/admin/replica writer is fenced. Those surfaces
need their own owner-preserving contracts. No live memory data is accessed by
these tests.

Rollback is to revert the source delta. Do not lower a live epoch, erase
revocations, or restore previously authorized content to compensate. Tests that
set epochs directly are isolated disposable fixtures only.

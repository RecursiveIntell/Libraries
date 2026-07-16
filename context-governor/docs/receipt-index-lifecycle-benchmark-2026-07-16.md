# Receipt-index lifecycle benchmark — 2026-07-16

## Claim boundary

This is a local, copied-corpus lifecycle measurement. It proves the measured candidate was faster than the installed JSON-index baseline on this host and snapshot. It is not a cross-machine, cold-disk, or production-SLO claim.

## Fixed inputs

- Authoritative source: `~/.hermes/context-governor/` (read-only)
- Fixed snapshot: 130 receipt JSON files, 249,733,481 bytes
- Snapshot metadata-manifest SHA-256: `53d27241fe3863838123becd8c25db5b0baba9d43456f0d7cbd87763cd44e450`
- Query: latest receipt ID in the snapshot, `ctxr_2840ba00cddd4b9bb7c517166476eed0`
- `top_k`: 1000
- Repetitions: 5
- Baseline binary SHA-256: `61cea7c637add180d7d28f29b941201a8fa187c6307d2236aa192df7c81bdfca`
- Candidate/installed binary SHA-256: `41fedcd745aa35ee900e2a13b1a6cd4502194c7988dab3354553be4c9d8b4ddc`
- Full JSON receipt from the run: `/tmp/context-governor-receipt-index-bench-v4/benchmark-v4.json`

Both implementations operated on separate copies. The post-save result tuples `(receipt_id, source, item_id, content_blake3, snippet)` were required to match; both returned two hits from the queried receipt.

## Results

| Lifecycle operation | JSON baseline | Incremental SQLite candidate | Candidate change |
|---|---:|---:|---:|
| Fresh-index build, receipt bytes prewarmed (median) | 8.056 s | 1.406 s | 5.73x faster |
| Persisted-index search, new CLI process (median) | 2.540 s | 0.0675 s | 37.65x faster |
| Same-ID overwrite save | 0.243 s | 0.0433 s | 5.62x faster |
| Save → first search | 15.542 s | 0.0703 s | 221.06x faster |
| Derived-index bytes | 101,745,583 | 21,356,544 | 79.0% smaller |

Raw samples:

- Baseline fresh build: `[6.1045, 8.4097, 5.9964, 8.0562, 28.3035]` seconds
- Candidate fresh build: `[5.6916, 10.8522, 1.3264, 1.3363, 1.4057]` seconds
- Baseline persisted search: `[2.4590, 2.4277, 2.5395, 2.6517, 17.5904]` seconds
- Candidate persisted search: `[0.06284, 0.06746, 0.06319, 0.06821, 0.06899]` seconds

## Caveats

- Receipt bytes were explicitly read before each fresh-index build. That isolates build/parse/index work but is not cold-disk latency.
- Each persisted search used a new CLI process. Filesystem cache was not evicted or otherwise controlled.
- Concurrent live compaction produced visible outliers. Medians are therefore reported with all raw samples retained.
- The canonical source gained one independently written receipt during the run (130 → 131), but the benchmark used a fixed copy and never wrote the source.
- The installed baseline has a correctness limitation for some punctuated substring queries: once any exact whitespace-token candidate exists, it can miss other substring matches. The final benchmark therefore used a receipt-ID query and required complete baseline/candidate hit-tuple equality.

## Correctness and recovery gates

The implementation keeps receipt JSON authoritative and the SQLite index disposable. Tests cover incremental append, same-ID overwrite, prune, external replacement reconciliation, index/signature corruption, tampered exact content, concurrent same-ID writers, stale temporary files, and deterministic rehydration.

The full verification run passed:

- `cargo test`: 106 tests
- `cargo test --all-features`: 113 tests
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo package --allow-dirty`: 73 packaged files; packaged `Cargo.lock` and `src/receipt_index.rs` present
- Hermes plugin suite: 61 tests

## SQLite/rusqlite design evidence

Primary sources checked on 2026-07-16:

- SQLite transactions: <https://www.sqlite.org/lang_transaction.html>
  - `BEGIN IMMEDIATE` acquires the write transaction up front and may itself return `SQLITE_BUSY`.
- SQLite result codes: <https://www.sqlite.org/rescode.html>
  - `SQLITE_BUSY` can occur at begin, write/update, or commit; a busy handler/timeout and immediate transaction make contention behavior explicit.
- SQLite synchronous pragma: <https://www.sqlite.org/pragma.html#pragma_synchronous>
  - WAL + `synchronous=NORMAL` remains atomic/consistent but can lose the last derived transaction on power loss. That trade is acceptable because fsync'd receipt JSON is authoritative and fingerprint reconciliation repairs the index.
- SQLite quick check: <https://www.sqlite.org/pragma.html#pragma_quick_check>
  - `quick_check` is `O(N)`, so it is reserved for recovery/suspicion rather than every query.
- SQLite WAL: <https://www.sqlite.org/wal.html>
  - WAL permits concurrent readers/writer on one host and uses fewer sync operations.
- SQLite atomic commit: <https://www.sqlite.org/atomiccommit.html>
  - Flush/fsync assumptions and commit boundaries support the temporary-build, file-sync, atomic-rename, parent-directory-fsync publication sequence.
- rusqlite 0.32.1 `Connection`: <https://docs.rs/rusqlite/0.32.1/rusqlite/struct.Connection.html>
  - `busy_timeout` installs the connection busy handler; `transaction_with_behavior` exposes `TransactionBehavior::Immediate`; `SQLITE_OPEN_NO_MUTEX` is the documented default choice because rusqlite enforces connection thread safety.

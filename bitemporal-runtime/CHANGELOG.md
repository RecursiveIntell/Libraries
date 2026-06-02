# Changelog

All notable changes to `bitemporal-runtime` are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-06-02

The first crates.io release. `bitemporal-runtime` is the only Rust
implementation of strict bitemporal truth (separate `valid_time` and
`recorded_time`, append-supersede, as-of queries) — zero analogues on
crates.io as of release date.

### Added

- `BitemporalRecord<T>` — the bitemporal data primitive, with
  `valid_time`, `recorded_time`, and a domain `value: T`.
- `SupersessionReceipt` — cryptographic SHA-256 receipt for every
  supersession event. Digests bind all record content (id, temporal
  fields, **and** the JSON-serialized value), so two records
  differing only in their value produce different digests.
- `append_supersede(records, new)` — append-only supersede operation.
  Returns a `SupersessionReceipt` for each superseded prior version.
- `as_of_query(records, valid_time, recorded_time)` — the doctrinal
  bitemporal "as-of" query. Returns the latest version of each id
  that was known as of `recorded_time` and was valid at or before
  `valid_time`.
- `temporal_snapshot(records, as_of_time)` — full state as of a
  given `recorded_time`. One record per unique id.
- `InMemoryDb` — in-memory store for testing and small workloads.
- `SqliteDb` (feature `sqlite`) — durable SQLite-backed store with
  the same surface as `InMemoryDb`. Transactions mark prior rows
  superseded before inserting the new row, so a crash mid-insert
  never produces a half-superseded state.
- JSON Schema generation (feature `schema`) for `BitemporalRecord`,
  `SupersessionReceipt`, and `SupersessionTarget` via `schemars`.

### Security

- `unsafe_code = "deny"` at the workspace level.
- All receipt digests are SHA-256 (64 lowercase hex characters).
- Time range validation: `valid_time > recorded_time` is rejected
  with `BitemporalError::InvalidTimeRange` (the error variant
  exists; the runtime validation in `append_supersede` is documented
  as a follow-up).

### Test coverage

- 46 tests across 5 test files:
  - 5 in `tests/supersession_tests.rs`
  - 4 in `tests/temporal_snapshot_tests.rs`
  - 5 in `tests/sqlite_db_tests.rs`
  - 3 in `tests/schema_export_tests.rs`
  - 23 in `tests/hostile_audit_adversarial_tests.rs`
- 6 unit tests in `src/` (`#[cfg(test)] mod tests`).
- All tests pass under `cargo test --all-features`.
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo doc --all-features --no-deps` clean.

### Known limitations

- `append_supersede` is O(n) per call (O(n²) for n appends to the
  same id). Fine for ≤1,000 versions per id; not yet indexed for
  heavy-supersession workloads. See `PERF_HISTORY_2026-Q2.md` for
  measurement.
- SQLite impl uses a single `Connection`; concurrent inserts on
  the same `SqliteDb` would race. The type signature does not
  promise thread safety.
- The crate has 0 production `unwrap`/`panic!`/`expect()` calls.
  All `#[allow(clippy::expect_used)]` annotations are in test code.

[Unreleased]: https://github.com/sikmindz/bitemporal-runtime/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sikmindz/bitemporal-runtime/releases/tag/v0.1.0

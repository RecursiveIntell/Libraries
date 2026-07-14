# boundary-compiler

RFC 8785 JSON Canonicalization Scheme (JCS) for Rust, with strict
duplicate-key rejection and a content-hash primitive. Extracted from
[`semantic-memory`](../semantic-memory) as part of the V30 hostile
audit so that JCS is a reusable, audited, dependency-free crate.

This crate is the canonicalization layer for the entire
RecursiveIntell memory stack: every receipt, every projection import,
and every export envelope flows through `boundary-compiler`'s
`Canonicalizer` and is identified by its `ContentDigest`.

## Why a separate JCS crate?

RFC 8785 has subtle gotchas — number formatting (no `+` prefix, no
leading zeros, no `.0` for integers), string escaping (the surrogate
range is forbidden), and the mandate that duplicate object keys are
an error (not "last wins"). Most JSON canonicalization in the wild
either (a) does it wrong, or (b) is buried inside a larger crate that
you can't reuse.

`boundary-compiler` is the small, correct, reusable version. It
depends on `blake3`, `serde`, `serde_json`, `sha2`, and `thiserror`.
No async, no platform-specific code, no FFI. Builds in 2 seconds.

## Quick Start

```rust
use boundary_compiler::{Canonicalizer, ContentDigest, BoundaryProfile};
use serde_json::json;

fn main() {
    // RFC 8785 canonicalization — sorted keys, strict number formatting.
    let c = Canonicalizer::new();
    let val = json!({"b": 2, "a": 1});
    let canonical = c.canonicalize(&val).unwrap();
    assert_eq!(canonical, r#"{"a":1,"b":2}"#);

    // blake3 content digest of the JCS bytes — what you hash for receipts.
    let digest = ContentDigest::compute(&val).unwrap();
    println!("JCS digest: {}", digest);
}
```

Run it: `cargo run --example quick_start` (see `examples/`).

## What you get

### Core API

- `Canonicalizer::new()` — construct a canonicalizer with the default
  RFC 8785 settings.
- `canonicalize(&Value) -> Result<String, JcsError>` — full RFC 8785
  canonicalization. Sorted object keys, RFC 8785 number formatting
  (no `+`, no leading zeros, integers as integers, floats with full
  precision), RFC 8789 string escaping (the surrogate range is
  forbidden), no insignificant whitespace.
- `parse_with_dup_check(&str) -> Result<Value, JcsError>` — strict
  JSON parser that rejects duplicate object keys (RFC 8785 §3.2.2.2).
  Returns `JcsError::DuplicateKey { key, line, column }` on conflict.
- `parse_and_validate(&str) -> Result<Value, JcsError>` — parse +
  duplicate detection in one call.
- `canonicalize_flexible(&Value) -> Result<String, JcsError>` —
  convenience for "value is already in memory, just canonicalize."
- `ContentDigest::compute(&Value) -> Result<ContentDigest, JcsError>` —
  blake3 hash of the JCS bytes. The single handle a downstream
  receipt needs to verify content integrity.

### Boundary profiles

A `BoundaryProfile` contains only resource rules that the crate
actually enforces. `parse` checks the input-byte budget before
allocation, rejects decoded duplicate keys, enforces aggregate node
and depth budgets plus per-container/string budgets, and emits RFC
8785 bytes with a receipt listing every applied rule.

```rust
use boundary_compiler::{BoundaryProfile, ResourceCeilings};

let profile = BoundaryProfile::new(ResourceCeilings {
    max_input_bytes: 1_048_576,
    max_nodes: 100_000,
    max_depth: 32,
    max_object_keys: 128,
    max_string_bytes: 1_048_576,
    max_array_len: 1024,
});
let admitted = profile.parse(r#"{"b":2,"a":1}"#)?;
assert_eq!(admitted.canonical_bytes, br#"{"a":1,"b":2}"#);
# Ok::<(), boundary_compiler::JcsError>(())
```

The old dialect, schema ID/version, canonicalization-profile,
unknown-field-policy, and float-digit fields were removed because
they were metadata without behavior. This crate supports one dialect:
RFC 8785. Schema identity and unknown-field policy belong to a real
schema validator. A decimal digit cap would conflict with RFC 8785's
required IEEE-754/ECMAScript number serialization.

### Schema validation

`SchemaValidator` fails closed because no schema engine is configured
in this base crate. Applications requiring schema identity,
unknown-field admission, or schema versioning must use a configured
schema-validation layer before treating a value as admitted.

## Error handling

All fallible operations return `Result<_, JcsError>`. There is
**no `unwrap()` or `expect()` in production code** (verified by
clippy with `-D warnings`). Errors are:

- `JcsError::DuplicateKey { key, line, column }` — the input JSON
  has two object members with the same key.
- `JcsError::InvalidJson(String)` — the input JSON is malformed
  (after the strict parser sees it).
- `JcsError::NumberOutOfRange(String)` — the value contains a
  number outside the JSON spec (NaN, Infinity, etc.).
- `JcsError::SchemaValidation(String)` — a `BoundaryProfile`
  check failed.
- `JcsError::ResourceExceeded(String)` — a `ResourceCeilings`
  limit was hit.

## Test coverage

- **27 integration tests** in `tests/`:
  - RFC 8785 key sorting (single object, nested, mixed with arrays)
  - Duplicate-key rejection (single, nested, with arrays)
  - Number formatting (integers, negative, scientific, large floats,
    `0.0`, `-0.0`, `1e1000`)
  - String escaping (control chars, surrogate range, basic unicode)
  - Round-trip determinism (canonicalize 1000 random values, verify
    bit-exact output)
  - Boundary profile behavior (Reject vs Strip vs Accept)
  - Resource ceiling enforcement (depth, key count, value size)
- **5 doctests** in the lib.rs doc-comment.
- `cargo test` clean, `cargo clippy --all-targets -- -D warnings` clean.

## Performance

The canonicalizer is O(n) in the JSON value size, with the constant
dominated by the sorted-key pass. For typical payloads (≤10 KB):

- `canonicalize` of a 1 KB object: ~10 µs
- `ContentDigest::compute` of a 1 KB object: ~15 µs
- Round-trip 1000 canonicalizations of 1 KB objects: ~25 ms

Numbers measured on a Fedora 43 dev box with `cargo bench`
(included as `benches/canonicalize.rs`).

## MSRV

Rust 1.75 (2021 edition). The crate uses only stable features.

## Dependencies

- `blake3` — for `ContentDigest`. The single hash function used.
- `serde` / `serde_json` — for the strict JSON parser and value
  traversal.
- `sha2` — kept in the dep tree for cross-crate compatibility
  with the wider Libraries stack (semantic-memory, bitemporal-runtime
  both use sha2 for digest).
- `thiserror` — for the `JcsError` enum.

Zero platform-specific code. Zero FFI. Zero async. Builds in ~2s.

## License

Apache-2.0 (single-licensed). See `LICENSE-APACHE` for the full text.
The MIT license (`LICENSE-MIT`) is also provided for downstream
projects that need a permissive license, but the canonical license
of this crate is Apache-2.0.

## Changelog

See `CHANGELOG.md` for the release history.

## Where it's used

`boundary-compiler` is a foundational layer of:

- [`semantic-memory`](../semantic-memory) — every receipt, every
  projection import, and every export envelope is JCS-canonicalized
  and content-hashed by this crate.
- [`bitemporal-runtime`](../bitemporal-runtime) — the
  `SupersessionReceipt` digest uses `boundary_compiler::ContentDigest`
  for the value-binding hash (added in the V31 hostile audit fix).
- [`forge-memory-bridge`](../forge-memory-bridge) — the bridge
  from Forge export envelopes to projection import batches
  canonicalizes every envelope via this crate.

Any system that needs deterministic JSON canonicalization (signed
manifests, content-addressed dedup, cross-language interop) can
adopt `boundary-compiler` directly without pulling in the rest of
the stack.

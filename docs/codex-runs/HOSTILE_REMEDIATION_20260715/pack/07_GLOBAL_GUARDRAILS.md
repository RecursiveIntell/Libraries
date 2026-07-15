# Global guardrails

## Authority

- `stack-ids` owns cross-crate ID types, parsing, rendering, derivation, and minting law.
- Domain crates own referents, persistence, and when IDs are required.
- Raw/SQLite state remains authoritative where documented.
- Indexes, HNSW, compressed vectors, caches, projections, and receipts are derived/evidentiary.
- One canonical crate owns codec/profile/wire contracts; codec crates own implementations.
- Adapters route/validate and never invent semantics.

## Prohibited

- Error-to-success conversion or false completion.
- Missing/corrupt evidence converted to false/default/empty/allow.
- Placeholder decoding labeled decompression.
- Competing ID authorities or direct random generation outside stack-ids.
- Silent lossy scope conversion.
- Parse filters that discard malformed material records.
- Generated receipts treated as independent proof.
- Compatibility shims without owner, removal condition, and test.
- Semantic widening to make tests pass.
- Quantitative/release claims without source-bound evidence.

## Migration law

Dual-read/single-write; append plus supersession; preserve original bytes/IDs; preflight/snapshot/
postcondition/reverse path; do not collapse bitemporal/currentness semantics.

## Error law

Parse, storage, integrity, governance, capability, and infrastructure errors remain typed.
Best-effort behavior requires explicit caller policy and result/receipt state.

## Test law

Every bug gets a regression that fails on audited behavior. Golden fixtures are versioned.
Required skips block closure. Tests assert contracts and emitted status/receipt state.

## Evidence law

Verify is read-only. Recording is separate. Commands bind argv, cwd, commit/tree, dirty state,
toolchain/platform, exit code, output hashes, and logs. Final verification leaves a clean tree.

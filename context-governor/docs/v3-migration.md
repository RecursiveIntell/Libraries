# Context Governor V3 side-by-side migration

`context_governor::migrate_v2_store` creates a disposable V3 projection without changing V2 receipt authority or the default writer.

## Guarantees

- V2 JSON receipts remain immutable and authoritative.
- V3 manifests store the parent receipt, local source IDs, covered-source digest, compacted transcript digest, and content-addressed exact-evidence references.
- Exact `Message` bytes are compressed with zstd and can be encrypted with AES-256-GCM.
- Encryption keys are supplied in memory and are never serialized into migration options or manifests; only a SHA-256 key ID is recorded.
- Missing keys, wrong keys, digest mismatches, and authenticated decryption failures fail closed.
- Migration mismatches are retained in `V3MigrationReportV1` and make `complete` false.

## Activation boundary

This is migration tooling only. It does not enable V3 default writes, replace V2 reads, delete V2 receipts, or promote CAS/index metadata to authority. A full-corpus activation still requires an immutable manifest, exact reconstruction of every migrated V2 source/fallback/compacted projection, secret/disclosure scanning, and a separate operator activation decision.

Example shape:

```rust,no_run
let options = V3MigrationOptions {
    encryption_key: Some(vec![0x42; 32]),
    require_encryption: true,
    ..Default::default()
};
let report = migrate_v2_store(&store, "/tmp/context-governor-v3", &options)?;
assert!(report.complete);
```

# Security and Portability Gates

## Required gates

| Gate | Failure mode | Required behavior |
|---|---|---|
| Path traversal | Archive entry writes outside extraction root | Reject paths containing `..`, absolute paths, drive roots, or unsafe separators. |
| Symlink escape | Link points outside repo/root | Exclude or error unless explicitly allowed and target is inside allowed root. |
| Hardlink/device/special files | Non-portable or dangerous archive entry | Exclude and report. |
| Unicode normalization collision | Same visible path differs by normalization | Error in strict mode. |
| Case-insensitive collision | `Readme.md` and `README.md` collide on Windows/macOS | Warning or error by mode. |
| Windows reserved names | `CON`, `PRN`, `NUL`, `AUX`, `COM1`, etc. | Error for release/context packages. |
| Secret-like filenames | `.env`, keys, tokens, credentials | Exclude by default; content scan when included. |
| Secret content patterns | Leaked private keys/tokens | Error in strict mode. |
| Oversized files | Handoff package bloat | Exclude or warn by policy. |
| Nested archives | Hidden stale packages/secrets | Exclude unless mode explicitly allows. |
| Compression ratio anomaly | Zip-bomb-like payload | Warn/error if compressed/uncompressed ratio exceeds policy. |
| Binary allowlist | Unknown binary files | Exclude unless explicitly allowed. |
| Generated output in source package | Dist/build/cache pollution | Move to archive or exclude. |
| Absolute build-machine paths | Non-transferable validation | Strip or include only as recorded evidence, never as required verifier path. |

## Portability verifier

Add a verifier that opens the produced archive without trusting the source tree:

```bash
python3 z.py verify --package package.zip --manifest package.manifest.json --strict
```

It must verify:

- every manifest-included path exists in the package;
- package files hash to manifest hashes;
- no package entry missing from manifest unless synthetic/sidecar policy declares it;
- archive entry names are safe;
- no excluded required file was accidentally omitted;
- optional checksums/provenance/SBOM subject digests match package digest.

## Secret scanning boundary

`z.py` should remain a conservative source-package scanner, not a full DLP system. It should catch common high-risk leaks and report its scan scope:

- bytes scanned;
- files skipped due size/binary policy;
- patterns enabled;
- redaction state in reports.

## Transferability rule

No final validation path may require `/home/<user>/...` or any build-machine path. The manifest may record original paths as evidence, but verification must accept `--package` and `--repo-root` overrides.

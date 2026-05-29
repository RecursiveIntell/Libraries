# PHASE 06 — Determinism, checksums, provenance-lite, SBOM-lite

Tasks:

1. Bump `SCRIPT_VERSION`.
2. Add `SOURCE_DATE_EPOCH` support.
3. Emit checksums file when configured.
4. Optionally emit tar.gz with normalized metadata.
5. Emit provenance-lite predicate with package subject digest, command, inputs, git state, script version, and policy digest.
6. Emit SBOM-lite component inventory from manifests when configured.

Acceptance:

- Two packages from unchanged tree have identical content manifest.
- Provenance subject digest matches package digest.
- Checksums file includes package and sidecars.

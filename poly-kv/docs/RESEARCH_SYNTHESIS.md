# Research Synthesis: What mature packagers do that z.py should learn from

## Sources checked

- Cargo manifest/package include/exclude and `cargo package --list`: https://doc.rust-lang.org/cargo/reference/manifest.html
- npm publish/package inclusion and `npm pack --dry-run`: https://docs.npmjs.com/cli/v11/commands/npm-publish/
- Python packaging/setuptools distribution file control: https://setuptools.pypa.io/en/latest/userguide/miscellaneous.html
- Python wheel binary distribution format and RECORD hashes: https://packaging.python.org/specifications/binary-distribution-format/
- PyPA build frontend behavior: https://build.pypa.io/
- Docker build context and `.dockerignore`: https://docs.docker.com/build/concepts/context/
- Git archive and `.gitattributes export-ignore`: https://git-scm.com/docs/git-archive
- Go module zip restrictions: https://golang.google.cn/cmd/vendor/golang.org/x/mod/zip/
- Reproducible Builds / SOURCE_DATE_EPOCH: https://reproducible-builds.org/docs/source-date-epoch/
- SLSA: https://slsa.dev/
- in-toto attestations: https://github.com/in-toto/attestation
- SPDX: https://spdx.dev/
- CycloneDX: https://cyclonedx.org/
- GitHub artifact attestations: https://docs.github.com/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds
- Sigstore/cosign blob signing: https://docs.sigstore.dev/cosign/signing/signing_with_blobs/

## Main lesson

Mature packagers are not merely archive writers. They combine:

1. **Declared inclusion semantics** — manifest fields, ignore files, package-data rules, and required always-included files.
2. **Dry-run visibility** — `cargo package --list`, `npm pack --dry-run`, Python build artifacts, Docker build context pruning, Git archive previews.
3. **Reproducibility controls** — deterministic ordering, stable timestamps, source-date epoch, normalized metadata, and strong per-file hashes.
4. **Integrity and provenance** — RECORD hashes, checksums, SLSA/in-toto attestations, SBOMs, signing, and build-origin metadata.
5. **Registry/package hygiene** — excluding build products, secrets, caches, generated residue, irrelevant logs, and platform-specific garbage while preserving required typed/package metadata.

`z.py` should stay a **source/context/handoff certifier**, not become Cargo/npm/PyPI/Docker. Its high-ROI role is to produce audit-grade packages and compare them against ecosystem packager expectations where possible.

## Design implication

`z.py` needs three clean layers:

```text
Repo facts + local package policy
      ↓
Ecosystem adapters and hygiene engines
      ↓
Archive writer + manifest/provenance/verifier
```

The current script already has useful pieces: include/exclude decisions, secret scanning, root package archival, deterministic ZIP timestamps, content manifests, Codex run hygiene, and sidecar reports. The next pass should make these features more portable, configurable, ecosystem-aware, and validator-friendly.

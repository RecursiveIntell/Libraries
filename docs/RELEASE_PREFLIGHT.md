# Clean-source release preflight

Historical dirty packages and tags are immutable evidence and must not be rewritten. Future patch
releases run `scripts/release_preflight.sh CRATE COMMIT` from the repository root.

The preflight rejects staged, unstaged, and untracked changes affecting the crate or workspace
manifests/lockfile, requires the relevant source to match the named commit, and runs
`cargo package -p CRATE --locked`. It then invokes the required `PACKAGE_VERIFY_HOOK` as:

```text
hook path/to/CRATE-VERSION.crate COMMIT CRATE
```

The repository/release pipeline supplies that hook to unpack the Cargo archive and compare it with
the named commit while allowing only reviewed Cargo packaging normalization (for example generated
manifest metadata and `.cargo_vcs_info.json`). The hook must also verify that `.cargo_vcs_info.json`
names the requested commit and records `dirty: false`. Preserve the hook output and package checksum
as release evidence. Signing and provenance attestation are separate release steps; this preflight
does not claim they ran.

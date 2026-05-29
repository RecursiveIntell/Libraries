# Final State and Acceptance Gates

## Final state

`z.py` is accepted when it can package any RecursiveIntell repo as a transferable, auditable handoff artifact with:

- clean root hygiene;
- required typed/package metadata included;
- command/diff/receipt evidence preserved in context modes;
- ecosystem parity checks where applicable;
- deterministic content manifest;
- portable verification from copied package + manifest;
- security/portability gates;
- optional provenance/SBOM/checksum outputs;
- clear include/exclude explanations;
- strict hostile-auditor handoff.

## Required commands

```bash
python3 z.py --help
python3 z.py --root . --mode next-codex-context --strict
python3 z.py --root . --mode source-clean --strict --dry-run
python3 z.py --verify-package <package.zip> --manifest <manifest.json> --strict
python3 scripts/test_zpy_hygiene_regression.py
python3 scripts/test_zpy_universal_packager_regression.py
python3 scripts/assert_source_package_hygiene.py --repo-root . --manifest <manifest.json> --package <package.zip> --mode manifest
```

If subcommands are introduced, equivalent subcommand invocations are acceptable, but old invocations must continue to work.

## Fixture requirements

Create fixtures under `tests/fixtures/zpy/` or equivalent:

1. `rust_basic` — Cargo crate, include refs, target dir, Cargo.lock.
2. `python_typed` — pyproject, package, `py.typed`, `.pyi`, tests, dist residue.
3. `node_basic` — package.json, files field, `.npmignore`, node_modules residue.
4. `git_export_ignore` — `.gitattributes export-ignore`, dirty/untracked files.
5. `docker_context` — Dockerfile, `.dockerignore`, COPY refs.
6. `unsafe_paths` — symlink escape, reserved names, Unicode/case collisions.
7. `codex_audit` — commands log, patch, final report, stale sidecars.
8. `monorepo_mixed` — Rust + Python + Node components.

## Non-goals for this pass

- Do not publish packages.
- Do not require Docker/Cargo/npm/Python toolchains to be installed for static validation.
- Do not implement full SBOM dependency resolution if manifests are enough for SBOM-lite.
- Do not sign artifacts by default.
- Do not make release claims based on context-package success.

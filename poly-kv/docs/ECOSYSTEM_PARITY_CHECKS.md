# Ecosystem Parity Checks

## Philosophy

`z.py` should not guess silently when a standard packager can tell us what it would package. The goal is not to copy registry package output. The goal is to detect drift between:

1. `z.py` package contents;
2. ecosystem package contents;
3. local source/handoff policy.

## Rust adapter

Detection:

- `Cargo.toml`
- `Cargo.lock`
- workspace members
- `include` / `exclude` manifest keys
- path dependencies

Optional commands:

```bash
cargo package --list --allow-dirty
cargo package --allow-dirty --no-verify
cargo metadata --format-version 1
```

Checks:

- `Cargo.toml`, `Cargo.lock`, crate source, README/license included where expected.
- Files referenced by `include_str!` / `include_bytes!` included.
- External path dependencies either bundled or reported.
- `target/` excluded.
- `cargo package --list` compared to z.py release-like mode.

## Python adapter

Detection:

- `pyproject.toml`
- `setup.cfg`, `setup.py`
- `MANIFEST.in`
- package directories
- `py.typed`, `*.pyi`

Optional commands:

```bash
python -m build --sdist --wheel
python -m pip install build
```

Checks:

- `py.typed` included if package advertises typing or contains it.
- `*.pyi` included.
- `MANIFEST.in` and pyproject/package-data rules not contradicted.
- Built wheel `RECORD` hashes inspected when wheel exists.
- Native extension stubs are not dropped from handoff packages.

## Node/npm adapter

Detection:

- `package.json`
- `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`
- `.npmignore`
- `files` field

Optional commands:

```bash
npm pack --dry-run --json
npm pack --json
```

Checks:

- Required npm package files (`package.json`, README/license where present) included.
- `node_modules/`, `dist/`, build output are mode-controlled.
- `.npmignore` / `files` behavior surfaced in parity report.

## Docker/build-context adapter

Detection:

- `Dockerfile`, `Containerfile`
- `.dockerignore`

Optional commands:

```bash
docker buildx build --progress=plain --dry-run .  # only if available and safe; otherwise do static checks
```

Checks:

- `.dockerignore` is parsed for context pruning expectations.
- Dockerfile `COPY` and `ADD` paths exist in the handoff package unless intentionally excluded.
- Large/context-sensitive files surfaced.

## Git adapter

Detection:

- `.git` directory
- `.gitattributes`
- `.gitignore`

Optional commands:

```bash
git status --short
git ls-files
git check-attr export-ignore -- <path>
git archive --format=tar HEAD | tar -tf -
```

Checks:

- Dirty/untracked files inventoried.
- `.gitattributes export-ignore` compared for release-like modes.
- `.git/` excluded.
- Git metadata recorded but not treated as source truth when archive is copied without `.git`.

## Go adapter

Detection:

- `go.mod`
- `go.sum`
- `go.work`

Optional commands:

```bash
go list -m -json all
go mod verify
go test ./...
```

Checks:

- `go.mod` and `go.sum` included.
- Vendor and module-cache directories excluded unless policy requires them.
- Go module zip restrictions used as path-portability inspiration: no path traversal, clean names, consistent extraction.

## Adapter failure policy

Adapters can fail for missing tools. Missing tools must not silently pass.

```text
missing command + parity=warn  → warning finding
missing command + parity=error → strict error
missing command + parity=off   → report only
```

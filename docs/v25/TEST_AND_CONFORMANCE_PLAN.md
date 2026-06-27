# Test and conformance plan — v25

## What this pass can prove here

This environment can prove:

- required v25 files exist,
- v25 schemas, examples, manifests, and fixture bundles parse as JSON,
- fixture manifests and file sets agree,
- current taught-surface docs exist,
- and mirror sync plus apply scripts are present.

This environment cannot prove:

- `cargo check`,
- `cargo test`,
- or schema regeneration from `schemars`,

because the Rust toolchain is absent here.

## Required local checks when Rust tooling is available

```bash
cargo run -p contract-schema-gen -- schemas
cargo run -p contract-schema-gen -- --check schemas
cargo test -p stack-ids
cargo test -p verification-policy
cargo test -p profile-runtime
cargo test -p knowledge-runtime
```

## Python / shell checks included in this repo pack

```bash
bash scripts/check_v25_repo_truth.sh
python3 scripts/check_v25_json_surface.py
bash scripts/run_v25_local_checks.sh
```

## Required fixture classes

1. blocked baseline
2. explicit exception-admitted path
3. conflict path
4. policy-impact diff path
5. delegation / break-glass path
6. release readiness blocked path
7. continuity incident-mode diff path
8. vendor translation caveat path

## Release-blocking failures

The following are release-blocking:

- current repo entry points still teach the no-v25 terminal position as current law,
- a required v25 schema/example/fixture file is missing,
- fixture manifest and fixture files disagree,
- any v25 JSON file fails to parse,
- `profile-runtime` is not wired into the workspace,
- `contract-schema-gen` lacks any v25 family registration,
- or the mirror sync path list drifts away from the actual repo truth.

## Still-required release proof outside this environment

- Rust unit test pass,
- schema regeneration and check pass,
- integration proof for effect/control/adjudication consumers,
- and no-local-recomposition enforcement in CI.

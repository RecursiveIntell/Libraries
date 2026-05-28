# v25 implementation status

## Landed in this pass

- root-local v25 canonical spec file
- root-local v26 horizon spec file
- supersession note resolving historical no-v25 doc ambiguity
- docs/v25 execution pack
- broadened v25 fixture corpus and fixture manifest
- conformance manifest for v25
- repo-truth and JSON-surface scripts
- whole-tree `libraries-source/` mirror sync

## Landed earlier and preserved here

- `profile-runtime` crate scaffold and reference composition logic
- v25 IDs in `stack-ids`
- v25 schema registry in `contract-schema-gen`
- v25 runtime views in `knowledge-runtime`
- v25 examples, schemas, and baseline tests

## Still unproven in this environment

- Rust compilation
- Rust test execution
- schema regeneration from `schemars`
- consumer adoption in `effect-runtime`, `verification-control`, and `verification-adjudication`

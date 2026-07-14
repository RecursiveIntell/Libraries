# RecursiveIntell Libraries

This repository is a Rust library ecosystem for governed agent runtimes, durable memory, verification, causal tooling, and efficient inference. Repository claims come from executable manifests and gates: `repo_contract.toml` defines support scope, `schemas/` owns schema truth, and CI certifies every active Cargo workspace independently.

## Ecosystem map

| Area | Primary packages | Purpose |
| --- | --- | --- |
| Memory and retrieval | `semantic-memory`, `semantic-memory-mcp`, `semantic-memory-forge`, `forge-memory-bridge`, `knowledge-runtime` | Durable local memory, retrieval, projections, graph operations, and MCP access |
| Claims and verification | `claim-ledger`, `verification-*`, `assurance-runtime`, `attestation-exchange` | Evidence, proof debt, adjudication, calibration, policy, and release decisions |
| Governed execution | `forge-pilot`, `kernel-*`, `effect-runtime`, `mechanism-runtime`, `authority-delegation` | Policy-bounded planning, execution, effects, delegation, and conformance |
| Quantization and storage | `turbo-quant`, `fib-quant`, `hyperquant`, `poly-kv`, `compressed-scorer` | Quantized representations, compressed candidate stores, scoring, and evaluation |
| Agent applications | `AiDENs`, `agent-graph`, `agent-graph-mcp`, `llm-*` | Agent capability kits, orchestration, model pipelines, and tool runtimes |
| Causal and systems primitives | `Primitives`, `cea-bridge`, `context-governor`, `scr-runtime` | Causal attribution, sandboxing, context governance, and reference runtimes |

Package-local source, manifests, and tests remain authoritative for package behavior. This README describes repository organization and certification scope; it does not override crate contracts.

## Support and maturity

The canonical per-package values, owners, features, and required gates are in [`repo_contract.toml`](repo_contract.toml). The repository currently contains 112 unique active packages across overlapping workspaces.

| Maturity | Support tier | Packages | Meaning |
| --- | --- | ---: | --- |
| Production | Certified | 19 | Release-facing packages required to pass the declared certification gates |
| Supported | Governance | 7 | Default-enabled governance surfaces checked with the root workspace |
| Supported | Internal | 11 | Shared primitive packages supported for repository use |
| Beta | Preview | 41 | Independently checked preview APIs that may still evolve |
| Experimental | Incubating | 34 | Research, integration, benchmark, or early-stage surfaces without a production support claim |

The 17 production-certified root members are:

- `contract-schema-gen`
- `forge-memory-bridge`
- `forge-pilot`
- `kernel-conformance`
- `kernel-execution`
- `kernel-oracles`
- `knowledge-runtime`
- `living-memory/living-memory`
- `llm-tool-runtime`
- `recursive-kernel-core`
- `semantic-memory`
- `semantic-memory-forge`
- `stack-ids`
- `verification-adjudication`
- `verification-calibration`
- `verification-control`
- `verification-policy`

The two independently certified production packages are `context-governor` and `semantic-memory-mcp`. “Production” is a repository support classification, not a claim that every optional feature is certified in every environment.

## Workspace map

Every active `Cargo.toml` containing `[workspace]` has a CI lane. Package counts are workspace membership counts, so the nested `Primitives` members also appear in the root count.

| Workspace root | Members | CI scope |
| --- | ---: | --- |
| `.` | 64 | Root hardening and repository gates |
| `AiDENs` | 37 | Independent fmt, check, test, and clippy |
| `Primitives` | 11 | Independent fmt, check, test, and clippy |
| `agent-graph-mcp` | 1 | Independent fmt, check, test, and clippy |
| `cea-bridge` | 1 | Independent fmt, check, test, and clippy |
| `context-governor` | 1 | Independent fmt, check, test, and clippy |
| `knowledge-router` | 1 | Independent fmt, check, test, and clippy |
| `poly-kv` | 2 | Independent fmt, check, test, and clippy |
| `scr-runtime` | 4 | Independent fmt, check, test, and clippy |
| `semantic-memory-mcp` | 1 | Independent fmt, check, test, and clippy |
| `turbo-quant/tools/semantic_memory_harness` | 1 | Independent fmt, check, test, and clippy |

### Root workspace members (64)

Production-certified members are listed above. Governance-supported members are:

- `assurance-runtime`
- `attestation-exchange`
- `authority-delegation`
- `constitutional-memory`
- `continuity-runtime`
- `effect-runtime`
- `mechanism-runtime`

The remaining active root members are:

- `agent-graph`
- `agent-guard`
- `ai-batch-queue`
- `bitemporal-runtime`
- `boundary-compiler`
- `claim-ledger`
- `comfyui-rs`
- `compressed-scorer`
- `constraint-compiler`
- `discovery-portfolio`
- `fib-quant`
- `gpu-backend`
- `hnsw-bench`
- `hyperquant`
- `federated-settlement`
- `job-queue`
- `llm-output-parser`
- `llm-pipeline`
- `poly-kv/crates/quant-codec-core`
- `ollama-vision`
- `Primitives/cea-core`
- `Primitives/cea-sqlite`
- `Primitives/cea-store`
- `Primitives/check-runner`
- `Primitives/check-runner-sys`
- `Primitives/effect-signature`
- `Primitives/forge-policy`
- `Primitives/mindstate-core`
- `Primitives/sandbox-workspace`
- `Primitives/stabilizer-core`
- `Primitives/typed-patch`
- `quant-governor`
- `quant-eval`
- `receipt-bench`
- `profile-runtime`
- `remote-oracle-admission`
- `scr-runtime-compression`
- `spec-execution`
- `tauri-queue`
- `turbo-quant`

## Repository truth surfaces

- [`repo_contract.toml`](repo_contract.toml) is the source for workspace/package maturity, support tiers, owners, features, required gates, and certification lanes.
- [`SUPPORT_PROFILE.md`](SUPPORT_PROFILE.md) and [`scripts/lane_manifest.json`](scripts/lane_manifest.json) are generated views. `python3 scripts/generate_from_repo_contract.py --check` rejects drift.
- [`schemas/`](schemas/) is the authoritative schema registry. [`schemas/schema_manifest.json`](schemas/schema_manifest.json) records every schema ID, version, owner, compatibility rule, file, and byte digest.
- `contracts/schemas/` is a generated compatibility mirror. Its registry manifest must be byte-identical to the authoritative manifest; legacy wave manifests are non-authoritative views.
- [`.github/workflows/ci.yml`](.github/workflows/ci.yml) declares one lane for every active workspace root. Discovery ignores only archived, salvage, fixture, fuzz, and build-output trees.
- Dated plans, audits, dashboards, receipts, and run packets are evidence, not current runtime contracts. Generated run bundles belong under `docs/archive/runs/<id>/` and must not become active repository truth.

## Validation

Run the repository-truth gates from the repository root:

```bash
python3 scripts/check_schema_registry.py
python3 scripts/discover_workspaces.py
python3 scripts/generate_from_repo_contract.py --check
```

Validate the root workspace:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Validate an independent workspace by replacing `<workspace>` with a path from the workspace map:

```bash
cargo fmt --manifest-path <workspace>/Cargo.toml --all -- --check
cargo check --manifest-path <workspace>/Cargo.toml --workspace --all-targets
cargo test --manifest-path <workspace>/Cargo.toml --workspace --all-targets
cargo clippy --manifest-path <workspace>/Cargo.toml --workspace --all-targets -- -D warnings
```

CI executes these independent-workspace commands as a matrix. Optional GPU, GUI, model-service, and platform integrations may require additional system dependencies; a passing command certifies only the manifest, features, targets, and environment it actually exercised.

## Stable operating doctrine

- Current source, Cargo metadata, generated-contract checks, schema digests, and test results outrank prose and historical receipts.
- One concern has one authority: support scope lives in `repo_contract.toml`; schema truth lives in `schemas/`; generated mirrors cannot self-promote.
- Retrieval candidates, caches, indexes, projections, compressed stores, and orchestration state do not become claim, evidence, or policy authority merely by being useful.
- Recall authority does not imply assertion or action authority. Boundary crossings require explicit typed decisions and receipts.
- Compatibility claims must name a schema ID and version. Breaking semantic changes require a new major version; unversioned schemas use exact compatibility.
- Certification claims must name the workspace, feature set, command, and environment. Partial or targeted checks are not whole-repository certification.
- Generated artifacts are reproducible views. Drift is a gate failure, and hand edits belong in the authoritative input instead.

## Documentation and licensing

Active documentation is indexed from [`docs/README.md`](docs/README.md); historical material lives under `docs/archive/`. Individual packages declare their own licenses in package manifests and license files, so no single license should be inferred for the entire repository.

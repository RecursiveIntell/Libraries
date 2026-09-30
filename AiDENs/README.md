# AiDENs

An orchestration, inspection, packaging, and supported-local runtime layer for the RecursiveIntell stack.

AiDENs wires providers, tools, receipts, profiles, and execution flows together. Canonical memory, governance, kernel, identity, and tool-contract semantics remain with their owner crates.

## Recorded support status

The checked-in [run record](docs/codex-runs/CURRENT_RUN.json) identifies `P32`, with certification status `candidate`, support label `p32-schema-compat-candidate`, and last certified run `P30`. It records schema compatibility as enabled and boundary-compiler work as no longer deferred.

That record also has `extracted_replay_certified=false`. Recorded build and packaging results are historical evidence, not a fresh certification of every later Libraries revision. The README does not promote the candidate to a certified release or claim production/cloud readiness.

## Getting started

From the Libraries repository root:

```bash
cd AiDENs
cargo metadata --no-deps --format-version 1
cargo test --workspace --locked --all-targets
```

For the broader verification workflow, first read [AGENTS.md](AGENTS.md), the [support profile](SUPPORT_PROFILE.md), and the scripts it invokes, then run:

```bash
bash scripts/verify_current.sh .
```

The verifier writes logs under `target/verify-current` by default and checks documentation/run consistency before Rust gates. Some paths referenced by historical run records, including `docs/codex-runs/BUILD_SCOPE.md`, are absent from this checkout; treat those as evidence gaps rather than assuming the old result has been reproduced.

## Workspace guide

- [Cargo.toml](Cargo.toml): current workspace members and dependencies
- [crates/](crates/): contract, provider, tool, security, receipt, runner, CLI, profile, and integration-test packages
- [examples/](examples/): local coding, memory-grounded, and daemon-oriented examples
- [scripts/](scripts/README.md): verification and packaging entry points
- [tests/](tests/README.md) and [schemas/](schemas/README.md): test and contract guidance
- [docs/codex-runs/](docs/codex-runs/): recorded run state and historical artifacts
- [CANONICAL_OWNER_MAP.md](CANONICAL_OWNER_MAP.md): ownership boundaries

Choose an example's instructions and required capabilities before running it. A fixture or packaging check does not establish broad autonomous execution, provider compatibility, or deployment readiness.

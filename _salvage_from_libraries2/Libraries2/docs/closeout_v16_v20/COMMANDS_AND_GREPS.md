# Commands and Greps

Run these inside the repo root in Codex.

## Fast file sanity

```bash
rg -n "SelfHostingBuildReceiptV1|SharedReplaySliceV1|SharedDivergenceReportV1|TreatySuspensionV1"   stack-ids spec-execution federated-settlement contract-schema-gen contracts kernel-conformance
```

## Targeted tests

```bash
cargo test -p federated-settlement
cargo test -p mechanism-runtime
cargo test -p discovery-portfolio
cargo test -p constitutional-memory
cargo test -p spec-execution
cargo test -p kernel-conformance
```

## Schema check

```bash
cargo run -p contract-schema-gen -- --check schemas
```

## Suggested final sweep

```bash
cargo test --workspace
```

## Grep proofs for crate-local docs

```bash
test -f federated-settlement/README.md
test -f federated-settlement/AGENTS.md
test -f mechanism-runtime/README.md
test -f mechanism-runtime/AGENTS.md
test -f discovery-portfolio/README.md
test -f discovery-portfolio/AGENTS.md
test -f constitutional-memory/README.md
test -f constitutional-memory/AGENTS.md
test -f spec-execution/README.md
test -f spec-execution/AGENTS.md
```

## Grep proof for v20 gap closure

```bash
rg -n "SelfHostingBuildReceiptV1|self_hosting_build_receipt_v1|self-hosting-build"   stack-ids spec-execution contracts/schemas/v20 contracts/fixtures/v20
```

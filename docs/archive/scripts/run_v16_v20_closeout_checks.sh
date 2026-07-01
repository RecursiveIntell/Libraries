#!/usr/bin/env bash
set -euo pipefail

echo "[1/5] targeted crate tests"
cargo test -p federated-settlement
cargo test -p mechanism-runtime
cargo test -p discovery-portfolio
cargo test -p constitutional-memory
cargo test -p spec-execution
cargo test -p kernel-conformance

echo "[2/5] schema check"
cargo run -p contract-schema-gen -- --check schemas

echo "[3/5] crate-local docs exist"
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

echo "[4/5] grep proof for key closeout artifacts"
rg -n "SelfHostingBuildReceiptV1|SharedReplaySliceV1|SharedDivergenceReportV1|TreatySuspensionV1"   stack-ids spec-execution federated-settlement contract-schema-gen contracts kernel-conformance

echo "[5/5] optional workspace sweep"
cargo test --workspace

echo "closeout checks passed"

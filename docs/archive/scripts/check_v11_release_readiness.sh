#!/usr/bin/env bash
set -euo pipefail

echo "[v11] checking canonical schemas"
cargo run -p contract-schema-gen -- --check schemas

echo "[v11] testing stack ids"
cargo test -p stack-ids

echo "[v11] testing semantic contracts"
cargo test -p semantic-memory-forge

echo "[v11] testing control-plane proof governance"
cargo test -p verification-control

echo "[v11] testing pilot proof/exactness integration"
cargo test -p forge-pilot --lib
cargo test -p forge-pilot --test verification_control_tests

echo "[v11] testing reference interpreters and conformance"
cargo test -p kernel-conformance --lib
cargo test -p kernel-conformance --test verification_control_v0

echo "[v11] readiness checks passed"

#!/usr/bin/env bash
set -euo pipefail

# Validate release artifacts without activating services or publishing anything.
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"

cargo fmt --all -- --check
cargo test -p stack-observation
cargo test -p stack-monitor --all-features
cargo test -p stack-monitor-desktop
cargo clippy -p stack-observation --all-targets -- -D warnings
cargo clippy -p stack-monitor --all-targets --all-features -- -D warnings
cargo clippy -p stack-monitor-desktop --all-targets -- -D warnings

cargo build --release -p stack-monitor --bin stack-monitor-collector
cargo build --release -p stack-monitor-desktop
(cd stack-monitor-desktop && cargo tauri build --ci)

for artifact in \
  target/release/stack-monitor-collector \
  target/release/stack-monitor-desktop \
  stack-monitor-desktop/tauri.conf.json \
  stack-monitor-desktop/packaging/stack-monitor-collector.service \
  stack-monitor-desktop/frontend/dist/index.html; do
  test -e "$artifact" || { printf 'missing artifact: %s\n' "$artifact" >&2; exit 1; }
done

printf '%s\n' 'RELEASE_VALIDATION=PASS'
sha256sum target/release/stack-monitor-collector target/release/stack-monitor-desktop

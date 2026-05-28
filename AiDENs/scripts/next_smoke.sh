#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

bash scripts/assert_no_fake_completion.sh .

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; smoke limited to no-placeholder check"
  exit 0
fi

cargo run -p aidens-cli -- profile list
cargo run -p aidens-cli -- profile explain coding-agent
cargo run -p aidens-cli -- plan validate --config examples/aidens.mock.toml
mkdir -p target
cargo run -p aidens-cli -- plan compile --config examples/aidens.mock.toml --out target/aidens-plan.json
test -s target/aidens-plan.json
cargo run -p aidens-cli -- doctor --config examples/aidens.mock.toml
cargo run -p aidens-cli -- provider-check --config examples/aidens.mock.toml
cargo run -p aidens-cli -- list-tools
cargo run -p aidens-cli -- run --config examples/aidens.mock.toml "hello"

set +e
cargo run -p aidens-cli -- run --config examples/aidens.disabled.toml "hello"
status=$?
set -e
if [ "$status" -eq 0 ]; then
  echo "Disabled provider unexpectedly answered successfully" >&2
  exit 1
fi

echo "Next-run smoke complete."

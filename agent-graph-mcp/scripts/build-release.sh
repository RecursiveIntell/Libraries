#!/bin/bash
# Build a release artifact with provenance manifest.
#
# Usage: ./scripts/build-release.sh [--install]
#
# Produces:
#   target/release/agent-graph-mcp          — release binary
#   target/release/agent-graph-mcpd         — daemon binary
#   target/release/build-manifest.json      — provenance manifest
#   target/release/cargo-audit.json         — audit output (if cargo-audit installed)

set -euo pipefail

WORKTREE="/home/sikmindz/Coding/worktrees/agent-graph-remediation"
cd "$WORKTREE"

echo "=== Building release artifacts ==="

# Verify clean working tree (only agent-graph related files)
DIRTY=$(git status --short -- agent-graph agent-graph-mcp | wc -l)
if [ "$DIRTY" -gt 0 ]; then
  echo "WARNING: working tree has $DIRTY uncommitted agent-graph changes"
  echo "Release builds should use a clean committed tree."
  echo "Continue anyway? (y/N)"
  read -r response
  [ "$response" = "y" ] || exit 1
fi

# Record source provenance
GIT_COMMIT=$(git rev-parse HEAD)
GIT_BRANCH=$(git branch --show-current)
GIT_DIRTY=$(git status --short -- agent-graph agent-graph-mcp | wc -l)
CARGO_LOCK_SHA256=$(sha256sum Cargo.lock | awk '{print $1}')
RUSTC_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version)
TARGET="x86_64-unknown-linux-gnu"
BUILD_TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

echo "Source commit: $GIT_COMMIT"
echo "Branch: $GIT_BRANCH"
echo "Dirty files: $GIT_DIRTY"
echo "Cargo.lock SHA-256: $CARGO_LOCK_SHA256"
echo "Rustc: $RUSTC_VERSION"

# Build release
cargo build --release -p agent-graph-mcp 2>&1

# Record artifact identity
BINARY_PATH="target/release/agent-graph-mcp"
BINARY_SHA256=$(sha256sum "$BINARY_PATH" | awk '{print $1}')
BINARY_SIZE=$(stat -c%s "$BINARY_PATH")

echo "Binary SHA-256: $BINARY_SHA256"
echo "Binary size: $BINARY_SIZE bytes"

# Run tests and capture receipts
echo "=== Running tests ==="
cargo test -p agent-graph --tests 2>&1 | tee target/release/engine-test-output.txt
cargo test -p agent-graph-mcp --no-fail-fast 2>&1 | tee target/release/mcp-test-output.txt
cargo fmt --check -p agent-graph -p agent-graph-mcp 2>&1 | tee target/release/fmt-output.txt || true
cargo clippy -p agent-graph -p agent-graph-mcp --all-targets -- -D warnings 2>&1 | tee target/release/clippy-output.txt || true

# Run cargo-audit if available
if command -v cargo-audit &>/dev/null; then
  cargo audit --json 2>&1 | tee target/release/cargo-audit.json || true
else
  echo '{"advisories": [], "note": "cargo-audit not installed"}' > target/release/cargo-audit.json
fi

# Generate manifest
cat > target/release/build-manifest.json << MANIFEST_EOF
{
  "manifest_version": 1,
  "build_timestamp": "$BUILD_TIMESTAMP",
  "source": {
    "git_commit": "$GIT_COMMIT",
    "git_dirty": $([ "$GIT_DIRTY" -gt 0 ] && echo "true" || echo "false"),
    "git_branch": "$GIT_BRANCH",
    "cargo_lock_sha256": "$CARGO_LOCK_SHA256",
    "workspace_root": "$WORKTREE"
  },
  "toolchain": {
    "rustc_version": "$RUSTC_VERSION",
    "cargo_version": "$CARGO_VERSION",
    "target": "$TARGET"
  },
  "artifact": {
    "name": "agent-graph-mcp",
    "path": "$BINARY_PATH",
    "sha256": "$BINARY_SHA256",
    "size_bytes": $BINARY_SIZE
  },
  "features": {
    "crate_features": [],
    "hash_algorithm_version": 1,
    "checkpoint_schema_version": 2,
    "graph_spec_version": "2",
    "migration_version": 1
  }
}
MANIFEST_EOF

echo "=== Build manifest ==="
cat target/release/build-manifest.json
echo ""
echo "=== Release build complete ==="
echo "Binary: $BINARY_PATH"
echo "Manifest: target/release/build-manifest.json"
echo "SHA-256: $BINARY_SHA256"

# Optional install
if [ "${1:-}" = "--install" ]; then
  echo "=== Installing to ~/.cargo/bin/ ==="
  cp "$BINARY_PATH" "$HOME/.cargo/bin/agent-graph-mcp"
  chmod 0700 "$HOME/.cargo/bin/agent-graph-mcp"
  INSTALLED_SHA256=$(sha256sum "$HOME/.cargo/bin/agent-graph-mcp" | awk '{print $1}')
  if [ "$INSTALLED_SHA256" != "$BINARY_SHA256" ]; then
    echo "FATAL: installed binary hash mismatch!"
    exit 1
  fi
  echo "Installed binary verified: $INSTALLED_SHA256"
fi
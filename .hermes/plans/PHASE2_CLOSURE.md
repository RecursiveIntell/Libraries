# Agent Graph — Phase 2 Closure Plan: Remaining Risks

## Baseline (2026-07-24)
- **112 tests passing**: 55 lib + 13 daemon_recovery + 44 integration
- **`cargo fmt --all --check`**: clean
- **`cargo clippy -p agent-graph-mcp -- -D warnings`**: clean
- **MCP daemon lifecycle**: `initialize → tools/list` (25 tools) proven over Unix socket
- **AG-003/023/025/026/028**: verified_closed in ledger

## Lane Assignments (3 parallel + controller convergence)

### Lane A (Delegate): Workspace Quality Gates — AG-010/011/012
**Goal**: Close clippy/MSRV, hook/preflight, and validator completeness gaps
**Tasks**:
1. Run `cargo test --workspace` full suite (exclude known-broken crates)
2. Run `cargo clippy --workspace -- -D warnings` and add workspace-level allows for pre-existing style debt in root Cargo.toml
3. Run `cargo audit` and create advisory-adjudication.json
4. Run release scripts: `agent-graph-mcp/scripts/build-release.sh` (if exists) or `cargo build --release -p agent-graph-mcp`
5. Verify hook/preflight scripts exist and validate against current manifest

### Lane B (Delegate): AG-001 Crash Recovery Live-Run Matrix  
**Goal**: Add crash-recovery integration tests
**Tasks**:
1. Add test: daemon crashes mid-execution → restart → run marked as `interrupted`, never `running`
2. Add test: kill -9 daemon during checkpoint write → restart → checkpoint is either present and valid or absent (no corruption)
3. Add test: daemon crash during graph_create → restart → graph not registered
4. Run `cargo test -p agent-graph-mcp --test daemon_recovery -- --nocapture`

### Lane C (Controller): AG-002/005 Authority + Daemon Socket Integration Tests
**Goal**: Close approval authority and daemon-proxy integration gaps
**Tasks (Controller)**:
1. Add process-boundary test: daemon + proxy → tools/list over socket
2. Replace `--direct` usage in key integration tests with daemon socket mode
3. Add authority negative test: forged actor label rejected

## Convergence (Controller)
1. Merge Lane A changes (workspace Cargo.toml, clippy allows)
2. Verify Lane B tests pass
3. Run final `cargo test --workspace` (scoped)
4. Run `cargo fmt --all --check`
5. Update closure ledger with AG-001/002/005/010/011/012 dispositions
6. Issue final readiness verdict

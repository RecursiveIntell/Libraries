# Agent Graph Final Closure — Implementation Plan

## Closure List (6 blockers → readiness)

### B1: Integration Test Transport Hang
- **Symptom:** `cargo test -p agent-graph-mcp --test mcp_integration` hangs (600s+ timeout)
- **Root cause:** `Mcp::new()` spawns `agent-graph-mcp` with no args → proxy mode → tries to connect to socket → no daemon → binary exits 69 → test `read_line` blocks on closed stdout
- **Recovery:** Add `--direct` flag to `Mcp::new()` constructor, or fix stdin/stdout EOF handling to fail fast

### B2: Daemon MCP Lifecycle Unproven
- **Symptom:** No process-boundary evidence of `initialize → notifications/initialized → tools/list` over socket transport
- **Requirement:** Spawn daemon, connect framed socket client, capture 3-step JSON-RPC transcript

### B3: AG-003 Mixed Keyless/Key-Enabled (still_open)
- **Gap:** No passing test proving mixed-mode rejection at process boundary
- **Existing:** `daemon_startup_mode_is_durable_across_restarts` + `repeated_same_mode_startup_is_allowed`
- **Need:** Test that flipping key presence between restarts is rejected

### B4: AG-023 Process/Watchdog Multiplicity (still_open)
- **Gap:** `second_daemon_against_same_data_dir_is_rejected` only covers one lock boundary
- **Need:** Watchdog exclusivity test + process count invariant

### B5: AG-025/026/028 Partially Closed
- **AG-025:** No live watchdog-vs-health failure injection
- **AG-026:** Observer transport failure visibility not exercised
- **AG-028:** Universal evidence invariant not proven

### B6: Quality Gates
- `cargo test` (all crates)
- `cargo clippy -- -D warnings`
- `cargo fmt --all --check`
- Closure ledger update with evidence links
- `.hermes/evidence/` artifacts

---

## Implementation Plan — 3 Parallel Lanes + Controller

### Lane A (Controller): Fix integration tests + daemon MCP transport
**Files:** `agent-graph-mcp/tests/mcp_integration.rs`, `agent-graph-mcp/src/bin/agent-graph-mcpd.rs`
1. Fix `Mcp::new()` to pass `--direct` flag (or handle EOF gracefully)
2. Verify `legacy_contract_and_exact_tool_names` passes single-threaded
3. Run full integration suite with `--test-threads=1`
4. Create process-boundary smoke test script for daemon socket MCP

### Lane B (Delegate): Close AG-003/023/025/026/028
**Files:** `agent-graph-mcp/src/daemon.rs`, `agent-graph-mcp/tests/daemon_recovery.rs`
1. AG-003: Add cross-mode rejection test (keyed→keyless flip rejected)
2. AG-023: Add watchdog multiplicity guard test
3. AG-025: Add health-vs-watchdog failure injection test
4. AG-026: Add observer failure visibility test
5. AG-028: Add evidence-universal-invariant test

### Lane C (Delegate): Quality gates + evidence artifacts
**Files:** Ledger, `.hermes/evidence/`
1. Run `cargo test` full suite
2. Run `cargo clippy -- -D warnings`
3. Run `cargo fmt --all --check`
4. Create MCP lifecycle evidence transcript
5. Update closure ledger with dispositions

---

## Delegation Assignments

| Lane | Agent | Scope | Risk |
|------|-------|-------|------|
| A | Controller | Daemon fix + test unblock | High (complex Rust) |
| B | Subagent 1 | AG findings closure tests | Medium (focused tests) |
| C | Subagent 2 | Quality gates + evidence | Low (run-and-report) |

## Acceptance Gates
- [ ] All 44 integration tests pass (single-threaded)
- [ ] All 8 daemon_recovery tests pass
- [ ] All 54 lib tests pass
- [ ] Socket MCP lifecycle transcript exists
- [ ] Closure ledger updated with AG-003/023/025/026/028 dispositions
- [ ] `cargo clippy` clean (or pre-existing allows documented)
- [ ] `cargo fmt` passes
- [ ] Go/no-go verdict with evidence links

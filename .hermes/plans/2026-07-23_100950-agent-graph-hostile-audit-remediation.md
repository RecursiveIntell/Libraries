# Agent Graph Hostile-Audit Remediation Implementation Plan

> **For Hermes:** Execute this plan in an isolated worktree with TDD and independent hostile review after every phase. Do **not** use Agent Graph to orchestrate its own P0–P2 remediation; the audited runtime remains quarantined until Phase 9. If delegation is needed, use bounded filesystem-capable workers and independently verify every claimed edit and command in the controller session.

**Goal:** Remediate every confirmed engine, MCP, persistence, authority, template, Hermes-integration, kit, and release-provenance defect; migrate the live installation without losing state; and restore only claims proven by installed-process evidence.

**Architecture:** Replace the current many-stdio-process/shared-SQLite topology with one long-lived, lock-owning Agent Graph daemon and thin per-session stdio proxies over a private Unix-domain socket. Make execution lifecycle, checkpoints, terminal projections, idempotency, cancellation, and graph-version binding canonical and transactional. Keep model-facing control separate from operator authority: the MCP surface cannot mint human approval, and privileged decisions require an external OS-authenticated operator path.

**Tech stack:** Rust, Tokio, rmcp, SQLite/rusqlite, Unix-domain sockets, Linux peer credentials, Polkit for operator authorization, systemd user services, Python 3 for Hermes hooks/scripts, Cargo test/clippy/audit/deny, SHA-256 release manifests.

**Planning checkpoint:** 2026-07-23T10:09:50-05:00

**Canonical paths under review:**

- Workspace: `/home/sikmindz/Coding/Libraries`
- Engine: `/home/sikmindz/Coding/Libraries/agent-graph`
- MCP server: `/home/sikmindz/Coding/Libraries/agent-graph-mcp`
- Hermes runtime config: `/home/sikmindz/.hermes/config.yaml`
- Hermes hooks: `/home/sikmindz/.hermes/agent-hooks`
- Hermes scripts: `/home/sikmindz/.hermes/scripts`
- Multi-host kits: `/home/sikmindz/Coding/agent-graph-kits`
- Live database: `/home/sikmindz/.agent-graph/agent-graph.db`
- Installed binary: `/home/sikmindz/.cargo/bin/agent-graph-mcp`

---

## 1. Current evidence and non-claims

### 1.1 Mutable source snapshot

At the planning checkpoint:

```text
branch: fix/hostile-remediation-20260715
HEAD: 9577d4462fdf87691d835723a4f61f4c3efe6ad3
global dirty entries: 193
agent-graph-mcp/: untracked
agent-graph tracked diff: 6 files, 202 insertions, 47 deletions
new engine test: agent-graph/tests/checkpoint_store_failure_contract.rs
```

This is historical evidence only. Implementation must refresh it before editing.

### 1.2 Verified positives

- Engine integration tests passed at audit time.
- MCP suite passed 60 tests: 16 unit and 44 integration.
- Release build passed.
- Installed and local release binaries shared SHA-256:
  `f4c7ba0495431fac56a4f9f947cb1ff94a394e723d63fc8d864c8c51f6c200d2`.
- Disposable MCP initialize/tools-list smoke passed.
- SQLite integrity check returned `ok`.
- Certification correctly remained `NO_GO` / `UNCERTIFIED_RUNTIME`.

### 1.3 Current blocks

Until this plan reaches the corresponding gates, do not claim:

- multi-client durable execution;
- restart-safe resume;
- authenticated human approval;
- replay beyond integrity/projection reads;
- source-verified research;
- production or enterprise readiness;
- safe effectful parallel execution;
- committed/reproducible release provenance;
- certified multi-host integration.

### 1.4 Quarantine rules during implementation

1. Do not use live Agent Graph for approvals, deployment, release, filesystem mutation, memory mutation, or infrastructure mutation.
2. Do not run disposable server probes against `/home/sikmindz/.agent-graph`; every process-boundary test uses a fresh temporary directory.
3. Do not stop services, kill processes, migrate the live database, modify Hermes configuration, or install binaries until the explicit Phase 9 operator gate.
4. Do not commit or reset unrelated parent-workspace dirt.
5. Do not expose the daemon on TCP. The supported transport is a private local Unix socket only.
6. Do not call a self-asserted actor string “human approval.”
7. Do not silently downgrade missing integrity, storage, policy, or authority capability.
8. Do not use `graph_policy_check` as authorization; it is preflight only.

---

## 2. Target invariants

The implementation is complete only when all invariants below are mechanically true.

### Runtime ownership

- Exactly one daemon owns a durable Agent Graph data directory.
- The daemon acquires an OS-released exclusive lock before opening/migrating SQLite.
- Stdio MCP processes are stateless proxies and never run startup recovery.
- Starting a second proxy cannot change any run status.
- Starting a second daemon against the same directory fails before database mutation.

### Persistence and lifecycle

- Each run carries `owner_instance_id`, graph version, canonical request digest, and explicit lifecycle generation.
- Startup recovery touches only runs owned by the dead prior daemon generation.
- Terminal transition is atomic and monotonic.
- Terminal execution, bounded terminal events, receipt, bundle/index, and receipt digest are published in one transaction or not at all.
- Every read tool uses one canonical durable projection after restart.
- Zero never means unavailable; capability/read failures are typed.

### Checkpoint and resume

- One canonical checkpoint type stores exact iteration, step, phase, resume boundary, active/remaining nodes, graph version, and state digest.
- Conditional routing semantics are included in graph identity or the graph is non-resumable.
- A checkpoint is consumed exactly once.
- Resume cannot duplicate or skip an effectful node.

### Cancellation and timeout

- Cancellation propagates through retry sleep, provider request, and parallel branches.
- First fatal branch failure cancels and joins sibling tasks before return.
- Sync timeout returns `completion_unknown` plus cancellation disposition; it never implies the run stopped.
- No new node attempt starts after cancellation wins.

### Authority

- Model-facing MCP clients cannot approve their own work.
- Privileged approval and destructive administrative action require an OS-authenticated operator route unavailable to ordinary MCP calls.
- Approval is bound to approval ID, checkpoint digest, graph ID/version, decision set, expiration, operator principal, and nonce.
- A decision is consumed once; replay and alteration fail closed.

### Data security

- Data/config/runtime directories are `0700`.
- SQLite, WAL, SHM, keys, sockets, backups, and manifests containing sensitive metadata are `0600` unless a stricter owner requires otherwise.
- Unsafe symlinks, foreign ownership, and permissive modes fail startup in durable mode.

### Templates and research

- Every registered node type is executable or the graph is rejected before registration.
- Built-in templates pass semantic black-box tests, not only schema validation.
- LLM-only synthesis is not called web research or source verification.
- Evidence-shape validation is distinct from source-authority verification.

### Release identity

- Tested source is committed and clean.
- Build manifest binds source revision, dirty flag, lockfile digest, toolchain, features, artifact hash, config schema, and migration version.
- Installed binary equals the certified artifact.
- Previous binary, config, and database backup have verified rollback handles.

---

## 3. Finding-to-task traceability

| Finding | Closure tasks |
|---|---|
| AG-001 global startup recovery | 2.1–2.5, 4.1–4.4, 9.4 |
| AG-002 unauthenticated approval | 6.1–6.5 |
| AG-003 mixed keyless processes | 2.2, 8.1, 8.5, 9.4–9.6 |
| AG-004 world-readable database | 2.3, 9.3 |
| AG-005 forgeable template promotion | 8.3 |
| AG-006 misleading templates | 7.2–7.6 |
| AG-007 source/binary provenance | 1.1–1.4, 10.1–10.4 |
| AG-008 destructive kit installer | 8.6–8.8 |
| AG-009 misleading status | 5.3, 8.5 |
| AG-010 clippy/MSRV/panics/audit | 10.1–10.3 |
| AG-011 hook/preflight drift | 8.2–8.5 |
| AG-012 weak kit validator/env expansion | 8.6–8.8 |
| AG-013 fabricated interrupt cursor | 3.1–3.3 |
| AG-014 router semantics absent from hash | 3.4–3.5 |
| AG-015 store silent no-op | 3.8–3.9 |
| AG-016 retry cancellation delay | 3.6 |
| AG-017 parallel cancellation leakage | 3.7 |
| AG-018 terminal transition race | 3.9, 5.1 |
| AG-019 sync timeout continues | 5.5 |
| AG-020 nullable idempotency digest | 5.4 |
| AG-021 unsupported node classes | 7.1 |
| AG-022 CLI ignores malformed args | 2.1, 4.2 |
| AG-023 process/watchdog multiplicity | 8.4–8.5, 9.4–9.6 |
| AG-024 malformed guard input fails open | 8.2 |
| AG-025 watchdog green while health red | 8.4 |
| AG-026 observer failures invisible | 8.3 |
| AG-027 stale hook allowlist | 8.9 |
| AG-028 graphs without durable evidence | 9.2, 9.7 |

---

# Phase 0 — Freeze source identity and create a bounded remediation worktree

## Task 0.1 — Capture a target-only baseline receipt

**Owner:** Controller

**Objective:** Preserve the exact current target state without touching unrelated workspace changes.

**Output:** `/home/sikmindz/Coding/Libraries/.hermes/evidence/agent-graph-remediation/<timestamp>/`

**Steps:**

1. Record date, host, user, workspace root, branch, HEAD, Rust/Cargo versions, scoped status, and scoped diff stat.
2. Save `git diff --binary -- agent-graph` as `agent-graph.patch`.
3. Archive untracked `agent-graph-mcp/` without `target/` or caches.
4. Archive the untracked checkpoint-store test separately if not represented by the patch.
5. Generate SHA-256 for every receipt/archive.
6. Re-read the receipt and verify archive listing before proceeding.

**Commands:**

```bash
cd /home/sikmindz/Coding/Libraries
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
E=.hermes/evidence/agent-graph-remediation/$STAMP
mkdir -p "$E"
{
  date --iso-8601=seconds
  hostname
  id
  git rev-parse --show-toplevel
  git branch --show-current
  git rev-parse HEAD
  rustc --version --verbose
  cargo --version
  git status --short -- agent-graph agent-graph-mcp
  git diff --stat -- agent-graph agent-graph-mcp
} > "$E/baseline.txt"
git diff --binary -- agent-graph > "$E/agent-graph.patch"
tar --exclude='target' --exclude='__pycache__' -czf "$E/agent-graph-mcp.tgz" agent-graph-mcp
sha256sum "$E"/* > "$E/SHA256SUMS"
sha256sum -c "$E/SHA256SUMS"
```

**Gate:** Hash verification passes and the archive contains the expected MCP source/tests/manifests.

**Abort:** If source changes while the receipt is being captured, discard only the incomplete receipt directory and repeat.

## Task 0.2 — Prove workspace membership and dependency ownership

**Objective:** Establish whether both crates are canonical members of the parent workspace.

**Inspect:**

- `/home/sikmindz/Coding/Libraries/Cargo.toml`
- `/home/sikmindz/Coding/Libraries/Cargo.lock`
- `/home/sikmindz/Coding/Libraries/agent-graph/Cargo.toml`
- `/home/sikmindz/Coding/Libraries/agent-graph-mcp/Cargo.toml`

**Commands:**

```bash
cd /home/sikmindz/Coding/Libraries
cargo metadata --locked --no-deps --format-version 1 > /tmp/agent-graph-metadata.json
python3 - <<'PY'
import json
m=json.load(open('/tmp/agent-graph-metadata.json'))
for p in m['packages']:
    if p['name'] in {'agent-graph','agent-graph-mcp'}:
        print(p['name'], p['manifest_path'])
PY
```

**Decision rule:** Keep `agent-graph-mcp` in the parent workspace only if metadata binds it there and the parent manifest already owns it. Otherwise stop and create a separate repository decision record before changing package boundaries. Do not create a shadow copy.

## Task 0.3 — Create an isolated implementation worktree

**Objective:** Prevent unrelated dirty-tree edits from entering remediation commits.

**Target:** `/home/sikmindz/Coding/worktrees/agent-graph-remediation`

**Steps:**

1. Create a branch from the recorded HEAD.
2. Apply only the recorded engine patch.
3. Extract only the recorded MCP archive.
4. Compare target-path hashes with the baseline receipt.
5. Commit a clearly labelled baseline import; this commit establishes reviewability, not correctness.

**Commands:**

```bash
cd /home/sikmindz/Coding/Libraries
git worktree add -b fix/agent-graph-full-remediation \
  /home/sikmindz/Coding/worktrees/agent-graph-remediation \
  9577d4462fdf87691d835723a4f61f4c3efe6ad3
```

Apply from the exact receipt path selected in Task 0.1. Do not use global `git add -A`; stage explicit target paths only.

**Gate:** `git status --short` in the worktree contains only intentional Agent Graph baseline files before commit, then is clean after the baseline commit.

## Task 0.4 — Create the closure ledger

**Create:** `agent-graph-mcp/docs/remediation/hostile-audit-closure-ledger.md`

**Required columns:**

```text
finding | severity | reviewed revision | owner | RED test | fix revision |
focused gate | integration gate | process-boundary gate | rollback | disposition
```

Populate AG-001 through AG-028 as `open`. Every later task must update only its rows with receipt paths and exact revisions.

**Phase 0 gate:** Source identity, scoped baseline, canonical workspace ownership, clean remediation worktree, and open finding ledger all exist. No runtime behavior has changed.

---

# Phase 1 — Immediate source-level containment

## Task 1.1 — Replace permissive argument parsing with a strict CLI contract

**Files:**

- Modify: `agent-graph-mcp/Cargo.toml`
- Create: `agent-graph-mcp/src/cli.rs`
- Modify: `agent-graph-mcp/src/main.rs`
- Create/Test: `agent-graph-mcp/tests/cli_contract.rs`

**RED cases:**

- unknown `--dat-dir` exits nonzero;
- `--model` without a value exits nonzero;
- durable mode without `--data-dir` exits nonzero;
- `--require-integrity-key` without a readable key exits nonzero;
- malformed/non-http provider URL exits nonzero;
- MCP transport never starts after parse failure.

**Implementation:** Use `clap` derive or an existing workspace CLI parser. Make memory-only mode explicit as `--ephemeral`; never infer it from a missing/malformed path.

**Focused gate:**

```bash
cargo test -p agent-graph-mcp --test cli_contract -- --nocapture
```

## Task 1.2 — Add durable-startup integrity requirements

**Files:**

- Modify: `agent-graph-mcp/src/cli.rs`
- Modify: `agent-graph-mcp/src/main.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

**RED:** Durable mode with missing/unreadable/wrong-length integrity key refuses startup before DB mutation. Ephemeral mode reports every durability-dependent capability unavailable.

**GREEN:** Resolve key path once at startup, validate metadata and exact key length without logging key material, and pass a typed `IntegrityCapability` into server construction. Remove implicit environment lookup from lower layers.

## Task 1.3 — Enforce private data-store permissions and ownership

**Files:**

- Create: `agent-graph-mcp/src/fs_security.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Modify: `agent-graph-mcp/src/main.rs`
- Create/Test: `agent-graph-mcp/tests/fs_security.rs`

**RED matrix:** permissive umask, pre-existing `0755` directory, `0644` DB/WAL/SHM, symlink data directory, symlink key, foreign owner, and non-regular key file.

**GREEN:** Create directories with `0700`, files with `0600`, reject unsafe symlinks/ownership, and recheck sidecar modes after WAL initialization.

**Rollback:** Permission tightening is reversible via recorded modes, but do not restore insecure modes after successful migration.

## Task 1.4 — Add exclusive durable-owner locking

**Files:**

- Create: `agent-graph-mcp/src/owner_lock.rs`
- Modify: `agent-graph-mcp/src/main.rs`
- Create/Test: `agent-graph-mcp/tests/owner_lock.rs`

**RED:** Two durable server owners start against one temporary data directory; the second currently reaches startup.

**GREEN:** Acquire a nonblocking exclusive lock on a private lock file before DB open or recovery. Return typed `DATA_DIR_ALREADY_OWNED` with no database writes.

**Important:** This is containment, not the final multi-client design. The daemon/proxy architecture in Phase 3 restores multiple MCP client connections safely.

**Phase 1 gate:**

```bash
cargo fmt --check -p agent-graph-mcp
cargo clippy -p agent-graph-mcp --all-targets -- -D warnings
cargo test -p agent-graph-mcp --test cli_contract --test fs_security --test owner_lock
cargo test -p agent-graph-mcp --no-fail-fast
```

Safe claim after Phase 1: a single durable owner starts fail-closed. Do not claim multi-client operation yet.

---

# Phase 2 — Repair engine execution semantics

## Task 2.1 — Define one canonical execution cursor

**Files:**

- Create: `agent-graph/src/execution_cursor.rs`
- Modify: `agent-graph/src/checkpoint.rs`
- Modify: `agent-graph/src/lib.rs`
- Modify: `agent-graph/src/prelude.rs`
- Test: `agent-graph/tests/interrupt_tests.rs`

**Schema fields:**

```text
iteration
step_number
interrupt_phase: before | after
resume_boundary
active_nodes
remaining_nodes
completed_nodes_in_superstep
graph_version
state_digest
```

Use ordered collections/canonical sorting for every hashed or serialized set.

**RED:** An interrupt after one node in a fan-out records zero iteration and empty active nodes.

**GREEN:** Eliminate fabricated defaults. The same cursor instance feeds in-memory return and durable checkpoint persistence.

## Task 2.2 — Make before/after resume semantics exact

**Files:**

- Modify: `agent-graph/src/engine.rs`
- Modify: `agent-graph/src/checkpoint.rs`
- Test: `agent-graph/tests/interrupt_tests.rs`
- Test: `agent-graph/tests/interrupt_failure_contract.rs`

**RED tests:**

1. `interrupt_before` executes the node once after resume.
2. `interrupt_after` does not re-execute the completed node.
3. Two-branch partial superstep executes each remaining branch exactly once.
4. State and iteration limits survive resume.

**Rollback trigger:** Any existing interrupt test changes expected behavior without a documented compatibility decision.

## Task 2.3 — Version and migrate checkpoint encoding

**Files:**

- Modify: `agent-graph/src/checkpoint.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

**Policy:** Introduce checkpoint schema version 2. V1 checkpoints remain readable only as `legacy_non_resumable` unless all missing cursor fields can be reconstructed without guessing.

**RED:** A V1 checkpoint with missing active-node data must never silently resume.

## Task 2.4 — Bind conditional routing semantics to graph version

**Files:**

- Modify: `agent-graph/src/graph.rs`
- Modify: `agent-graph/src/error.rs`
- Test: `agent-graph/tests/routing_tests.rs`
- Test: `agent-graph/tests/interrupt_tests.rs`

**Design:** Replace opaque closure-only resumability with an explicit `router_semantic_digest` or canonical router specification. Opaque conditional closures may execute but must be marked non-resumable.

**RED:** Identical nodes with different route outcomes produce the same graph hash.

**GREEN:** Hash differs, or checkpoint creation returns `NON_RESUMABLE_ROUTER` before durable resume is advertised.

## Task 2.5 — Version graph hashing

**Files:**

- Modify: `agent-graph/src/graph.rs`
- Modify: `agent-graph-mcp/src/spec.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Test: `agent-graph/tests/routing_tests.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

Store `hash_algorithm_version` beside every graph/checkpoint/run. Never reinterpret an old hash as a new hash.

## Task 2.6 — Make retry waits cancellation-aware

**Files:**

- Modify: `agent-graph/src/engine.rs`
- Modify: `agent-graph/Cargo.toml` if `tokio-util::sync::CancellationToken` is not already available
- Test: `agent-graph/tests/retry_tests.rs`

**RED:** Cancel during a multi-second backoff; current execution waits and retries.

**GREEN:** `tokio::select!` returns promptly with typed cancellation and no later `NodeStart`.

## Task 2.7 — Contain parallel branch failure and cancellation

**Files:**

- Modify: `agent-graph/src/engine.rs`
- Test: `agent-graph/tests/parallel_tests.rs`
- Test: `agent-graph/tests/streaming_tests.rs`

**RED:** A failing branch plus delayed side-effect branch allows the delayed marker after parent return.

**GREEN:** Cancel sibling branches, await all task termination, emit branch terminal/cancel events, then return the parent error. Record `external_effect_may_have_escaped` if the provider boundary cannot prove cancellation.

## Task 2.8 — Replace silent checkpoint-store no-ops with typed failures

**Files:**

- Modify: `agent-graph/src/checkpoint_store.rs`
- Modify: `agent-graph/src/error.rs`
- Test: `agent-graph/tests/checkpoint_store_failure_contract.rs`

**RED:** Every mutator called with unknown run/attempt ID returns `Ok(())`.

**GREEN:** Return `RunNotFound`, `AttemptNotFound`, `AttemptRunMismatch`, or `InvalidTransition` as appropriate.

## Task 2.9 — Enforce atomic monotonic terminal transitions

**Files:**

- Modify: `agent-graph/src/checkpoint_store.rs`
- Modify: `agent-graph/src/state.rs`
- Test: `agent-graph/tests/checkpoint_store_failure_contract.rs`
- Test: `agent-graph/tests/runtime_tests.rs`

**RED:** Race `complete_run` and `fail_run`; both report success or later caller overwrites terminal state.

**GREEN:** Exactly one legal terminal transition wins. Repeating the same terminal transition is idempotent; conflicting terminal states return `TerminalStateConflict`.

**Phase 2 gate:**

```bash
cargo fmt --check -p agent-graph
cargo clippy -p agent-graph --all-targets -- -D warnings
cargo test --locked -p agent-graph --test interrupt_tests -- --nocapture
cargo test --locked -p agent-graph --test retry_tests -- --nocapture
cargo test --locked -p agent-graph --test parallel_tests -- --nocapture
cargo test --locked -p agent-graph --test checkpoint_store_failure_contract -- --nocapture
cargo test --locked -p agent-graph --tests -- --nocapture
```

Safe claim after Phase 2: engine checkpoint/cancellation semantics pass synthetic in-process tests. Do not claim process-restart durability.

---

# Phase 3 — Replace multi-process ownership with daemon + stdio proxy

## Task 3.1 — Freeze the local transport ADR

**Create:** `agent-graph-mcp/docs/adr/0001-single-daemon-stdio-proxy.md`

**Must specify:**

- one daemon per durable data directory;
- private Unix socket under `$XDG_RUNTIME_DIR/agent-graph/`;
- socket directory `0700`, socket `0600`;
- no TCP listener;
- peer UID verification;
- line/frame limits;
- connection/session ID;
- daemon lock acquisition and shutdown behavior;
- proxy backpressure and stderr/stdout separation;
- compatibility behavior when daemon is unavailable;
- no automatic ephemeral fallback.

**Gate:** Controller and independent reviewer agree the ADR closes AG-001 without creating a second truth store.

## Task 3.2 — Create daemon instance schema and migration

**Files:**

- Modify: `agent-graph-mcp/src/store.rs`
- Create: `agent-graph-mcp/src/migrations.rs`
- Create/Test: `agent-graph-mcp/tests/migrations.rs`

**Schema:**

```text
schema_migrations(version, applied_at, migration_digest)
server_instances(instance_id, started_at, heartbeat_at, stopped_at, binary_digest)
executions.owner_instance_id NOT NULL for new runs
```

**RED:** Start with a legacy database and prove active rows are globally rewritten.

**GREEN:** Migration is transactional, versioned, idempotent, and backup-compatible. Legacy active rows become `legacy_owner_unknown` and non-resumable; they are not assigned to the new daemon.

## Task 3.3 — Build the lock-owning daemon

**Files:**

- Create: `agent-graph-mcp/src/daemon.rs`
- Create: `agent-graph-mcp/src/bin/agent-graph-mcpd.rs`
- Modify: `agent-graph-mcp/src/lib.rs`
- Test: `agent-graph-mcp/tests/process_boundary.rs`

**RED:** A second independent server startup mutates a run owned by process A.

**GREEN:** Daemon owns DB, run manager, recovery, and Unix listener. Lock is acquired before DB open. New proxy connections do not instantiate stores or invoke recovery.

## Task 3.4 — Build the thin stdio proxy

**Files:**

- Create: `agent-graph-mcp/src/proxy.rs`
- Modify: `agent-graph-mcp/src/main.rs`
- Test: `agent-graph-mcp/tests/proxy_stdio.rs`

**Contract:**

- stdin/stdout carry only MCP JSON-RPC;
- logs go to stderr;
- frames have a bounded maximum size;
- disconnect propagates cleanly;
- daemon absence produces a typed startup error;
- no database path or integrity key is needed by the proxy.

## Task 3.5 — Add daemon liveness and graceful shutdown

**Files:**

- Modify: `agent-graph-mcp/src/daemon.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Test: `agent-graph-mcp/tests/process_boundary.rs`

**RED cases:** SIGTERM during idle, active provider request, checkpoint publication, and SQLite transaction.

**GREEN:** Stop accepting new runs, request cancellation, bounded wait, commit or roll back transactions, mark instance stopped when clean, and let lock release on process exit.

## Task 3.6 — Prove concurrent proxy isolation

**Test:** `agent-graph-mcp/tests/process_boundary.rs`

**Required scenario:**

1. Start daemon with temporary key/data/socket.
2. Connect proxy A and start a deliberately blocked run.
3. Connect proxies B through N.
4. Query status and create/read unrelated graphs.
5. Confirm A remains running and DB status unchanged.
6. Kill A proxy; daemon ownership remains.
7. Connect a new proxy; run remains coherent.
8. Stop daemon; restart daemon; only prior daemon-owned active runs become interrupted.

**Phase 3 gate:** At least 100 concurrent proxy connect/disconnect cycles pass with one daemon PID and no status corruption, protocol leakage, deadlock, or orphan child.

---

# Phase 4 — Canonical durable lifecycle and terminal publication

## Task 4.1 — Centralize lifecycle classification

**Files:**

- Create: `agent-graph-mcp/src/lifecycle.rs`
- Modify: `agent-graph-mcp/src/run_manager.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/lifecycle.rs`

One function must classify accepted/running/completed/failed/cancelled/interrupted states. No handler may invent its own mapping.

## Task 4.2 — Owner-bound restart recovery

**Files:**

- Modify: `agent-graph-mcp/src/daemon.rs`
- Modify: `agent-graph-mcp/src/store.rs`
- Test: `agent-graph-mcp/tests/process_boundary.rs`

**RED:** Starting a client globally updates all active rows.

**GREEN:** On exclusive daemon startup, recover only active rows belonging to a previous daemon instance. Recovery emits a durable reason and event. Starting a proxy performs no recovery SQL.

## Task 4.3 — Publish terminal projection atomically

**Files:**

- Modify: `agent-graph-mcp/src/store.rs`
- Modify: `agent-graph-mcp/src/run_manager.rs`
- Modify: `agent-graph-mcp/src/evidence.rs`
- Test: `agent-graph-mcp/tests/terminal_projection.rs`

**Transaction contents:**

- terminal execution row;
- bounded terminal event set;
- canonical receipt;
- receipt digest;
- bundle/index backpointer;
- final state digest;
- graph ID/version and request digest.

**Failure injection:** Fail after each SQL write and assert the entire transaction rolls back.

## Task 4.4 — Make all post-restart reads converge

**Files:**

- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/run_manager.rs`
- Test: `agent-graph-mcp/tests/process_boundary.rs`

**Tools covered:**

- `graph_run_get`
- `graph_run_state`
- `graph_run_events`
- `graph_run_receipt`
- `graph_status(resource=run)`

Every tool must return the same durable disposition after restart. In-memory registry is a cache, not authority.

## Task 4.5 — Migrate idempotency to non-null canonical request binding

**Files:**

- Modify: `agent-graph-mcp/src/store.rs`
- Modify: `agent-graph-mcp/src/migrations.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/migrations.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

**Canonical digest includes:** tool operation, graph ID/version, execution mode, normalized input, budgets, checkpoint flag, and caller session/capability scope where material.

**Legacy policy:** Null-digest rows are quarantined and cannot satisfy a request. Migration must not invent a digest.

## Task 4.6 — Make synchronous timeout disposition explicit

**Files:**

- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/run_manager.rs`
- Test: `agent-graph-mcp/tests/process_boundary.rs`

**Response fields:**

```text
timed_out: true
completion: unknown
cancellation: requested | acknowledged | provider_may_continue
run_id
idempotency_key
```

Do not claim cancellation when the provider request remains live.

**Phase 4 gate:** Restart, SQLite failure injection, idempotency conflict, timeout, and terminal publication tests all pass against the real daemon/proxy process boundary.

---

# Phase 5 — Authority and authenticated operator decisions

## Task 5.1 — Define the threat model and capability matrix

**Create:** `agent-graph-mcp/docs/security/threat-model.md`

**Principals:** MCP model client, stdio proxy, daemon, local operator, untrusted local user, provider endpoint, hook/script, installer.

**Capabilities:** graph read/create/run/cancel, witness capture/read, checkpoint request/read, approval decide, graph delete, database migration, config/install.

**Rule:** Model-facing MCP capability excludes approval decisions, permanent deletion, migration, and release/install authority.

## Task 5.2 — Enforce server-side capability policy

**Files:**

- Create: `agent-graph-mcp/src/auth.rs`
- Modify: `agent-graph-mcp/src/daemon.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/tools.rs`
- Create/Test: `agent-graph-mcp/tests/authorization.rs`

**RED:** Caller changes `actor` string and approves/deletes.

**GREEN:** Daemon derives principal/capability from the connection channel, not request fields. Unauthorized tools are absent from that connection’s tool list or return stable `FORBIDDEN` without mutation.

## Task 5.3 — Remove self-asserted approval from model MCP

**Files:**

- Modify: `agent-graph-mcp/src/tools.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/templates.rs`
- Test: `agent-graph-mcp/tests/authorization.rs`

Remove or permanently forbid `graph_approval_decide` on model-facing connections. Keep approval-list/get read-only if content policy permits. Rename current historical actor field to `claimed_actor_label` during migration so it cannot be mistaken for authenticated identity.

## Task 5.4 — Add an OS-authenticated operator helper

**Files:**

- Create: `agent-graph-mcp/src/bin/agent-graph-approve.rs`
- Create: `agent-graph-mcp/packaging/polkit/com.recursiveintell.agent-graph.approve.policy`
- Create: `agent-graph-mcp/packaging/polkit/agent-graph-approve-helper`
- Create/Test: `agent-graph-mcp/tests/operator_authority.rs`

**Design:** Polkit action uses `auth_self` or stricter policy; no password, token, or private key enters MCP, chat, argv, or environment. The helper receives bounded decision material through a protected pipe, re-reads canonical approval metadata from the daemon, and submits the authenticated OS principal.

**Approval binding:**

```text
approval_id
checkpoint_id and checkpoint digest
graph_id and graph_version
allowed_decisions
chosen decision
expiration
nonce
operator UID/principal
authority mechanism and policy version
```

**RED:** Model-facing caller, wrong UID, expired decision, altered checkpoint, replay, and second decision all fail.

**Safety:** Installation of Polkit policy is a separate privileged action requiring explicit user approval in Phase 9.

## Task 5.5 — Implement exact-once decision consumption

**Files:**

- Modify: `agent-graph-mcp/src/store.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/operator_authority.rs`

Use one transaction to validate decision binding, set decision, and consume checkpoint/approval exactly once. Concurrent approve/reject attempts must have one winner.

## Task 5.6 — Keep HITL unavailable until the real gate is installed

**Files:**

- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/templates.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

`graph_status.hitl` becomes available only when daemon persistence, integrity key, operator authority policy, and approval helper are all active and verified. Missing any dependency is a typed unavailable state, never fallback.

**Phase 5 gate:** Independent hostile test proves the model-accessible tool surface cannot make an approval decision even when it supplies plausible actor metadata or invokes the operator binary without completing OS authentication.

---

# Phase 6 — Executable node contract, policy, templates, and evidence

## Task 6.1 — Reject unsupported node types before registration

**Files:**

- Modify: `agent-graph-mcp/src/spec.rs`
- Modify: `agent-graph-mcp/src/compiler.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/mcp_integration.rs`

For each declared node type, add a conformance row: schema parse, compiler support, runtime support, resume eligibility, effect classification. Unsupported types return `UNSUPPORTED_NODE_TYPE` before persistence.

## Task 6.2 — Replace the false research template contract

**Files:**

- Modify: `agent-graph-mcp/src/templates.rs`
- Modify: `agent-graph-mcp/README.md`
- Test: `agent-graph-mcp/tests/template_semantics.rs`

**Decision:** Rename the source-free template to `analysis_pipeline`. Add `witnessed_research_pipeline` only if input requires caller-supplied source witness IDs and runtime verifies them before synthesis. Do not claim the graph fetches sources.

## Task 6.3 — Implement real validation routing

**Files:**

- Modify: `agent-graph-mcp/src/templates.rs`
- Test: `agent-graph-mcp/tests/template_semantics.rs`

Validator false must route through a bounded correction loop or terminate failed; formatter must never run on invalid output. Add loop limit and terminal reason.

## Task 6.4 — Repair council workstream dataflow

**Files:**

- Modify: `agent-graph-mcp/src/templates.rs`
- Modify if needed: `agent-graph-mcp/src/nodes.rs`
- Test: `agent-graph-mcp/tests/template_semantics.rs`

Each analyst must consume its declared coordinator workstream, not the original ingress. Preserve immutable original input separately.

## Task 6.5 — Repair classifier routing without ingress overwrite

**Files:**

- Modify: `agent-graph-mcp/src/templates.rs`
- Modify if needed: `agent-graph-mcp/src/nodes.rs`
- Test: `agent-graph-mcp/tests/template_semantics.rs`

Store label at `classification.label`; downstream handler receives original report plus label. Never write routing labels into `__input__`.

## Task 6.6 — Make approval template capability-gated

**Files:**

- Modify: `agent-graph-mcp/src/templates.rs`
- Test: `agent-graph-mcp/tests/template_semantics.rs`

The approval template is not listed executable unless Phase 5 authority and the required resume boundary are active. On unavailable systems, template instantiation returns a capability error rather than a misleading graph.

## Task 6.7 — Separate evidence shape, integrity, and factual support

**Files:**

- Modify: `agent-graph-mcp/src/evidence.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Modify: `agent-graph-mcp/src/tools.rs`
- Test: `agent-graph-mcp/tests/evidence_contract.rs`

Expose distinct dispositions:

```text
shape_valid
integrity_verified
source_witness_bound
source_authority_unverified | verified_by_controller
factual_support_unjudged | supported | contested
```

No model output may promote itself to source-verified.

## Task 6.8 — Make policy preflight substantive but non-authoritative

**Files:**

- Create: `agent-graph-mcp/src/policy.rs`
- Modify: `agent-graph-mcp/src/server.rs`
- Test: `agent-graph-mcp/tests/policy.rs`

Policy checks graph version, executable nodes, effect classes, budgets, provider destination, witness requirements, capability availability, and approval requirements. Authorization remains a separate runtime gate.

**Phase 6 gate:** Every built-in executes in a disposable daemon, hits each intended branch, and passes semantic assertions. Tool descriptions and README match live behavior.

---

# Phase 7 — Hermes hooks, governance scripts, watchdogs, and kits

## Task 7.1 — Make the graph guard deterministic and fail closed at its own boundary

**Files:**

- Modify: `/home/sikmindz/.hermes/agent-hooks/agent-graph-guard.py`
- Create: `/home/sikmindz/.hermes/agent-hooks/tests/test_agent_graph_guard.py`

**RED inputs:** malformed JSON, `null`, array, missing tool name, wrong input types, string-encoded budgets, stale approval fields, unknown graph ID.

**GREEN:** Always emit bounded structured JSON; never traceback. Because Hermes hooks may fail open, duplicate every material rule in server-side policy. Document the hook as advisory defense-in-depth only.

## Task 7.2 — Repair preflight schema discovery

**Files:**

- Modify: `/home/sikmindz/.hermes/agent-hooks/preflight_map.py`
- Test: its existing tests or create `/home/sikmindz/.hermes/agent-hooks/tests/test_preflight_map.py`

Read `mcp_servers`, not obsolete `mcp.tools`. Verify only names/enabled/command identities; never print secret env values.

## Task 7.3 — Make observer telemetry safe and visible

**Files:**

- Modify: `/home/sikmindz/.hermes/agent-hooks/agent-graph-observer.py`
- Modify: `/home/sikmindz/.hermes/scripts/agent_graph_ops.py`
- Test: create `/home/sikmindz/.hermes/scripts/test_agent_graph_ops.py`

**Changes:**

- use `tempfile` with `0600` and exclusive creation;
- require explicit structured success;
- emit bounded telemetry counters on observer failure;
- parse JSON-string specs safely;
- do not silently admit failed/unknown graph creation.

## Task 7.4 — Rebuild template promotion from canonical receipts

**Files:**

- Modify: `/home/sikmindz/.hermes/scripts/agent_graph_ops.py`
- Test: `/home/sikmindz/.hermes/scripts/test_agent_graph_ops.py`

**RED:** Three arbitrary receipt strings and claimed operator names promote a candidate; a later bad outcome leaves it approved.

**GREEN:** Promotion verifies each terminal receipt in canonical Agent Graph storage, graph/template/version binding, integrity status, distinct run IDs, and authenticated operator decision. Any bad/revoked outcome demotes to quarantine. Projection rebuild from canonical receipts produces identical state.

## Task 7.5 — Make watchdog process- and capability-aware

**Files:**

- Modify: `/home/sikmindz/.hermes/scripts/agent_graph_watchdog.py`
- Modify if used: `/home/sikmindz/.hermes/scripts/agent_graph_ops.py`
- Test: create `/home/sikmindz/.hermes/scripts/test_agent_graph_watchdog.py`

**Checks:** one daemon owner, any number of bounded proxies, no legacy DB-owning server, config digest parity, key availability in daemon only, socket permissions, binary/build digest, DB integrity, migrations, live status, receipt/event continuity, and stale process detection.

## Task 7.6 — Make approval inbox read-only and authority-aware

**Files:**

- Modify: `/home/sikmindz/.hermes/scripts/agent_graph_approval_inbox.py`
- Test: create `/home/sikmindz/.hermes/scripts/test_agent_graph_approval_inbox.py`

It may notify the user but must not decide approvals. Include exact operator command/GUI route without exposing prompt body or credentials.

## Task 7.7 — Fix multi-host installer merge and rollback

**Files:**

- Modify: `/home/sikmindz/Coding/agent-graph-kits/shared/scripts/setup-host.py`
- Create: `/home/sikmindz/Coding/agent-graph-kits/tests/test_setup_host.py`

**RED:** Existing unrelated MCP servers/rules disappear; repeated install overwrites `.bak`.

**GREEN:** Schema-aware merge of a managed Agent Graph block, timestamped digest-backed backup, same-directory temporary write, fsync, atomic replace, preserved mode/owner, and uninstall that removes only the managed block.

## Task 7.8 — Fix environment/path handling in kits

**Files:**

- Modify: kit launcher/config files under `/home/sikmindz/Coding/agent-graph-kits/*`
- Modify: `/home/sikmindz/Coding/agent-graph-kits/shared/scripts/run-server.sh`
- Test: `/home/sikmindz/Coding/agent-graph-kits/tests/test_launchers.py`

Resolve home paths in the launcher, not by assuming host clients expand literal `${HOME}` values. Never put secret material in generated JSON.

## Task 7.9 — Replace parse-only kit validation with contract tests

**Files:**

- Modify: `/home/sikmindz/Coding/agent-graph-kits/scripts/validate-all-kits.py`
- Modify: `/home/sikmindz/Coding/agent-graph-kits/shared/scripts/doctor.py`
- Test: `/home/sikmindz/Coding/agent-graph-kits/tests/`

For each supported host verify schema, preservation of unrelated config, launcher executable identity, daemon/proxy arguments, environment resolution, initialize/tools-list, stderr cleanliness, and uninstall rollback.

## Task 7.10 — Canonicalize the hook allowlist

**Files:**

- Modify through Hermes-supported flow: `/home/sikmindz/.hermes/shell-hooks-allowlist.json`
- Create audit utility or extend daily health check.

Bind entries to resolved path, interpreter, content digest, owner, and mode. Remove stale/duplicate paths only after capturing a rollback copy and proving they are inactive.

**Phase 7 gate:** Python syntax/tests, `hermes hooks doctor`, direct exact-wire-format hook probes, watchdog negative fixtures, and all kit contract tests pass. No live config is switched yet.

---

# Phase 8 — Quality, supply chain, and release provenance

## Task 8.1 — Eliminate production panic paths and restore strict clippy

**Files:** All clippy-cited production files, especially:

- `agent-graph-mcp/src/evidence.rs`
- `agent-graph-mcp/src/run_manager.rs`
- `agent-graph-mcp/src/server.rs`
- `agent-graph-mcp/src/spec.rs`

Replace `expect`/poison panics with typed errors. Add negative tests before each replacement.

**Gate:**

```bash
cargo clippy -p agent-graph -p agent-graph-mcp --all-targets -- -D warnings
```

## Task 8.2 — Enforce declared Rust 1.75 compatibility

**Files:** `agent-graph-mcp/Cargo.toml` and call sites using post-1.75 APIs.

**Decision:** Preserve MSRV 1.75 unless an ADR explicitly raises it. Replace `Option::is_none_or` and any other newer APIs.

**Gate:** Build/test using Rust 1.75 in a clean toolchain/container plus current stable.

## Task 8.3 — Close dependency advisory and policy gates

**Files:** `Cargo.lock`, `deny.toml` if already canonical or create one in the workspace root after ownership review.

**Commands:**

```bash
cargo audit --json > .hermes/evidence/cargo-audit.json
cargo deny check
cargo tree -d
```

Every advisory gets fixed, explicitly unaffected with evidence, or quarantined with an owner/date. Nonzero unexplained exit is a release block.

## Task 8.4 — Commit and version the multi-host kits

Establish a canonical Git baseline for `/home/sikmindz/Coding/agent-graph-kits`, add CI, and pin compatible server protocol/build identity. Do not publish from an all-untracked directory.

## Task 8.5 — Generate a real build manifest and SBOM

**Create:**

- `agent-graph-mcp/build.rs` or release script that captures non-secret build identity
- `agent-graph-mcp/docs/release/manifest.schema.json`
- `agent-graph-mcp/scripts/build-release.sh`
- generated release manifest and CycloneDX/SPDX SBOM in the release evidence directory

**Manifest fields:** source commit, dirty=false, Cargo.lock SHA-256, Rust/Cargo versions, target, features, graph/checkpoint/schema versions, artifact SHA-256, build timestamp, test receipt digests.

## Task 8.6 — Make certification executable and source-bound

**Files:**

- Modify: `/home/sikmindz/.hermes/scripts/agent-graph-certify.py`
- Modify: `/home/sikmindz/.hermes/scripts/test-agent-graph-certify.py`
- Replace placeholder manifest generation.

Certification must fail on dirty source, absent commit, binary mismatch, stale config digest, failed process-boundary test, missing authority gate, or open critical/high ledger entry.

**Phase 8 gate:** Clean worktree, MSRV/current stable gates, fmt, strict clippy, full tests, audit/deny, SBOM, build manifest, and certification candidate all succeed.

---

# Phase 9 — Controlled live migration and activation

> This phase has external side effects. Obtain explicit user approval immediately before service/process/config/database changes. Do not bundle that approval into earlier coding consent.

## Task 9.1 — Create a timestamped rollback pack

Include:

- SQLite online backup plus WAL checkpoint status;
- current DB/WAL/SHM modes and hashes;
- installed binary and hash;
- Hermes config and hash;
- integrity-key path metadata only, never key material in reports;
- active process/service inventory;
- cron registry;
- hooks/allowlist/scripts changed by the remediation;
- exact rollback commands.

Verify backup with `PRAGMA integrity_check` on the copy.

## Task 9.2 — Classify existing graphs/runs before migration

Export a non-secret inventory of 8 graphs and 12 historical execution rows. Mark all rows without terminal receipts/events as `legacy_volatile_unverified`. Do not manufacture missing receipts or events. Preserve them as audit records or quarantine them in a versioned legacy table.

## Task 9.3 — Dry-run migration on a copied database

Run every migration against the backup copy first. Verify:

- row counts by table/status;
- null idempotency rows quarantined;
- legacy active owner state handled honestly;
- receipt/witness HMAC verification;
- permissions after WAL creation;
- old binary still reads the untouched original backup.

## Task 9.4 — Stop legacy ownership cleanly

With explicit approval:

1. Disable new Agent Graph execution at Hermes policy/config boundary.
2. Wait for or cancel active runs with recorded disposition.
3. Stop only identified legacy Agent Graph MCP processes/watchdogs/services.
4. Verify no process owns the SQLite/WAL files.
5. Never use broad `pkill agent` patterns.

## Task 9.5 — Install daemon, proxy, service, and operator policy atomically

**Install targets:**

- `/home/sikmindz/.cargo/bin/agent-graph-mcp` — stdio proxy
- `/home/sikmindz/.cargo/bin/agent-graph-mcpd` — daemon
- operator helper binary/policy at reviewed locations
- systemd user service for daemon

Install via temporary file + fsync + atomic rename. Preserve prior binaries in rollback pack. Verify hashes after installation.

## Task 9.6 — Update Hermes config safely

Back up and YAML-parse `~/.hermes/config.yaml`. Change only the Agent Graph stanza:

- proxy command only;
- daemon socket path, not DB/key, for proxy;
- no secret material;
- retain disabled/quarantine state until live tests finish.

Run:

```bash
hermes config check
hermes mcp test agent_graph
```

A passing disposable connection is not activation proof.

## Task 9.7 — Start daemon and restart/reload every owner path

Verify:

- one daemon PID;
- zero legacy DB-owning MCP PIDs;
- expected bounded proxy count;
- daemon key path present without printing value;
- private socket/data modes;
- service enabled/active;
- gateway/desktop/new session all report the same capabilities and build digest.

Existing sessions must be reloaded or replaced; stale sessions are not silently accepted.

## Task 9.8 — Run installed-process acceptance matrix

Using the installed binaries and disposable directories where destructive:

1. initialize/tools-list JSON cleanliness;
2. concurrent proxy isolation;
3. second-daemon lock rejection;
4. active-run proxy churn;
5. daemon kill/restart disposition;
6. cancellation during retry and provider request;
7. parallel branch failure containment;
8. idempotency same-request replay and changed-request conflict;
9. approval model denial and OS-authenticated success/replay rejection;
10. event cursor boundaries;
11. receipt/source-witness tamper detection;
12. SQLite failure rollback;
13. config reload parity;
14. watchdog detects injected stale/missing-capability fixture.

## Task 9.9 — Re-enable only certified capability tiers

**Tier A:** read-only graph registry/status.

**Tier B:** non-effectful ephemeral analysis.

**Tier C:** durable deterministic-local runs.

**Tier D:** checkpoint/resume for proven resumable graphs.

**Tier E:** authenticated approval-gated effects.

Advance one tier at a time. Failure rolls back to the last proven tier; it does not trigger fallback.

**Live rollback triggers:** multiple daemons, binary mismatch, config divergence, DB integrity failure, missing receipt/event, unexpected legacy recovery, approval bypass, orphaned provider requests, or unexplained cron failure.

---

# Phase 10 — Independent hostile re-audit and closure

## Task 10.1 — Re-run the full source gauntlet from a frozen revision

Capture HEAD and scoped status before and after every long command. Any source change invalidates the run.

```bash
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo clippy -p agent-graph -p agent-graph-mcp --all-targets --locked -- -D warnings
cargo test --locked -p agent-graph --tests -- --nocapture
cargo test --locked -p agent-graph-mcp --no-fail-fast -- --nocapture
cargo audit
cargo deny check
python3 -m unittest /home/sikmindz/.hermes/scripts/test-agent-graph-certify.py
python3 /home/sikmindz/Coding/agent-graph-kits/scripts/validate-all-kits.py
```

## Task 10.2 — Commission independent hostile review lanes

At minimum:

1. engine checkpoint/hash/cancellation;
2. daemon/proxy/store/restart/idempotency;
3. authority/approval/ACL/provider network policy;
4. Hermes hooks/watchdogs/config/cron;
5. kits/release/provenance/install/rollback.

Each reviewer receives the exact frozen revision and returns file:line evidence, negative tests, and verdict. The controller re-verifies decisive citations and tests.

## Task 10.3 — Close the finding ledger

Every AG-001 through AG-028 row must be one of:

- `verified_closed` with source revision and receipt;
- `quarantined` with a mechanical disabled path and test;
- `accepted_risk` with explicit user authority and expiry;
- `open` — which blocks the corresponding capability tier.

No “fixed by inspection” disposition is allowed for critical/high findings.

## Task 10.4 — Produce the auditor-rerunnable certification bundle

Bundle:

- source/build/config identities;
- migration and rollback receipts;
- exact commands and outputs;
- installed artifact hashes;
- process/service inventory;
- capability matrix;
- closed finding ledger;
- known limitations;
- public-safe claim text.

## Task 10.5 — Final claim boundary

Only after all gates pass may the system claim:

> Agent Graph is a local single-user daemon-backed graph orchestration runtime with private SQLite persistence, restart-coherent terminal projections, owner-bound interruption recovery, request-bound idempotency, bounded cancellation, graph-versioned checkpoints for explicitly resumable graphs, and an OS-authenticated external approval path for enabled privileged workflows.

Even then, do not claim:

- factual research verification without controller-verified sources;
- guaranteed external-effect cancellation;
- hostile same-account isolation against arbitrary code execution;
- remote/multi-user service security;
- compliance or enterprise readiness;
- provider-independent reproducibility of LLM output.

---

## 4. Commit and review discipline

1. Work only in the isolated remediation worktree.
2. One semantic fix per commit; do not mix engine, MCP, Hermes config, and kits in one commit.
3. Each mutating task follows RED → observed failure → minimal GREEN → focused test → broader gate → diff review → commit.
4. Suggested commit families:

```text
chore: establish agent graph remediation baseline
test(engine): expose checkpoint cursor corruption
fix(engine): persist canonical interrupt cursor
test(engine): expose router hash drift
fix(engine): bind router semantics to graph version
fix(engine): propagate cancellation through retries and branches
fix(store): reject invalid checkpoint transitions
feat(mcp): add single-owner daemon and stdio proxy
fix(mcp): publish terminal projections atomically
fix(mcp): bind idempotency to canonical requests
feat(auth): separate model and operator authority
fix(templates): enforce semantic execution contracts
fix(hermes): harden agent graph hooks and watchdogs
fix(kits): merge host configuration atomically
chore(release): add attested build and certification bundle
```

5. After each delegated edit, the controller must re-read files, inspect scoped diff, and rerun acceptance tests. Subagent reports are not receipts.
6. Do not merge to the original dirty branch until the remediation branch passes Phase 10 and the user approves the integration strategy.

---

## 5. Master acceptance checklist

### Source and artifact

- [ ] Canonical committed source exists for engine, MCP, and kits.
- [ ] Remediation worktree is clean at certified revision.
- [ ] Cargo.lock digest is recorded.
- [ ] MSRV and current stable pass.
- [ ] fmt/check/clippy/tests/audit/deny pass.
- [ ] Built and installed hashes match.
- [ ] SBOM and manifest are real, not placeholders.

### Runtime

- [ ] One daemon owns the live data directory.
- [ ] Proxies never touch SQLite or recovery.
- [ ] Second daemon fails before mutation.
- [ ] Proxy churn cannot interrupt a run.
- [ ] Daemon restart produces coherent owner-bound interruption.
- [ ] Every terminal run has events, receipt, and bundle/index.

### Security and authority

- [ ] Data/socket/key/backup permissions are private.
- [ ] Unsafe symlinks/ownership fail closed.
- [ ] Model tools cannot approve or administratively delete.
- [ ] Operator decision requires real OS authentication.
- [ ] Approval replay/alteration/expiry tests fail closed.
- [ ] Provider endpoint policy rejects metadata/internal destinations unless explicitly authorized.

### Engine

- [ ] Interrupt cursor contains exact execution position.
- [ ] Before/after resume has no duplicate/skip.
- [ ] Router semantics are hash/version bound or non-resumable.
- [ ] Retry and branch cancellation are prompt and joined.
- [ ] Invalid store IDs/transitions return typed errors.
- [ ] Conflicting terminal writes cannot overwrite each other.

### MCP semantics

- [ ] Legacy null idempotency rows are quarantined.
- [ ] Sync timeout reports completion unknown.
- [ ] Post-restart read tools agree.
- [ ] Unsupported nodes are rejected before registration.
- [ ] Status distinguishes zero, unavailable, unverified, and verified.

### Templates and ecosystem

- [ ] Every built-in has a semantic black-box test.
- [ ] Research template requires witnessed input or is honestly named analysis.
- [ ] Validator routing actually controls formatter execution.
- [ ] Council and classifier preserve declared dataflow.
- [ ] Guard never tracebacks and server duplicates material enforcement.
- [ ] Observer failures are visible.
- [ ] Template promotion is canonical-receipt and operator-authority bound.
- [ ] Watchdog checks daemon ownership and capability parity.
- [ ] Kit install/reinstall/uninstall preserves unrelated host config.
- [ ] Every supported host completes an actual initialize/tools-list smoke.

### Migration and rollback

- [ ] Online backup passes integrity check.
- [ ] Legacy graphs/runs are classified honestly.
- [ ] Migration passes against copied DB before live DB.
- [ ] Previous binary/config/database/service state can be restored.
- [ ] Rollback was dry-run in a disposable environment.
- [ ] Live capability tiers were enabled incrementally.

---

## 6. Execution order and hard dependencies

```text
Phase 0 source freeze
  → Phase 1 containment
    → Phase 2 engine semantics
      → Phase 3 daemon/proxy ownership
        → Phase 4 lifecycle persistence
          → Phase 5 authority
            → Phase 6 templates/policy/evidence
              → Phase 7 Hermes/kits
                → Phase 8 release provenance
                  → Phase 9 approved live migration
                    → Phase 10 re-audit/certification
```

Parallel work is allowed only where files and contracts do not overlap:

- Phase 2 cancellation tests may run in parallel with store-transition tests after cursor schema is frozen.
- Phase 7 hook tests may run in parallel with kit installer tests after the daemon/proxy CLI contract is frozen.
- Documentation can trail implementation but cannot define behavior contrary to code.
- Authority, schema migration, terminal publication, and release installation are serialized owner-boundary work.

There is no evidence-backed wall-clock estimate. Track actual duration per completed task and update subsequent scheduling from measured data rather than inventing estimates.

---

## 7. Definition of done

This remediation is not done when code compiles, when 60 tests pass, when a daemon starts, or when the installed hash matches a local build. It is done only when:

1. AG-001 through AG-028 have evidence-backed dispositions;
2. all critical/high findings are verified closed or mechanically quarantined;
3. the installed daemon/proxy passes the process-boundary matrix;
4. live migration and rollback receipts exist;
5. operator authority is external to the model-facing tool surface;
6. every public capability label matches observed runtime behavior;
7. an independent hostile re-audit returns no open critical/high finding for the enabled capability tier.

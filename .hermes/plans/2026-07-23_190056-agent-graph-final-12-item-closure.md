# Agent Graph Final 12-Item Closure Plan

**Created:** 2026-07-23 19:00:56 UTC  
**Status:** controller-validated implementation plan  
**Canonical source baseline:** `/home/sikmindz/Coding/Libraries` at `3218a450257361ff0e84eab88297f0d977a55f1b`  
**Canonical Agent Graph kits baseline:** `/home/sikmindz/Coding/agent-graph-kits` at `30e27ec2decc89237fbd7ad394ada1cff81dcd10`  
**Predecessor plan:** `.hermes/plans/2026-07-23_100950-agent-graph-hostile-audit-remediation.md`  
**Independent re-audit:** `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260723_132435_950917.txt`  
**Purpose:** close the eleven unresolved audit findings plus one end-to-end migration/certification item without silently widening authority, inventing evidence, or treating source changes as installed-runtime proof.

---

## 1. Verdict and scope lock

The earlier remediation materially improved the engine and MCP implementation, but current source and runtime do **not** support a full-closure claim. The active stdio process still owns SQLite directly, `agent-graph-mcpd` is a placeholder, model-facing approval accepts caller-controlled actor metadata, template promotion is heuristic, release gates suppress failures, the kit deep smoke fails, the watchdog misidentifies process roles, and the hook allowlist contains stale/duplicate entries.

This follow-up closes exactly these twelve items:

| Item | Audit/phase surface | Current disposition | Closure proof required |
|---:|---|---|---|
| 1 | AG-001 + remaining Phase 4 lifecycle work | partially closed | daemon-owned startup recovery and atomic terminal publication proven across process restart |
| 2 | AG-002 | still open | OS-authenticated, action-scoped operator authority; model path cannot decide approvals |
| 3 | AG-003 | still open | exactly one durable/key-owning daemon; every model client is database-blind |
| 4 | AG-005 | still open | canonical-receipt-bound template promotion with authenticated operator decision |
| 5 | AG-007 | partially closed | clean source-to-artifact-to-installed provenance bundle, manifest, SBOM, and receipt digests |
| 6 | AG-008 | partially closed | schema-aware reversible kit install/reinstall/uninstall with conflict refusal |
| 7 | AG-010 | partially closed | MSRV/current toolchain gates and adjudicated dependency advisories |
| 8 | AG-012 | partially closed | semantic host-kit contract validation and real daemon/proxy deep smoke |
| 9 | AG-023 | still open | active daemon/proxy/systemd topology with bounded process behavior |
| 10 | AG-025 | partially closed | role-aware watchdog with live capability, artifact, service, socket, and store checks |
| 11 | AG-027 | partially closed | governed allowlist audit/apply/rollback with zero unexplained stale or duplicate entries |
| 12 | Phase 9 + Phase 10 closure | pending | dry-run, approved live migration, installed acceptance, rollback drill, closure ledger, independent hostile re-audit |

No other feature work is admitted. If implementation discovers an unrelated defect, record it in `deferred-findings.md`; do not silently widen this branch.

---

## 2. Current evidence snapshot

Observed on 2026-07-23 before writing this plan:

- `/home/sikmindz/Coding/Libraries` branch: `fix/hostile-remediation-20260715`.
- Baseline HEAD: `3218a450257361ff0e84eab88297f0d977a55f1b`.
- The canonical checkout has approximately 186 dirty paths; Agent Graph target scope currently includes a modified `Cargo.lock`. This checkout is **not** an admissible implementation or release workspace.
- Installed binary: `/home/sikmindz/.cargo/bin/agent-graph-mcp`.
- Installed binary SHA-256 at snapshot: `3a354373b1d4310f654ba5e7830d30d9c3f295a3988ed7e82b9a18d3bd641099`.
- Live Hermes config points directly to that binary and passes `--base-url`, `--model`, and `--data-dir`; the environment exposes `AGENT_GRAPH_INTEGRITY_KEY_PATH` to the stdio child.
- Live process shape is watchdog wrapper → direct `agent-graph-mcp` durable server, not proxy → daemon.
- `agent-graph-mcp/src/main.rs:61-77` acquires the owner lock, opens the durable server, and serves stdio.
- `agent-graph-mcp/src/bin/agent-graph-mcpd.rs:1-4` is a placeholder that exits with code 78.
- `agent-graph-mcp/src/server.rs:1706-1800` exposes `graph_approval_decide` and passes `claimed_actor_label` to the store.
- `agent-graph-mcp/src/auth.rs` declares capabilities but has no authenticated principal construction or production enforcement.
- `~/.hermes/scripts/agent_graph_ops.py` treats `operator:*` text as authority and promotes after three local `good` outcomes.
- `agent-graph-mcp/scripts/build-release.sh:14-15` hardcodes a deleted worktree; lines 60-67 suppress fmt, clippy, and audit failures.
- Agent Graph kits repo is clean at `30e27ec2decc89237fbd7ad394ada1cff81dcd10`.
- Required Python tests: 36 passed, but `shared/scripts/doctor.py --deep` failed with zero MCP responses.
- `agent_graph_watchdog.py` reports `multiple daemon owners` because it counts process text rather than typed roles.
- `audit_hook_allowlist.py` reports stale and duplicate entries but exits successfully and has no governed apply/rollback path.
- A controller-run `cargo audit` from the workspace root reports eight vulnerabilities and twenty-one allowed warnings. A separate planning lane ran `cargo audit --json` from `agent-graph-mcp/` and observed zero vulnerabilities. This disagreement is itself a provenance defect: the commands may have selected different lockfiles/build contexts. Neither result is admitted until G0 records every `Cargo.lock`, proves which lockfile the release build consumes, and binds the audit result to that exact digest. At minimum, the root-lock result reports `anyhow 1.0.102` affected by RUSTSEC-2026-0190 and patched in `>=1.0.103`. Other advisories must be proven target-reachable or target-unreachable, not waved away.

These observations outrank previous completion summaries. Earlier “Phase 9 complete” language is withdrawn: a binary replacement and permission change did not activate the planned daemon/proxy authority topology.

### 2.1 Independent planning-lane convergence

Three read-only lanes independently inspected the same baseline:

- Runtime/lifecycle: `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260723_134654_577544.txt`
- Authority/promotion: `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260723_134733_091070.txt`
- Release/kits/operations: `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260723_134659_231164.txt`

All three agree that the current runtime is `NO_GO / UNCERTIFIED_RUNTIME`, that the dirty canonical checkout is not a release source, and that source-present modules do not prove active topology or authority.

Controller resolutions of material lane observations:

1. **Accepted:** `daemon.rs` currently uses a create/delete sentinel lock that can survive abnormal death. Item 1 therefore requires replacement with the existing safe OS-held advisory-lock pattern; deleting a stale file is never ownership proof.
2. **Accepted:** durable `PersistentStore` construction is still reachable from ordinary server construction. Items 1/3 require a sealed daemon-owned construction capability, not convention or CLI-only enforcement.
3. **Accepted:** module-import tests are insufficient. Items 1/3/9 require spawned production-binary tests.
4. **Accepted:** root and nested audit results conflict. Item 7 treats this as an unresolved build-context problem and blocks release until lockfile selection is proved.
5. **Accepted:** installer mutation must be explicit and dry-run/plan-only by default.
6. **No dissent:** caller labels are not authentication, heuristic outcome counts are not promotion authority, and installed-artifact parity is currently unverified.

---

## 3. Canonical ownership and anti-shadow-truth rules

| Surface | Canonical owner | Derived/installed projection |
|---|---|---|
| Engine, MCP, daemon, proxy, operator protocol, SQLite migrations | `/home/sikmindz/Coding/Libraries/agent-graph*` | release binaries under `dist/`; installed binaries under `~/.local/libexec` or `/usr/libexec` |
| systemd and Polkit packaging templates | `agent-graph-mcp/packaging/` | `~/.config/systemd/user/`, `/usr/share/polkit-1/actions/`, `/usr/libexec/` |
| Host kits and Hermes operational scripts/tests | `/home/sikmindz/Coding/agent-graph-kits` | `~/.hermes/scripts/`, host project configurations |
| Durable run, receipt, approval, and promotion authority | daemon-owned Agent Graph SQLite | local reports/caches; never a second writable authority DB |
| Build/release truth | clean committed worktree + `Cargo.lock` + manifest/SBOM/receipt bundle | installed files verified against manifest digests |
| Hook allowlist | active `~/.hermes/shell-hooks-allowlist.json`, mutated only by governed audited transaction | audit reports and backups under evidence directory |
| Closure status | `agent-graph-mcp/docs/remediation/hostile-audit-closure-ledger.md` plus immutable evidence bundle | summaries generated from ledger; never the reverse |

`~/.hermes/scripts/agent_graph_ops.py` is currently an orphaned live script. Before modifying behavior, create the canonical version at `agent-graph-kits/hermes/scripts/agent_graph_ops.py`, move comprehensive tests into `agent-graph-kits/tests/test_agent_graph_ops.py`, and make installation digest-verifiable. Do not keep independent behavioral implementations in both places.

---

## 4. Non-negotiable invariants

1. **One durable owner:** only `agent-graph-mcpd` may open Agent Graph SQLite, load the integrity key, run migrations, recover runs, or publish terminal projections.
2. **Database-blind proxy:** `agent-graph-mcp` may only translate newline-delimited stdio JSON-RPC to bounded framed Unix-socket messages. It receives no data directory, key path, model credential, or migration authority.
3. **No silent direct fallback:** if the daemon socket is absent, the proxy returns typed `DAEMON_UNAVAILABLE` and exits nonzero. It must never open SQLite as a fallback.
4. **Transport-bound authority:** model traffic and operator traffic use distinct sockets and protocols. Supplying `actor`, `claimed_actor_label`, policy output, or a valid-looking receipt on the model socket grants no authority.
5. **OS-derived operator identity:** operator identity originates from verified peer credentials and Polkit/pkexec context, not request JSON.
6. **Action/resource scope:** each operator authorization is bound to one action, one resource, one canonical state digest, one nonce, and one expiry.
7. **Exactly once:** approval and promotion authorization nonces are consumed atomically with the state transition. Replays never relaunch or repromote.
8. **Canonical promotion:** outcome counts are evidence only. Only the daemon can move a template into `approved`, and only with canonical terminal receipt bindings plus authenticated operator authorization.
9. **Rebuildable projections:** Python reports and local operational caches can be regenerated from the daemon’s canonical records.
10. **Atomic lifecycle:** terminal state, terminal events, receipt, idempotency completion, and projection status commit in one SQLite transaction or remain visibly incomplete/volatile.
11. **Fail-closed release:** dirty source, missing tools, failing tests, failing policy, missing receipt, or hash mismatch aborts build/install. No `|| true`, interactive override, or fabricated empty result.
12. **Installed evidence:** source tests are not installed-runtime proof. Every closure claim names the exact installed artifact digest, config digest, service/unit digest, database migration version, and acceptance receipt.
13. **Reversible migration:** database, WAL/SHM, binaries, config, user units, Polkit policy/helper, kits, allowlist, and operational scripts have verified rollback artifacts before activation.
14. **No retroactive authority:** historical caller-label approvals and heuristic promotions become `legacy_unverified`/`quarantined`; they are never reclassified as authenticated.
15. **Least privilege:** no TCP operator listener, no model access to admin sockets, no secrets in process arguments/manifests, private runtime directories, strict socket modes, and no `unsafe` code.

---

## 5. Execution environment and branch discipline

### 5.1 Create isolated worktrees

Do not implement in the dirty canonical checkout.

```bash
cd /home/sikmindz/Coding/Libraries
test "$(git rev-parse HEAD)" = "3218a450257361ff0e84eab88297f0d977a55f1b" || exit 42
git worktree add -b fix/agent-graph-final-closure-20260723 \
  /home/sikmindz/Coding/worktrees/agent-graph-final-closure \
  3218a450257361ff0e84eab88297f0d977a55f1b

cd /home/sikmindz/Coding/agent-graph-kits
test "$(git rev-parse HEAD)" = "30e27ec2decc89237fbd7ad394ada1cff81dcd10" || exit 43
git worktree add -b fix/agent-graph-kits-final-closure-20260723 \
  /home/sikmindz/Coding/worktrees/agent-graph-kits-final-closure \
  30e27ec2decc89237fbd7ad394ada1cff81dcd10
```

If either HEAD changed, stop and amend this plan’s baseline before creating branches. Never stash, reset, clean, or overwrite the dirty canonical checkout.

### 5.2 Evidence root

Each run creates:

```text
/home/sikmindz/Coding/Libraries/.hermes/evidence/agent-graph-final-closure/<UTC_TIMESTAMP>/
```

Required root files:

- `baseline.json`
- `commands.jsonl`
- `source-status.txt`
- `kits-status.txt`
- `installed-processes.txt`
- `installed-hashes.json`
- `config-redacted.json`
- `db-schema-summary.json`
- `finding-matrix.json`
- `deferred-findings.md`
- `rollback-manifest.json`
- `final-certification.json`

Every command receipt records exact argv, cwd, UTC start/end, exit code, stdout/stderr digests, and output path. Never store keys, tokens, raw sensitive prompts, or full environment dumps.

### 5.3 Commit boundaries

Use one reviewable commit per closure boundary after its focused tests pass:

- R1: daemon transport/lifecycle foundation — Items 1, 3, 9
- R2: authenticated operator approval — Item 2
- R3: canonical promotion — Item 4
- R4: release provenance and dependency policy — Items 5, 7
- K1: reversible kit installer and semantic validation — Items 6, 8
- K2: watchdog, allowlist, and canonical Hermes operational scripts — Items 10, 11
- R5/K3: migration packaging, certification harness, and ledger — Item 12

Do not squash away review boundaries until the independent audit has cited them.

---

# Item 1 — AG-001 and remaining Phase 4: owner-bound startup recovery and terminal publication

## Objective

Make startup recovery a responsibility of the sole daemon generation and complete the existing durable terminal publication path. Arbitrary stdio client startup must never recover, reclassify, or resume global state.

## Files

**Modify:**

- `agent-graph-mcp/src/daemon.rs`
- `agent-graph-mcp/src/lifecycle.rs`
- `agent-graph-mcp/src/migrations.rs`
- `agent-graph-mcp/src/store.rs`
- `agent-graph-mcp/src/server.rs`
- `agent-graph-mcp/src/run_manager.rs`
- `agent-graph-mcp/src/owner_lock.rs`
- `agent-graph-mcp/src/lib.rs`

**Create:**

- `agent-graph-mcp/tests/daemon_recovery.rs`
- `agent-graph-mcp/tests/terminal_publication_process_boundary.rs`
- `agent-graph-mcp/tests/fixtures/legacy-lifecycle-v1.sql`

## RED first

Add tests that currently fail:

1. Starting two daemon generations against the same copied database admits one owner and rejects the other before migration/recovery.
2. Starting any number of proxies does not change run status, recovery count, schema version, or receipt count.
3. A daemon crash between engine completion and terminal publication leaves a typed recoverable `publication_pending` state, never a false terminal result.
4. Restart publishes a pending deterministic terminal bundle exactly once.
5. Legacy/non-deterministic/effectful interrupted runs become `legacy_unverified` or `manual_review_effects_unknown`; they are not resumed.
6. A transaction fault after terminal events but before run/receipt publication rolls back all terminal projection rows.
7. Existing `persist_terminal_projection` tests are exercised through an actual daemon process, not only direct store calls.

Focused RED command:

```bash
cargo test -p agent-graph-mcp --test daemon_recovery --test terminal_publication_process_boundary
```

Expected before implementation: compile/test failure because daemon generation/recovery and process-boundary helpers do not exist.

## Minimal implementation

1. Replace `daemon.rs` create/delete sentinel ownership with the safe OS-held exclusive lock implementation used by `owner_lock.rs` (or consolidate to one implementation). The lock must be released by process death; a leftover diagnostic path must never block or authorize startup. No unsafe libc calls.
2. Add `daemon_instances` and `run_publication_state` migrations with explicit schema versioning.
3. Generate a daemon `instance_id` and monotonically increasing `generation` after exclusive owner-lock acquisition.
4. Persist `boot_id`, PID, executable digest, source/build manifest digest, start timestamp, heartbeat timestamp, and clean-shutdown timestamp. Treat PID as diagnostic only; authority comes from lock + generation.
5. Move all startup classification into `daemon::recover_owned_state(instance_id, generation)`.
6. Classify each nonterminal run using existing lifecycle semantics and executable node class:
   - deterministic/local and complete checkpoint → eligible for explicit resume policy;
   - terminal bundle pending → republish idempotently;
   - effectful/unknown/legacy → `manual_review_effects_unknown`;
   - invalid/missing checkpoint → `recovery_blocked` with typed reason.
7. Wire existing `PersistentStore::persist_terminal_projection` into the daemon-owned completion path so run terminal state, events, receipt, idempotency completion, and publication marker are one transaction.
8. Record old daemon generation and new daemon generation in recovery events.
9. On graceful shutdown, stop accepting connections, cancel admitted work according to policy, drain terminal publication, checkpoint remaining eligible runs, write clean-shutdown state, then release the owner lock.
10. No startup scan may run in proxy `main` or per-client server construction.
11. Add an abnormal-exit test: SIGKILL daemon A, then prove daemon B acquires the OS-held lock without manual lock-file deletion and recovers only generation-A rows.

## GREEN and integration gates

```bash
cargo fmt --check -p agent-graph -p agent-graph-mcp
cargo clippy -p agent-graph-mcp --all-targets -- -D warnings
cargo test -p agent-graph-mcp --test daemon_recovery --test terminal_publication_process_boundary
cargo test -p agent-graph-mcp --test lifecycle --test terminal_projection --test process_boundary
```

## Acceptance

- One and only one process owns the lock/database/key.
- Restart classification is deterministic and emits durable events.
- Terminal publication is atomic and idempotent across crash/restart.
- No effectful/unknown legacy run resumes automatically.
- A proxy start/stop produces zero lifecycle mutations.

## Evidence, rollback, claim

- Evidence: `item-01/daemon-recovery/` containing copied DB fixtures, before/after table summaries, process logs, and command receipts.
- Rollback: migration down is not run against live state. Restore the pre-migration online backup and prior binary set; keep the new database quarantined for diagnosis.
- Claim after focused tests: “daemon-owned recovery and terminal publication passed local process-boundary tests.” Do not claim installed closure until Item 12 passes.

---

# Item 2 — AG-002: authenticated operator approval and administrative authority

## Objective

Remove privileged mutations from model authority and establish a separate OS-authenticated operator path terminating at the sole daemon.

## Authority contract

- Model clients may request/list/get approvals, but may not decide them.
- Model clients may not delete durable graphs, migrate storage, install configuration, or promote templates.
- `Principal::LocalOperator` is constructible only from verified operator transport context.
- Polkit/pkexec authenticates the user. The daemon verifies the helper through Unix peer credentials and creates the canonical decision receipt.
- The helper never opens SQLite and never supplies a free-form actor label.

## Files

**Modify:**

- `agent-graph-mcp/src/auth.rs`
- `agent-graph-mcp/src/tools.rs`
- `agent-graph-mcp/src/server.rs`
- `agent-graph-mcp/src/store.rs`
- `agent-graph-mcp/src/migrations.rs`
- `agent-graph-mcp/src/error.rs`
- `agent-graph-mcp/src/lib.rs`
- `agent-graph-mcp/tests/mcp_integration.rs`

**Create:**

- `agent-graph-mcp/src/operator_auth.rs`
- `agent-graph-mcp/src/operator_ipc.rs`
- `agent-graph-mcp/src/bin/agent-graph-operator.rs`
- `agent-graph-mcp/src/bin/agent-graph-operator-helper.rs`
- `agent-graph-mcp/tests/operator_authority.rs`
- `agent-graph-mcp/tests/operator_process_boundary.rs`
- `agent-graph-mcp/packaging/polkit/io.recursiveintell.agentgraph.policy`
- `agent-graph-mcp/packaging/polkit/10-agent-graph.rules` only if policy testing proves a rule is required; otherwise do not add it.

## Protocol

Version the admin frame as `agent_graph.operator.v1`. Reject unknown fields and versions. Request fields:

```text
request_id, action, resource_kind, resource_id, expected_state_digest,
nonce, issued_at, expires_at, decision_material
```

Authenticated context is not accepted from JSON. The daemon derives:

```text
peer_uid, peer_gid, pkexec_uid, daemon_instance_id, daemon_generation
```

The helper runs as root through `pkexec`, forwards the authenticated original UID from trusted `PKEXEC_UID`, and connects only to the private operator socket. The daemon requires root peer credentials on the operator socket and requires `pkexec_uid` to equal the daemon owner UID. Root is in the trust boundary; arbitrary same-user processes are not.

Operator receipt fields:

```text
receipt_version, receipt_id, request_digest, action, resource_kind,
resource_id, expected_state_digest, operator_uid, daemon_instance_id,
daemon_generation, nonce, issued_at, expires_at, consumed_at, result_digest
```

No bearer receipt is handed back to the model and later replayed. The daemon creates and atomically consumes it while applying the privileged transition.

## RED first

1. Call `graph_approval_decide` with `operator`, `operator:alice`, `root`, and a forged receipt. Expect `AUTHENTICATED_OPERATOR_REQUIRED`, no decision, and no resumed run.
2. Verify privileged tools are absent from `tools/list` on the model socket.
3. Send an operator frame through the model socket. Expect `WRONG_AUTHORITY_TRANSPORT`.
4. Connect to the operator socket as the daemon owner UID rather than root helper. Expect rejection before store mutation.
5. Exercise missing/expired/future-dated/reused nonce; wrong action/resource/state digest; oversized frame; extra fields; malformed UTF-8/JSON. All fail closed.
6. Race two valid decisions for one approval. Exactly one transition and resumed run occurs.
7. Create an approval, restart daemon, decide through the helper test harness, and verify one durable decision/launch.

RED command:

```bash
cargo test -p agent-graph-mcp --test operator_authority --test operator_process_boundary
```

## Minimal implementation

1. Split model tool registration from admin protocol handling. Remove `graph_approval_decide` and destructive/admin tools from model `tools/list`; for one compatibility release, direct calls return stable `AUTHENTICATED_OPERATOR_REQUIRED` without mutation.
2. Replace `claimed_actor_label` as authority. Preserve it only in legacy read models as untrusted metadata.
3. Use Tokio Unix peer credentials; do not add unsafe libc calls.
4. Bind the operator action to canonical approval/checkpoint/state digests read by the daemon.
5. Add transactional tables for authorization receipts/nonces and decision linkage.
6. Mark existing label-derived decisions `legacy_unverified`; never upgrade them in place.
7. Add bounded operator CLI UX:
   - read canonical approval metadata/digest;
   - show resource/action/expiry;
   - require explicit human decision;
   - invoke `pkexec` helper;
   - emit receipt ID/digest only, never key material.
8. Ensure cancellation, denial, helper failure, or Polkit failure leaves the approval pending.

## Gates

```bash
cargo fmt --check -p agent-graph-mcp
cargo clippy -p agent-graph-mcp --all-targets -- -D warnings
cargo test -p agent-graph-mcp --test operator_authority --test operator_process_boundary
cargo test -p agent-graph-mcp --test mcp_integration approval
```

Add a packaging test using a fake Polkit/pkexec boundary. Real Polkit activation is deferred to Item 12 and requires explicit user approval.

## Acceptance

- Privileged mutations are absent from model `tools/list` and direct forged calls cannot mutate state.
- The operator path derives identity from peer/Polkit context, binds action/resource/state/nonce/expiry, and consumes authorization exactly once.
- Restart and concurrency tests prove one durable decision and one resumed run.

## Rollback/quarantine/claim

- Until live operator verification succeeds, effectful approval remains unavailable; do not fall back to actor strings.
- Rollback removes helper/policy and leaves privileged routes disabled. Restoring the old model-decision path is forbidden.
- Claim only “model approval decisions are blocked and the authenticated operator path passed local tests” until installed Polkit and peer-credential tests pass.

---

# Item 3 — AG-003: eliminate mixed keyless/direct durable processes

## Objective

Ensure only the daemon receives storage/model/key configuration. A proxy has no code path or environment contract capable of becoming a durable owner.

## Files

- Modify `agent-graph-mcp/src/cli.rs`, `main.rs`, `proxy.rs`, `fs_security.rs`, `owner_lock.rs`.
- Create `agent-graph-mcp/src/daemon_cli.rs`, `agent-graph-mcp/tests/proxy_confinement.rs`, and `agent-graph-mcp/tests/process_ownership.rs`.
- Modify kit launchers only in Items 6/8 after the binary contract stabilizes.

## RED first

1. `agent-graph-mcp --data-dir …`, `--integrity-key …`, `--base-url …`, or `--model …` fails with `LEGACY_DIRECT_DURABLE_UNSUPPORTED`.
2. Proxy environment containing data/key variables is scrubbed and never forwarded or read.
3. Proxy cannot open the database, WAL, SHM, owner lock, or key; verify via `/proc/<pid>/fd` in Linux process tests.
4. Twenty concurrent proxies connect to one daemon without extra owners or migrations.
5. Missing/foreign/symlink/permissive socket returns typed failure; no local DB fallback.
6. A proxy built from a different protocol major is rejected before forwarding requests.

## Implementation

- `agent-graph-mcp` accepts only `--socket`, `--connect-timeout-ms`, and diagnostic `--version/--help`.
- Default socket: `$XDG_RUNTIME_DIR/agent-graph/mcp.sock`; resolve once, reject symlinks, require daemon-owner UID and mode 0600.
- Remove durable arguments and key/data/model environment reads from proxy startup.
- Make durable store constructors crate-private and require a sealed `DaemonStoreAuthority`/owned-state token whose private field can be created only after daemon lock acquisition. `AgentGraphServer::new` must no longer accept a data directory; daemon construction receives an already-owned durable state handle.
- `agent-graph-mcpd` exclusively accepts `--socket`, `--operator-socket`, `--data-dir`, `--integrity-key`, `--base-url`, `--model`, and bounded operational settings.
- No hidden compatibility switch may turn the proxy into a durable server. If an ephemeral in-memory server is retained for tests, compile it only into test helpers or a separately named development binary; never the installed proxy.

## Gates

```bash
cargo test -p agent-graph-mcp --test proxy_confinement --test process_ownership
cargo test -p agent-graph-mcp --test cli_integration --test process_boundary
```

## Acceptance

Only the daemon holds file descriptors for DB/WAL/SHM/key/owner lock. Proxy count may vary; durable owner count remains exactly one. Model traffic cannot choose keyless versus keyed durable operation.

---

# Item 4 — AG-005: canonical, operator-authorized template promotion

## Objective

Replace local heuristic promotion with a daemon-owned canonical candidate/outcome/promotion state machine. Outcome collection is advisory; approval is an authenticated operator action.

## Files

**Rust:**

- Modify `agent-graph-mcp/src/store.rs`, `migrations.rs`, `evidence.rs`, `operator_auth.rs`, `operator_ipc.rs`, and operator binaries.
- Create `agent-graph-mcp/src/promotion.rs`.
- Create `agent-graph-mcp/tests/template_promotion.rs` and `template_promotion_process_boundary.rs`.

**Kits/Hermes:**

- Create canonical `agent-graph-kits/hermes/scripts/agent_graph_ops.py`.
- Create `agent-graph-kits/tests/test_agent_graph_ops.py`.
- Modify active `~/.hermes/scripts/agent_graph_ops.py` only during Item 12 installation.
- Retire `~/.hermes/agent-graph-ops/agent-graph-ops.db` as authority; import it only as quarantined legacy evidence.

## Canonical schema

Versioned daemon tables must represent:

- `template_candidates(template_id, spec_digest, graph_id, graph_version, source_ref, state, created_at, updated_at)`
- `template_outcome_links(template_id, run_id, terminal_receipt_id, receipt_digest, disposition, evidence_digest, recorded_at)` with unique `(template_id, run_id)`
- `template_promotion_decisions(template_id, from_state, to_state, evidence_set_digest, operator_receipt_id, decision_digest, decided_at)`

Foreign keys bind every outcome to a canonical run/receipt. An outcome is eligible only if it binds exact `run_id`, `graph_id`, `graph_version`, `template_id`, `spec_digest`, terminal state, and integrity disposition. Distinct receipts from one run count once.

State machine:

```text
legacy_unverified -> quarantined
candidate -> candidate | quarantined | approved
approved -> approved | quarantined | revoked
quarantined -> candidate only through new authenticated re-admission
revoked -> terminal
```

Three good outcomes and no bad outcomes make a candidate **eligible**, not approved. Promotion requires an authenticated `promote_template` operator action bound to the evidence-set digest. A contradictory/bad/revoked/unverifiable outcome blocks or quarantines promotion.

## RED first

- `operator:fake` text cannot authorize.
- A row forged in the local Python DB cannot become canonical.
- A real receipt with wrong run/graph/version/template/spec digest is rejected.
- Three good outcomes without operator decision leave state `candidate`.
- Duplicate run IDs do not increase eligibility count.
- Nonterminal/cancelled/failed/legacy-unverified receipts do not qualify unless an explicit versioned policy admits a specific failure case.
- One bad outcome quarantines; concurrent promote/quarantine has one deterministic winner.
- Reused operator nonce cannot promote a second template.
- Delete/rebuild of Python projection cannot change canonical state.
- Daemon restart preserves candidate, evidence, and decision.

## Implementation sequence

1. Add RED Rust/store/process tests.
2. Add canonical schema and typed state machine.
3. Add read-only model tools for candidate/status/eligible evidence only; no promotion mutation.
4. Add admin `promote_template` action through Item 2 boundary.
5. Convert Python `record-outcome` to submit evidence only.
6. Add explicit Python `promote-template` command that invokes the operator CLI; it never writes approval state itself.
7. Add `rebuild-projection` from daemon export and verify deterministic digest.
8. Import historical local rows as `legacy_unverified`/`quarantined` with source digest; never auto-approve them.

## Gates

```bash
cargo test -p agent-graph-mcp --test template_promotion --test template_promotion_process_boundary
python3 -m pytest /home/sikmindz/Coding/worktrees/agent-graph-kits-final-closure/tests/test_agent_graph_ops.py -v
```

## Acceptance

No local row, outcome count, caller label, mismatched receipt, duplicate run, or replay can approve a template. One authenticated operator action over a fully bound canonical evidence set produces one durable promotion decision.

## Rollback/claim

Rollback disables promotion and restores candidate/quarantine read state; it must never restore heuristic auto-promotion. Claim canonical promotion only after live operator and receipt-binding acceptance in Item 12.

---

# Item 5 — AG-007: complete source/artifact/installed provenance

## Objective

Produce a deterministic, machine-validated release bundle that binds clean source and lockfile to every built artifact, test/policy receipt, SBOM, packaging file, installed file, config, and service unit.

## Files

**Modify:**

- `agent-graph-mcp/scripts/build-release.sh`
- `agent-graph-mcp/docs/release/manifest-schema.md`
- `agent-graph-mcp/Cargo.toml`

**Create:**

- `agent-graph-mcp/docs/release/manifest-v2.schema.json`
- `agent-graph-mcp/scripts/validate-release.py`
- `agent-graph-mcp/scripts/install-release.sh`
- `agent-graph-mcp/scripts/release-toolchain.lock.json`
- `agent-graph-mcp/tests/release_manifest.rs`
- `agent-graph-mcp/tests/test_release_scripts.py`

## RED first

Release must fail when any of the following is true:

- source or `Cargo.lock` is dirty;
- HEAD/lockfile changes during build;
- expected tool is absent or unpinned;
- fmt/clippy/test/MSRV/advisory/SBOM/schema gate fails;
- daemon/proxy/operator/helper artifact is missing;
- receipt path exists but its digest/result is absent or non-pass;
- manifest has unknown/missing fields;
- installed binary/unit/policy/config digest differs;
- release path records an ephemeral/deleted worktree as canonical source;
- manifest includes secret-like environment values.

## Build contract

1. Resolve repository root dynamically with `git rev-parse --show-toplevel`; remove the hardcoded worktree path.
2. Require a clean full source tree for admitted paths including root `Cargo.toml`, `Cargo.lock`, `agent-graph`, and `agent-graph-mcp`. No prompt/override.
3. Use `cargo build --release --locked` for proxy, daemon, operator client, and helper.
4. Record exact tool versions from a committed lock file; never invoke unpinned “latest.”
5. Run gates without `|| true`. Capture each gate via a receipt wrapper that preserves the real exit code.
6. Generate CycloneDX JSON and package-specific dependency metadata. Validate SBOM schema and bind its SHA-256.
7. Emit artifacts under:

```text
dist/agent-graph-mcp/<crate-version>/<git-commit>/<target>/
```

8. Manifest v2 binds:
   - commit, branch, full dirty status, source tree digest, Cargo.lock digest;
   - rustc/cargo/target/profile/features;
   - artifact paths, sizes, modes, SHA-256;
   - unit/policy/config-template digests;
   - migration/schema/protocol versions;
   - test/gate command, exit code, output digest;
   - SBOM and advisory-policy digest;
   - build start/end and source date epoch.
9. The install script consumes an immutable bundle, verifies schema/digests before copying, stages every file, atomically activates, verifies installed digests, and emits an install receipt. It does not rebuild.

## Gates

```bash
python3 -m pytest agent-graph-mcp/tests/test_release_scripts.py -v
cargo test -p agent-graph-mcp --test release_manifest
agent-graph-mcp/scripts/build-release.sh
python3 agent-graph-mcp/scripts/validate-release.py dist/.../build-manifest.json --verify-tree --verify-receipts
```

## Acceptance

The bundle can be moved to a clean verification directory and independently validated without access to mutable build outputs. Installed identity remains `unverified` until install receipt and post-install checks match the same manifest.

---

# Item 6 — AG-008: reversible, conflict-safe host-kit installation

## Objective

Make install/reinstall/uninstall preserve unrelated configuration, refuse malformed or concurrently modified state, and provide digest-backed rollback.

## Files

- Modify `agent-graph-kits/shared/scripts/setup-host.py`.
- Modify host configuration templates and launchers only to the proxy contract.
- Create `agent-graph-kits/tests/test_setup_host.py` and fixture directories for all nine hosts.
- Create `agent-graph-kits/shared/schemas/install-state-v1.schema.json`.

## RED first

For each host (`claude`, `codex`, `hermes`, `cursor`, `windsurf`, `cline`, `roo-code`, `continue`, `opencode`):

1. Preserve unrelated keys and rules before **and after** the managed marker.
2. Fail on malformed JSON/YAML/TOML; never replace it with `{}`.
3. Reject symlink targets, foreign ownership, and parent traversal.
4. Preserve mode/owner where permitted; use private mode for new sensitive config.
5. Reinstall is idempotent and changes no digest.
6. If current digest differs from recorded installed digest, uninstall refuses rather than deleting user edits.
7. Backup → install → uninstall restores byte-identical original content and mode.
8. A fault before rename leaves original intact; a fault after rename is recoverable from state manifest.
9. Dry-run emits an exact change plan and performs no writes.

## Implementation

- Parse each host’s real schema and merge only the Agent Graph-owned key/marker region.
- Use a versioned managed block with one start and one end marker; reject nested/duplicate/unbalanced markers.
- Write through a same-directory temporary file, fsync file, atomic replace, then fsync parent directory.
- Store backup and install-state manifests beneath `$XDG_STATE_HOME/agent-graph-kits/<workspace-hash>/<host>/<timestamp>/`.
- State manifest binds canonical workspace path, target paths, pre/post digests, modes, ownership, kit commit, and operation ID.
- Add subcommands: `plan`, `install`, `reinstall`, `uninstall`, `rollback`, `status`.
- Default invocation is plan/dry-run only. Every mutation requires an explicit `--apply` plus a precondition digest; no interactive prompt or implicit write mode.
- Never auto-prune unrelated host entries.

## Gates

```bash
python3 -m pytest tests/test_setup_host.py -v
python3 scripts/validate-all-kits.py
```

## Acceptance

All nine host fixtures pass byte-preserving install/reinstall/uninstall and injected-failure rollback. Malformed, symlinked, foreign, or concurrently modified targets are refused without mutation.

## Rollback/claim

Rollback requires digest match or explicit operator selection of a recorded backup. Claim only the nine host/schema combinations actually tested.

---

# Item 7 — AG-010: MSRV, clippy, and dependency-advisory closure

## Objective

Make release-quality checks fail closed while distinguishing vulnerabilities reachable from shipped Agent Graph artifacts from unrelated workspace lockfile entries through reproducible evidence.

## Files

- Modify root `Cargo.lock` only through admitted dependency updates in the isolated worktree.
- Modify relevant `Cargo.toml` constraints where required.
- Create `agent-graph-mcp/deny.toml`.
- Create `agent-graph-mcp/docs/release/advisory-adjudication.json`.
- Create `agent-graph-mcp/scripts/validate-advisories.py` and tests.
- Modify release toolchain lock and manifest from Item 5.

## Precondition: lockfile ownership decision

The canonical checkout’s root `Cargo.lock` is dirty, and root-vs-subdirectory audit commands produced contradictory results. In the isolated worktree:

1. enumerate every `Cargo.lock` under the workspace and record path/digest;
2. prove which manifest/workspace/lockfile each build, `cargo metadata`, `cargo audit`, `cargo deny`, and SBOM command consumes;
3. record the committed root-lock digest and any nested-lock digest;
4. run `cargo metadata --locked` from the exact release root;
5. compare the dirty canonical lock only as external evidence;
6. do not copy or merge it unless every changed package has a named reason and test;
7. reject an audit receipt whose recorded lockfile digest differs from the release manifest;
8. commit the final authoritative lockfile with the dependency-policy commit.

## Advisory procedure

For every cargo-audit advisory/warning:

1. capture advisory ID, affected package/version, patched versions, severity/type;
2. run package inverse trees for proxy, daemon, operator client, and helper with all targets/features;
3. classify `reachable`, `build-only`, `target-unreachable`, or `unknown`;
4. update every reachable vulnerable package to a patched version, beginning with `anyhow >=1.0.103`;
5. rerun MSRV/current builds and tests;
6. a waiver is allowed only for target-unreachable or informational unmaintained entries and must include owner, reason, reachability command/output digest, created date, expiry ≤90 days, and tracking issue;
7. `unknown` blocks release;
8. a high/critical reachable vulnerability cannot be waived in this plan.

Known snapshot advisories to adjudicate include RUSTSEC-2026-0204 (`crossbeam-epoch`), 2026-0194/0195 (`quick-xml`), 2026-0185 (`quinn-proto`), 2026-0049/0098/0099/0104 (`rustls-webpki`), 2026-0190 (`anyhow`), and the reported `rand` advisory. Re-query at implementation time; this list is not a frozen advisory database.

## RED/GREEN gates

```bash
cargo +1.75.0 check --locked -p agent-graph -p agent-graph-mcp --all-targets
cargo +1.75.0 test --locked -p agent-graph --tests
cargo +1.75.0 test --locked -p agent-graph-mcp --no-fail-fast
cargo +stable fmt --check -p agent-graph -p agent-graph-mcp
cargo +stable clippy --locked -p agent-graph -p agent-graph-mcp --all-targets -- -D warnings
cargo +stable test --locked -p agent-graph --tests
cargo +stable test --locked -p agent-graph-mcp --no-fail-fast
cargo audit --json > <evidence>/cargo-audit.json
cargo deny --manifest-path agent-graph-mcp/Cargo.toml --config agent-graph-mcp/deny.toml check
python3 agent-graph-mcp/scripts/validate-advisories.py --audit <...> --sbom <...> --adjudication <...>
```

The wrapper must capture a nonzero raw `cargo audit` result honestly. A release may receive `qualified_pass` only when the validator proves all remaining entries target-unreachable/informational under unexpired reviewed waivers. Public language must say “Agent Graph artifact dependency policy passed with N documented workspace waivers,” not “workspace audit clean.”

## Acceptance

- Rust 1.75 and current-stable checks/tests pass against the authoritative locked graph.
- Every advisory is linked to the exact lockfile and shipped-artifact reachability result.
- No critical/high target-reachable vulnerability remains, no unknown classification remains, and every exception is explicit, reviewed, and unexpired.

---

# Item 8 — AG-012: semantic host-kit contract validation

## Objective

Replace file-exists validation with host-specific config semantics and real MCP transport tests against a disposable daemon/proxy topology.

## Files

- Modify `agent-graph-kits/scripts/validate-all-kits.py`.
- Modify `agent-graph-kits/shared/scripts/doctor.py`.
- Modify `agent-graph-kits/shared/scripts/run-server.sh` and all nine host launchers to invoke proxy only.
- Create `agent-graph-kits/tests/test_validate_all_kits.py`, `test_doctor.py`, `test_launchers.py`, and host fixtures.

## RED first

- Wrong root key (`mcpServers` vs host-specific equivalent) fails.
- Wrong command/args/env shape fails.
- Launcher containing data-dir/key/model flags fails.
- Launcher output pollution fails.
- Daemon absent yields typed unavailable result, not success.
- Disposable daemon + proxy initialize and tools/list must pass for all nine generated host configurations.
- Tool list must contain required model-safe tools and exclude approval decision, promotion, delete, migration, and install tools.
- Two concurrent host clients share one daemon; only daemon owns DB/key.
- Host install/uninstall round-trip remains byte-identical.

## Implementation

- Encode host schemas and required command contract explicitly.
- Validate resolved absolute paths, executable identity, socket location, and absence of unresolved variables.
- `doctor.py --deep` must create a temporary private runtime/data directory, start a disposable daemon, wait for readiness with a deadline, run proxy initialize/tools-list/status, terminate cleanly, and emit a JSON receipt. It must not collide with or inspect the live DB.
- Always report skipped/unavailable checks as such; never convert them to PASS.

## Gates

```bash
python3 -m pytest tests/test_validate_all_kits.py tests/test_doctor.py tests/test_launchers.py -v
python3 scripts/validate-all-kits.py
python3 shared/scripts/doctor.py --deep --json-receipt <evidence>/kit-doctor.json
```

## Acceptance

Each host configuration passes schema, exact argv/environment, proxy-only capability, install round-trip, and live initialize/tools-list checks against a disposable daemon. The validator reports unavailable/skipped work without calling it validated.

---

# Item 9 — AG-023: activate the daemon/proxy/systemd runtime topology

## Objective

Ship and activate a bounded single-daemon topology rather than merely compiling daemon/proxy modules.

## Transport and paths

- Model socket: `$XDG_RUNTIME_DIR/agent-graph/mcp.sock`
- Operator socket: `$XDG_RUNTIME_DIR/agent-graph/operator.sock`
- Runtime directory mode: 0700
- Socket mode: 0600
- Model framing: 4-byte big-endian length followed by UTF-8 JSON payload; maximum frame size fixed and tested.
- Stdio side: one JSON-RPC object per line, no stdout logging.
- Operator protocol: distinct schema/version and socket; never routed through model framing.

## Files

- Complete `agent-graph-mcp/src/daemon.rs`, `proxy.rs`, `src/bin/agent-graph-mcpd.rs`, and `main.rs`.
- Create `agent-graph-mcp/src/transport.rs` and `tests/daemon_proxy_process_boundary.rs`.
- Create packaging:
  - `agent-graph-mcp/packaging/systemd/agent-graph-mcpd.service`
  - `agent-graph-mcp/packaging/systemd/agent-graph-mcpd.env.example`
  - `agent-graph-mcp/packaging/systemd/README.md`

## Service contract

Use a user service with `Type=simple`; socket readiness is proven by a bounded health probe, not assumed from process existence. Required hardening where supported:

```text
UMask=0077
Restart=on-failure
RestartSec=2s
RuntimeDirectory=agent-graph
RuntimeDirectoryMode=0700
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=%h/.agent-graph %t/agent-graph
LockPersonality=true
RestrictSUIDSGID=true
```

`ExecStart` invokes the exact installed daemon artifact, not a shell. Configuration contains paths and nonsecret model endpoint settings; secrets remain in private files and are not placed on argv.

## RED first

1. Daemon handles graceful SIGTERM and removes only sockets it owns.
2. Stale socket with no live owner is quarantined/replaced; live foreign socket is rejected.
3. Oversized/malformed frame kills only the client connection.
4. Slow client backpressure is bounded; connection and global limits are enforced.
5. Killing a proxy during a long run does not cancel daemon-owned run unless an explicit cancel request was accepted.
6. Killing/restarting daemon gives proxies typed disconnect/completion-unknown; no fabricated completion.
7. 100 short-lived proxies over a test duration leave one daemon, bounded FDs/tasks, and no zombie processes.
8. Service restart preserves durable state and follows Item 1 lifecycle policy.

## Gates

```bash
cargo test -p agent-graph-mcp --test daemon_proxy_process_boundary --test process_ownership
systemd-analyze --user verify agent-graph-mcp/packaging/systemd/agent-graph-mcpd.service
```

Run systemd activation only in Item 12 after explicit approval.

## Acceptance

Spawned production binaries prove one OS-released durable owner, secure model/operator sockets, bounded proxy churn, clean shutdown/restart, no proxy-owned persistence, and no silent fallback.

---

# Item 10 — AG-025: role-aware watchdog and capability parity

## Objective

Report health from authoritative process/service/socket/capability/artifact evidence rather than substring process counts or direct optimistic probes.

## Files

- Canonicalize into `agent-graph-kits/hermes/scripts/agent_graph_watchdog.py`.
- Create `agent-graph-kits/tests/test_agent_graph_watchdog.py`.
- Modify installed `~/.hermes/scripts/agent_graph_watchdog.py` only in Item 12.
- Add/read daemon status fields through model-safe `graph_system_status` or equivalent.

## Typed result

```text
ok | degraded | unavailable | misconfigured | uncertified
```

Each check returns `status`, `code`, `observed`, `expected`, `authority`, and `checked_at`. Unknown is never OK.

## Required checks

1. systemd unit load/active state and exact MainPID;
2. MainPID executable digest equals installed manifest daemon digest;
3. exactly one daemon owner generation;
4. model/operator socket owner, type, path, and mode;
5. live handshake and protocol version through proxy;
6. actual model tool set equals manifest capability contract and excludes privileged tools;
7. daemon reports expected schema/migration version, integrity-key loaded state, recovery status, and last durable integrity verification;
8. config, unit, policy, proxy, daemon, helper, and manifest digests match install receipt;
9. proxy count is bounded/diagnostic but not confused with owner count;
10. DB/key directory and file modes are private;
11. allowlist audit and kit doctor receipts are fresh and passing;
12. no direct DB owner outside MainPID, verified via `/proc` on Linux.

Do not run an uncoordinated full SQLite integrity scan from the watchdog against the live DB. The daemon owns coordinated integrity verification and reports its timestamp/result.

## RED/GREEN

Fixtures cover missing service, stale PID, same-name unrelated process, two daemons, many valid proxies, wrong executable hash, permissive socket, wrong capabilities, stale manifest, daemon unavailable, and probe timeout.

```bash
python3 -m pytest tests/test_agent_graph_watchdog.py -v
python3 hermes/scripts/agent_graph_watchdog.py --json
```

Expected live result before Item 12 activation: `uncertified` or `degraded`, never false OK.

## Acceptance

Every fixture yields the documented typed state, one snapshot drives both JSON and exit status, and `ok` is possible only when service, process role, socket, capability, artifact, config, migration, key, and durable-integrity evidence all agree.

---

# Item 11 — AG-027: governed hook allowlist cleanup and drift prevention

## Objective

Eliminate unexplained stale/duplicate allowlist entries through a reviewed atomic transaction and make future drift a failing health signal.

## Files

- Canonicalize `audit_hook_allowlist.py` into `agent-graph-kits/hermes/scripts/audit_hook_allowlist.py`.
- Create `agent-graph-kits/tests/test_audit_hook_allowlist.py`.
- Modify `~/.hermes/shell-hooks-allowlist.json` only during approved Item 12 migration.
- Add install receipt linkage to the kit manifest.

## RED first

- Duplicate resolved script path is a nonzero failure.
- Missing/stale path is nonzero.
- Same script invoked through path aliases resolves to one identity.
- Malformed shell quoting is rejected using `shlex`; no whitespace splitting.
- Interpreter + script identity is captured separately.
- Symlink target, owner, mode, executable bit, and content digest are validated.
- Audit mode performs no writes.
- Apply without backup/review manifest is refused.
- Injected write failure restores byte-identical original.
- Entries outside the approved removal set are preserved.

## Workflow

1. `audit` emits deterministic report and exits nonzero on issues.
2. `plan` creates a proposed patch containing exact precondition digest and classified actions: keep, deduplicate, update digest, quarantine, remove.
3. Every non-Agent-Graph stale entry has a named owning subsystem. Do not silently delete another subsystem’s entry.
4. User approval in Item 12 admits the exact plan digest.
5. `apply --plan <file>` verifies precondition digest, writes timestamped backup, stages and validates JSON, atomically replaces, fsyncs parent, reruns audit, and rolls back on failure.
6. `rollback --receipt <file>` restores only if current digest matches the applied digest.
7. Add a scheduled/watchdog check; do not auto-apply future changes.

## Gates

```bash
python3 -m pytest tests/test_audit_hook_allowlist.py -v
python3 hermes/scripts/audit_hook_allowlist.py audit <fixture-allowlist> --json-receipt <evidence>/allowlist-audit.json
python3 hermes/scripts/audit_hook_allowlist.py plan <fixture-allowlist> --output <evidence>/allowlist-plan.json
python3 hermes/scripts/audit_hook_allowlist.py apply <fixture-allowlist> --plan <evidence>/allowlist-plan.json --apply
python3 hermes/scripts/audit_hook_allowlist.py audit <fixture-allowlist>
python3 hermes/scripts/audit_hook_allowlist.py rollback <fixture-allowlist> --receipt <apply-receipt>
```

The mutation commands above run only on disposable fixtures during source verification. Applying the plan to the active Hermes allowlist is deferred to Item 12 and requires explicit approval.

## Acceptance

Final audit has zero unexplained stale or duplicate entries. If another subsystem cannot be repaired in this scope, quarantine its stale entry and document the owning follow-up; do not mark AG-027 closed while a stale executable remains approved.

---

# Item 12 — controlled migration, installed acceptance, rollback drill, and independent hostile certification

## Objective

Convert tested source into an installed, operator-governed topology and close the ledger only from installed evidence.

## 12.1 Pre-migration dry run — no live mutation

1. Freeze clean Rust and kits commits after Items 1–11.
2. Build the release bundle from the clean Rust worktree.
3. Copy the live DB using SQLite online backup while the old owner is running; separately archive WAL/SHM metadata for diagnosis.
4. Copy/redact config, units, binaries, integrity-key metadata (not contents), kits, scripts, allowlist, and active process inventory.
5. Run all migrations and acceptance tests against copied state and disposable runtime paths.
6. Exercise rollback from migrated copy to pre-migration copy.
7. Produce `migration-readiness.json`. Any `unknown`, nonzero gate, missing rollback hash, or unsupported legacy state blocks live migration.

## 12.2 Rollback pack

Before approval, create and verify:

```text
rollback/<timestamp>/
  manifest.json
  agent-graph.db.sqlite-backup
  previous-binaries/
  previous-config-redacted.json
  previous-config-file.backup
  previous-user-units/
  previous-polkit-policy/
  previous-helper/
  previous-kits/
  previous-hermes-scripts/
  previous-allowlist.json
  restore.sh
  verify-restore.py
```

`manifest.json` binds every file digest/mode/owner and the commands required to restore. Test `restore.sh` against a sandbox; do not first test rollback on live state.

## 12.3 Explicit authority gate

Stop immediately before any of the following:

- stopping the active MCP owner;
- modifying Hermes config;
- installing/enabling systemd units;
- installing root-owned helper or Polkit policy;
- migrating the live DB;
- modifying the live allowlist or Hermes scripts.

Present the exact release manifest digest, migration plan digest, allowlist plan digest, rollback manifest digest, expected downtime, and rollback trigger set. Require explicit user approval for that exact tuple. Any rebuilt artifact invalidates approval.

## 12.4 Live migration order

After approval:

1. Block new durable work and verify no active uncheckpointed effectful runs; otherwise abort.
2. Create final online DB backup and re-hash rollback pack.
3. Stop old watchdog/direct owner gracefully; verify zero DB owners.
4. Install verified proxy, daemon, operator client, root helper, policy, and user unit from the release bundle using staged atomic replacement.
5. Run schema migration once through daemon maintenance mode; record before/after versions and row classifications.
6. Start daemon; wait for typed readiness; verify one owner and private sockets.
7. Update Hermes MCP config from direct durable args to proxy `--socket` only. Remove key/data/model settings from proxy environment.
8. Install canonical kits/scripts, run kit validation/deep doctor, then apply the approved allowlist plan.
9. Restart/reload affected Hermes MCP sessions without killing unrelated work.
10. Run installed acceptance matrix.

## 12.5 Installed Acceptance Matrix

All must pass:

### Identity and topology

- exact installed hashes match release manifest;
- user unit/policy/config-template hashes match install receipt;
- exactly one daemon MainPID/instance/generation;
- only daemon has DB/WAL/SHM/key/owner-lock file descriptors;
- 1, 20, and 100 short-lived proxies never create another owner;
- model and operator sockets have correct type/owner/mode and distinct protocols.

### Model capability

- initialize/tools-list succeeds through installed proxy;
- model tool set matches capability manifest;
- approval decision, promotion, delete, migration, and install are absent/rejected;
- create/get/list/run/cancel/status/receipt paths work;
- sync timeout returns completion-unknown without losing daemon-owned work;
- proxy kill during run does not cancel the daemon run.

### Lifecycle

- graceful daemon restart preserves terminal receipts and eligible checkpoints;
- crash/restart test on a disposable copied DB demonstrates Item 1 classifications;
- terminal publication and idempotency remain exact once;
- legacy effectful/unknown runs remain quarantined.

### Operator authority

- same-user direct connection to operator socket is rejected;
- forged actor/receipt/nonce is rejected;
- Polkit denial leaves approval pending;
- one explicitly user-approved non-effectful test approval succeeds, produces one operator receipt, and resumes once;
- replay is rejected;
- no raw secret or key appears in argv, logs, receipts, or manifests.

### Promotion

- three good outcomes remain candidate;
- forged/mismatched receipt rejected;
- one explicitly user-approved synthetic template promotion succeeds exactly once;
- contradictory outcome quarantines; Python projection rebuild matches canonical digest.

### Kits/watchdog/allowlist

- all nine host config validators pass;
- disposable deep doctor passes;
- watchdog returns `ok` only after every authoritative check passes;
- allowlist audit has zero unexplained stale/duplicate entries;
- uninstall/rollback sandbox test is byte-identical.

### Security/release

- database/key/runtime modes private;
- MSRV/stable fmt/clippy/tests pass on frozen source;
- release manifest/SBOM/advisory policy validate;
- installed/runtime/config/service digests match;
- no critical/high target-reachable advisory remains.

## 12.6 Rollback triggers and procedure

Rollback immediately on:

- more than one durable owner;
- daemon cannot acquire/release ownership cleanly;
- migration or integrity mismatch;
- model sees any privileged tool;
- any approval/promotion without authenticated operator receipt;
- nonce replay succeeds;
- proxy can open DB/key;
- installed hash differs from manifest;
- Hermes cannot reconnect through proxy;
- terminal publication or durable receipt regression;
- watchdog cannot distinguish roles or reports false OK;
- allowlist apply leaves stale/duplicate entries;
- unexplained data loss or legacy state reclassification.

Rollback order:

1. disable new proxy connections;
2. stop daemon and verify zero DB owners;
3. preserve failed migrated state for forensics;
4. restore pre-migration DB through verified backup, not file copy over an open DB;
5. restore prior binaries/config/units/policy/scripts/allowlist from manifest;
6. disable/remove new operator route;
7. start prior known-good runtime only if its known risks are explicitly accepted; otherwise remain safely unavailable;
8. verify restore digests and database integrity;
9. publish a failed-migration receipt. Never call rollback success without restored acceptance checks.

## 12.7 Independent hostile re-audit

Freeze the installed release digest before audit. Use independent read-only lanes with no implementation ownership:

- runtime/lifecycle/process lane: AG-001, AG-003, AG-023;
- authority/promotion lane: AG-002, AG-005;
- release/kits/operations lane: AG-007, AG-008, AG-010, AG-012, AG-025, AG-027;
- controller reruns all commands and verifies installed handles/digests.

For each finding, record:

```text
finding, severity, disposition, exact source evidence, exact installed evidence,
test/receipt, consequence, residual risk, rollback/quarantine state, auditor identity
```

Allowed final dispositions: `verified_closed`, `quarantined`, `partially_closed`, `still_open`. Full certification requires every item in this plan to be `verified_closed`; a critical/high `quarantined` item yields a safe but non-certified runtime.

## 12.8 Closure ledger and final artifacts

Update `agent-graph-mcp/docs/remediation/hostile-audit-closure-ledger.md` with all AG-001–AG-028 rows, not only the remaining eleven. Each closed row must cite:

- source commit;
- installed artifact digest;
- focused test receipt;
- installed acceptance receipt where applicable;
- migration/rollback receipt;
- independent auditor disposition.

Create final bundle:

```text
final-certification/
  certification.json
  closure-ledger.md
  finding-matrix.json
  release-manifest.json
  install-receipt.json
  sbom.cdx.json
  advisory-adjudication.json
  migration-receipt.json
  rollback-receipt.json
  installed-acceptance.json
  independent-audit.json
  checksums.sha256
```

Verify `checksums.sha256` in a separate process and record the verifier command/output.

---

## 6. Phase gates and serialization

| Gate | Must pass before | Required proof |
|---|---|---|
| G0 Baseline admission | any source edit | isolated clean worktrees, baseline digests, dirty canonical checkout untouched |
| G1 Runtime foundation | authority work integration | Items 1/3/9 process tests; one owner; proxy blind; lifecycle atomic |
| G2 Authority | promotion integration | Item 2 negative/process tests; model privilege absent; operator exact-once |
| G3 Promotion | operational scripting | Item 4 canonical binding/restart/replay tests |
| G4 Release/security | bundle production | Items 5/7 clean build, schema, SBOM, MSRV/stable gates, adjudicated advisories |
| G5 Kits/ops | migration readiness | Items 6/8/10/11 tests, deep doctor, watchdog fixtures, allowlist plan |
| G6 Dry-run | user approval request | copied-state migration and rollback drill, exact bundle/plan digests |
| G7 Live activation | certification audit | explicit approval, installed acceptance, rollback pack still valid |
| G8 Independent audit | closure claim | all 12 controller-verified dispositions and complete ledger |

Parallelism is permitted only where state does not overlap:

- after G0, runtime transport/lifecycle tests and release-script test harness may proceed in parallel;
- authority implementation waits for the daemon admin boundary contract;
- promotion waits for authority receipt schema;
- kits may develop against a frozen proxy contract but cannot activate before G1;
- watchdog waits for status/capability schema;
- live migration, Polkit install, config update, allowlist apply, and final ledger are serialized.

At every phase boundary: re-read `git status`, diff only admitted paths, rerun focused gates, update finding matrix, and stop on invariant violation.

---

## 7. Full certification command matrix

Run from clean Rust worktree unless stated otherwise:

```bash
cargo fmt --check -p agent-graph -p agent-graph-mcp
cargo clippy --locked -p agent-graph -p agent-graph-mcp --all-targets -- -D warnings
cargo test --locked -p agent-graph --tests
cargo test --locked -p agent-graph-mcp --no-fail-fast
cargo +1.75.0 check --locked -p agent-graph -p agent-graph-mcp --all-targets
cargo +1.75.0 test --locked -p agent-graph --tests
cargo +1.75.0 test --locked -p agent-graph-mcp --no-fail-fast
cargo audit --json
cargo deny --manifest-path agent-graph-mcp/Cargo.toml --config agent-graph-mcp/deny.toml check
```

Run from clean kits worktree:

```bash
python3 -m pytest tests/ -v
python3 scripts/validate-all-kits.py
python3 shared/scripts/doctor.py --deep --json-receipt <evidence>/kit-doctor.json
```

Run release and installed checks:

```bash
agent-graph-mcp/scripts/build-release.sh
python3 agent-graph-mcp/scripts/validate-release.py <manifest> --verify-tree --verify-receipts
systemd-analyze --user verify <service-file>
python3 <canonical-watchdog> --json
python3 <canonical-allowlist-auditor> audit <active-allowlist>
sha256sum -c final-certification/checksums.sha256
```

Do not summarize counts by parsing only final lines. Preserve raw outputs and JUnit/JSON where supported. A timeout is a failure or `unknown`, never a pass.

---

## 8. Implementation handoff checklist

Before editing:

- [ ] Baseline HEAD and kits HEAD still match this plan or plan is amended.
- [ ] Dirty canonical checkout is untouched.
- [ ] New worktrees are clean.
- [ ] Current `Cargo.lock` ownership/diff is adjudicated.
- [ ] Evidence root and command recorder exist.
- [ ] Finding matrix contains all twelve items and exact expected proof.
- [ ] Model approval/promotion remains quarantined.

Before requesting live approval:

- [ ] Items 1–11 focused and integration tests pass.
- [ ] Clean release bundle validates independently.
- [ ] No critical/high target-reachable advisory remains.
- [ ] Copied-state migration passes.
- [ ] Sandbox rollback restores byte-identical state.
- [ ] Allowlist change plan is exact and reviewed.
- [ ] Release, migration, allowlist, and rollback digests are frozen.
- [ ] Expected downtime and rollback triggers are stated.

Before declaring completion:

- [ ] Installed topology, authority, lifecycle, promotion, kits, watchdog, and allowlist acceptance all pass.
- [ ] Independent auditors inspected the frozen installed release.
- [ ] Controller reran and verified every material command.
- [ ] Closure ledger cites real evidence for all 28 original findings.
- [ ] Final checksums verify in a separate process.
- [ ] Remaining risks and any skipped gates are explicit.
- [ ] Rollback artifacts remain retained and tested.

---

## 9. Evidence-safe completion language

Before Item 12 installed acceptance, acceptable wording is:

> The follow-up implementation exists in isolated source worktrees and passed the named local tests. The live Agent Graph runtime remains unmodified or quarantined; installed closure is not yet established.

After migration but before independent audit:

> The frozen release was installed and passed the recorded acceptance matrix. Independent hostile closure is pending.

Only after G8:

> The eleven previously unresolved findings and the final migration/certification item were independently verified closed for the exact installed artifact and configuration identified by the final certification bundle.

Do not claim production readiness, enterprise security, authenticated human approval, canonical promotion, or full audit closure from source inspection or unit tests alone.

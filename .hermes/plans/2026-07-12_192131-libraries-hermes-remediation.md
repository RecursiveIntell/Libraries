# Libraries + Hermes Remediation and Integration Plan

> **For Hermes:** Execute task-by-task with strict TDD, controller-owned verification, local commits only, and no remote push.

**Goal:** Repair the verified hostile-audit defects, stabilize the package/live truth boundary, and add only high-ROI Libraries capabilities to Hermes through narrow plugin/MCP seams.

**Architecture:** Preserve `semantic-memory`/MCP as the memory waist and `context-governor` as the context-engine waist. Harden Libraries contracts before exposing them to Hermes. Add claim/trace/execution governance as observer or preflight layers—not parallel agent runtimes, databases, or model tools. Deploy live changes only from named commits with copied-store canaries and rollback receipts.

**Tech Stack:** Rust/Cargo, Python/pytest, Hermes plugin/context-engine APIs, MCP stdio/HTTP, SQLite, systemd user services, npm/TypeScript.

---

## Evidence-backed current state

- Repositories:
  - `/home/sikmindz/Coding/Libraries` at clean `0f75a56595ce7b52b81e6b4cb48f42886bc236bf` on `feat/full-integration`.
  - `/home/sikmindz/.hermes/hermes-agent` at clean `9b134cd2ea6a810b1bae80b534e4b8791f5fef77` on `main`, ahead 5 / behind upstream 97.
- Root workspace baseline: prior 2,745 tests passed; fresh root test/clippy rerun started as `/tmp/remediation-root-{test,clippy}.log`.
- Fresh `cargo deny check`: still red.
- Fresh AiDENs tests: compile fails at:
  - `AiDENs/crates/aidens-autonomous/src/capture.rs:145`
  - `AiDENs/crates/aidens-memory-kit/src/lib.rs:669`
  - both omit `valid_time`/`recorded_time` in `AddGraphEdgeParams`.
- Hermes doctor: config v33 and core runtime healthy.
- Context-governor plugin: 45 tests pass; one test is environment-blocked under the current shell Python because PyYAML is absent, not because of a product assertion.
- Live context receipts are split:
  - `~/.hermes/context-governor`: 60
  - `~/.hermes/context-governor-store`: 4
  - `~/.local/share/context-governor/receipts`: 126
- Claim boundary: no release-ready, authority-safe AiDENs, unified context-store, or freshly deployed semantic-memory/MCP claim is allowed until its phase gate passes.

## Hard no list

- No push, crates.io publish, or PR without explicit user authorization.
- No direct SQLite row copying between semantic-memory stores.
- No wholesale AiDENs/agent-graph/queue/runtime integration into Hermes.
- No parallel Python memory implementation.
- No quantized backend as sole authoritative memory index.
- No fake provenance, synthesized canonical lineage, or self-asserted permit authority.
- No live binary swap before copied-store canary and rollback artifact.
- No multi-hour PPL/LLM benchmark without a separate explicit request.

---

## Phase 0 — Preserve state and create execution receipts

### Task 0.1: Capture immutable pre-change receipts

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/.hermes/receipts/remediation-preflight-20260712/`
- Create: `STATUS.txt`, `DIFF.patch`, `HEADS.txt`, `GATES.txt`

**Steps:**
1. Record both repo HEADs, branches, status, diff checks, installed binary hashes, active services, and store counts.
2. Save current deny and failing-gate logs.
3. Verify both repos remain clean before feature edits.

**Gate:** receipt directory contains hashes and exact commands; `git status --short` remains empty in both repos.

---

## Phase 1 — CRITICAL trust and provenance fixes

### Task 1.1: Replace self-asserted AiDENs permits with issuer-bound grants

**Files:**
- Modify: `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- Modify: `AiDENs/crates/aidens-permit-kit/src/lib.rs`
- Modify: `AiDENs/crates/aidens-tool-kit/src/registry.rs`
- Test: `AiDENs/crates/aidens-permit-kit/src/lib.rs` tests
- Test: `AiDENs/crates/aidens-tool-kit/tests/p30_tool_hardening.rs`

**TDD:**
1. RED: deserialized/caller-constructed grant without trusted issuer proof must not authorize.
2. RED: altered grant, unknown issuer, revoked grant, and replayed use outside run/attempt must deny.
3. Add private `PermitIssuer`, issuer-bound digest/signature-equivalent receipt contract, revocation reference/status, and verification before scope matching.
4. Keep read-only default behavior unchanged.
5. GREEN: focused permit/tool tests.

**Gate:** no public constructor can mint an admissible authority-bearing grant; all side-effect paths recheck verified grants at dispatch.

### Task 1.2: Make AiDENs permit/use IDs deterministic and cross-process stable

**Files:** same contract/permit files plus receipt tests.

**TDD:**
1. RED: same canonical grant material in two processes produces same ID.
2. RED: changing issuer, scope, risk, expiry, run, or attempt changes ID.
3. Use domain-separated BLAKE3 over canonical material; retain unstable display IDs only for non-authority UI objects.

**Gate:** cross-process golden fixture passes.

### Task 1.3: Reject missing canonical lineage in Forge V3

**Files:**
- Modify: `forge-memory-bridge/src/transform.rs`
- Test: `forge-memory-bridge/tests/forge_bridge_memory_proof.rs`
- Test: `forge-memory-bridge/src/transform_tests.rs`

**TDD:**
1. RED: V3 claim/relation missing canonical IDs returns typed error.
2. RED: serialization failure cannot become empty derivation material.
3. Keep deterministic synthesis only in explicitly named legacy V1/V2 migration APIs; emit migration provenance.
4. GREEN: V3 refusal and legacy migration tests.

**Gate:** `cargo test -p forge-memory-bridge --all-targets`; no `unwrap_or_default()` in identity derivation.

### Task 1.4: Complete bitemporal event/version identity

**Files:**
- Modify: `bitemporal-runtime/src/types.rs`
- Modify: `bitemporal-runtime/src/sqlite.rs`
- Modify: `bitemporal-runtime/src/lib.rs`
- Modify: `bitemporal-runtime/src/queries.rs`
- Test: `bitemporal-runtime/tests/hostile_audit_adversarial_tests.rs`
- Test: `bitemporal-runtime/tests/sqlite_db_tests.rs`

**TDD:**
1. RED: supersession names exact old/new event IDs, not stable record ID.
2. RED: migration identity is content-derived and stable across two DBs with different rowids.
3. RED: serialization failure returns error rather than tie-key collapse.
4. Add first-class event IDs to records/receipts and a durable migration receipt/schema table.
5. Preserve wire compatibility through versioned serde migration where possible.

**Gate:** all-feature bitemporal tests and downstream semantic-memory tests pass.

### Task 1.5: Fix AiDENs temporal graph-edge API drift

**Files:**
- Modify: `AiDENs/crates/aidens-autonomous/src/capture.rs`
- Modify: `AiDENs/crates/aidens-memory-kit/src/lib.rs`
- Add focused temporal-contract tests in each crate.

**TDD:** explicit valid/recorded time semantics must survive capture and memory adapter round-trip.

**Gate:** `cargo test --workspace --all-targets` from `AiDENs/` passes.

---

## Phase 2 — HIGH execution and package integrity

### Task 2.1: Harden `check-runner` container execution

**Files:**
- Modify: `Primitives/check-runner/src/lib.rs`
- Modify: `Primitives/check-runner-sys/src/lib.rs`
- Add integration tests under `Primitives/check-runner/tests/`.

**TDD:**
1. RED: shell metacharacters remain one argv value and cannot execute a second command.
2. RED: disallowed caller env keys are rejected.
3. RED: timed-out container is force-removed using tracked CID/name.
4. RED: sealed source mount is read-only with a separate writable output mount.
5. Return/verify process-group kill errors.

**Gate:** all-feature tests + Clippy; optional live Docker smoke when available.

### Task 2.2: Repair poly-kv Python workspace packaging

**Files:**
- Modify: `poly-kv/Cargo.toml`
- Modify: `poly-kv/crates/poly-kv-python/Cargo.toml`
- Modify: `poly-kv/pyproject.toml`
- Add: `poly-kv/tests/test_python_package_smoke.py` or package-local equivalent.

**TDD:** clean venv wheel build, install, import, minimal encode/decode operation.

**Gate:** Rust tests plus `maturin build --release` and installed-wheel pytest.

### Task 2.3: Enforce path dependency version coherence

**Files:**
- Create: `scripts/check_path_dependency_versions.py`
- Create: `tests/test_path_dependency_versions.py` or script self-tests.
- Modify stale manifests only after the gate identifies them.
- Wire into `scripts/release_preflight.sh`.

**TDD:** mismatched `{path, version}` fixture fails; matching and intentionally unpublished dependencies pass with explicit policy.

**Gate:** zero unexplained mismatches; package-build smoke for publishable crates.

### Task 2.4: Clear dependency advisories without blind upgrades

**Files:** affected Cargo/npm manifests and lockfiles identified by reverse dependency trees.

**Steps:**
1. Record `cargo tree -i` reachability for each advisory.
2. Update minimum dependency set.
3. Run root, AiDENs, MCP, context, and Node gates.
4. Do not suppress advisories without a documented non-reachability proof and expiry.

**Gate:** root and AiDENs `cargo deny check` pass; npm audit has zero high/critical findings.

### Task 2.5: Restore strict Clippy cleanliness

Fix the turbo-quant benchmark warnings and AiDENs Clippy findings with behavior-preserving edits and focused tests.

**Gate:** root and AiDENs Clippy with `-D warnings` pass.

---

## Phase 3 — Hermes context and memory activation

### Task 3.1: Propagate configured context policy into plugin engines

**Repo:** `/home/sikmindz/.hermes/hermes-agent`

**Files:**
- Modify: `agent/agent_init.py`
- Test: `tests/plugins/test_context_governor_plugin.py`
- Test/add: fresh-agent non-default configuration contract.

**TDD:** RED with threshold, target/output reservation, protect-head/tail values deliberately different from defaults; GREEN when passed to `update_model()` and reflected by live engine state.

**Gate:** plugin suite using the Hermes venv, then relevant agent/context tests.

### Task 3.2: Consolidate context-governor stores safely

**Files:**
- Create migration script under `~/.hermes/scripts/` only if no existing supported migration command exists.
- Create a receipt manifest and backups under `~/.hermes/backups/`.

**Steps:**
1. Identify writer processes/configs for all three stores.
2. Stop or reconfigure legacy writers using supported config/service mechanisms.
3. Merge by receipt ID and digest, preserving conflicts for review; never overwrite blindly.
4. Validate union counts, exact search/expand, and canonical-only future writes.
5. Keep rollback archives.

**Gate:** one canonical store, no active writer to legacy paths, union preserved, search/expand smoke passes.

### Task 3.3: Build and canary semantic-memory/MCP from a named commit

**Files/services:** no source edits unless a gate fails; build artifacts and deployment receipt only.

**Steps:**
1. Run semantic-memory all-feature and MCP stable/full matrices.
2. Build release binary and record source commit/hash.
3. Snapshot canonical store via SQLite-supported backup.
4. Run candidate against copied store on isolated port/token.
5. Verify integrity, stats, witnessed search, durable receipt reload/replay, forged authority denial, forgetting closure, profile tool list, and maintenance check.
6. Install with previous binary retained for rollback; restart via supported service command.

**Gate:** installed hash equals candidate hash; fresh MCP and HTTP smoke pass; rollback tested.

---

## Phase 4 — New Hermes features from Libraries

### Task 4.1: Add claim-ledger observer/finalization gate

**Design:** A narrow Hermes plugin observes material tool outcomes and final claim promotion. It does not become a model-visible tool by default and does not create a second truth database.

**Files:**
- Libraries: add/verify a small claim-ledger CLI or MCP adapter if absent.
- Hermes: create plugin under `plugins/` using existing hook APIs.
- Tests: successful/failed tool calls, unsupported claim, session/tool/request lineage, chain verification.

**TDD:** no ledger receipt means no “verified/completed” promotion; read-only conversation remains unaffected.

**Gate:** plugin tests, claim-ledger tests, one fresh Hermes smoke with a real tool receipt.

### Task 4.2: Add `stack-ids` trace translation

**Design:** Internal translation joins Hermes session/task/API request IDs with context, memory-search, permit, and claim receipts. No new model tool.

**TDD:** cross-process stable golden trace; no collision or lost parent/child edge.

**Gate:** plugin/adapter tests and one end-to-end trace packet.

### Task 4.3: Add canonical boundary digests to Hermes evidence receipts

Use `boundary-compiler` only after profile/schema domain separation is implemented. Canonicalize tool arguments and claim evidence at plugin/MCP boundaries without replacing provider schema normalization.

**Gate:** duplicate-key rejection, RFC 8785 vectors, cross-language digest parity, and no tool-call compatibility regression.

### Task 4.4: Add governed execution preflight behind an opt-in plugin

Use hardened `check-runner` for explicitly selected coding checks. Keep Hermes terminal behavior unchanged by default. Expose preflight/commit receipts to policy middleware, not every model prompt.

**Gate:** read-only commands unaffected; dangerous execution requires approved permit and emits cleanup/evidence receipts.

### Task 4.5: Add receipt-bench proof packets to release verification

Generate local machine-readable proof packets containing dataset/config hashes, raw per-case rows, recomputable aggregate metrics, machine fingerprint, command, and artifact digest.

**Gate:** deterministic fixture receipt tests; no public quality claim beyond measured workload.

---

## Phase 5 — Full verification and local commits

### Task 5.1: Libraries gauntlet

Run and preserve outputs:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
(cd AiDENs && cargo fmt --all -- --check && cargo test --workspace --all-targets && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check)
(cd poly-kv && cargo test --workspace --all-targets)
(cd semantic-memory-mcp && cargo test --all-targets && cargo test --features full --all-targets && cargo clippy --all-targets -- -D warnings)
(cd context-governor && cargo test --all-targets && cargo clippy --all-targets -- -D warnings)
```

### Task 5.2: Hermes gauntlet

```bash
python -m pytest tests/plugins/test_context_governor_plugin.py -q -o 'addopts='
python -m pytest tests/agent tests/plugins -q -o 'addopts='
hermes doctor
hermes mcp test semantic_memory
```

Run the repository's broader canonical suite if targeted changes pass.

### Task 5.3: Review and commit

- Run diff/security/spec review.
- Commit Libraries and Hermes separately with scoped messages.
- Do not push.
- Update the hostile-audit report with shipped/deferred items and final receipt hashes.

## Final claim boundary

Safe only after all gates:

- Libraries root/AiDENs/package gates are green at named commits.
- AiDENs permits are issuer-verified, stable, and revocable.
- Forge V3 never invents canonical lineage.
- Bitemporal supersession identifies exact events with migration receipts.
- Context-governor has one authoritative store and receives configured policy.
- Live semantic-memory/MCP binary is commit-addressed and canary-certified.
- New Hermes claim/trace/execution features are narrow, opt-in where side-effecting, receipt-backed, and tested.

Not safe without separate evidence: production maturity, compliance, external superiority, customer adoption, or model-quality wins from quantization.
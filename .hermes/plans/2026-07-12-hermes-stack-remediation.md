# Hermes Stack Remediation and Feature Integration Plan

> **For Hermes:** Execute task-by-task with strict RED/GREEN verification. Preserve unrelated dirty work. Commit locally only; never push without explicit approval.

**Goal:** Harden the Libraries trust spine, wire the highest-ROI capabilities into live Hermes without parallel shadow systems, and leave exact receipts for every enabled path.

**Architecture:** Keep Hermes as the host/orchestrator and use narrow adapters: `context-governor` remains the context engine; `semantic-memory-mcp` remains the sole semantic-memory boundary; shell hooks emit compact deterministic receipts; hardened Libraries contracts remain behind those boundaries until their own authority and provenance gates pass. Canonical stores are explicit and legacy stores are treated as migration inputs, never co-equal writers.

**Tech Stack:** Python 3.11+, Hermes Agent 6.0.3, Rust/Cargo, pytest, semantic-memory MCP/HTTP, context-governor CLI, shell hooks, SQLite.

---

## Evidence-backed current state

- Audit root: `/home/sikmindz/Coding/Libraries`; extensive pre-existing dirty work must be preserved.
- Audit report: `/home/sikmindz/Downloads/recursiveintell-libraries-hostile-audit-2026-07-12.md`.
- Root tests previously passed: 2,745 passed, 4 ignored. Root Clippy is red on `hnsw-bench` warnings; this is not a release-clean workspace.
- Root and AiDENs `cargo deny` are red. Exact advisory/policy diagnostics must be regenerated after code stabilization.
- Live Hermes: `/home/sikmindz/.hermes/hermes-agent`, version 6.0.3, local branch diverged from upstream; do not run a blind update.
- Active context engine: `context_governor`; canonical store intent: `/home/sikmindz/.hermes/context-governor`.
- Active semantic MCP store: `/home/sikmindz/.hermes/semantic-memory.db`; alternate `/home/sikmindz/.local/share/semantic-memory` must not remain an active writer.
- `post_tool_call` is configured and fires, but deterministic host-lineage receipts and canonical/installed plugin parity require completion.
- Hard trust blockers: caller-mintable AiDENs permits; Forge V3 synthetic lineage; incomplete bitemporal event identity/migration receipts.

## Phase 1 — P0 trust and receipt correctness

### Task 1.1: Deterministic tool receipt lineage

**Files**
- Modify: `/home/sikmindz/Coding/agent-memory-kits/shared/scripts/tool_receipts.py`
- Modify: `/home/sikmindz/Coding/agent-memory-kits/hermes/hooks/sm-auto-edge.py`
- Test: `/home/sikmindz/Coding/agent-memory-kits/tests/test_hermes_tool_receipts.py`

**Steps**
1. Keep the existing failing regression proving identical host lineage yields the same trace and changed `tool_call_id` changes it.
2. Derive `trace_id` from a domain-separated canonical preimage containing tool digest and explicit host lineage.
3. Persist host lineage in the receipt, not only in the digest preimage.
4. Forward `task_id`, `tool_call_id`, `api_request_id`, and parent trace from the Hermes payload.
5. Run the focused receipt suite; require all tests green.

### Task 1.2: Canonical/installed hook parity and live registration

**Files**
- Canonical: `/home/sikmindz/Coding/agent-memory-kits/hermes/{plugin.json,hooks/sm-auto-edge.py,hooks/sm-auto-edge.sh}`
- Installed: `/home/sikmindz/.hermes/plugins/semantic-memory-mcp/{plugin.json,hooks/sm-auto-edge.py,hooks/sm-auto-edge.sh}`
- Config: `/home/sikmindz/.hermes/config.yaml` via `hermes config`, not manual YAML edits.

**Steps**
1. Verify deployment-aware shared-script discovery in source and installed layouts.
2. Make source and installed hook logic byte-identical where deployment paths permit.
3. Verify `hermes hooks list`, `hermes hooks doctor`, and synthetic `post_tool_call` execution.
4. Verify an actual non-noisy tool call creates a `tool-receipts` fact with the expected deterministic trace and lineage.
5. Keep hook failure non-blocking for agent execution but observable through doctor/log output.

**Phase gate**
```bash
python3 -m pytest tests/test_hermes_tool_receipts.py -q
hermes hooks doctor
hermes hooks test post_tool_call --for-tool terminal
```

## Phase 2 — Context and store coherence

### Task 2.1: Forward the complete host compression policy

**Files**
- Modify: `/home/sikmindz/.hermes/hermes-agent/agent/agent_init.py`
- Test: `/home/sikmindz/.hermes/hermes-agent/tests/agent/test_context_engine_policy_forwarding.py`
- Regression suite: `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`

**Steps**
1. Preserve the RED regression for non-default threshold/protection/max-token fields.
2. Pass the complete policy into external engine `update_model()`.
3. Run focused helper and plugin tests.
4. Instantiate a fresh `AIAgent` and verify effective values on `ContextGovernorEngine`.

### Task 2.2: Resolve context-governor store fragmentation

1. Enumerate receipt IDs, counts, overlap, unique IDs, and newest timestamps across:
   - `/home/sikmindz/.hermes/context-governor`
   - `/home/sikmindz/.hermes/context-governor-store`
   - `/home/sikmindz/.local/share/context-governor/receipts`
2. Identify current writers from config, hooks, services, and process environments.
3. Copy unique legacy receipts into a backed-up canonical staging store only after validating receipt integrity.
4. Point every writer to `/home/sikmindz/.hermes/context-governor`.
5. Verify compact → store → exact search → expand against the canonical store.
6. Do not delete legacy stores; rename/archive only after proving no active writer.

### Task 2.3: Resolve semantic-memory store ownership

1. Verify live MCP child args, HTTP service args, hooks, maintenance job, and current MCP stats.
2. Compare fact IDs/content digests between canonical and alternate stores.
3. Import unique facts through governed APIs or a copied-store migration; never raw-merge a live database.
4. Restart only the affected MCP/gateway processes after backups.
5. Verify witnessed search → durable receipt → replay on the canonical store.

**Phase gate**
```bash
PYTHONDONTWRITEBYTECODE=1 python -m pytest tests/agent/test_context_engine_policy_forwarding.py tests/plugins/test_context_governor_plugin.py -q -o 'addopts=' -p no:cacheprovider
context-governor status --dir /home/sikmindz/.hermes/context-governor
hermes mcp test semantic_memory
```

## Phase 3 — Harden Libraries before exposing new authority

### Task 3.1: AiDENs trusted permit issuance

1. Replace caller-constructible authority with issuer-bound grants.
2. Bind issuer identity, content-derived grant ID, signature/MAC or canonical authority receipt, expiry, scope, and revocation reference.
3. Reject deserialized/unverified grants at every capability gate.
4. Repair `AddGraphEdgeParams` temporal fields and run the AiDENs workspace.
5. Keep AiDENs disabled as Hermes action authority until adversarial tests pass.

### Task 3.2: Forge and bitemporal provenance

1. Make canonical Forge V3 reject missing claim/version/relation lineage.
2. Fence synthetic derivation into explicitly labeled legacy migration code.
3. Use event IDs—not stable record IDs—for bitemporal supersession links.
4. Add deterministic migration IDs and migration receipts; remove `rowid` identity dependence.

### Task 3.3: Harden command execution

1. Preserve structured argv; forbid `sh -c` reconstruction for adversarial inputs.
2. Validate caller environment keys/values.
3. Track and force-remove timed-out containers.
4. Mount source read-only with a separate writable output path.
5. Surface process-group cleanup failures.
6. Do not expose this as a general Hermes executor until a real container timeout integration test passes.

### Task 3.4: Packaging and dependency truth

1. Repair `poly-kv` Python workspace metadata and clean-wheel import smoke.
2. Add a read-only path/version coherence gate.
3. Fix current Clippy blockers without hiding warnings.
4. Regenerate exact `cargo deny` diagnostics and upgrade/replace vulnerable dependencies where compatible.

**Phase gate**
```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
(cd AiDENs && cargo test --workspace --all-targets && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check)
```

## Phase 4 — Add only high-ROI Hermes features

### Task 4.1: Receipt-backed post-tool lineage (enable now)
- Ship the deterministic `post_tool_call` receipt path after Phase 1 gates pass.
- Store only compact status/digests; no raw large tool output.

### Task 4.2: Context policy observability (enable now)
- Expose effective threshold, target, protections, canonical store, and last persistence error through existing `context_status`—no parallel tool.

### Task 4.3: Governed action authority (incubate)
- Integrate only after AiDENs trusted issuance/revocation passes.
- Adapter must consume canonical authority receipts, not caller-provided permit JSON.

### Task 4.4: Sandboxed check execution (incubate)
- Add as an optional toolset after real container lifecycle tests.
- Default to sealed source, writable output, explicit image, CPU/memory/time limits.

### Task 4.5: Boundary/bitemporal provenance enrichment (incubate)
- Use `boundary-compiler` for canonical payload admission and bitemporal event receipts behind semantic-memory MCP.
- No second memory database or Python shadow ledger.

### Task 4.6: Quantized retrieval (research-only)
- Keep proveKV/poly-kv/turbo-quant behind optional adapters and exact f32 reranking.
- Require external retrieval workloads and quality gates before changing the authoritative store.

## Phase 5 — Live validation and local checkpoint

1. Run focused Hermes suites, then full Hermes pytest.
2. Run hook doctor, MCP test, live witnessed retrieval/replay, and context exact recovery.
3. Run root/AiDENs tests, Clippy, deny, Node audit/build, and Python packaging smoke.
4. Record exact commands, exit codes, counts, artifact hashes, store paths, and untested boundaries.
5. Update the hostile-audit report with only current evidence.
6. Commit logical changes locally by repository; do not push.

## Claim boundary

Safe after focused gates: deterministic tool receipts, registered live hook, and complete context-policy forwarding work in the tested paths.

Not safe until all red gates close: release-ready Libraries, trusted AiDENs action authority, canonical Forge provenance, production-safe container execution, published-package compatibility, or quantized retrieval superiority.

## Hard no list

- No blind `hermes update` on the diverged customized branch.
- No raw merge of live SQLite stores.
- No deletion of legacy stores before unique-ID migration and writer shutdown proof.
- No AiDENs permits as authorization while caller-mintable.
- No synthetic lineage represented as authoritative provenance.
- No second semantic-memory implementation beside the MCP boundary.
- No release-ready claim while Clippy, dependency policy, packaging, or consent gates remain red.

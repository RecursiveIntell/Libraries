# 04 — Extraction Procedure

## Principle

Extract by **semantic ownership**, not by moving current files wholesale.

Current Recall files often contain several concerns in one module. The extraction should split those concerns cleanly and then update Recall to use AiDENs as its own app kit.

## Phase 0 — Freeze current Recall as source baseline

Actions:

1. Tag the current Recall archive/repo state.
2. Save a source inventory:
   - crate list,
   - module list,
   - public type list,
   - tests list,
   - schema list,
   - current `Cargo.lock` if present.
3. Record static findings from this packet.
4. Define the extraction branch.

Exit criteria:

```text
current Recall still builds if it previously built
source inventory checked in
AiDENs extraction issue matrix opened
```

## Phase 1 — Foundation crates

Create:

```text
aidens-contracts
aidens-boundary-kit
aidens-config
aidens-receipts
aidens-capability-kit
aidens-testkit
```

### Extract `aidens-contracts`

From:

```text
recall-contracts/src/lib.rs
stack-ids
verification-control selected types
verification-policy selected types
```

Procedure:

1. Move shared base types only.
2. Keep Recall-specific app views separate or behind `recall-compat` feature.
3. Generate schemas from Rust types.
4. Meta-validate generated schemas.
5. Add compatibility tests for schema evolution.

Acceptance:

```text
schema generation passes
schema meta-validation passes
no runtime crate dependency
```

### Extract `aidens-boundary-kit`

From:

```text
deps/llm-output-parser
recall-session/src/session/tool_dispatch.rs parser extraction/repair logic
recall-session/src/path_safety.rs
recall-session/src/query_safety.rs
```

Procedure:

1. Define strict JSON acceptance.
2. Define structured-output repair outputs.
3. Define boundary repair receipts.
4. Define JSON patch/merge patch gates if patch tools are used.
5. Add differential parser fixtures.

Acceptance:

```text
raw LLM JSON cannot reach tool execution in examples
repair emits receipt
duplicate key / malformed / oversized cases are rejected or repaired with explicit artifact
```

### Extract `aidens-config`

From:

```text
recall-session/src/config.rs
recall-app config state
recall-daemon apply config paths
```

Procedure:

1. Split config model from runtime status.
2. Add config generation IDs.
3. Add atomic apply plan.
4. Add redaction API.
5. Add blocked/missing/default-created states.

Acceptance:

```text
config apply validates before commit
secrets redacted in all debug/status output
run pins config_generation_id
```

### Extract `aidens-receipts`

From:

```text
recall-contracts QueryReceiptV2 and receipt types
recall-session/src/control.rs
llm-tool-runtime receipt sink patterns
recall-daemon scheduler receipts
```

Procedure:

1. Define append-only receipt ledger trait.
2. Provide in-memory and file/sqlite dev ledgers.
3. Bridge tool receipts into run receipts.
4. Bridge control receipts into run receipts.
5. Add receipt consistency assertions.

Acceptance:

```text
every runner path emits a RunReceipt
receipt exists for fallback, denial, timeout, approval, boundary repair
```

### Extract `aidens-capability-kit`

From:

```text
RuntimeCapabilityTruthV1
RuntimeTruthV1
build_runtime_status
tool_capability_statuses
provider/web/scheduler status logic
```

Procedure:

1. Define truth surfaces.
2. Separate configured/registered/exposed/executable/attempted.
3. Add digest generation.
4. Add doctor checks.

Acceptance:

```text
disabled tools are shown as disabled, not ready
parser fallback is shown as fallback, not native
provider unavailable blocks readiness
```

## Phase 2 — Usable agent core

Create:

```text
aidens-provider-kit
aidens-tool-kit
aidens-security-kit
aidens-permit-kit
aidens-arbiter-kit
aidens-budget-kit
aidens-runner
```

### Provider extraction

From:

```text
recall-session/src/provider.rs
recall-session/src/provider_bridge.rs
deps/llm-pipeline
```

Fixes:

- unknown native provider must not default to OpenAI chat,
- provider route must record actual backend,
- parser fallback must be explicit degraded route,
- API key/model failures must block readiness early.

### Tool extraction

From:

```text
llm-tool-runtime
recall-session/src/tool_catalog.rs
generic tool descriptor/view logic
```

Fixes:

- disabled means absent,
- per-turn exposure required,
- tool ID includes namespace/name/version,
- full registry exposure is debug-only.

### Permit/security extraction

From:

```text
recall-session/src/approval.rs
path_safety.rs
scheduler permit types
```

Fixes:

- UI does not own approval truth,
- auto approval is explicit and bounded,
- future permits attenuate rather than expand authority,
- shell/write/network policies are separate gates.

### Arbiter extraction

From:

```text
session/arbiter.rs
session/arbiter_fast_signals.rs
session/arbiter_intents.rs
graph_query.rs route decisions
```

Fixes:

- no substring-only `needs_tools`,
- no-tool route is first-class,
- fallback ladder recorded,
- route decision references capability truth.

### Runner extraction

From:

```text
session/mod.rs query path
session/tool_dispatch.rs native tool query
session/prompt.rs prompt assembly
control.rs receipt building
```

Fixes:

- runner owns one run only,
- no app/daemon/UI lifecycle inside runner,
- all paths emit receipts,
- native tool execution is happy path where supported,
- parser fallback is explicit and receipt-bearing.

Exit criteria for Phase 2:

```text
one provider works
one read-only tool works
one write tool requires approval
provider fallback truth recorded
run receipt emitted
no daemon/tauri dependency
```

## Phase 3 — App speed layer

Create:

```text
aidens-app-kit
aidens-cli
aidens-profile-coding
```

Procedure:

1. Define `AiDENsAppPlanV1`.
2. Define profile expansion.
3. Define starter templates.
4. Implement `aidens new`.
5. Implement `aidens doctor`.
6. Add generated tests.

Exit criteria:

```bash
aidens new coding-agent my-agent
cd my-agent
aidens doctor
cargo test
cargo run
```

works with local defaults.

## Phase 4 — Memory, queue, schedule, daemon, UI

Create:

```text
aidens-memory-kit
aidens-queue-kit
aidens-schedule-kit
aidens-daemon-kit
aidens-wake-kit
aidens-tauri-kit
```

Procedure:

1. Extract memory adapters without making runtime a DB.
2. Extract queue leases/attempt families.
3. Extract schedule law separately from host wake.
4. Extract daemon IPC lifecycle.
5. Extract Tauri presentation bridge.

Exit criteria:

```text
memory disabled/optional/required modes work
queued action has lease/attempt lineage
schedule fires exactly once per policy
host wake is projection only
UI reads capability truth
```

## Phase 5 — Advanced runtime

Create:

```text
aidens-kernel-kit
aidens-delegation-kit
aidens-plan-kit
aidens-repair-kit
future aidens-federation-kit
future aidens-mechanism-kit
```

Exit criteria:

```text
kernel consumes snapshots
regions exchange typed artifacts
repair emits records
child agents cannot expand authority
external artifacts import as evidence only
```

## Migration of Recall itself

After AiDENs v0.1 is usable, Recall should become an AiDENs app:

```text
Recall before: recall-session owns everything
Recall after: recall-app/daemon call aidens-app-kit + app-specific Recall tools
```

Steps:

1. Replace provider construction with `aidens-provider-kit`.
2. Replace tool registry assembly with `aidens-tool-kit`.
3. Replace approval policy with `aidens-permit-kit`.
4. Replace query path with `aidens-runner`.
5. Replace runtime status with `aidens-capability-kit`.
6. Replace receipts with `aidens-receipts`.
7. Keep Recall-specific tools in `recall-session` or new `recall-tools` until generalized.
8. Eventually slim `recall-session` into a Recall app profile/tools crate.

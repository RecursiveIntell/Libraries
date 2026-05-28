# Codex Prompt — AiDENs Next Run: Real Platform Path

You are in:

```bash
~/Coding/Libraries/AiDENs
```

This is the **current AiDENs scaffold**. It is not finished. Your job is to complete the next real implementation pass so AiDENs becomes a usable app-construction front door, not a documentation scaffold.

## Source roots

Use these paths:

```text
~/Coding/Libraries/AiDENs       # current AiDENs workspace
~/Coding/Recall                 # primary extraction source
~/Coding/Recall-Coding          # secondary source, coding-tool patterns only
~/Coding/Libraries              # canonical libraries
~/Coding/Libraries2             # additional libraries
```

## Critical context

The current AiDENs scaffold likely has:

- good crate boundaries;
- config, receipts, capability truth, provider route truth skeletons;
- safe tool exposure skeleton;
- CLI scaffolding;
- many advanced crates intentionally placeholder/skeletal;
- but `aidens-runner` still returns fake/placeholder output.

That is not acceptable for this pass.

## First commands

Run these before editing:

```bash
bash scripts/next_preflight.sh || true
bash scripts/inspect_next_sources.sh || true
bash scripts/assert_no_fake_completion.sh || true
```

Then inspect these files manually:

```text
~/Coding/Libraries/AiDENs/PASS_STATUS.md
~/Coding/Libraries/AiDENs/crates/aidens-runner/src/lib.rs
~/Coding/Libraries/AiDENs/crates/aidens-provider-kit/src/lib.rs
~/Coding/Libraries/AiDENs/crates/aidens-tool-kit/src/lib.rs
~/Coding/Libraries/AiDENs/crates/aidens-app-kit/src/lib.rs
~/Coding/Libraries/AiDENs/crates/aidens-cli/src/lib.rs
~/Coding/Libraries/AiDENs/crates/aidens-contracts/src/lib.rs

~/Coding/Recall/recall-session/src/provider.rs
~/Coding/Recall/recall-session/src/provider_bridge.rs
~/Coding/Recall/recall-session/src/session/tool_dispatch.rs
~/Coding/Recall/recall-session/src/tool_catalog.rs
~/Coding/Recall/recall-session/src/approval.rs
~/Coding/Recall/recall-session/src/control.rs
~/Coding/Recall/recall-session/src/config.rs
~/Coding/Recall/recall-session/src/path_safety.rs
~/Coding/Recall/deps/llm-pipeline/src/tool_loop.rs
~/Coding/Recall/deps/llm-pipeline/src/lib.rs

~/Coding/Recall-Coding/recall-session/src/tools/workspace_audit.rs
~/Coding/Recall-Coding/recall-session/src/tools/workspace_patch.rs
~/Coding/Recall-Coding/recall-session/src/tools/run_checks.rs
```

If any source file is missing, use `find`/`rg` to locate the equivalent and record the substitution in `PASS_STATUS.md`.

## Objective

Implement the **real P0/P1 AiDENs path**:

```text
Profile/config -> AiDENsAppPlanV1 -> validate -> compile -> app/runner -> provider execution -> tool execution -> receipts -> doctor truth
```

Do not widen into memory/queue/kernel/daemon unless needed to keep the public API honest. Those can report `disabled`, `deferred`, or `not configured` in doctor output.

## Required implementation

### 1. Remove fake runner completion

`aidens-runner` must not return placeholder strings.

Required behavior:

- provider kind `disabled` fails honestly;
- provider kind `mock` returns only an explicitly configured mock response;
- provider kind `ollama` uses a real provider boundary or reports unavailable if not configured/reachable;
- provider route receipts reflect the actual execution path;
- run receipt warnings must not contain placeholder text.

Forbidden strings in runtime code:

```text
AiDENs placeholder response
placeholder runner output
wire provider implementation next
fake success
TODO runtime
```

### 2. Add an executable provider boundary

In `aidens-provider-kit`, add a trait or equivalent:

```rust
#[async_trait::async_trait]
pub trait AiDENsProvider: Send + Sync {
    fn provider_kind(&self) -> &str;
    fn model(&self) -> Option<&str>;
    fn capabilities(&self) -> ProviderCapabilitiesV1;
    async fn complete(&self, request: AiDENsCompletionRequestV1) -> anyhow::Result<AiDENsCompletionResponseV1>;
}
```

Minimum implementations:

- `DisabledProvider` — always returns an error and never produces an answer.
- `MockProvider` — deterministic, explicit test/smoke provider only.
- `OllamaProvider` or `LlmPipelineProvider` — wraps Recall/llm-pipeline provider logic where feasible.

Use Recall as primary source:

- `recall-session/src/provider.rs` has completion request/response, retry summary, provider capabilities, execution mode labels, Ollama provider behavior.
- `recall-session/src/provider_bridge.rs` shows how Recall wraps `llm_pipeline::ExecCtx` + `LlmCall`.
- `deps/llm-pipeline/src/tool_loop.rs` is the native tool-loop source for later native tool execution.

If direct llm-pipeline wiring fails due dependency path issues, implement the trait and mock/disabled behavior now, add the path dependency attempt, and record exact blockers in `PASS_STATUS.md`. Do not return placeholder text as a substitute.

### 3. Add real read-only tool dispatch

`aidens-tool-kit` must be able to execute at least one real sandboxed read-only tool.

Minimum tool:

```text
aidens:repo-read:1
```

Expected input:

```json
{ "path": "README.md" }
```

Rules:

- sandbox root is configured explicitly;
- path traversal is rejected;
- absolute path escape is rejected;
- disabled/dangerous tools are absent by default;
- invocation emits a `ToolAttemptReceiptV1` or equivalent receipt;
- tool output is structured enough for runner/CLI tests.

Use Recall's path safety and tool exposure patterns:

- `recall-session/src/path_safety.rs`
- `recall-session/src/session/tool_dispatch.rs`
- `recall-session/src/tool_catalog.rs`

Use Recall-Coding only for coding-specific tool shape ideas, not as the primary runtime source.

### 4. Implement AppPlan and doctor flow

Add or finish:

```text
AiDENsAppPlanV1
AiDENsCompiledPlanV1 or equivalent
AiDENsDoctorReportV1
AiDENsProfile list/explain
plan validate
plan compile
```

CLI commands required:

```bash
aidens profile list
aidens profile explain coding-agent
aidens plan validate --config examples/aidens.mock.toml
aidens plan compile --config examples/aidens.mock.toml --out target/aidens-plan.json
aidens doctor --config examples/aidens.mock.toml
aidens provider-check --config examples/aidens.mock.toml
aidens list-tools
aidens run --config examples/aidens.mock.toml "hello"
aidens new coding-agent /tmp/aidens-smoke
```

Doctor output must be structured truth, not vibes. It must distinguish:

```text
configured
available
healthy
registered
exposed_this_turn
executable_this_turn
disabled
blocked_by_policy
degraded
fallback_only
unavailable
deferred
```

Required doctor sections:

```text
config
provider
tools
security
receipts
memory
queue
schedule
daemon
runtime
```

Deferred sections may report disabled/deferred, but must not pretend to be healthy.

### 5. Generated app must use only the facade

`aidens new coding-agent my-app` should generate a simple app using the root `aidens`/`aidens-app-kit` facade.

Good generated app:

```rust
let app = AiDENsApp::from_config("aidens.toml").build().await?;
let output = app.run_once("hello").await?;
println!("{}", output.text);
```

Bad generated app:

```rust
let provider = ProviderStack::new(...);
let tools = ToolRegistry::new(...);
let receipts = ReceiptLedger::new(...);
```

### 6. Keep advanced crates honest

These may remain skeletal in this pass:

```text
aidens-memory-kit
aidens-kernel-kit
aidens-queue-kit
aidens-schedule-kit
aidens-daemon-kit
aidens-delegation-kit
aidens-plan-kit
aidens-repair-kit
aidens-governance-kit
```

But doctor must report them as disabled/deferred/not configured, not healthy.

## Required tests

The overlay includes tests you must satisfy or adapt minimally without weakening intent:

```text
crates/aidens-testkit/tests/next_no_fake_completion.rs
crates/aidens-runner/tests/next_runner_provider_contract.rs
crates/aidens-tool-kit/tests/next_repo_read_dispatch.rs
crates/aidens-cli/tests/next_cli_plan_doctor.rs
crates/aidens-app-kit/tests/next_app_plan_facade.rs
```

Do not delete these tests to pass. If names/API differ, update the tests to the equivalent public API while preserving the same assertions.

## Required acceptance commands

Run and update `PASS_STATUS.md` with exact results:

```bash
bash scripts/assert_no_fake_completion.sh
bash scripts/next_smoke.sh
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo test -p aidens-testkit --test next_no_fake_completion
cargo test -p aidens-runner --test next_runner_provider_contract
cargo test -p aidens-tool-kit --test next_repo_read_dispatch
cargo test -p aidens-cli --test next_cli_plan_doctor
cargo test -p aidens-app-kit --test next_app_plan_facade
```

If the environment lacks cargo, say so in `PASS_STATUS.md`. Do not claim build success.

## Completion standard

You may claim completion only if:

- no fake completion patterns remain;
- disabled provider cannot answer;
- explicit mock provider can answer deterministically;
- at least one real read-only tool executes with a receipt;
- CLI profile/plan/doctor/run commands work for explicit mock config;
- generated app uses facade-only API;
- cargo check/tests pass or exact blockers are documented.

Implementation first. Documentation only as needed to keep the run understandable.

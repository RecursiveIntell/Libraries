# Tool Dispatch Spec

## Minimum dispatch API

Implement in `aidens-tool-kit`:

```rust
pub struct ToolDispatcher { ... }

impl ToolDispatcher {
    pub fn new(registry: ToolRegistryV1) -> Self;
    pub async fn invoke(&self, tool_id: &str, input: serde_json::Value) -> anyhow::Result<ToolInvocationOutcomeV1>;
}
```

## Minimum tool

`aidens:repo-read:1`

Input:

```json
{ "path": "README.md" }
```

Output should include file text and a receipt.

## Sandbox law

Reject:

```text
../secret
/etc/passwd
symlink escapes if feasible
absolute paths outside sandbox
home secret prefixes
```

Use Recall's `path_safety.rs` as the source pattern.

## Receipt law

Every invocation returns or appends a tool attempt receipt containing:

```text
tool_id
attempt_id or receipt_id
input digest if available
succeeded bool
error class/reason if failed
started/completed timestamps if available
```

## Exposure law

`registered != exposed != executable != attempted`.

Default coding profile exposes only read-only/safe tools.

# P22 Security and Redaction Plan

## Current known issue

The current source package reports two warnings for `secret-content-named-secret-assignment` at:

- `crates/aidens-app-kit/src/lib.rs` line 370
- `crates/aidens-cli/src/lib.rs` line 2777

The source lines are field-copy patterns, not hardcoded secret literals. P22 must close this without weakening actual secret detection.

## Required scanner distinction

Allowed/no warning:

```rust
api_key: provider.api_key.clone(),
api_key: cfg.provider.api_key.clone(),
```

Still warning/error:

```rust
api_key: "sk-...".to_string(),
let api_key = "sk-...";
OPENAI_API_KEY=sk-...
```

## Required redaction rule

Any provider report, receipt, archive manifest, or debug output that references credential presence must print only:

```text
configured=true
source=env|config|none
redacted=true
```

Never print secret values.

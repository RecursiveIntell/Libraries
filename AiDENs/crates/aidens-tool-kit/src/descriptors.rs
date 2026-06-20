use aidens_contracts::{CanonicalToolSideEffectClass, ToolDescriptorV1, ToolSchemaV1};

pub fn safe_coding_tool_plan() -> Vec<(ToolDescriptorV1, bool)> {
    vec![
        (repo_read_descriptor(), true),
        (repo_list_descriptor(), true),
        (file_stat_descriptor(), true),
        (repo_search_descriptor(), true),
        (patch_propose_descriptor(), true),
        (patch_apply_descriptor(), true),
        (run_checks_descriptor(), true),
        (
            side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
            false,
        ),
        (
            side_effect_descriptor("shell", CanonicalToolSideEffectClass::Admin),
            false,
        ),
        (
            side_effect_descriptor("network", CanonicalToolSideEffectClass::Analysis),
            false,
        ),
        (
            side_effect_descriptor("memory-write", CanonicalToolSideEffectClass::Write),
            false,
        ),
        (
            side_effect_descriptor("schedule", CanonicalToolSideEffectClass::Admin),
            false,
        ),
    ]
}

pub fn safe_coding_tool_declarations() -> Vec<ToolDescriptorV1> {
    safe_coding_tool_plan()
        .into_iter()
        .map(|(descriptor, _enabled)| descriptor)
        .collect()
}

pub fn repo_read_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-read".into(),
        version: "1".into(),
        description: "Read one UTF-8 file inside the configured repository sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "bytes", "content"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "bytes": { "type": "integer", "minimum": 0 },
                    "content": { "type": "string" }
                }
            }),
            parser_fallback_hint: "Call aidens:repo-read:1 with JSON {\"path\":\"relative/file\"}."
                .into(),
        },
    }
}

pub fn patch_propose_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "patch-propose".into(),
        version: "1".into(),
        description: "Propose a patch without applying it; this tool never mutates files.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["summary", "diff"],
                "properties": {
                    "summary": { "type": "string" },
                    "diff": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "proposal", "mutates_files"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "proposal": { "type": "object" },
                    "mutates_files": { "type": "boolean" }
                }
            }),
            parser_fallback_hint: "Call aidens:patch-propose:1 with JSON {\"summary\":\"...\",\"diff\":\"unified diff\"}."
                .into(),
        },
    }
}

pub fn repo_list_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-list".into(),
        version: "1".into(),
        description: "List entries inside one directory under the repository sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": { "type": "string" },
                    "max_entries": { "type": "integer", "minimum": 1, "maximum": 1000 }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "entries", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "entries": { "type": "array" },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint: "Call aidens:repo-list:1 with JSON {\"path\":\"relative/dir\"}."
                .into(),
        },
    }
}

pub fn file_stat_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "file-stat".into(),
        version: "1".into(),
        description: "Inspect metadata for one file or directory inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "is_file", "is_dir", "bytes"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "is_file": { "type": "boolean" },
                    "is_dir": { "type": "boolean" },
                    "bytes": { "type": "integer", "minimum": 0 },
                    "content_digest": { "type": ["object", "null"] }
                }
            }),
            parser_fallback_hint: "Call aidens:file-stat:1 with JSON {\"path\":\"relative/file\"}."
                .into(),
        },
    }
}

pub fn repo_search_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-search".into(),
        version: "1".into(),
        description: "Search UTF-8 files inside the sandbox for a literal string.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["query"],
                "properties": {
                    "query": { "type": "string" },
                    "path": { "type": "string" },
                    "max_matches": { "type": "integer", "minimum": 1, "maximum": 200 }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "query", "path", "matches"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "query": { "type": "string" },
                    "path": { "type": "string" },
                    "matches": { "type": "array" }
                }
            }),
            parser_fallback_hint:
                "Call aidens:repo-search:1 with JSON {\"query\":\"needle\",\"path\":\".\"}.".into(),
        },
    }
}

pub fn patch_apply_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "patch-apply".into(),
        version: "1".into(),
        description: "Apply an approved unified patch inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::Write,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["diff"],
                "properties": {
                    "diff": { "type": "string" },
                    "check_only": { "type": "boolean" },
                    "dry_run": { "type": "boolean" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "applied", "dry_run_checked", "changed_files", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "applied": { "type": "boolean" },
                    "dry_run_checked": { "type": "boolean" },
                    "changed_files": { "type": "array", "items": { "type": "string" } },
                    "semantic_status": { "type": "string" },
                    "failure_kind": { "type": "string" },
                    "touched_paths": { "type": "array", "items": { "type": "string" } },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint:
                "aidens:patch-apply:1 requires explicit scoped file-write permit evidence.".into(),
        },
    }
}

pub fn run_checks_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "run-checks".into(),
        version: "1".into(),
        description: "Run an allowlisted local check command inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::Admin,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["command"],
                "properties": {
                    "command": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 1
                    }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "command", "succeeded", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "command": { "type": "array", "items": { "type": "string" } },
                    "succeeded": { "type": "boolean" },
                    "exit_code": { "type": ["integer", "null"] },
                    "stdout": { "type": "string" },
                    "stderr": { "type": "string" },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint:
                "aidens:run-checks:1 requires explicit scoped shell permit evidence.".into(),
        },
    }
}

pub fn side_effect_descriptor(
    name: &str,
    risk_class: CanonicalToolSideEffectClass,
) -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: name.into(),
        version: "1".into(),
        description: format!("{name} is a side-effect tool and requires explicit permit evidence."),
        risk_class,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            parser_fallback_hint: format!("{name} is not registered by the safe coding profile."),
        },
    }
}

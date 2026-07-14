//! App-level path safety helpers and canonical tool side-effect classification.

use llm_tool_runtime::ToolSideEffectClass;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpTrustReportV1 {
    pub tool_name: String,
    pub side_effect_class: String,
    pub requires_permit: bool,
    pub is_dangerous: bool,
    pub risk_label: String,
    pub recommendation: String,
}

pub fn evaluate_mcp_tool_safety(descriptor: &llm_tool_runtime::ToolDescriptor) -> McpTrustReportV1 {
    let side_effect_class = descriptor.side_effect_class.to_string();
    let requires_permit = requires_explicit_permit(&descriptor.side_effect_class);
    let is_dangerous = is_dangerous_without_permit(&descriptor.side_effect_class);

    let risk_label = match descriptor.side_effect_class {
        ToolSideEffectClass::ReadOnly => "low",
        ToolSideEffectClass::Analysis | ToolSideEffectClass::PreviewWrite => "medium",
        ToolSideEffectClass::Write => "high",
        ToolSideEffectClass::Admin => "critical",
    };

    let recommendation = match risk_label {
        "low" => "allow",
        "medium" | "high" => "permit-required",
        "critical" => "deny-by-default",
        _ => "allow",
    };

    McpTrustReportV1 {
        tool_name: descriptor.name.clone(),
        side_effect_class,
        requires_permit,
        is_dangerous,
        risk_label: risk_label.to_string(),
        recommendation: recommendation.to_string(),
    }
}

pub fn attest_tool_invocation(
    receipt: &llm_tool_runtime::ToolReceipt,
) -> attestation_exchange::AttestationEnvelopeV1 {
    let attestation_envelope_id = receipt
        .attestation_envelope_id
        .clone()
        .unwrap_or_else(|| stack_ids::AttestationEnvelopeId::new(format!("attn_{}", receipt.receipt_id)));

    let trace_context = receipt.trace_ctx.trace_id.clone();
    let signing_time = if !receipt.finished_at.is_empty() {
        receipt.finished_at.clone()
    } else {
        receipt.started_at.clone()
    };
    let trust_root_set_id = if trace_context.is_empty() {
        "trs_unknown".to_string()
    } else {
        format!("trs_{}", &trace_context)
    };
    let disclosure_policy_id = if trace_context.is_empty() {
        "dp_unknown".to_string()
    } else {
        format!("dp_{}", &trace_context)
    };
    let signer_identity = if trace_context.is_empty() {
        receipt.tool_name.clone()
    } else {
        trace_context.clone()
    };
    let provenance_summary = if trace_context.is_empty() {
        format!(
            "invocation {} with attempt {}",
            receipt.tool_name, receipt.attempt_id
        )
    } else {
        format!(
            "invocation {} with trace {} and attempt {}",
            receipt.tool_name, trace_context, receipt.attempt_id,
        )
    };

    if let Ok(envelope) = attestation_exchange::AttestationEnvelopeV1::new(
        attestation_envelope_id.clone(),
        "mcp-tool-invocation",
        "v1",
        receipt.input_digest.clone(),
        "mcp-tool",
        signer_identity.clone(),
        signing_time.clone(),
        stack_ids::TrustRootSetId::new(trust_root_set_id.clone()),
        provenance_summary.clone(),
        stack_ids::DisclosurePolicyId::new(disclosure_policy_id.clone()),
        None,
        attestation_exchange::AttestationReplayabilityClassV1::Replayable,
        Vec::new(),
        Vec::new(),
    ) {
        envelope
    } else {
        attestation_exchange::AttestationEnvelopeV1 {
            schema_version: attestation_exchange::ATTESTATION_ENVELOPE_V1_SCHEMA.to_string(),
            attestation_envelope_id,
            artifact_family: "mcp-tool-invocation".into(),
            artifact_version: "v1".into(),
            content_digest: receipt.input_digest.clone(),
            schema_identity: "mcp-tool".into(),
            signer_identity,
            signing_time,
            trust_root_set_id: stack_ids::TrustRootSetId::new(trust_root_set_id),
            provenance_summary,
            disclosure_policy_id: stack_ids::DisclosurePolicyId::new(disclosure_policy_id),
            artifact_admission_policy_id: None,
            replayability_class: attestation_exchange::AttestationReplayabilityClassV1::Replayable,
            revocation_refs: Vec::new(),
            supersession_refs: Vec::new(),
        }
    }
}

pub fn requires_explicit_permit(side_effect: &ToolSideEffectClass) -> bool {
    !matches!(
        side_effect,
        ToolSideEffectClass::ReadOnly | ToolSideEffectClass::Analysis
    )
}

pub fn is_dangerous_without_permit(side_effect: &ToolSideEffectClass) -> bool {
    matches!(
        side_effect,
        ToolSideEffectClass::PreviewWrite | ToolSideEffectClass::Write | ToolSideEffectClass::Admin
    )
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathSafetyError {
    #[error("path traversal is not allowed")]
    TraversalNotAllowed,
    #[error("path is outside sandbox root {root}")]
    OutsideSandbox { root: String },
    #[error("path is in sensitive prefix {prefix}")]
    SensitivePrefix { prefix: String },
    #[error("path contains hidden or sensitive component {component}")]
    HiddenOrSensitiveComponent { component: String },
}

pub fn path_contains_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

pub fn expand_home_path(path: &str, home: &Path) -> PathBuf {
    if path == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(path)
}

pub fn validate_sandbox_path(path: &Path, sandbox_root: &Path) -> Result<(), PathSafetyError> {
    if path_contains_parent_dir(path) {
        return Err(PathSafetyError::TraversalNotAllowed);
    }
    if !path.starts_with(sandbox_root) {
        return Err(PathSafetyError::OutsideSandbox {
            root: "<sandbox-root>".into(),
        });
    }

    let relative = path.strip_prefix(sandbox_root).unwrap_or(path);
    for component in relative.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        let name = name.to_string_lossy();
        let folded = name.to_ascii_lowercase();
        if is_sensitive_component(&folded) {
            return Err(PathSafetyError::SensitivePrefix {
                prefix: name.into_owned(),
            });
        }
        if folded.starts_with('.') {
            return Err(PathSafetyError::HiddenOrSensitiveComponent {
                component: name.into_owned(),
            });
        }
    }
    Ok(())
}

fn is_sensitive_component(component: &str) -> bool {
    matches!(
        component,
        ".git"
            | ".git-credentials"
            | ".env"
            | ".env.local"
            | ".npmrc"
            | ".aws"
            | ".ssh"
            | ".gnupg"
            | ".cargo"
            | ".recall"
            | ".password-store"
            | "id_rsa"
            | "id_ed25519"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use llm_tool_runtime::{
        McpSurfaceKind, ToolApprovalKind, ToolBackendKind, ToolExposureMode, ToolIdempotencyClass,
        ToolOutputMode,
    };

    fn mcp_descriptor(side_effect_class: ToolSideEffectClass) -> llm_tool_runtime::ToolDescriptor {
        llm_tool_runtime::ToolDescriptor {
            name: "mcp.tool".to_string(),
            version: "1.0.0".to_string(),
            description: Some("test tool".to_string()),
            backend_kind: ToolBackendKind::RemoteMcp,
            input_schema: serde_json::json!({"type": "object"}),
            output_mode: ToolOutputMode::StructuredJson,
            read_only: true,
            side_effect_class,
            idempotency_class: ToolIdempotencyClass::Idempotent,
            approval_kind: ToolApprovalKind::None,
            timeout_ms: 1000,
            concurrency_key: None,
            cache_ttl_ms: None,
            exposure_mode: ToolExposureMode::Auto,
            mcp_surface_kind: McpSurfaceKind::Tool,
            exposure_policy: Default::default(),
            receipt_persistence: Default::default(),
            output_size_limit_bytes: None,
            provider_payload: None,
        }
    }

    #[test]
    fn side_effects_require_canonical_permit_classification() {
        assert!(requires_explicit_permit(&ToolSideEffectClass::Write));
        assert!(!requires_explicit_permit(&ToolSideEffectClass::ReadOnly));
        assert!(is_dangerous_without_permit(&ToolSideEffectClass::Admin));
    }

    #[test]
    fn evaluate_mcp_tool_safety_classifies_readonly_as_allow() {
        let descriptor = mcp_descriptor(ToolSideEffectClass::ReadOnly);
        let report = evaluate_mcp_tool_safety(&descriptor);

        assert_eq!(report.tool_name, "mcp.tool");
        assert_eq!(report.side_effect_class, "read-only");
        assert!(!report.requires_permit);
        assert!(!report.is_dangerous);
        assert_eq!(report.risk_label, "low");
        assert_eq!(report.recommendation, "allow");
    }

    #[test]
    fn evaluate_mcp_tool_safety_classifies_write_as_deny_by_default() {
        let descriptor = mcp_descriptor(ToolSideEffectClass::Write);
        let report = evaluate_mcp_tool_safety(&descriptor);

        assert_eq!(report.tool_name, "mcp.tool");
        assert_eq!(report.side_effect_class, "write");
        assert!(report.requires_permit);
        assert!(report.is_dangerous);
        assert_eq!(report.risk_label, "high");
        assert_eq!(report.recommendation, "permit-required");
    }

    #[test]
    fn path_safety_rejects_traversal_and_sensitive_prefixes() {
        let home = Path::new("/home/user");

        assert!(path_contains_parent_dir(Path::new("../secret.txt")));
        assert_eq!(expand_home_path("~/repo", home), home.join("repo"));
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.ssh/id_rsa"), home),
            Err(PathSafetyError::SensitivePrefix {
                prefix: ".ssh".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.npmrc"), home),
            Err(PathSafetyError::SensitivePrefix {
                prefix: ".npmrc".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.hidden"), home),
            Err(PathSafetyError::HiddenOrSensitiveComponent {
                component: ".hidden".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/tmp/file.txt"), home),
            Err(PathSafetyError::OutsideSandbox {
                root: "<sandbox-root>".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/repo"), home),
            Ok(())
        );
    }
}

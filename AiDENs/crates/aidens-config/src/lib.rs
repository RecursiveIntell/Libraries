//! Config loading, validation, redaction, and atomic apply planning.

use aidens_contracts::{MemoryModeV1, ReportLevelV1};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const REDACTION_MASK: &str = "********";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiDENsConfigV1 {
    pub app_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    pub provider: ProviderConfigV1,
    #[serde(default)]
    pub tools: ToolsConfigV1,
    #[serde(default)]
    pub security: SecurityConfigV1,
    #[serde(default)]
    pub receipts: EventLogConfigV1,
    #[serde(default, skip_serializing_if = "MemoryConfigV1::is_empty")]
    pub memory: MemoryConfigV1,
    pub memory_mode: MemoryModeV1,
    pub receipt_level: ReportLevelV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfigV1 {
    pub kind: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mock_response: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ToolsConfigV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_root: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_bundles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventLogConfigV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MemoryConfigV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_root: Option<String>,
}

impl MemoryConfigV1 {
    pub fn is_empty(&self) -> bool {
        self.store_root.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfigV1 {
    pub approval_mode: String,
    pub network_policy: String,
    pub write_policy: String,
}

impl Default for SecurityConfigV1 {
    fn default() -> Self {
        Self {
            approval_mode: "require-side-effects".into(),
            network_policy: "disabled".into(),
            write_policy: "approval-required".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {reason}")]
    ParseToml { path: PathBuf, reason: String },
    #[error("failed to render config as toml: {0}")]
    RenderToml(String),
    #[error("app_id must not be empty")]
    EmptyAppId,
    #[error("provider kind must not be empty")]
    EmptyProviderKind,
    #[error("dangerous or unsupported config field at {path}: {reason}")]
    DangerousField { path: String, reason: String },
    #[error("provider secret is not allowed for provider kind {kind}")]
    ProviderSecretNotAllowed { kind: String },
    #[error("provider endpoint is not allowed: {reason}")]
    ProviderEndpointNotAllowed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLoadOutcomeV1 {
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub source_fingerprint: String,
    pub reason_codes: Vec<String>,
    pub config: AiDENsConfigV1,
}

impl AiDENsConfigV1 {
    pub fn safe_default(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            profile_id: Some("chat-only".into()),
            provider: ProviderConfigV1 {
                kind: "disabled".into(),
                model: None,
                api_key: None,
                base_url: None,
                mock_response: None,
            },
            tools: ToolsConfigV1::default(),
            security: SecurityConfigV1::default(),
            receipts: EventLogConfigV1::default(),
            memory: MemoryConfigV1::default(),
            memory_mode: MemoryModeV1::Disabled,
            receipt_level: ReportLevelV1::Standard,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.app_id.trim().is_empty() {
            return Err(ConfigError::EmptyAppId);
        }
        if self.provider.kind.trim().is_empty() {
            return Err(ConfigError::EmptyProviderKind);
        }
        let provider_kind = self.provider.kind.trim().to_ascii_lowercase();
        if self.provider.api_key.is_some()
            && matches!(
                provider_kind.as_str(),
                "disabled" | "mock" | "local" | "ollama"
            )
        {
            return Err(ConfigError::ProviderSecretNotAllowed {
                kind: self.provider.kind.clone(),
            });
        }
        if let Some(base_url) = self.provider.base_url.as_deref() {
            validate_provider_base_url(base_url)?;
        }
        validate_security_defaults(self)?;
        Ok(())
    }

    pub fn redacted(&self) -> Self {
        let mut copy = self.clone();
        if copy.provider.api_key.is_some() {
            copy.provider.api_key = Some(REDACTION_MASK.into());
        }
        if let Some(base_url) = copy.provider.base_url.as_deref() {
            copy.provider.base_url = Some(redact_url_auth(base_url));
        }
        copy
    }

    pub fn redacted_json(&self) -> Result<Value, serde_json::Error> {
        let mut value = serde_json::to_value(self)?;
        redact_sensitive_json(&mut value, None);
        Ok(value)
    }

    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|e| ConfigError::RenderToml(e.to_string()))
    }
}

pub fn load_config_file(path: impl AsRef<Path>) -> Result<ConfigLoadOutcomeV1, ConfigError> {
    let path = path.as_ref().to_path_buf();
    let contents = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;
    let raw_toml = contents
        .parse::<toml::Value>()
        .map_err(|e| ConfigError::ParseToml {
            path: path.clone(),
            reason: e.to_string(),
        })?;
    reject_unknown_dangerous_fields(&raw_toml)?;
    let config =
        toml::from_str::<AiDENsConfigV1>(&contents).map_err(|e| ConfigError::ParseToml {
            path: path.clone(),
            reason: e.to_string(),
        })?;
    config.validate()?;
    let canonical_path = path.canonicalize().map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;
    let source_fingerprint = format!("sha256:{}", hex_sha256(contents.as_bytes()));
    Ok(ConfigLoadOutcomeV1 {
        path,
        canonical_path,
        source_fingerprint,
        reason_codes: vec![
            "config-source-canonicalized".into(),
            "config-fingerprint-sha256".into(),
        ],
        config,
    })
}

pub fn redacted_json_value(mut value: Value) -> Value {
    redact_sensitive_json(&mut value, None);
    value
}

fn redact_sensitive_json(value: &mut Value, key: Option<&str>) {
    match value {
        Value::Object(map) => {
            for (child_key, child_value) in map {
                redact_sensitive_json(child_value, Some(child_key));
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_sensitive_json(item, key);
            }
        }
        Value::String(text) => {
            if key.is_some_and(is_sensitive_key) || looks_like_secret_value(text) {
                *text = REDACTION_MASK.into();
            } else {
                *text = redact_url_auth(text);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    key.contains("api_key")
        || key.contains("apikey")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("credential")
        || key.contains("accesskey")
        || key.contains("privatekey")
        || key.contains("authorization")
        || key.contains("bearer")
        || key == "auth"
}

fn looks_like_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.starts_with("xoxb-")
        || lower.starts_with("ghp_")
        || lower.starts_with("github_pat_")
        || lower.starts_with("bearer ")
}

fn redact_url_auth(value: &str) -> String {
    let Some((scheme, rest)) = value.split_once("://") else {
        return value.to_string();
    };
    let Some((_, host_and_path)) = rest.split_once('@') else {
        return value.to_string();
    };
    format!("{scheme}://{REDACTION_MASK}@{host_and_path}")
}

fn reject_unknown_dangerous_fields(value: &toml::Value) -> Result<(), ConfigError> {
    fn walk(path: &str, value: &toml::Value) -> Result<(), ConfigError> {
        match value {
            toml::Value::Table(table) => {
                for (key, child) in table {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    if is_sensitive_key(key) && child_path != "provider.api_key" {
                        return Err(ConfigError::DangerousField {
                            path: child_path,
                            reason: "secret-like keys are only accepted at provider.api_key".into(),
                        });
                    }
                    walk(&child_path, child)?;
                }
            }
            toml::Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    walk(&format!("{path}[{index}]"), item)?;
                }
            }
            toml::Value::String(text) => {
                if looks_like_secret_value(text) && path != "provider.api_key" {
                    return Err(ConfigError::DangerousField {
                        path: path.into(),
                        reason: "secret-like values are only accepted at provider.api_key".into(),
                    });
                }
            }
            toml::Value::Integer(_)
            | toml::Value::Float(_)
            | toml::Value::Boolean(_)
            | toml::Value::Datetime(_) => {}
        }
        Ok(())
    }
    walk("", value)
}

fn validate_provider_base_url(base_url: &str) -> Result<(), ConfigError> {
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err(ConfigError::ProviderEndpointNotAllowed {
            reason: "only http or https endpoints are accepted".into(),
        });
    }
    if base_url
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('@'))
        .is_some()
    {
        return Err(ConfigError::ProviderEndpointNotAllowed {
            reason: "embedded endpoint credentials are not allowed".into(),
        });
    }
    Ok(())
}

fn validate_security_defaults(config: &AiDENsConfigV1) -> Result<(), ConfigError> {
    if config.security.write_policy != "approval-required" {
        return Err(ConfigError::DangerousField {
            path: "security.write_policy".into(),
            reason: "write/admin behavior requires approval-required policy".into(),
        });
    }
    if !matches!(
        config.security.network_policy.as_str(),
        "disabled" | "local-only"
    ) {
        return Err(ConfigError::DangerousField {
            path: "security.network_policy".into(),
            reason: "network policy must be disabled or local-only in supported-local configs"
                .into(),
        });
    }
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key() {
        let cfg = AiDENsConfigV1 {
            app_id: "app".into(),
            provider: ProviderConfigV1 {
                kind: "openai".into(),
                model: None,
                api_key: Some("secret".into()),
                base_url: None,
                mock_response: None,
            },
            profile_id: None,
            tools: ToolsConfigV1::default(),
            security: SecurityConfigV1::default(),
            receipts: EventLogConfigV1::default(),
            memory: MemoryConfigV1::default(),
            memory_mode: MemoryModeV1::Disabled,
            receipt_level: ReportLevelV1::Full,
        };
        assert_eq!(
            cfg.redacted().provider.api_key.as_deref(),
            Some(REDACTION_MASK)
        );
    }

    #[test]
    fn redacted_json_hides_secret_values_and_url_auth() {
        let cfg = AiDENsConfigV1 {
            app_id: "app".into(),
            provider: ProviderConfigV1 {
                kind: "openai".into(),
                model: Some("model".into()),
                api_key: Some("sk-secret".into()),
                base_url: Some("https://user:pass@example.com/v1".into()),
                mock_response: None,
            },
            profile_id: None,
            tools: ToolsConfigV1::default(),
            security: SecurityConfigV1::default(),
            receipts: EventLogConfigV1::default(),
            memory: MemoryConfigV1::default(),
            memory_mode: MemoryModeV1::Disabled,
            receipt_level: ReportLevelV1::Full,
        };

        let rendered = serde_json::to_string(&cfg.redacted_json().unwrap()).unwrap();
        assert!(!rendered.contains("sk-secret"));
        assert!(!rendered.contains("user:pass"));
        assert!(rendered.contains(REDACTION_MASK));
    }

    #[test]
    fn redacted_json_hides_adversarial_secret_keys_and_values() {
        let value = serde_json::json!({
            "api-key": "sk-test-value",
            "nested": {
                "authorization": "Bearer abc123",
                "privateKey": "ghp_should_not_render"
            },
            "array": [
                {"credential": "xoxb-credential"}
            ],
            "safe": "not secret"
        });

        let rendered = redacted_json_value(value).to_string();

        assert!(!rendered.contains("sk-test-value"));
        assert!(!rendered.contains("Bearer abc123"));
        assert!(!rendered.contains("ghp_should_not_render"));
        assert!(!rendered.contains("xoxb-credential"));
        assert!(rendered.contains("not secret"));
        assert!(rendered.contains(REDACTION_MASK));
    }

    #[test]
    fn safe_default_is_valid_and_provider_disabled() {
        let cfg = AiDENsConfigV1::safe_default("agent");

        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.provider.kind, "disabled");
        assert_eq!(cfg.memory_mode, MemoryModeV1::Disabled);
    }

    #[test]
    fn config_round_trips_through_toml_file() {
        let dir = std::env::temp_dir().join(format!("aidens-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidens.toml");
        let cfg = AiDENsConfigV1 {
            app_id: "agent".into(),
            provider: ProviderConfigV1 {
                kind: "openai".into(),
                model: Some("gpt-test".into()),
                api_key: Some("sk-secret".into()),
                base_url: Some("https://api.openai.com/v1".into()),
                mock_response: None,
            },
            profile_id: Some("coding-agent".into()),
            tools: ToolsConfigV1 {
                sandbox_root: Some(".".into()),
                enabled_bundles: vec!["safe-coding".into()],
            },
            security: SecurityConfigV1::default(),
            receipts: EventLogConfigV1 {
                store_root: Some("target/aidens-receipts/agent".into()),
            },
            memory: MemoryConfigV1 {
                store_root: Some("target/aidens-memory/agent".into()),
            },
            memory_mode: MemoryModeV1::Optional,
            receipt_level: ReportLevelV1::Full,
        };
        std::fs::write(&path, cfg.to_toml_string().unwrap()).unwrap();

        let loaded = load_config_file(&path).unwrap();

        assert_eq!(loaded.config, cfg);
        assert_eq!(loaded.path, path);
        assert!(loaded.canonical_path.is_absolute());
        assert!(loaded.source_fingerprint.starts_with("sha256:"));
        assert!(loaded
            .reason_codes
            .contains(&"config-source-canonicalized".into()));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn dangerous_unknown_secret_fields_are_rejected_before_deserialize_ignore() {
        let dir = std::env::temp_dir().join(format!(
            "aidens-config-dangerous-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidens.toml");
        std::fs::write(
            &path,
            r#"
app_id = "agent"
memory_mode = "disabled"
receipt_level = "standard"
slack_token = "xoxb-hidden"

[provider]
kind = "mock"
mock_response = "ok"
"#,
        )
        .unwrap();

        let error = load_config_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::DangerousField { .. }));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn provider_secrets_and_endpoint_credentials_are_rejected() {
        let mut cfg = AiDENsConfigV1::safe_default("agent");
        cfg.provider.kind = "mock".into();
        cfg.provider.api_key = Some("sk-secret".into());
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::ProviderSecretNotAllowed { .. })
        ));

        cfg.provider.kind = "openai".into();
        cfg.provider.api_key = Some("sk-secret".into());
        cfg.provider.base_url = Some("https://user:pass@example.com/v1".into());
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::ProviderEndpointNotAllowed { .. })
        ));
    }

    #[test]
    fn unsafe_write_or_network_defaults_are_rejected() {
        let mut cfg = AiDENsConfigV1::safe_default("agent");
        cfg.security.write_policy = "allow".into();
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::DangerousField { .. })
        ));

        cfg.security.write_policy = "approval-required".into();
        cfg.security.network_policy = "internet".into();
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::DangerousField { .. })
        ));
    }

    #[test]
    fn safe_default_plan_config_matches_reference_interpreter() {
        let cfg = AiDENsConfigV1::safe_default("agent");
        let case = aidens_contracts::ReferenceCaseV1::new(
            aidens_contracts::ReferenceDomainV1::PlanConfig,
            "safe default plan config",
            serde_json::json!({
                "memory_mode": aidens_testkit::json_string(&cfg.memory_mode),
                "receipt_level": aidens_testkit::json_string(&cfg.receipt_level),
                "memory_store_configured": cfg.memory.store_root.is_some()
            }),
            serde_json::json!({}),
        )
        .with_memory_mode(cfg.memory_mode.clone())
        .with_receipt_level(cfg.receipt_level.clone());
        let durable_receipts_required = cfg.receipt_level != ReportLevelV1::Minimal;
        let memory_blocked =
            cfg.memory_mode == MemoryModeV1::Required && cfg.memory.store_root.is_none();
        let actual = serde_json::json!({
            "memory_mode": aidens_testkit::json_string(&cfg.memory_mode),
            "receipt_level": aidens_testkit::json_string(&cfg.receipt_level),
            "memory_store_configured": cfg.memory.store_root.is_some(),
            "durable_receipts_required": durable_receipts_required,
            "memory_blocked": memory_blocked,
            "reason_codes": [
                format!("memory-mode-{}", cfg.memory_mode),
                format!("receipt-level-{}", cfg.receipt_level)
            ]
        });
        let report = aidens_testkit::compare_case_to_actual(
            &case,
            "aidens-config::AiDENsConfigV1::safe_default",
            actual,
        );

        assert!(
            report.passed,
            "{}",
            report
                .findings
                .iter()
                .map(|finding| finding.human_diff.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

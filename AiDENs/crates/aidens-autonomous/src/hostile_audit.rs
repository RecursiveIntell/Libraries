//! Hostile audit gate — cross-checks captured facts with a different LLM.
//!
//! The Codex Super-Pass Protocol uses hostile audits as the verification
//! gate. The autonomous loop should too. When viscosity is Strict or
//! Frozen, facts are cross-checked by a different model before promotion.
//!
//! If the auditor disagrees or finds problems, the fact is downgraded
//! from Promote to Quarantine.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of a hostile audit on a captured fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// Whether the fact survived the audit.
    pub survived: bool,
    /// Auditor's assessment text.
    pub assessment: String,
    /// Specific issues found (if any).
    pub issues: Vec<String>,
    /// Confidence score from auditor (0.0-1.0).
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// Gate
// ---------------------------------------------------------------------------

/// A hostile audit gate that cross-checks facts using a different LLM.
#[derive(Debug, Clone)]
pub struct HostileAuditGate {
    /// Auditor provider URL (should be different from executor's Ollama).
    auditor_url: String,
    /// Auditor model name (should differ from executor's model).
    auditor_model: String,
}

impl HostileAuditGate {
    /// Create a new audit gate.
    pub fn new(auditor_url: impl Into<String>, auditor_model: impl Into<String>) -> Self {
        let mut url = auditor_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            auditor_url: url,
            auditor_model: auditor_model.into(),
        }
    }

    /// Audit a captured fact. Returns true if the fact survives audit.
    ///
    /// The audit prompt asks a different model to find every reason the
    /// claim might be wrong. If it cannot find issues, the fact survives.
    pub async fn audit(&self, claim: &str, context: &str) -> Result<AuditResult> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        let prompt = format!(
            "You are a hostile auditor. Your job is to find every reason the \
             following claim might be wrong, unsupported, or misleading.\n\n\
             CLAIM:\n{claim}\n\n\
             CONTEXT:\n{context}\n\n\
             If you cannot find any issues, respond with exactly: SURVIVES\n\
             If you find issues, list them as bullet points starting with '- '.\n\
             End your response with a confidence score 0.0-1.0 on the line \
             'CONFIDENCE: <score>'."
        );

        let body = serde_json::json!({
            "model": self.auditor_model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 500,
            },
        });

        let url = format!("{}/api/generate", self.auditor_url);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("audit request failed: {e}"))?;

        if !resp.status().is_success() {
            // If the auditor is unavailable, fail open (allow the fact)
            // but record the audit failure. Better to proceed than block
            // the entire loop because the auditor is down.
            return Ok(AuditResult {
                survived: true,
                assessment: "auditor unavailable — fail open".to_string(),
                issues: vec!["auditor service unavailable".to_string()],
                confidence: 0.5,
            });
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse audit response: {e}"))?;

        let response_text = data
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse the response.
        let survived = response_text.contains("SURVIVES")
            && !response_text.contains("- ");

        // Extract issues (lines starting with "- ").
        let issues: Vec<String> = response_text
            .lines()
            .filter(|l| l.trim_start().starts_with("- "))
            .map(|l| l.trim_start_matches("- ").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Extract confidence score.
        let confidence = response_text
            .lines()
            .find_map(|l| {
                let l = l.trim();
                if let Some(rest) = l.strip_prefix("CONFIDENCE:") {
                    rest.trim().parse::<f64>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0.5);

        Ok(AuditResult {
            survived,
            assessment: response_text,
            issues,
            confidence,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_gate_construction() {
        let gate = HostileAuditGate::new("http://localhost:11434", "llama3");
        // Just verify it constructs without error.
        assert_eq!(gate.auditor_model, "llama3");
    }

    #[test]
    fn test_audit_result_serde() {
        let result = AuditResult {
            survived: true,
            assessment: "SURVIVES".to_string(),
            issues: vec![],
            confidence: 0.95,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: AuditResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.survived);
        assert_eq!(parsed.confidence, 0.95);
    }
}
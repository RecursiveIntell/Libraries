//! Governance policy definition and evaluation.

use serde::{Deserialize, Serialize};

use crate::decision::{CodecDecision, CodecProfile};
use crate::error::GovernorError;

/// Content type for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContentType {
    /// Text content
    Text,
    /// Image data
    Image,
    /// Audio data
    Audio,
    /// Video data
    Video,
    /// Structured data
    Structured,
    /// Model weights
    Model,
    /// Other/unknown
    #[default]
    Other,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::Image => write!(f, "image"),
            ContentType::Audio => write!(f, "audio"),
            ContentType::Video => write!(f, "video"),
            ContentType::Structured => write!(f, "structured"),
            ContentType::Model => write!(f, "model"),
            ContentType::Other => write!(f, "other"),
        }
    }
}

/// Admissibility class for content routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AdmissibilityClass {
    /// Critical content requiring highest fidelity
    Critical,
    /// High priority content
    HighPriority,
    /// Standard content
    #[default]
    Standard,
    /// Compressible content
    Compressible,
    /// Best effort content
    BestEffort,
}

/// Request for codec decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRequest {
    /// Content type for routing
    pub content_type: ContentType,

    /// Size of content in bytes
    pub size_bytes: u64,

    /// Required accuracy (0.0 to 1.0)
    pub accuracy_requirement: f64,

    /// Maximum latency tolerance in milliseconds
    pub latency_tolerance_ms: u64,

    /// Admissibility class
    pub admissibility: AdmissibilityClass,
}

impl Default for GovernanceRequest {
    fn default() -> Self {
        Self {
            content_type: ContentType::Other,
            size_bytes: 0,
            accuracy_requirement: 0.95,
            latency_tolerance_ms: 1000,
            admissibility: AdmissibilityClass::Standard,
        }
    }
}

/// Governance policy for codec selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    /// Maximum degradation allowed (0.0 to 1.0)
    max_degradation: f64,

    /// Size threshold for small content bypass (bytes)
    small_content_threshold: u64,

    /// Minimum accuracy for raw codec
    raw_min_accuracy: f64,

    /// Policy name for debugging
    name: String,

    /// Codec profiles admitted by this policy (CMP-001 admission contract).
    admitted_codecs: Vec<CodecProfile>,

    /// Maximum accepted content size in bytes (None = unbounded).
    byte_budget: Option<u64>,

    /// Recorded corpus this admission contract applies to.
    corpus: String,

    /// Recorded exact baseline digest (optional).
    exact_baseline: Option<String>,
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            max_degradation: 0.1,
            small_content_threshold: 256,
            raw_min_accuracy: 0.99,
            name: "default".to_string(),
            // CMP-001: honest default — only Raw has a registered decoder in
            // this workspace until CMP-003 registers real codec decoders.
            admitted_codecs: vec![CodecProfile::Raw],
            byte_budget: None,
            corpus: "unregistered".to_string(),
            exact_baseline: None,
        }
    }
}

impl GovernancePolicy {
    /// Create a new governance policy with custom settings.
    pub fn new(max_degradation: f64, small_content_threshold: u64, raw_min_accuracy: f64) -> Self {
        Self {
            max_degradation,
            small_content_threshold,
            raw_min_accuracy,
            name: "custom".to_string(),
            admitted_codecs: vec![CodecProfile::Raw],
            byte_budget: None,
            corpus: "unregistered".to_string(),
            exact_baseline: None,
        }
    }

    /// Create a policy optimized for storage efficiency.
    pub fn storage_efficient() -> Self {
        Self {
            max_degradation: 0.15,
            small_content_threshold: 512,
            raw_min_accuracy: 0.90,
            name: "storage_efficient".to_string(),
            admitted_codecs: vec![CodecProfile::Raw, CodecProfile::Q8, CodecProfile::Q4],
            byte_budget: None,
            corpus: "storage_bench".to_string(),
            exact_baseline: None,
        }
    }

    /// Create a policy optimized for low latency.
    pub fn low_latency() -> Self {
        Self {
            max_degradation: 0.12,
            small_content_threshold: 1024,
            raw_min_accuracy: 0.92,
            name: "low_latency".to_string(),
            admitted_codecs: vec![CodecProfile::Raw, CodecProfile::Q8, CodecProfile::Turbo],
            byte_budget: None,
            corpus: "latency_bench".to_string(),
            exact_baseline: None,
        }
    }

    /// Create a policy optimized for accuracy.
    pub fn accuracy_oriented() -> Self {
        Self {
            max_degradation: 0.05,
            small_content_threshold: 128,
            raw_min_accuracy: 0.999,
            name: "accuracy_oriented".to_string(),
            admitted_codecs: vec![CodecProfile::Raw, CodecProfile::Q8, CodecProfile::Fib],
            byte_budget: None,
            corpus: "accuracy_bench".to_string(),
            exact_baseline: None,
        }
    }

    /// Evaluate a governance request and produce a codec decision.
    pub fn evaluate(&self, request: GovernanceRequest) -> Result<CodecDecision, GovernorError> {
        // CMP-001: byte budget is enforced before any routing.
        if let Some(budget) = self.byte_budget {
            if request.size_bytes > budget {
                return Err(GovernorError::ByteBudgetExceeded {
                    requested_bytes: request.size_bytes,
                    budget_bytes: budget,
                });
            }
        }

        // Small content bypass
        if request.size_bytes <= self.small_content_threshold
            && request.admissibility != AdmissibilityClass::Critical
        {
            return Ok(CodecDecision::direct(
                CodecProfile::Raw,
                self.max_degradation,
            ));
        }

        // Critical content always gets raw
        if request.accuracy_requirement >= self.raw_min_accuracy
            || request.admissibility == AdmissibilityClass::Critical
        {
            return Ok(CodecDecision::direct(CodecProfile::Raw, 0.0));
        }

        // Select codec based on content type and requirements
        let codec = self.select_codec(&request)?;

        // CMP-001: admission gate — a selected codec without a registered/
        // admitted decoder is rejected with a typed error, never silently
        // substituted with raw or another codec.
        if !self.admitted_codecs.contains(&codec) {
            return Err(GovernorError::UnsupportedCodec {
                profile: codec,
                policy: self.name.clone(),
                reason: "codec has no registered decoder admitted by this policy".to_string(),
            });
        }

        // CMP-001: latency budget — a selected codec whose declared latency
        // estimate exceeds the request tolerance is rejected.
        let estimated_ms = codec.estimated_latency_ms();
        if estimated_ms > request.latency_tolerance_ms {
            return Err(GovernorError::LatencyBudgetExceeded {
                profile: codec,
                requested_ms: request.latency_tolerance_ms,
                estimated_ms,
            });
        }

        let degradation = codec.default_degradation_threshold();

        Ok(CodecDecision::direct(codec, degradation))
    }

    /// Select appropriate codec based on request.
    fn select_codec(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        match request.content_type {
            ContentType::Text => self.select_for_text(request),
            ContentType::Image => self.select_for_image(request),
            ContentType::Audio => self.select_for_audio(request),
            ContentType::Video => self.select_for_video(request),
            ContentType::Structured => self.select_for_structured(request),
            ContentType::Model => self.select_for_model(request),
            ContentType::Other => Ok(CodecProfile::Q8),
        }
    }

    fn select_for_text(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.accuracy_requirement >= 0.98 {
            Ok(CodecProfile::Raw)
        } else if request.size_bytes > 1_000_000 {
            Ok(CodecProfile::Turbo)
        } else {
            Ok(CodecProfile::Q8)
        }
    }

    fn select_for_image(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.accuracy_requirement >= 0.95 {
            Ok(CodecProfile::Q8)
        } else if request.size_bytes > 5_000_000 {
            Ok(CodecProfile::Q4)
        } else {
            Ok(CodecProfile::Q8)
        }
    }

    fn select_for_audio(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.latency_tolerance_ms < 100 {
            Ok(CodecProfile::Turbo)
        } else if request.accuracy_requirement >= 0.97 {
            Ok(CodecProfile::Fib)
        } else {
            Ok(CodecProfile::Q8)
        }
    }

    fn select_for_video(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.latency_tolerance_ms < 50 {
            Ok(CodecProfile::Turbo)
        } else {
            Ok(CodecProfile::Q4)
        }
    }

    fn select_for_structured(
        &self,
        request: &GovernanceRequest,
    ) -> Result<CodecProfile, GovernorError> {
        if request.accuracy_requirement >= 0.99 {
            Ok(CodecProfile::Raw)
        } else {
            Ok(CodecProfile::Q8)
        }
    }

    fn select_for_model(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.admissibility == AdmissibilityClass::Critical {
            Ok(CodecProfile::Raw)
        } else if request.accuracy_requirement >= 0.98 {
            Ok(CodecProfile::Fib)
        } else if request.size_bytes > 100_000_000 {
            Ok(CodecProfile::Q4)
        } else {
            Ok(CodecProfile::Q8)
        }
    }

    /// Returns the policy name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns max degradation setting.
    pub fn max_degradation(&self) -> f64 {
        self.max_degradation
    }

    /// Returns the codec profiles admitted by this policy (CMP-001).
    pub fn admitted_codecs(&self) -> &[CodecProfile] {
        &self.admitted_codecs
    }

    /// Builder: admit the given codec profiles (registered decoders).
    pub fn with_admitted_codecs<I: IntoIterator<Item = CodecProfile>>(mut self, codecs: I) -> Self {
        self.admitted_codecs = codecs.into_iter().collect();
        self
    }

    /// Returns the configured byte budget (None = unbounded).
    pub fn byte_budget(&self) -> Option<u64> {
        self.byte_budget
    }

    /// Builder: set the maximum accepted content size in bytes.
    pub fn with_byte_budget(mut self, budget: u64) -> Self {
        self.byte_budget = Some(budget);
        self
    }

    /// Returns the recorded corpus for this admission contract.
    pub fn corpus(&self) -> &str {
        &self.corpus
    }

    /// Builder: record the corpus this admission contract applies to.
    pub fn with_corpus(mut self, corpus: impl Into<String>) -> Self {
        self.corpus = corpus.into();
        self
    }

    /// Returns the recorded exact baseline digest (optional).
    pub fn exact_baseline(&self) -> Option<&str> {
        self.exact_baseline.as_deref()
    }

    /// Builder: record the exact baseline digest.
    pub fn with_exact_baseline(mut self, baseline: impl Into<String>) -> Self {
        self.exact_baseline = Some(baseline.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_evaluation() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest::default();

        let result = policy.evaluate(request);
        assert!(result.is_ok());
    }

    #[test]
    fn small_content_bypass() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            size_bytes: 100,
            admissibility: AdmissibilityClass::Standard,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Raw);
    }

    #[test]
    fn critical_content_gets_raw() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            admissibility: AdmissibilityClass::Critical,
            accuracy_requirement: 0.5,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Raw);
    }

    #[test]
    fn image_content_routing() {
        // CMP-001: routing to Q4 requires an admitting policy (registered
        // decoder); storage_efficient admits Raw/Q8/Q4 explicitly.
        let policy = GovernancePolicy::storage_efficient();

        // Large image with lower accuracy gets Q4
        let request = GovernanceRequest {
            content_type: ContentType::Image,
            size_bytes: 10_000_000,
            accuracy_requirement: 0.8,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Q4);
    }

    #[test]
    fn default_policy_rejects_unadmitted_codec() {
        // CMP-001 RED: the default policy admits only Raw; a request that
        // routes to Turbo must be rejected with a typed UnsupportedCodec,
        // never silently substituted.
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            content_type: ContentType::Text,
            size_bytes: 2_000_000,
            accuracy_requirement: 0.9,
            ..Default::default()
        };

        let err = policy.evaluate(request).unwrap_err();
        match err {
            GovernorError::UnsupportedCodec { profile, .. } => {
                assert_eq!(profile, CodecProfile::Turbo);
            }
            other => panic!("expected UnsupportedCodec, got {other:?}"),
        }
    }

    #[test]
    fn admitted_codec_is_selected_when_explicitly_admitted() {
        // CMP-001: explicitly admitting Turbo makes the same request succeed.
        let policy = GovernancePolicy::default().with_admitted_codecs([
            CodecProfile::Raw,
            CodecProfile::Q8,
            CodecProfile::Turbo,
        ]);
        let request = GovernanceRequest {
            content_type: ContentType::Text,
            size_bytes: 2_000_000,
            accuracy_requirement: 0.9,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Turbo);
    }

    #[test]
    fn latency_budget_exceeded_rejected_typed() {
        // CMP-001: Q8 declares 5ms latency; a 1ms tolerance must reject.
        let policy =
            GovernancePolicy::default().with_admitted_codecs([CodecProfile::Raw, CodecProfile::Q8]);
        let request = GovernanceRequest {
            content_type: ContentType::Text,
            size_bytes: 100_000,
            accuracy_requirement: 0.9,
            latency_tolerance_ms: 1,
            ..Default::default()
        };

        let err = policy.evaluate(request).unwrap_err();
        match err {
            GovernorError::LatencyBudgetExceeded {
                profile,
                requested_ms,
                estimated_ms,
            } => {
                assert_eq!(profile, CodecProfile::Q8);
                assert_eq!(requested_ms, 1);
                assert_eq!(estimated_ms, 5);
            }
            other => panic!("expected LatencyBudgetExceeded, got {other:?}"),
        }
    }

    #[test]
    fn byte_budget_exceeded_rejected_typed() {
        // CMP-001: content larger than the byte budget is rejected before
        // any routing.
        let policy = GovernancePolicy::default().with_byte_budget(1000);
        let request = GovernanceRequest {
            size_bytes: 2000,
            ..Default::default()
        };

        let err = policy.evaluate(request).unwrap_err();
        match err {
            GovernorError::ByteBudgetExceeded {
                requested_bytes,
                budget_bytes,
            } => {
                assert_eq!(requested_bytes, 2000);
                assert_eq!(budget_bytes, 1000);
            }
            other => panic!("expected ByteBudgetExceeded, got {other:?}"),
        }
    }

    #[test]
    fn admission_contract_records_corpus_baseline_and_budgets() {
        // CMP-001: the admission contract records supported codecs, corpus,
        // exact baseline, and budgets — all queryable.
        let policy = GovernancePolicy::default()
            .with_admitted_codecs([CodecProfile::Raw, CodecProfile::Q8])
            .with_corpus("beir-nfcorpus")
            .with_exact_baseline("sha256:deadbeef")
            .with_byte_budget(64 * 1024 * 1024);

        assert_eq!(
            policy.admitted_codecs(),
            &[CodecProfile::Raw, CodecProfile::Q8]
        );
        assert_eq!(policy.corpus(), "beir-nfcorpus");
        assert_eq!(policy.exact_baseline(), Some("sha256:deadbeef"));
        assert_eq!(policy.byte_budget(), Some(64 * 1024 * 1024));
    }

    #[test]
    fn model_content_routing() {
        let policy = GovernancePolicy::default();

        // Critical model gets raw
        let request = GovernanceRequest {
            content_type: ContentType::Model,
            admissibility: AdmissibilityClass::Critical,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Raw);
    }

    #[test]
    fn low_latency_audio_gets_turbo() {
        // Use low_latency policy so small_content_threshold doesn't block
        // the latency-sensitive path before the audio routing decision fires
        let policy = GovernancePolicy::low_latency();
        let request = GovernanceRequest {
            content_type: ContentType::Audio,
            size_bytes: 2000, // exceeds small_content_threshold of 1024
            latency_tolerance_ms: 50,
            accuracy_requirement: 0.8,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Turbo);
    }

    #[test]
    fn policy_presets() {
        let storage = GovernancePolicy::storage_efficient();
        assert_eq!(storage.name(), "storage_efficient");

        let latency = GovernancePolicy::low_latency();
        assert_eq!(latency.name(), "low_latency");

        let accuracy = GovernancePolicy::accuracy_oriented();
        assert_eq!(accuracy.name(), "accuracy_oriented");
    }
}

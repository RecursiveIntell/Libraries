//! Governance policy definition and evaluation.

use serde::{Deserialize, Serialize};

use crate::decision::{CodecDecision, CodecProfile, GovernanceDecisionReceipt};
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
    /// Vector embeddings
    Embedding,
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
            ContentType::Embedding => write!(f, "embedding"),
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
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            max_degradation: 0.1,
            small_content_threshold: 256,
            raw_min_accuracy: 0.99,
            name: "default".to_string(),
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
        }
    }

    /// Create a policy optimized for storage efficiency.
    pub fn storage_efficient() -> Self {
        Self {
            max_degradation: 0.15,
            small_content_threshold: 512,
            raw_min_accuracy: 0.90,
            name: "storage_efficient".to_string(),
        }
    }

    /// Create a policy optimized for low latency.
    pub fn low_latency() -> Self {
        Self {
            max_degradation: 0.12,
            small_content_threshold: 1024,
            raw_min_accuracy: 0.92,
            name: "low_latency".to_string(),
        }
    }

    /// Create a policy optimized for accuracy.
    pub fn accuracy_oriented() -> Self {
        Self {
            max_degradation: 0.05,
            small_content_threshold: 128,
            raw_min_accuracy: 0.999,
            name: "accuracy_oriented".to_string(),
        }
    }

    /// Evaluate a governance request and produce a codec decision.
    pub fn evaluate(&self, request: GovernanceRequest) -> Result<CodecDecision, GovernorError> {
        // Small content bypass
        if request.size_bytes <= self.small_content_threshold
            && request.admissibility != AdmissibilityClass::Critical
        {
            return Ok(CodecDecision::with_governance(
                CodecProfile::Raw,
                self.max_degradation,
                GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: request.content_type.to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale: "Small content bypass: routing directly to Raw.".to_string(),
                    blocked_profiles: vec![],
                    candidate_profiles: vec![CodecProfile::Raw],
                },
            ));
        }

        // Critical content always gets raw
        if request.accuracy_requirement >= self.raw_min_accuracy
            || request.admissibility == AdmissibilityClass::Critical
        {
            return Ok(CodecDecision::with_governance(
                CodecProfile::Raw,
                0.0,
                GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: request.content_type.to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale: "Critical policy path: raw codec required by accuracy or admissibility.".to_string(),
                    blocked_profiles: vec![CodecProfile::Q8, CodecProfile::Q4],
                    candidate_profiles: vec![CodecProfile::Raw],
                },
            ));
        }

        // Select codec and optional governance trace for policy-facing decisions.
        let (codec, governance_trace) = self.select_codec(&request)?;
        let degradation = codec.default_degradation_threshold();

        match governance_trace {
            Some(trace) => Ok(CodecDecision::with_governance(
                codec,
                degradation,
                trace,
            )),
            None => Ok(CodecDecision::direct(codec, degradation)),
        }
    }

    /// Select appropriate codec based on request.
    fn select_codec(
        &self,
        request: &GovernanceRequest,
    ) -> Result<(CodecProfile, Option<GovernanceDecisionReceipt>), GovernorError> {
        match request.content_type {
            ContentType::Text => Ok((self.select_for_text(request)?, None)),
            ContentType::Image => Ok((self.select_for_image(request)?, None)),
            ContentType::Audio => Ok((self.select_for_audio(request)?, None)),
            ContentType::Video => Ok((self.select_for_video(request)?, None)),
            ContentType::Structured => Ok((self.select_for_structured(request)?, None)),
            ContentType::Embedding => self.select_for_embedding(request),
            ContentType::Model => Ok((self.select_for_model(request)?, None)),
            ContentType::Other => Ok((CodecProfile::Q8, None)),
        }
    }

    fn select_for_text(&self, request: &GovernanceRequest) -> Result<CodecProfile, GovernorError> {
        if request.accuracy_requirement >= 0.98 {
            Ok(CodecProfile::Raw)
        } else if request.size_bytes > 1_000_000 {
            Ok(CodecProfile::Turbo)
        } else if request.latency_tolerance_ms < 50 {
            // Low-latency text workloads (search, retrieval, RAG re-rank)
            // benefit from QJL's constant-size sketches and Polar's
            // asymmetric inner-product scoring. Pick the right one based
            // on the size budget: tiny vectors use Polar for finer
            // granularity, large vectors use QJL for fixed-cost sketches.
            if request.size_bytes > 50_000 {
                Ok(CodecProfile::Qjl)
            } else {
                Ok(CodecProfile::Polar)
            }
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

    fn select_for_embedding(
        &self,
        request: &GovernanceRequest,
    ) -> Result<(CodecProfile, Option<GovernanceDecisionReceipt>), GovernorError> {
        if request.accuracy_requirement >= self.raw_min_accuracy {
            return Ok((CodecProfile::Raw, None));
        }

        if request.admissibility == AdmissibilityClass::HighPriority {
            if request.size_bytes > 1_000_000 && self.within_budget(CodecProfile::Q4) {
                return Ok((
                    CodecProfile::Q4,
                    Some(GovernanceDecisionReceipt {
                        policy_name: self.name.clone(),
                        content_type: "Embedding".to_string(),
                        admissibility: format!("{request:?}", request = request.admissibility),
                        rationale: "HighPriority embedding routes avoid HyperQuant by policy.".to_string(),
                        blocked_profiles: vec![CodecProfile::Hyperquant],
                        candidate_profiles: vec![
                            CodecProfile::Hyperquant,
                            CodecProfile::Q4,
                            CodecProfile::Q8,
                            CodecProfile::Turbo,
                        ],
                    }),
                ));
            }
            return Ok((
                CodecProfile::Q8,
                Some(GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: "Embedding".to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale:
                        "HighPriority embedding did not satisfy Q4 threshold, fallback to Q8 by policy.".to_string(),
                    blocked_profiles: vec![CodecProfile::Hyperquant, CodecProfile::Q4],
                    candidate_profiles: vec![CodecProfile::Hyperquant, CodecProfile::Q4, CodecProfile::Q8],
                }),
            ));
        }

        // Low-latency policy should stay on asymmetric/sketch codecs and avoid
        // lattice quantization in the hot-path to preserve first-pass latency.
        if self.name == "low_latency"
            && request.latency_tolerance_ms < 100
            && request.size_bytes >= 256_000
            && self.within_budget(CodecProfile::Turbo)
        {
            return Ok((
                CodecProfile::Turbo,
                Some(GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: "Embedding".to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale:
                        "Low-latency embedding routing prefers Turbo over HyperQuant to satisfy latency policy.".to_string(),
                    blocked_profiles: vec![CodecProfile::Hyperquant, CodecProfile::Q4],
                    candidate_profiles: vec![
                        CodecProfile::Hyperquant,
                        CodecProfile::Turbo,
                        CodecProfile::Q8,
                    ],
                }),
            ));
        }

        let storage_oriented = self.name == "storage_efficient";
        let accuracy_oriented = self.name == "accuracy_oriented";

        let prefer_hyperquant = if accuracy_oriented {
            false
        } else if storage_oriented {
            request.accuracy_requirement <= 0.95
                && (request.size_bytes >= 256_000
                    || request.admissibility == AdmissibilityClass::BestEffort)
                && self.within_budget(CodecProfile::Hyperquant)
        } else {
            request.accuracy_requirement <= 0.90
                && (request.size_bytes >= 1_000_000
                    || request.admissibility == AdmissibilityClass::BestEffort)
                && self.within_budget(CodecProfile::Hyperquant)
        };

        if prefer_hyperquant {
            return Ok((
                CodecProfile::Hyperquant,
                Some(GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: "Embedding".to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale: if storage_oriented {
                        "storage_efficient policy admitted HyperQuant under budget and threshold requirements."
                            .to_string()
                    } else {
                        "Default policy admitted HyperQuant under budget and size threshold requirements."
                            .to_string()
                    },
                    blocked_profiles: vec![],
                    candidate_profiles: vec![
                        CodecProfile::Hyperquant,
                        CodecProfile::Q4,
                        CodecProfile::Q8,
                        CodecProfile::Turbo,
                    ],
                }),
            ));
        }

        let blocked = if self.within_budget(CodecProfile::Q4) {
            Vec::new()
        } else {
            vec![CodecProfile::Q4]
        };

        if request.size_bytes > 1_000_000 && self.within_budget(CodecProfile::Q4) {
            Ok((
                CodecProfile::Q4,
                Some(GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: "Embedding".to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale: "Embedding is large, but HyperQuant not admitted for this policy profile.".to_string(),
                    blocked_profiles: {
                        let mut blocked_profiles = vec![CodecProfile::Hyperquant];
                        if storage_oriented {
                            blocked_profiles.push(CodecProfile::Turbo);
                        }
                        blocked_profiles
                    },
                    candidate_profiles: vec![
                        CodecProfile::Hyperquant,
                        CodecProfile::Q4,
                        CodecProfile::Q8,
                        CodecProfile::Turbo,
                    ],
                }),
            ))
        } else {
            Ok((
                CodecProfile::Q8,
                Some(GovernanceDecisionReceipt {
                    policy_name: self.name.clone(),
                    content_type: "Embedding".to_string(),
                    admissibility: format!("{request:?}", request = request.admissibility),
                    rationale: if self.within_budget(CodecProfile::Hyperquant) {
                        "HyperQuant was blocked by profile/size gate; Q4 would miss quality threshold or size is low.".to_string()
                    } else {
                        "HyperQuant was blocked by policy max_degradation budget.".to_string()
                    },
                    blocked_profiles: if blocked.is_empty() {
                        vec![CodecProfile::Hyperquant]
                    } else {
                        let mut blocked_profiles = vec![CodecProfile::Hyperquant];
                        blocked_profiles.extend(blocked);
                        blocked_profiles
                    },
                    candidate_profiles: vec![
                        CodecProfile::Hyperquant,
                        CodecProfile::Q4,
                        CodecProfile::Q8,
                    ],
                }),
            ))
        }
    }

    fn within_budget(&self, profile: CodecProfile) -> bool {
        profile.default_degradation_threshold() <= self.max_degradation
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
        let policy = GovernancePolicy::default();

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
    fn embedding_large_best_effort_prefers_hyperquant() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            content_type: ContentType::Embedding,
            size_bytes: 2_000_000,
            accuracy_requirement: 0.88,
            latency_tolerance_ms: 200,
            admissibility: AdmissibilityClass::BestEffort,
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Hyperquant);

        if let crate::decision::CodecReceipt::Governance { governance, .. } = decision.receipt {
            assert_eq!(governance.policy_name, "default");
            assert!(governance.rationale.contains("HyperQuant"));
            assert_eq!(governance.blocked_profiles.len(), 0);
        } else {
            panic!("expected governance receipt for embedding decision");
        }
    }

    #[test]
    fn embedding_rejects_hyperquant_when_budget_disallows_or_size_is_low() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            content_type: ContentType::Embedding,
            size_bytes: 16_000,
            accuracy_requirement: 0.97,
            latency_tolerance_ms: 200,
            ..Default::default()
        };

        let decision = policy.evaluate(request).unwrap();
        assert_eq!(decision.codec, CodecProfile::Q8);

        if let crate::decision::CodecReceipt::Governance { governance, .. } = decision.receipt {
            assert_eq!(governance.policy_name, "default");
            assert!(governance.rationale.contains("HyperQuant was blocked"));
            assert!(governance.blocked_profiles.contains(&CodecProfile::Hyperquant));
            assert!(governance.content_type.eq("Embedding"));
        } else {
            panic!("expected governance receipt for embedding decision");
        }
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

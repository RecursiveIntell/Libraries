//! Integration tests for quant-governor policy routing.

use quant_governor::{
    evaluate, AdmissibilityClass, CodecDecision, CodecProfile, ContentType, GovernancePolicy,
    GovernanceRequest,
};

/// Test 1: strict_policy_requires_exact_fallback — accuracy 0.999 → Raw, degradation 0.
#[test]
fn strict_policy_requires_exact_fallback() {
    let policy = GovernancePolicy::accuracy_oriented();
    let request = GovernanceRequest {
        content_type: ContentType::Structured,
        size_bytes: 1000,
        accuracy_requirement: 0.999,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision = evaluate(request, &policy).unwrap();
    assert_eq!(decision.codec, CodecProfile::Raw);
    assert_eq!(decision.degradation_budget, 0.0);
}

/// Test 2: audio_low_latency_selects_turbo — latency<100ms → Turbo codec.
#[test]
fn audio_low_latency_selects_turbo() {
    // Use low_latency policy to avoid small_content_threshold bypassing
    let policy = GovernancePolicy::low_latency();
    let request = GovernanceRequest {
        content_type: ContentType::Audio,
        size_bytes: 2000, // exceeds small_content_threshold of 1024 in low_latency
        latency_tolerance_ms: 50, // < 100ms threshold
        accuracy_requirement: 0.8,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision = evaluate(request, &policy).unwrap();
    assert_eq!(decision.codec, CodecProfile::Turbo);
}

/// Test 3: model_critical_selects_raw — Critical admissibility → Raw regardless of accuracy.
#[test]
fn model_critical_selects_raw() {
    let policy = GovernancePolicy::default();
    let request = GovernanceRequest {
        content_type: ContentType::Model,
        size_bytes: 1_000_000,
        accuracy_requirement: 0.5, // low accuracy, but Critical should override
        latency_tolerance_ms: 1000,
        admissibility: AdmissibilityClass::Critical,
    };

    let decision = evaluate(request, &policy).unwrap();
    assert_eq!(decision.codec, CodecProfile::Raw);
}

/// Test 4: degradation_budget_accounted — degradation budget consumed in receipt.
#[test]
fn degradation_budget_accounted() {
    let policy = GovernancePolicy::default();
    let request = GovernanceRequest {
        content_type: ContentType::Image,
        size_bytes: 10_000_000,
        accuracy_requirement: 0.8,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision = evaluate(request, &policy).unwrap();
    // Large image with lower accuracy should get Q4
    assert_eq!(decision.codec, CodecProfile::Q4);
    // Q4 has default degradation threshold of 0.10
    assert_eq!(decision.degradation_budget, 0.10);
}

/// Test 5: content_type_routing_matrix — verify all 7 content types route correctly.
#[test]
fn content_type_routing_matrix() {
    let policy = GovernancePolicy::default();

    // Text — high accuracy gets Raw
    let text_req = GovernanceRequest {
        content_type: ContentType::Text,
        size_bytes: 1000,
        accuracy_requirement: 0.99,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };
    let text_decision = evaluate(text_req, &policy).unwrap();
    assert_eq!(text_decision.codec, CodecProfile::Raw);

    // Image — large size with low accuracy gets Q4
    let image_req = GovernanceRequest {
        content_type: ContentType::Image,
        size_bytes: 10_000_000,
        accuracy_requirement: 0.8,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };
    let image_decision = evaluate(image_req, &policy).unwrap();
    assert_eq!(image_decision.codec, CodecProfile::Q4);

    // Audio — latency < 100ms gets Turbo
    let audio_req = GovernanceRequest {
        content_type: ContentType::Audio,
        size_bytes: 2000,
        latency_tolerance_ms: 50,
        accuracy_requirement: 0.8,
        admissibility: AdmissibilityClass::Standard,
    };
    let audio_decision = evaluate(audio_req, &policy).unwrap();
    assert_eq!(audio_decision.codec, CodecProfile::Turbo);

    // Video — latency < 50ms gets Turbo
    let video_req = GovernanceRequest {
        content_type: ContentType::Video,
        size_bytes: 1_000_000,
        latency_tolerance_ms: 40,
        accuracy_requirement: 0.8,
        admissibility: AdmissibilityClass::Standard,
    };
    let video_decision = evaluate(video_req, &policy).unwrap();
    assert_eq!(video_decision.codec, CodecProfile::Turbo);

    // Structured — high accuracy gets Raw
    let structured_req = GovernanceRequest {
        content_type: ContentType::Structured,
        size_bytes: 1000,
        accuracy_requirement: 0.99,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };
    let structured_decision = evaluate(structured_req, &policy).unwrap();
    assert_eq!(structured_decision.codec, CodecProfile::Raw);

    // Model — Critical gets Raw regardless of accuracy
    let model_req = GovernanceRequest {
        content_type: ContentType::Model,
        size_bytes: 1_000_000,
        accuracy_requirement: 0.5,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Critical,
    };
    let model_decision = evaluate(model_req, &policy).unwrap();
    assert_eq!(model_decision.codec, CodecProfile::Raw);

    // Other — defaults to Q8
    let other_req = GovernanceRequest {
        content_type: ContentType::Other,
        size_bytes: 1000,
        accuracy_requirement: 0.8,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };
    let other_decision = evaluate(other_req, &policy).unwrap();
    assert_eq!(other_decision.codec, CodecProfile::Q8);
}

/// Test: small content bypasses compression.
#[test]
fn small_content_bypasses_compression() {
    let policy = GovernancePolicy::default();
    let request = GovernanceRequest {
        content_type: ContentType::Text,
        size_bytes: 100, // below small_content_threshold of 256
        accuracy_requirement: 0.8,
        latency_tolerance_ms: 500,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision = evaluate(request, &policy).unwrap();
    // Small content should bypass compression and use Raw
    assert_eq!(decision.codec, CodecProfile::Raw);
}

/// Test: codec profile degradation thresholds.
#[test]
fn codec_profile_degradation_thresholds() {
    assert_eq!(CodecProfile::Raw.default_degradation_threshold(), 0.0);
    assert_eq!(CodecProfile::Q8.default_degradation_threshold(), 0.05);
    assert_eq!(CodecProfile::Q4.default_degradation_threshold(), 0.10);
    assert_eq!(CodecProfile::Turbo.default_degradation_threshold(), 0.08);
    assert_eq!(CodecProfile::Fib.default_degradation_threshold(), 0.03);
}

/// Test: codec decision is direct (no fallback) for standard case.
#[test]
fn codec_decision_is_direct() {
    let decision = CodecDecision::direct(CodecProfile::Q8, 0.05);
    assert_eq!(decision.codec, CodecProfile::Q8);
    assert!(!decision.exact_fallback);
    assert!(!decision.had_fallback());
}

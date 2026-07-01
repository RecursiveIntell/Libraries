use hyperquant::LatticeKind;
use quant_eval::{
    run_governed_hyperquant_eval, GovernedHyperQuantEvalConfig, GovernedHyperQuantPolicyPreset,
    HyperQuantEvalConfig,
};
use quant_governor::AdmissibilityClass;

fn base_config(policy_preset: GovernedHyperQuantPolicyPreset) -> GovernedHyperQuantEvalConfig {
    GovernedHyperQuantEvalConfig {
        fixture: HyperQuantEvalConfig {
            dim: 16,
            vectors: 16,
            seed: 7,
            scale: 8.0,
        },
        policy_preset,
        size_bytes: 500_000,
        accuracy_requirement: 0.89,
        latency_tolerance_ms: 200,
        admissibility: AdmissibilityClass::Standard,
    }
}

#[test]
fn governed_storage_efficient_admits_hyperquant_with_measured_receipt() {
    let receipt = run_governed_hyperquant_eval(&base_config(
        GovernedHyperQuantPolicyPreset::StorageEfficient,
    ))
    .expect("governed eval succeeds");

    assert!(receipt.admitted);
    assert_eq!(receipt.decision.selected_codec, "hyperquant");
    assert_eq!(receipt.decision.policy_name, "storage_efficient");
    assert!(receipt.decision.rationale.contains("HyperQuant"));
    assert!(receipt
        .selected_hyperquant_profile
        .as_ref()
        .is_some_and(|profile| profile.kind == LatticeKind::D4));
    assert!(receipt
        .baselines
        .iter()
        .any(|baseline| baseline.codec == "hyperquant" && baseline.compression_ratio > 1.0));
    assert!(receipt
        .hyperquant_eval
        .profiles
        .iter()
        .all(|profile| profile.receipt_count == 16));
}

#[test]
fn governed_low_latency_blocks_hyperquant_with_rationale() {
    let mut config = base_config(GovernedHyperQuantPolicyPreset::LowLatency);
    config.size_bytes = 2_000_000;
    config.accuracy_requirement = 0.85;
    config.latency_tolerance_ms = 60;

    let receipt = run_governed_hyperquant_eval(&config).expect("governed eval succeeds");

    assert!(!receipt.admitted);
    assert_eq!(receipt.decision.selected_codec, "turbo");
    assert!(receipt
        .decision
        .rationale
        .to_lowercase()
        .contains("latency"));
    assert!(receipt
        .decision
        .blocked_profiles
        .iter()
        .any(|profile| profile == "hyperquant"));
}

#[test]
fn governed_accuracy_oriented_blocks_hyperquant() {
    let mut config = base_config(GovernedHyperQuantPolicyPreset::AccuracyOriented);
    config.accuracy_requirement = 0.93;

    let receipt = run_governed_hyperquant_eval(&config).expect("governed eval succeeds");

    assert!(!receipt.admitted);
    assert_eq!(receipt.decision.selected_codec, "q8");
    assert!(receipt
        .decision
        .blocked_profiles
        .iter()
        .any(|profile| profile == "hyperquant"));
}

#[test]
fn governed_strict_budget_blocks_hyperquant_and_q4() {
    let mut config = base_config(GovernedHyperQuantPolicyPreset::CustomStrict);
    config.size_bytes = 2_000_000;
    config.accuracy_requirement = 0.80;
    config.latency_tolerance_ms = 500;

    let receipt = run_governed_hyperquant_eval(&config).expect("governed eval succeeds");

    assert!(!receipt.admitted);
    assert_eq!(receipt.decision.selected_codec, "q8");
    assert!(receipt
        .decision
        .blocked_profiles
        .iter()
        .any(|profile| profile == "hyperquant"));
    assert!(receipt
        .decision
        .blocked_profiles
        .iter()
        .any(|profile| profile == "q4"));
}

#[test]
fn governed_hyperquant_receipt_round_trips_json() {
    let receipt = run_governed_hyperquant_eval(&base_config(
        GovernedHyperQuantPolicyPreset::StorageEfficient,
    ))
    .expect("governed eval succeeds");

    let json = serde_json::to_string(&receipt).expect("receipt serializes");
    let decoded: quant_eval::GovernedHyperQuantEvalReceipt =
        serde_json::from_str(&json).expect("receipt deserializes");

    assert_eq!(decoded, receipt);
}

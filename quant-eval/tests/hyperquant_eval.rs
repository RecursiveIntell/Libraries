use hyperquant::LatticeKind;
use quant_eval::{run_hyperquant_eval, HyperQuantEvalConfig};

#[test]
fn hyperquant_eval_reports_z1_and_a2_profiles() {
    let config = HyperQuantEvalConfig {
        dim: 8,
        vectors: 16,
        seed: 7,
        scale: 8.0,
    };

    let result = run_hyperquant_eval(&config).expect("hyperquant eval succeeds");

    assert_eq!(result.config.dim, 8);
    assert_eq!(result.config.vectors, 16);
    assert_eq!(result.profiles.len(), 2);
    assert_eq!(result.profiles[0].kind, LatticeKind::Z1);
    assert_eq!(result.profiles[1].kind, LatticeKind::A2);
    assert!(result
        .profiles
        .iter()
        .all(|profile| profile.mean_mse.is_finite()));
    assert!(result
        .profiles
        .iter()
        .all(|profile| profile.max_mse.is_finite()));
    assert!(result
        .profiles
        .iter()
        .all(|profile| profile.rejected_vectors == 0));
    assert!(result
        .profiles
        .iter()
        .all(|profile| profile.receipt_count == 16));
}

#[test]
fn hyperquant_eval_a2_matches_or_beats_z1_on_triangular_fixture() {
    let config = HyperQuantEvalConfig::triangular_fixture();

    let result = run_hyperquant_eval(&config).expect("hyperquant eval succeeds");
    let z1 = result.profile(LatticeKind::Z1).expect("z1 profile exists");
    let a2 = result.profile(LatticeKind::A2).expect("a2 profile exists");

    assert!(
        a2.mean_mse <= z1.mean_mse,
        "a2 mean {} > z1 mean {}",
        a2.mean_mse,
        z1.mean_mse
    );
    assert!(
        a2.max_mse <= z1.max_mse,
        "a2 max {} > z1 max {}",
        a2.max_mse,
        z1.max_mse
    );
}

#[test]
fn hyperquant_eval_rejects_invalid_config() {
    let config = HyperQuantEvalConfig {
        dim: 0,
        vectors: 4,
        seed: 1,
        scale: 1.0,
    };

    assert!(run_hyperquant_eval(&config).is_err());
}

#[test]
fn hyperquant_eval_result_is_json_serializable() {
    let result = run_hyperquant_eval(&HyperQuantEvalConfig::triangular_fixture())
        .expect("hyperquant eval succeeds");

    let json = serde_json::to_string(&result).expect("result serializes");
    let decoded: quant_eval::HyperQuantEvalResult =
        serde_json::from_str(&json).expect("result deserializes");

    assert_eq!(decoded, result);
}

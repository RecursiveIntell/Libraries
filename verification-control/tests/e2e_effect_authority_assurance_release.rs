use serde_json::Value;
use std::{fs, path::PathBuf};

fn load_bundle(path: PathBuf) -> Value {
    let text = fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn require_value<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(key).expect("missing key in fixture");
    }
    cursor
}

fn require_str<'a>(value: &'a Value, path: &[&str]) -> &'a str {
    require_value(value, path)
        .as_str()
        .expect("expected string in fixture")
}

#[test]
fn e2e_effect_authority_assurance_demo_chain_is_replayable_and_ready_for_release() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = root.parent().unwrap();

    let demo = load_bundle(
        repo_root.join("contracts/fixtures/demo/effect_authority_assurance_release.bundle.json"),
    );
    let v21 = load_bundle(repo_root.join("contracts/fixtures/v21/effect_happy_path.bundle.json"));
    let v22 = load_bundle(
        repo_root.join("contracts/fixtures/v22/delegated_effect_happy_path.bundle.json"),
    );
    let v23 = load_bundle(repo_root.join("contracts/fixtures/v23/release_happy_path.bundle.json"));

    assert_eq!(
        require_str(&demo, &["fixture_name"]),
        "effect_authority_assurance_release"
    );
    assert_eq!(require_str(&demo, &["wave"]), "v21-v22-v23");

    let v21_exec_receipt = require_str(
        &v21,
        &[
            "artifacts",
            "EffectExecutionReceiptV1",
            "effect_execution_receipt_id",
        ],
    );
    let v21_commit = require_str(
        &v21,
        &["artifacts", "EffectCommitDecisionV1", "decision_state"],
    );
    let v21_observation = require_str(
        &v21,
        &[
            "artifacts",
            "EffectObservationBundleV1",
            "observation_state",
        ],
    );

    assert_eq!(v21_commit, "authorized");
    assert_eq!(v21_observation, "complete");

    let v22_receipt = require_str(
        &v22,
        &[
            "artifacts",
            "ActingOnBehalfReceiptV1",
            "effect_execution_receipt_id",
        ],
    );
    let v22_authority_chain = require_str(
        &v22,
        &["artifacts", "AuthorityChainV1", "current_validity_state"],
    );
    let replay_requirements = require_value(
        &v22,
        &["artifacts", "DelegationBundleV1", "replay_requirements"],
    )
    .as_array()
    .unwrap()
    .iter()
    .map(|item| item.as_str().unwrap())
    .collect::<Vec<_>>();

    assert_eq!(v22_receipt, v21_exec_receipt);
    assert_eq!(v22_authority_chain, "valid");
    assert!(replay_requirements.contains(&"authority_chain"));
    assert!(replay_requirements.contains(&"effect_receipt"));

    let v23_decision = require_str(
        &v23,
        &["artifacts", "ReleaseReadinessDecisionV1", "decision_state"],
    );
    let v23_blocking = require_value(
        &v23,
        &["artifacts", "ReleaseReadinessDecisionV1", "blocking_gaps"],
    )
    .as_array()
    .unwrap();
    let v23_advisory_only = require_value(
        &v23,
        &["artifacts", "ReleaseReadinessDecisionV1", "advisory_only"],
    )
    .as_bool()
    .unwrap();

    let demo_cross_link = require_str(
        &demo,
        &[
            "chain",
            "cross_wave_links",
            "effect_to_delegation",
            "effect_execution_receipt",
        ],
    );
    let demo_release_link = require_str(
        &demo,
        &[
            "chain",
            "cross_wave_links",
            "delegation_to_release",
            "release_readiness_decision",
        ],
    );

    assert_eq!(demo_cross_link, "EffectExecutionReceiptV1:fxr_001");
    assert_eq!(demo_release_link, "ReleaseReadinessDecisionV1:rrd_001");

    assert_eq!(v23_decision, "approved_with_monitoring");
    assert!(v23_blocking.is_empty());
    assert!(!v23_advisory_only);

    let expected_output_preflight = require_str(&demo, &["expected_outputs", "preflight"]);
    let expected_output_release = require_str(&demo, &["expected_outputs", "release"]);

    assert_eq!(expected_output_preflight, "commit_eligible");
    assert_eq!(expected_output_release, "approved_with_monitoring");
}

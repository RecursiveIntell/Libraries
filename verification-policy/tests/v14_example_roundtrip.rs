#![allow(clippy::expect_used)]

use verification_policy::{
    ArtifactAdmissionPolicyV1, DisclosureBudgetV1, DisclosurePolicyV1, ExperimentBudgetV1,
    RefuterSuiteV1,
};

#[test]
fn v14_examples_roundtrip_and_validate() {
    let refuter_suite: RefuterSuiteV1 = serde_json::from_str(
        &std::fs::read_to_string("../examples/RefuterSuiteV1.example.json")
            .expect("read refuter suite"),
    )
    .expect("parse refuter suite");
    refuter_suite.validate().expect("validate refuter suite");

    let experiment_budget: ExperimentBudgetV1 = serde_json::from_str(
        &std::fs::read_to_string("../examples/ExperimentBudgetV1.example.json")
            .expect("read experiment budget"),
    )
    .expect("parse experiment budget");
    experiment_budget
        .validate()
        .expect("validate experiment budget");

    let artifact_admission: ArtifactAdmissionPolicyV1 = serde_json::from_str(
        &std::fs::read_to_string("../examples/ArtifactAdmissionPolicyV1.example.json")
            .expect("read artifact admission"),
    )
    .expect("parse artifact admission");
    artifact_admission
        .validate()
        .expect("validate artifact admission");

    let disclosure_policy: DisclosurePolicyV1 = serde_json::from_str(
        &std::fs::read_to_string("../examples/DisclosurePolicyV1.example.json")
            .expect("read disclosure policy"),
    )
    .expect("parse disclosure policy");
    disclosure_policy
        .validate()
        .expect("validate disclosure policy");

    let disclosure_budget: DisclosureBudgetV1 = serde_json::from_str(
        &std::fs::read_to_string("../examples/DisclosureBudgetV1.example.json")
            .expect("read disclosure budget"),
    )
    .expect("parse disclosure budget");
    disclosure_budget
        .validate()
        .expect("validate disclosure budget");

    let encoded = serde_json::to_string(&(
        refuter_suite,
        experiment_budget,
        artifact_admission,
        disclosure_policy,
        disclosure_budget,
    ))
    .expect("serialize tuple");
    let (
        refuter_suite,
        experiment_budget,
        artifact_admission,
        disclosure_policy,
        disclosure_budget,
    ): (
        RefuterSuiteV1,
        ExperimentBudgetV1,
        ArtifactAdmissionPolicyV1,
        DisclosurePolicyV1,
        DisclosureBudgetV1,
    ) = serde_json::from_str(&encoded).expect("deserialize tuple");

    refuter_suite.validate().expect("validate refuter suite");
    experiment_budget
        .validate()
        .expect("validate experiment budget");
    artifact_admission
        .validate()
        .expect("validate artifact admission");
    disclosure_policy
        .validate()
        .expect("validate disclosure policy");
    disclosure_budget
        .validate()
        .expect("validate disclosure budget");
}

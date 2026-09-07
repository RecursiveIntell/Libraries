use std::collections::{BTreeMap, BTreeSet};

use agent_graph::consumer_bridge::{
    project_execution_fact_for_application, ApplicationGrantV1, ConsumerBridgeError,
    ConsumerRequestV1, ExecutionFactV1, InMemoryConsumerOwners,
};

#[test]
fn app_01_two_apps_share_execution_fact_without_domain_truth_or_memory_leakage() {
    let fact = ExecutionFactV1 {
        fact_id: "fact:run-1".into(),
        run_id: "run-1".into(),
        receipt_digest: "sha256:receipt".into(),
        artifact_refs: vec!["artifact:one".into()],
    };
    let mut owners = InMemoryConsumerOwners {
        facts: BTreeMap::from([(fact.fact_id.clone(), fact.clone())]),
        grants: BTreeMap::new(),
        accepted_outputs: BTreeMap::new(),
    };
    owners.grants.insert(
        "app:audit".into(),
        ApplicationGrantV1 {
            app_id: "app:audit".into(),
            memory_namespace: "memory:audit".into(),
            permitted_fact_ids: BTreeSet::from([fact.fact_id.clone()]),
            output_policy_id: "policy:audit".into(),
            purpose: "audit".into(),
        },
    );
    owners.grants.insert(
        "app:report".into(),
        ApplicationGrantV1 {
            app_id: "app:report".into(),
            memory_namespace: "memory:report".into(),
            permitted_fact_ids: BTreeSet::from([fact.fact_id.clone()]),
            output_policy_id: "policy:report".into(),
            purpose: "report".into(),
        },
    );
    owners.accepted_outputs.insert(
        "app:audit:policy:audit".into(),
        BTreeSet::from(["audit-domain-ok".into()]),
    );
    owners.accepted_outputs.insert(
        "app:report:policy:report".into(),
        BTreeSet::from(["report-domain-ok".into()]),
    );

    let audit = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app:audit".into(),
            fact_id: fact.fact_id.clone(),
            memory_namespace: "memory:audit".into(),
            domain_output: "audit-domain-ok".into(),
            ui_cached_fact: Some(fact.clone()),
        },
        &owners,
    )
    .unwrap();
    let report = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app:report".into(),
            fact_id: fact.fact_id.clone(),
            memory_namespace: "memory:report".into(),
            domain_output: "report-domain-ok".into(),
            ui_cached_fact: Some(fact.clone()),
        },
        &owners,
    )
    .unwrap();

    assert_eq!(audit.execution_fact_id, report.execution_fact_id);
    assert_eq!(audit.receipt_digest, report.receipt_digest);
    assert_ne!(audit.output_policy_id, report.output_policy_id);
    assert_ne!(audit.domain_output, report.domain_output);
    assert_ne!(audit.memory_namespace, report.memory_namespace);
    assert!(!audit.memory_content_included && !report.memory_content_included);
    assert!(!audit.canonical_fact_created && !report.canonical_fact_created);
    assert_eq!(
        owners.facts.len(),
        1,
        "UI projections duplicated canonical facts"
    );

    let cross_app = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app:audit".into(),
            fact_id: fact.fact_id.clone(),
            memory_namespace: "memory:report".into(),
            domain_output: "audit-domain-ok".into(),
            ui_cached_fact: None,
        },
        &owners,
    );
    assert_eq!(cross_app, Err(ConsumerBridgeError::MemoryNamespaceMismatch));

    let wrong_domain = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app:report".into(),
            fact_id: fact.fact_id.clone(),
            memory_namespace: "memory:report".into(),
            domain_output: "audit-domain-ok".into(),
            ui_cached_fact: None,
        },
        &owners,
    );
    assert_eq!(wrong_domain, Err(ConsumerBridgeError::DomainOutputRejected));

    let mut forged = fact;
    forged.receipt_digest = "sha256:forged".into();
    let ui_conflict = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app:audit".into(),
            fact_id: "fact:run-1".into(),
            memory_namespace: "memory:audit".into(),
            domain_output: "audit-domain-ok".into(),
            ui_cached_fact: Some(forged),
        },
        &owners,
    );
    assert_eq!(ui_conflict, Err(ConsumerBridgeError::ConflictingUiFact));
    assert_eq!(owners.facts.len(), 1);
}

#[test]
fn pr11_owner_fact_must_match_permitted_identity() {
    let owner = InMemoryConsumerOwners {
        facts: BTreeMap::from([(
            "allowed".into(),
            ExecutionFactV1 {
                fact_id: "different".into(),
                run_id: "secret".into(),
                receipt_digest: "secret".into(),
                artifact_refs: vec![],
            },
        )]),
        grants: BTreeMap::from([(
            "app".into(),
            ApplicationGrantV1 {
                app_id: "app".into(),
                memory_namespace: "ns".into(),
                permitted_fact_ids: BTreeSet::from(["allowed".into()]),
                output_policy_id: "policy".into(),
                purpose: "test".into(),
            },
        )]),
        accepted_outputs: BTreeMap::new(),
    };
    let result = project_execution_fact_for_application(
        &ConsumerRequestV1 {
            app_id: "app".into(),
            fact_id: "allowed".into(),
            memory_namespace: "ns".into(),
            domain_output: "".into(),
            ui_cached_fact: None,
        },
        &owner,
    );
    assert_eq!(result, Err(ConsumerBridgeError::FactIdentityMismatch));
}

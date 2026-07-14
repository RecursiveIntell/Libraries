use chrono::{Duration, Utc};
use llm_tool_runtime::{
    AuthorityLineageEntry, EffectTargetSpec, ToolDescriptor, ToolExecutionPermit,
    ToolReceiptPersistence, ToolSideEffectClass,
};
use serde_json::json;
use stack_ids::{ContentDigest, ExecutionPermitId, PolicyDecisionId};
use std::sync::Arc;

fn permit(expires_at: Option<chrono::DateTime<Utc>>) -> ToolExecutionPermit {
    ToolExecutionPermit::new(
        ExecutionPermitId::generate(),
        PolicyDecisionId::generate(),
        None,
        "tests",
        "artifact-1",
        ContentDigest::compute(b"method"),
        ContentDigest::compute(b"effect"),
        expires_at,
        "nonce-1",
    )
}

#[test]
fn execution_permit_is_one_shot() {
    let permit = permit(Some(Utc::now() + Duration::minutes(1)));
    assert!(permit.consume().is_ok());
    assert!(permit.consume().is_err());
}

#[test]
fn execution_permit_rejects_expiry_and_modified_effect() {
    let expired = permit(Some(Utc::now() - Duration::seconds(1)));
    assert!(expired
        .validate_binding(
            &ContentDigest::compute(b"method"),
            &ContentDigest::compute(b"effect"),
            Utc::now(),
        )
        .is_err());

    let bound = permit(Some(Utc::now() + Duration::minutes(1)));
    assert!(bound
        .validate_binding(
            &ContentDigest::compute(b"method"),
            &ContentDigest::compute(b"modified-effect"),
            Utc::now(),
        )
        .is_err());
}

#[test]
fn execution_permit_concurrent_double_spend_has_one_winner() {
    let permit = Arc::new(permit(Some(Utc::now() + Duration::minutes(1))));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let permit = Arc::clone(&permit);
            std::thread::spawn(move || permit.consume().is_ok())
        })
        .collect();
    let successes = handles
        .into_iter()
        .filter(|handle| handle.join().unwrap())
        .count();
    assert_eq!(successes, 1);
}

#[test]
fn effect_intent_normalizes_aliases_and_represents_compound_targets() {
    let descriptor: ToolDescriptor = serde_json::from_value(json!({
        "name": "write_artifact",
        "version": "1",
        "backend_kind": "local_function",
        "input_schema": {},
        "output_mode": "structured_json",
        "read_only": false,
        "side_effect_class": "write",
        "idempotency_class": "non_idempotent",
        "approval_kind": "policy_required",
        "timeout_ms": 1000,
        "exposure_mode": "opt_in",
        "mcp_surface_kind": "tool",
        "effect_target": {"aliases": ["target", "artifact_id"], "compound": []}
    }))
    .unwrap();

    let first = descriptor
        .describe_effect(&json!({"target": "a", "value": 1}))
        .unwrap();
    let second = descriptor
        .describe_effect(&json!({"artifact_id": "a", "value": 1}))
        .unwrap();
    assert_eq!(first, second);

    let mut compound = descriptor;
    compound.effect_target = EffectTargetSpec {
        aliases: vec![],
        compound: vec!["source".into(), "destination".into()],
    };
    let intent = compound
        .describe_effect(&json!({"source": "a", "destination": "b"}))
        .unwrap();
    assert_eq!(intent.effect_class, ToolSideEffectClass::Write);
    assert_eq!(intent.scope.targets, vec!["a", "b"]);
}

#[test]
fn receipt_persistence_defaults_to_durable() {
    assert_eq!(
        ToolReceiptPersistence::default(),
        ToolReceiptPersistence::Durable
    );
}

#[test]
fn authority_lineage_verification_requires_complete_chain() {
    let missing = vec![AuthorityLineageEntry {
        origin_class: "request".into(),
        principal: "caller".into(),
        permit_id: "permit-1".into(),
        policy_version: "policy-1".into(),
    }];
    assert!(llm_tool_runtime::verify_authority_lineage(&missing).is_err());

    let complete = ["request", "policy", "approval", "permit", "effect"]
        .into_iter()
        .map(|origin_class| AuthorityLineageEntry {
            origin_class: origin_class.into(),
            principal: "caller".into(),
            permit_id: "permit-1".into(),
            policy_version: "policy-1".into(),
        })
        .collect::<Vec<_>>();
    assert!(llm_tool_runtime::verify_authority_lineage(&complete).is_ok());
}

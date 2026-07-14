use async_trait::async_trait;
use chrono::{Duration, Utc};
use llm_tool_runtime::{
    AuthorityLineageEntry, EffectTargetSpec, McpSurfaceKind, Tool, ToolApprovalKind,
    ToolBackendKind, ToolCall, ToolCtx, ToolDescriptor, ToolError, ToolErrorClass,
    ToolExecutionPermit, ToolExposureMode, ToolExposurePolicy, ToolIdempotencyClass,
    ToolOriginKind, ToolOutputMode, ToolPlannerStage, ToolReceipt, ToolReceiptPersistence,
    ToolReceiptSink, ToolRegistry, ToolResult, ToolRuntime, ToolSideEffectClass,
};
use serde_json::json;
use stack_ids::{
    AttemptId, ContentDigest, ExecutionPermitId, PolicyDecisionId, ScopeKey, TraceCtx, TrialId,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
    assert!(expired.consume().is_err());
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
        .map(|handle| handle.join().unwrap())
        .filter(|success| *success)
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
    assert_eq!(
        descriptor.receipt_persistence,
        ToolReceiptPersistence::Durable
    );

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
    let ephemeral: ToolReceiptPersistence = serde_json::from_str("\"ephemeral\"").unwrap();
    assert_eq!(ephemeral, ToolReceiptPersistence::Ephemeral);
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

struct EffectTool {
    descriptor: ToolDescriptor,
    invocations: Arc<AtomicUsize>,
}

#[async_trait]
impl Tool for EffectTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn invoke(&self, _ctx: &ToolCtx, _call: &ToolCall) -> Result<ToolResult, ToolError> {
        self.invocations.fetch_add(1, Ordering::SeqCst);
        Ok(ToolResult::json(json!({"ok": true})))
    }
}

struct FailingSink {
    fail_on_persist: usize,
    persist_count: AtomicUsize,
    unresolved: AtomicBool,
    receipts: Mutex<Vec<ToolReceipt>>,
}

#[async_trait]
impl ToolReceiptSink for FailingSink {
    async fn health_check(&self) -> Result<(), ToolError> {
        Ok(())
    }

    async fn persist(&self, receipt: &ToolReceipt) -> Result<(), ToolError> {
        let attempt = self.persist_count.fetch_add(1, Ordering::SeqCst) + 1;
        if attempt == self.fail_on_persist {
            return Err(ToolError::new(
                ToolErrorClass::ReceiptPersistence,
                "injected persistence failure",
            ));
        }
        self.receipts.lock().unwrap().push(receipt.clone());
        Ok(())
    }

    async fn mark_unresolved(&self, _preflight_receipt_id: &str) -> Result<(), ToolError> {
        self.unresolved.store(true, Ordering::SeqCst);
        Ok(())
    }
}

fn effect_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "effect".into(),
        version: "1".into(),
        description: None,
        backend_kind: ToolBackendKind::LocalFunction,
        input_schema: json!({
            "type": "object",
            "required": ["target"],
            "properties": {"target": {"type": "string"}},
            "additionalProperties": false
        }),
        output_mode: ToolOutputMode::StructuredJson,
        read_only: false,
        side_effect_class: ToolSideEffectClass::Write,
        idempotency_class: ToolIdempotencyClass::NonIdempotent,
        approval_kind: ToolApprovalKind::None,
        timeout_ms: 1_000,
        concurrency_key: None,
        cache_ttl_ms: None,
        exposure_mode: ToolExposureMode::OptIn,
        mcp_surface_kind: McpSurfaceKind::Tool,
        exposure_policy: ToolExposurePolicy::default(),
        receipt_persistence: ToolReceiptPersistence::Durable,
        effect_target: EffectTargetSpec {
            aliases: vec!["target".into()],
            compound: vec![],
        },
        output_size_limit_bytes: None,
        provider_payload: None,
    }
}

fn effect_ctx() -> ToolCtx {
    ToolCtx {
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::generate(),
        trial_id: TrialId::generate(),
        deadline: None,
        workload_class: None,
        budget_context: None,
        scope: Some(ScopeKey::namespace_only("tests")),
        dry_run: false,
        approval_grant: None,
        execution_permit: None,
        idempotency_key: None,
        caller: "authority-tests".into(),
        planner_stage: ToolPlannerStage::Execution,
        parent_receipt_id: None,
        family_receipt_id: None,
        replay_parent_receipt_id: None,
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        retry_owner: None,
    }
}

fn bound_effect_permit(descriptor: &ToolDescriptor, call: &ToolCall) -> Arc<ToolExecutionPermit> {
    let intent = descriptor.describe_effect(&call.arguments).unwrap();
    Arc::new(ToolExecutionPermit::new(
        ExecutionPermitId::generate(),
        PolicyDecisionId::generate(),
        None,
        "tests",
        intent.target_key.clone(),
        descriptor.method_digest(),
        intent.digest(),
        Some(Utc::now() + Duration::minutes(1)),
        uuid::Uuid::new_v4().to_string(),
    ))
}

#[tokio::test]
async fn preflight_persistence_failure_prevents_effect() {
    let invocations = Arc::new(AtomicUsize::new(0));
    let descriptor = effect_descriptor();
    let call = ToolCall::new(
        "effect",
        "1",
        json!({"target": "artifact-1"}),
        ToolOriginKind::Test,
    );
    let permit = bound_effect_permit(&descriptor, &call);
    let mut registry = ToolRegistry::new();
    registry.register(EffectTool {
        descriptor,
        invocations: Arc::clone(&invocations),
    });
    let sink = Arc::new(FailingSink {
        fail_on_persist: 1,
        persist_count: AtomicUsize::new(0),
        unresolved: AtomicBool::new(false),
        receipts: Mutex::new(vec![]),
    });
    let runtime = ToolRuntime::new(registry).with_receipt_sink(sink);

    let execution = runtime
        .execute(&effect_ctx(), &call, Some(permit), None)
        .await;
    assert_eq!(
        execution.result.unwrap_err().class,
        ToolErrorClass::ReceiptPersistence
    );
    assert_eq!(invocations.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn outcome_persistence_failure_marks_preflight_unresolved() {
    let invocations = Arc::new(AtomicUsize::new(0));
    let descriptor = effect_descriptor();
    let call = ToolCall::new(
        "effect",
        "1",
        json!({"target": "artifact-1"}),
        ToolOriginKind::Test,
    );
    let permit = bound_effect_permit(&descriptor, &call);
    let mut registry = ToolRegistry::new();
    registry.register(EffectTool {
        descriptor,
        invocations: Arc::clone(&invocations),
    });
    let sink = Arc::new(FailingSink {
        fail_on_persist: 2,
        persist_count: AtomicUsize::new(0),
        unresolved: AtomicBool::new(false),
        receipts: Mutex::new(vec![]),
    });
    let runtime = ToolRuntime::new(registry).with_receipt_sink(sink.clone());

    let execution = runtime
        .execute(&effect_ctx(), &call, Some(permit), None)
        .await;
    assert_eq!(
        execution.result.unwrap_err().class,
        ToolErrorClass::ReceiptPersistence
    );
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert!(sink.unresolved.load(Ordering::SeqCst));
}

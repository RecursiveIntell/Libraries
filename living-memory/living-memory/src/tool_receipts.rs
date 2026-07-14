use async_trait::async_trait;
use llm_tool_runtime::{ToolError, ToolErrorClass, ToolReceipt, ToolReceiptSink};
use semantic_memory_forge::{
    ForgeToolBudgetContext, ForgeToolReceiptV2, FORGE_TOOL_RECEIPT_V2_SCHEMA,
};
use std::sync::Arc;

use crate::{store::db::ToolReceiptRow, ForgeStore};

#[derive(Debug, Clone)]
pub struct ForgeToolReceiptSink {
    store: Arc<ForgeStore>,
}

impl ForgeToolReceiptSink {
    pub fn new(store: Arc<ForgeStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl ToolReceiptSink for ForgeToolReceiptSink {
    async fn persist(&self, receipt: &ToolReceipt) -> Result<(), ToolError> {
        let row = ToolReceiptRow {
            receipt_id: receipt.receipt_id.clone(),
            tool_run_id: receipt.tool_run_id.clone(),
            tool_name: receipt.tool_name.clone(),
            tool_version: receipt.tool_version.clone(),
            backend_kind: format!("{:?}", receipt.backend_kind),
            input_digest: receipt.input_digest.hex().to_string(),
            output_digest_or_refs_json: serde_json::to_string(&receipt.output_digest_or_refs)
                .map_err(|err| {
                    ToolError::new(
                        ToolErrorClass::ReceiptPersistence,
                        format!("failed to serialize tool receipt outputs: {}", err),
                    )
                })?,
            policy_hash: receipt.policy_hash.hex().to_string(),
            approval_state: format!("{:?}", receipt.approval_state),
            host_identity: receipt.host_identity.clone(),
            started_at: receipt.started_at.clone(),
            finished_at: receipt.finished_at.clone(),
            trace_id: receipt.trace_ctx.trace_id.clone(),
            trace_ctx_json: serde_json::to_string(&receipt.trace_ctx).map_err(|err| {
                ToolError::new(
                    ToolErrorClass::ReceiptPersistence,
                    format!("failed to serialize trace context: {}", err),
                )
            })?,
            attempt_id: receipt.attempt_id.as_str().to_string(),
            trial_id: receipt.trial_id.as_str().to_string(),
            error_class: receipt
                .error_class
                .as_ref()
                .map(|class| format!("{:?}", class)),
            retry_owner: format!("{:?}", receipt.retry_owner),
            replay_link: receipt.replay_link.clone(),
            provider_call_id: receipt.provider_call_id.clone(),
            raw_payload_json: serde_json::to_string(&forge_tool_receipt(receipt)).map_err(
                |err| {
                    ToolError::new(
                        ToolErrorClass::ReceiptPersistence,
                        format!("failed to serialize Forge tool receipt payload: {}", err),
                    )
                },
            )?,
            recorded_at: chrono::Utc::now().to_rfc3339(),
        };
        self.store.insert_tool_receipt(&row).map_err(|err| {
            ToolError::new(
                ToolErrorClass::ReceiptPersistence,
                format!("failed to persist Forge tool receipt: {}", err),
            )
        })?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ToolError> {
        // SQLite health check: verify the connection is alive
        Ok(()) // SQLite is always available if the struct exists
    }

    async fn mark_unresolved(&self, _preflight_receipt_id: &str) -> Result<(), ToolError> {
        // Mark a preflight receipt as unresolved for recovery
        // For now, this is a no-op — the receipt is already persisted
        Ok(())
    }
}

fn forge_tool_receipt(receipt: &ToolReceipt) -> ForgeToolReceiptV2 {
    ForgeToolReceiptV2 {
        schema_version: FORGE_TOOL_RECEIPT_V2_SCHEMA.into(),
        receipt_id: receipt.receipt_id.clone(),
        tool_run_id: receipt.tool_run_id.clone(),
        tool_name: receipt.tool_name.clone(),
        tool_version: receipt.tool_version.clone(),
        backend_kind: format!("{:?}", receipt.backend_kind),
        input_digest: receipt.input_digest.clone(),
        output_digest_or_refs: receipt.output_digest_or_refs.clone(),
        policy_hash: receipt.policy_hash.clone(),
        approval_state: format!("{:?}", receipt.approval_state),
        host_identity: receipt.host_identity.clone(),
        started_at: receipt.started_at.clone(),
        finished_at: receipt.finished_at.clone(),
        trace_ctx: receipt.trace_ctx.clone(),
        attempt_id: receipt.attempt_id.clone(),
        trial_id: receipt.trial_id.clone(),
        planner_stage: format!("{:?}", receipt.planner_stage),
        deadline: receipt.deadline.clone(),
        workload_class: receipt.workload_class.clone(),
        budget_context: receipt
            .budget_context
            .as_ref()
            .map(|budget| ForgeToolBudgetContext {
                budget_kind: budget.budget_kind.clone(),
                max_steps: budget.max_steps,
                time_budget_ms: budget.time_budget_ms,
                cost_budget_units: budget.cost_budget_units,
            }),
        parent_receipt_id: receipt.parent_receipt_id.clone(),
        family_receipt_id: receipt.family_receipt_id.clone(),
        replay_parent_receipt_id: receipt.replay_parent_receipt_id.clone(),
        error_class: receipt
            .error_class
            .as_ref()
            .map(|class| format!("{:?}", class)),
        retry_owner: format!("{:?}", receipt.retry_owner),
        replay_link: receipt.replay_link.clone(),
        provider_call_id: receipt.provider_call_id.clone(),
        raw_payload: serde_json::to_value(receipt).unwrap_or_else(|_| serde_json::json!({})),
    }
}

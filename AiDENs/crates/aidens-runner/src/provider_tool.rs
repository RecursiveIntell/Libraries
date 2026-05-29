//! Provider/tool turn helpers and parser-fallback boundary handling.
//!
//! This module routes local tool calls and parser fallback receipts; provider
//! and tool authority remains with `aidens-provider-kit` and `aidens-tool-kit`.

use super::*;

use boundary_compiler_core::{
    compile_json_boundary, BoundaryCompilerProfileV1, BoundaryDecisionV1,
};

pub(super) fn completion_request(
    prompt: String,
    tool_exposure: &ToolExposureSetV1,
    tool_results: Vec<ToolCallResultV1>,
) -> anyhow::Result<AiDENsCompletionRequestV1> {
    let mut request = AiDENsCompletionRequestV1::single_user(prompt);
    request.provider_tool_schemas = tool_exposure.provider_tool_schemas.clone();
    request.tool_results = tool_results;
    if !request.tool_results.is_empty() {
        let content = serde_json::to_string(&request.tool_results)
            .map_err(|error| anyhow!("tool-result serialization failed: {error}"))?;
        request.messages.push(AiDENsChatMessageV1 {
            role: "tool".into(),
            content,
        });
    }
    Ok(request)
}

pub(super) fn turn_mode_for(
    provider_route: &ProviderRouteReportV1,
    tool_exposure: &ToolExposureSetV1,
) -> TurnModeV1 {
    match provider_route.route {
        ProviderRouteKindV1::Disabled | ProviderRouteKindV1::Unavailable => {
            TurnModeV1::ProviderUnavailable
        }
        _ if tool_exposure.exposed_tool_ids.is_empty() => TurnModeV1::NoTools,
        _ if provider_route.native_tool_loop => TurnModeV1::NativeToolLoop,
        _ => TurnModeV1::ParserFallback,
    }
}

pub(super) fn turn_plan_reasons(mode: TurnModeV1) -> Vec<String> {
    match mode {
        TurnModeV1::NoTools => vec!["no-tools-exposed".into()],
        TurnModeV1::NativeToolLoop => vec!["native-tool-loop-selected".into()],
        TurnModeV1::ParserFallback => vec![
            "parser-fallback-selected".into(),
            "parser-fallback-degraded".into(),
        ],
        TurnModeV1::ProviderUnavailable => vec!["provider-unavailable".into()],
        TurnModeV1::BudgetExhausted => vec!["budget-exhausted".into()],
    }
}

pub(super) fn tool_calls_for_completion(
    completion: &AiDENsCompletionResponseV1,
    mode: TurnModeV1,
) -> CompletionToolCallsV1 {
    if !completion.tool_calls.is_empty() {
        return CompletionToolCallsV1 {
            calls: completion.tool_calls.clone(),
            boundary_repair_receipts: Vec::new(),
            json_repair_receipts: Vec::new(),
            degradation_reason_codes: Vec::new(),
        };
    }
    if mode != TurnModeV1::ParserFallback {
        if mode == TurnModeV1::NoTools && looks_like_tool_call_payload(&completion.text) {
            return parse_parser_fallback_tool_calls(&completion.text);
        }
        return CompletionToolCallsV1 {
            calls: Vec::new(),
            boundary_repair_receipts: Vec::new(),
            json_repair_receipts: Vec::new(),
            degradation_reason_codes: Vec::new(),
        };
    }
    parse_parser_fallback_tool_calls(&completion.text)
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CompletionToolCallsV1 {
    pub(super) calls: Vec<ToolCallRequestV1>,
    pub(super) boundary_repair_receipts: Vec<BoundaryRepairReportV1>,
    pub(super) json_repair_receipts: Vec<JsonBoundaryRepairDisplayReportV1>,
    pub(super) degradation_reason_codes: Vec<String>,
}

pub(super) fn parse_parser_fallback_tool_calls(text: &str) -> CompletionToolCallsV1 {
    // Phase 06 vertical slice: strict boundary validation before lenient parse.
    // If the text is valid JSON, run it through the strict boundary compiler
    // to detect duplicate keys, unknown fields, and resource ceiling violations.
    let strict_receipt = strict_boundary_compile_receipt(text);

    let Ok(outcome) = parse_json_boundary(text, BoundaryRepairPolicyV1::default()) else {
        if looks_like_tool_call_payload(text) {
            let mut degradation = vec![
                "malformed-parser-fallback-tool-call".into(),
                "parser-fallback-boundary-parse-failed".into(),
            ];
            degradation.extend(strict_receipt);
            degradation.sort();
            degradation.dedup();
            return CompletionToolCallsV1 {
                calls: Vec::new(),
                boundary_repair_receipts: Vec::new(),
                json_repair_receipts: Vec::new(),
                degradation_reason_codes: degradation,
            };
        }
        // Non-JSON text that isn't a tool call — no degradation, no strict receipt.
        // The turn will proceed normally with final text output.
        return CompletionToolCallsV1 {
            calls: Vec::new(),
            boundary_repair_receipts: Vec::new(),
            json_repair_receipts: Vec::new(),
            degradation_reason_codes: Vec::new(),
        };
    };
    let reason_codes = vec![
        "parser-fallback-tool-call".into(),
        "parser-fallback-degraded".into(),
    ];
    let mut completion_reason_codes = reason_codes.clone();
    if outcome.repair_receipt.changed {
        completion_reason_codes.push(format!(
            "boundary-repair:{}",
            outcome.repair_receipt.repair_kind
        ));
        completion_reason_codes.push("parser-fallback-repair-blocked".into());
    }
    if !strict_receipt.is_empty() {
        completion_reason_codes.extend(strict_receipt);
    }
    completion_reason_codes.sort();
    completion_reason_codes.dedup();
    let calls = if let Some(calls) = outcome
        .value
        .get("tool_calls")
        .and_then(serde_json::Value::as_array)
    {
        calls.clone()
    } else if let Some(call) = outcome.value.get("tool_call") {
        vec![call.clone()]
    } else if outcome.value.get("tool_id").is_some() {
        vec![outcome.value.clone()]
    } else {
        Vec::new()
    };
    let mut rejected_call_reasons = Vec::new();
    let mut parsed_calls = Vec::new();
    for (index, call) in calls.into_iter().enumerate() {
        let Some(tool_id) = call.get("tool_id").and_then(serde_json::Value::as_str) else {
            let digest = DisplayDigestV1::for_json_value(&call).digest;
            rejected_call_reasons.push(format!(
                "rejected-parser-fallback-tool-call:index={index}:reason=missing-tool-id:raw-digest={digest}"
            ));
            continue;
        };
        if tool_id.trim().is_empty() {
            let digest = DisplayDigestV1::for_json_value(&call).digest;
            rejected_call_reasons.push(format!(
                "rejected-parser-fallback-tool-call:index={index}:reason=empty-tool-id:raw-digest={digest}"
            ));
            continue;
        }
        let input = call
            .get("input")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        parsed_calls.push(ToolCallRequestV1::new(
            ToolCallSourceV1::ParserFallback,
            tool_id,
            input,
            Some(text.into()),
            completion_reason_codes.clone(),
        ));
    }
    let repair_receipt_changed = outcome.repair_receipt.changed;
    let boundary_repair_receipts = repair_receipt_changed.then_some(outcome.repair_receipt);
    let json_repair_receipts = outcome.json_repair_receipt.into_iter().collect();
    if !rejected_call_reasons.is_empty() {
        let mut degradation_reason_codes = completion_reason_codes;
        degradation_reason_codes.push("parser-fallback-rejected-malformed-tool-call".into());
        degradation_reason_codes.extend(rejected_call_reasons);
        degradation_reason_codes.sort();
        degradation_reason_codes.dedup();
        return CompletionToolCallsV1 {
            calls: parsed_calls,
            boundary_repair_receipts: boundary_repair_receipts.into_iter().collect(),
            json_repair_receipts,
            degradation_reason_codes,
        };
    }
    CompletionToolCallsV1 {
        calls: parsed_calls,
        boundary_repair_receipts: boundary_repair_receipts.into_iter().collect(),
        json_repair_receipts,
        degradation_reason_codes: if repair_receipt_changed {
            completion_reason_codes
        } else {
            Vec::new()
        },
    }
}

pub(super) fn looks_like_tool_call_payload(text: &str) -> bool {
    if let Ok(outcome) = parse_json_boundary(text, BoundaryRepairPolicyV1::default()) {
        match outcome.value {
            serde_json::Value::Object(object) => {
                object.contains_key("tool_call")
                    || object.contains_key("tool_calls")
                    || object
                        .get("tool_id")
                        .is_some_and(|tool_id| tool_id.is_string())
            }
            _ => false,
        }
    } else {
        // Text that starts with a tool_call: prefix is a malformed tool call attempt
        // even if JSON parsing fails (e.g., truncated JSON, prefix-only payloads).
        let trimmed = text.trim();
        trimmed.starts_with("tool_call:")
            || trimmed.starts_with("tool_calls:")
            || trimmed.starts_with("tool_call :")
            || trimmed.starts_with("tool_calls :")
    }
}

pub(super) fn tool_invocation_receipt_from_error(
    tool_id: &str,
    input: serde_json::Value,
    error: &anyhow::Error,
    context: &AidensRunContextV1,
) -> ToolInvocationReportV1 {
    if let Some(invocation_error) = error.downcast_ref::<ToolInvocationError>() {
        return invocation_error
            .receipt()
            .clone()
            .with_execution_context(context);
    }
    ToolInvocationReportV1::started(tool_id, input)
        .with_execution_context(context)
        .complete_failure(error.to_string())
}

pub(super) fn elapsed_millis(started_at: Instant) -> u64 {
    started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

/// Phase 06 vertical slice: run strict boundary compilation on provider tool output.
/// Returns reason codes if the strict compiler rejects or quarantines the output,
/// enabling receipt-bearing degradation downstream.
fn strict_boundary_compile_receipt(text: &str) -> Vec<String> {
    // Trim surrounding whitespace — JSON boundary compilation requires
    // RFC 8259 strict parsing (de.end() rejects trailing content).
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let profile = BoundaryCompilerProfileV1::strict_json_default();
    let result = compile_json_boundary(&profile, trimmed.as_bytes(), None, &[]);
    match result.decision {
        BoundaryDecisionV1::Accept => Vec::new(),
        BoundaryDecisionV1::Reject => {
            let mut codes = vec!["strict-boundary-rejected".into()];
            for error in &result.errors {
                codes.push(format!("strict-boundary-error:{:?}", error.kind));
            }
            codes
        }
        BoundaryDecisionV1::Quarantine => {
            let mut codes = vec!["strict-boundary-quarantined".into()];
            for error in &result.errors {
                codes.push(format!("strict-boundary-error:{:?}", error.kind));
            }
            codes
        }
        BoundaryDecisionV1::RepairedAccept => {
            vec!["strict-boundary-repaired-accept".into()]
        }
    }
}

//! Replay and receipt-log path helpers.
//!
//! Replay records here are local execution evidence paths. Canonical replay
//! authority remains with the verification/control owner crates.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static PLAN_ACT_VERIFY_RECEIPT_LOG_SEQ: AtomicU64 = AtomicU64::new(1);

pub(super) fn default_plan_act_verify_receipt_log_config() -> CanonicalEventLogConfig {
    let sequence = PLAN_ACT_VERIFY_RECEIPT_LOG_SEQ.fetch_add(1, Ordering::Relaxed);
    CanonicalEventLogConfig::for_root(format!(
        "target/aidens-receipts/plan-act-verify-loop/process-{}-{sequence}",
        std::process::id()
    ))
}

pub(super) fn default_runner_receipt_log_config(app_id: &str) -> CanonicalEventLogConfig {
    let sanitized = app_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    CanonicalEventLogConfig::for_root(format!(
        "target/aidens-receipts/runner/{sanitized}/process-{}-{nonce}",
        std::process::id()
    ))
}

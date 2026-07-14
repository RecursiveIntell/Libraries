use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use chrono::{Duration, Utc};
use llm_tool_runtime::ToolExecutionPermit;
use stack_ids::{ExecutionPermitId, PolicyDecisionId};

#[test]
fn loom_style_permit_single_consume() {
    let permit = Arc::new(ToolExecutionPermit::new(
        ExecutionPermitId::generate(),
        PolicyDecisionId::generate(),
        None,
        "tests",
        "artifact-1",
        stack_ids::ContentDigest::compute(b"method"),
        stack_ids::ContentDigest::compute(b"effect"),
        Some(Utc::now() + Duration::minutes(1)),
        uuid::Uuid::new_v4().to_string(),
    ));

    let consume_success = Arc::new(AtomicUsize::new(0));
    let threads = (0..16)
        .map(|_| {
            let permit = Arc::clone(&permit);
            let consume_success = Arc::clone(&consume_success);
            thread::spawn(move || {
                if permit.consume().is_ok() {
                    consume_success.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(consume_success.load(Ordering::SeqCst), 1);
}

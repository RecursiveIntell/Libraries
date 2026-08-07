use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message};
use std::time::Instant;

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn receipt_index_scales_to_10k_compactions() {
    let start = Instant::now();

    for i in 0..10_000 {
        let iter_start = Instant::now();
        let _response = compact_context(CompactRequest {
            hmac_key_path: None,
            session_id: format!("scale-{}", i),
            messages: vec![
                msg("system", &format!("scale test {}", i)),
                msg("user", &format!("message {}", i)),
            ],
            policy: CompactionPolicy::default(),
            focus: None,
        })
        .unwrap();

        let elapsed = iter_start.elapsed();

        // Verify every 1000 iterations
        if i > 0 && i % 1000 == 0 {
            assert!(
                elapsed.as_millis() < 200,
                "compaction {} took {}ms — indexing may be degrading",
                i,
                elapsed.as_millis()
            );
        }
    }

    let total = start.elapsed();
    assert!(
        total.as_secs() < 180,
        "10k compactions took {}s",
        total.as_secs()
    );
}

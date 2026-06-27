use context_governor::{
    archive_response_to_memory, compact_context, CompactRequest, CompactionPolicy,
    MemoryArchiveRecordV1, MemorySink, Message,
};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[derive(Default)]
struct RecordingSink {
    records: Vec<MemoryArchiveRecordV1>,
}

impl MemorySink for RecordingSink {
    fn archive_context_record(
        &mut self,
        record: &MemoryArchiveRecordV1,
    ) -> Result<Option<String>, context_governor::ContextGovernorError> {
        self.records.push(record.clone());
        Ok(Some(format!("fact:{}", record.item_id)))
    }
}

#[test]
fn memory_archive_records_are_explicit_and_result_bound() {
    let response = compact_context(CompactRequest {
        session_id: "memory".into(),
        messages: vec![
            msg("system", "system"),
            msg(
                "user",
                "Decision: archive this durable architecture choice.",
            ),
            msg("assistant", &"low value chatter ".repeat(400)),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            archive_memory_enabled: true,
            target_tokens: 120,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let mut sink = RecordingSink::default();
    let outcome = archive_response_to_memory(&response, &mut sink).unwrap();

    assert!(!outcome.record_ids.is_empty());
    assert_eq!(outcome.record_ids.len(), sink.records.len());
    assert!(sink
        .records
        .iter()
        .all(|r| r.receipt_id == response.receipt.receipt_id));
    assert!(sink.records.iter().all(|r| !r.content_blake3.is_empty()));
    assert!(sink
        .records
        .iter()
        .any(|r| r.archive_reason.contains("semantic")));
}

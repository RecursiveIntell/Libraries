use context_governor::{
    compact_context, compact_context_v2, finalize_compacted_response_v2, receipt_index,
    CheckpointStrategy, CompactRequest, CompactResponseV2, CompactionPolicy, ContextGovernorError,
    FileContextStore, Message, ReceiptActivationRequestV2, SearchScope, VersionedCompactResponse,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use tempfile::TempDir;

const OMITTED_MARKER: &str = "ARES_RECURSIVE_EXACT_MARKER_8f7d0b2a";
const CERTIFIED_KEY: [u8; 32] = [0x42; 32];

fn certified_store(path: impl AsRef<std::path::Path>) -> FileContextStore {
    FileContextStore::with_hmac_key(path, &CERTIFIED_KEY)
}

fn message(role: &str, content: impl Into<String>) -> Message {
    Message {
        role: role.to_string(),
        content: content.into(),
        ..Message::default()
    }
}

fn root_request(session_id: &str) -> CompactRequest {
    CompactRequest {
        session_id: session_id.to_string(),
        messages: vec![
            message(
                "system",
                "Preserve exact evidence and the active instruction.",
            ),
            message(
                "tool",
                format!(
                    "{}\n{}\n{}",
                    "old tool noise ".repeat(1_500),
                    OMITTED_MARKER,
                    "more old tool noise ".repeat(1_500)
                ),
            ),
            message("assistant", "The old tool output was inspected."),
            message("user", "Continue with the active verification gate."),
        ],
        policy: CompactionPolicy {
            target_tokens: 180,
            protect_first_n: 0,
            protect_last_n: 1,
            summary_max_chars: 320,
            ..CompactionPolicy::default()
        },
        focus: Some("recursive provenance verification".to_string()),
        hmac_key_path: None,
    }
}

fn next_request(parent: &CompactResponseV2, generation: u32) -> CompactRequest {
    let mut messages = parent.compacted_messages.clone();
    messages.push(message(
        "assistant",
        format!("generation {generation} projection checkpoint"),
    ));
    messages.push(message(
        "user",
        format!("Continue generation {generation}; do not quote omitted evidence."),
    ));
    CompactRequest {
        session_id: parent.receipt.session_id.clone(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 180,
            protect_first_n: 0,
            protect_last_n: 1,
            summary_max_chars: 320,
            ..CompactionPolicy::default()
        },
        focus: Some("recursive provenance verification".to_string()),
        hmac_key_path: None,
    }
}

fn bounded_growth_request(parent: &CompactResponseV2, generation: u32) -> CompactRequest {
    let mut messages = parent.compacted_messages.clone();
    messages.push(message(
        "tool",
        format!(
            "generation {generation} bounded-growth source\n{}",
            "deterministic disposable tool output ".repeat(1_500)
        ),
    ));
    messages.push(message(
        "assistant",
        format!("generation {generation} processed the new tool output"),
    ));
    messages.push(message(
        "user",
        format!("Continue generation {generation}; preserve the active gate."),
    ));
    CompactRequest {
        session_id: parent.receipt.session_id.clone(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 180,
            protect_first_n: 0,
            protect_last_n: 1,
            summary_max_chars: 320,
            max_lineage_generation: Some(32),
            max_provenance_bytes: Some(131_072),
            min_net_savings_tokens: Some(128),
            ..CompactionPolicy::default()
        },
        focus: Some("recursive provenance certification".to_string()),
        hmac_key_path: None,
    }
}

fn marker_source_id(response: &CompactResponseV2) -> String {
    response
        .source_evidence
        .iter()
        .find(|item| item.message.content.contains(OMITTED_MARKER))
        .expect("root receipt must archive the exact marker source")
        .source_id
        .clone()
}

fn save_root(store: &FileContextStore, session_id: &str) -> CompactResponseV2 {
    let root = store
        .compact_next_v2(root_request(session_id), None)
        .expect("generation 1 compaction");
    assert_eq!(root.receipt.generation, 1);
    assert!(root.receipt.parent_receipt.is_none());
    assert!(root.receipt.supersedes_receipt_id.is_none());
    assert!(!root
        .compacted_messages
        .iter()
        .any(|message| message.content.contains(OMITTED_MARKER)));
    store.save_v2(&root).expect("persist generation 1");
    store
        .load_v2(&root.receipt.receipt_id)
        .expect("reload persisted generation 1")
}

fn advance(
    store: &FileContextStore,
    parent: &CompactResponseV2,
    generation: u32,
) -> CompactResponseV2 {
    let response = store
        .compact_next_v2(next_request(parent, generation), None)
        .expect("construct next generation from canonical tip");
    assert_eq!(response.receipt.generation, generation);
    assert_eq!(
        response.receipt.supersedes_receipt_id.as_deref(),
        Some(parent.receipt.receipt_id.as_str())
    );
    store.save_v2(&response).expect("persist next generation");
    store
        .load_v2(&response.receipt.receipt_id)
        .expect("reload persisted next generation")
}

fn receipt_path(root: &TempDir, receipt_id: &str) -> std::path::PathBuf {
    root.path().join(format!("{receipt_id}.json"))
}

fn mutate_receipt(root: &TempDir, receipt_id: &str, mutate: impl FnOnce(&mut Value)) {
    let path = receipt_path(root, receipt_id);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    mutate(&mut value);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

#[test]
fn existing_v1_receipt_remains_readable_exact_and_parentless() {
    let tmp = TempDir::new().unwrap();
    let store = FileContextStore::new(tmp.path());
    let v1 = compact_context(root_request("legacy-v1")).unwrap();
    let exact = v1
        .exact_store
        .iter()
        .find(|item| item.content.contains(OMITTED_MARKER))
        .unwrap()
        .clone();
    store.save(&v1).unwrap();

    let loaded = store.load_versioned(&v1.receipt.receipt_id).unwrap();
    assert!(matches!(loaded, VersionedCompactResponse::V1(_)));
    let expanded = store
        .expand_lineage(&v1.receipt.receipt_id, &exact.item_id, usize::MAX)
        .unwrap();
    assert_eq!(expanded.content, exact.content);
    assert!(expanded.content.contains(OMITTED_MARKER));

    let raw: Value =
        serde_json::from_slice(&fs::read(receipt_path(&tmp, &v1.receipt.receipt_id)).unwrap())
            .unwrap();
    assert_eq!(raw["receipt"]["schema"], "ContextCompactionReceiptV1");
    assert!(raw["receipt"].get("parent_receipt").is_none());
    assert!(raw["receipt"].get("generation").is_none());
}

#[test]
fn mandatory_two_generation_restart_recovers_exact_omitted_marker() {
    let tmp = TempDir::new().unwrap();
    let first_store = certified_store(tmp.path());
    let first = save_root(&first_store, "mandatory-two-generation");
    let source_id = marker_source_id(&first);

    // Simulate a process restart: no in-memory parent or adapter state survives.
    drop(first_store);
    let restarted = certified_store(tmp.path());
    let second = advance(&restarted, &first, 2);
    assert!(!second
        .compacted_messages
        .iter()
        .any(|message| message.content.contains(OMITTED_MARKER)));
    assert!(second
        .receipt
        .covered_original_sources
        .iter()
        .any(|source| source.source_id == source_id));
    drop(restarted);

    let after_second_restart = certified_store(tmp.path());
    let expanded = after_second_restart
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap();
    assert!(expanded.content.contains(OMITTED_MARKER));
    assert_eq!(
        expanded.content_blake3,
        context_governor::hash_text(&expanded.content)
    );
}

#[test]
fn recursive_lineage_rejects_host_loss_of_non_text_message_metadata() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let mut request = root_request("host-metadata-roundtrip");
    request.messages[0].id = Some("provider-message-id".to_string());
    request.messages[0].name = Some("context_governor".to_string());
    let first = store.compact_next_v2(request, None).unwrap();
    store.save_v2(&first).unwrap();

    let mut child = next_request(&first, 2);
    child.messages[0].id = None;
    child.messages[0].name = None;

    assert!(matches!(
        store.compact_next_v2(child, None),
        Err(ContextGovernorError::LineageIntegrityMismatch { .. })
    ));
}

#[test]
fn recursive_lineage_still_rejects_parent_text_drift() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "host-text-drift");
    let mut child = next_request(&first, 2);
    child.messages[0].content.push_str(" tampered");

    assert!(matches!(
        store.compact_next_v2(child, None),
        Err(ContextGovernorError::LineageIntegrityMismatch { .. })
    ));
}

#[test]
fn generation_four_and_eight_survive_restart_between_every_generation() {
    for final_generation in [4_u32, 8_u32] {
        let tmp = TempDir::new().unwrap();
        let store = certified_store(tmp.path());
        let mut current = save_root(&store, &format!("repeat-{final_generation}"));
        let source_id = marker_source_id(&current);
        let root_original_tokens = current.receipt.original_approx_tokens;
        let root_compacted_tokens = current.receipt.compacted_approx_tokens;
        drop(store);

        for generation in 2..=final_generation {
            let restarted = certified_store(tmp.path());
            current = advance(&restarted, &current, generation);
            assert!(!current
                .compacted_messages
                .iter()
                .any(|message| message.content.contains(OMITTED_MARKER)));
        }

        let final_restart = certified_store(tmp.path());
        let expansion_started = std::time::Instant::now();
        let expanded = final_restart
            .expand_lineage(&current.receipt.receipt_id, &source_id, usize::MAX)
            .unwrap();
        let expansion_micros = expansion_started.elapsed().as_micros();
        assert!(expanded.content.contains(OMITTED_MARKER));
        assert_eq!(current.receipt.generation, final_generation);
        assert_eq!(
            current
                .receipt
                .covered_original_sources
                .iter()
                .filter(|source| source.source_id == source_id)
                .count(),
            1
        );
        if final_generation == 8 {
            let receipt_bytes = fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                })
                .map(|entry| entry.metadata().unwrap().len())
                .sum::<u64>();
            eprintln!(
                "LINEAGE_CERTIFICATION_METRICS {}",
                serde_json::json!({
                    "schema": "ContextGovernorLineageCertificationMetricsV1",
                    "generation": final_generation,
                    "root_original_tokens": root_original_tokens,
                    "root_compacted_tokens": root_compacted_tokens,
                    "final_compacted_tokens": current.receipt.compacted_approx_tokens,
                    "covered_original_sources": current.receipt.covered_original_sources.len(),
                    "receipt_json_bytes": receipt_bytes,
                    "exact_expansion_micros": expansion_micros,
                    "lineage_blake3": current.receipt.lineage_blake3,
                    "marker_blake3": expanded.content_blake3,
                })
            );
        }
    }
}

#[test]
#[ignore = "release certification gate: run cargo test --release --test lineage certified_growth_measurement_through_generation_thirty_two -- --ignored --nocapture"]
fn certified_growth_measurement_through_generation_thirty_two() {
    const PROMPT_SOURCE_ID_LIMIT: usize = 4;
    let tmp = TempDir::new().unwrap();
    let key = receipt_index::generate_hmac_key();
    let store = FileContextStore::with_hmac_key(tmp.path(), &key);
    let mut initial = root_request("growth-through-thirty-two");
    initial.policy.max_lineage_generation = Some(32);
    initial.policy.max_provenance_bytes = Some(131_072);
    initial.policy.min_net_savings_tokens = Some(128);
    let root = store.compact_next_v2(initial, None).unwrap();
    store.save_v2_with_hmac_key(&root, &key).unwrap();
    let mut current = store.load_v2(&root.receipt.receipt_id).unwrap();
    let source_id = marker_source_id(&current);
    let mut measurements = Vec::new();

    for generation in 1_u32..=32 {
        let compaction_micros = if generation > 1 {
            let restarted = FileContextStore::with_hmac_key(tmp.path(), &key);
            let started = std::time::Instant::now();
            let response = restarted
                .compact_next_v2(bounded_growth_request(&current, generation), None)
                .unwrap();
            restarted.save_v2_with_hmac_key(&response, &key).unwrap();
            current = restarted.load_v2(&response.receipt.receipt_id).unwrap();
            let elapsed = started.elapsed().as_micros();
            assert!(elapsed > 0);
            elapsed
        } else {
            0
        };
        if ![1, 2, 4, 8, 16, 32].contains(&generation) {
            continue;
        }
        let restarted = FileContextStore::with_hmac_key(tmp.path(), &key);
        let load_started = std::time::Instant::now();
        let loaded = restarted.load_v2(&current.receipt.receipt_id).unwrap();
        let restart_load_micros = load_started.elapsed().as_micros();
        let expand_started = std::time::Instant::now();
        let expanded = restarted
            .expand_lineage(&current.receipt.receipt_id, &source_id, usize::MAX)
            .unwrap();
        let exact_expand_micros = expand_started.elapsed().as_micros();
        assert!(expanded.content.contains(OMITTED_MARKER));
        let cumulative_receipt_bytes = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            })
            .map(|entry| entry.metadata().unwrap().len())
            .sum::<u64>();
        let receipt_bytes = fs::metadata(receipt_path(&tmp, &current.receipt.receipt_id))
            .unwrap()
            .len();
        let provenance_bytes = serde_json::to_vec(&loaded.receipt.covered_original_sources)
            .unwrap()
            .len();
        let prompt_provenance_bytes = loaded
            .receipt
            .covered_original_sources
            .iter()
            .take(PROMPT_SOURCE_ID_LIMIT)
            .map(|source| source.source_id.len() + 1)
            .sum::<usize>()
            + if loaded.receipt.covered_original_sources.len() > PROMPT_SOURCE_ID_LIMIT {
                72
            } else {
                0
            };
        measurements.push(serde_json::json!({
            "generation": generation,
            "receipt_bytes": receipt_bytes,
            "cumulative_receipt_bytes": cumulative_receipt_bytes,
            "provenance_bytes": provenance_bytes,
            "prompt_provenance_bytes": prompt_provenance_bytes,
            "covered_original_sources": loaded.receipt.covered_original_sources.len(),
            "input_tokens": loaded.receipt.original_approx_tokens,
            "output_tokens": loaded.receipt.compacted_approx_tokens,
            "net_saved_tokens": loaded.receipt.token_savings_estimate,
            "compaction_micros": compaction_micros,
            "restart_load_micros": restart_load_micros,
            "exact_expand_micros": exact_expand_micros,
        }));
    }

    eprintln!(
        "LINEAGE_GROWTH_CERTIFICATION_METRICS {}",
        serde_json::json!({
            "schema": "ContextGovernorLineageGrowthMetricsV2",
            "projection_source_id_limit": PROMPT_SOURCE_ID_LIMIT,
            "samples": measurements,
        })
    );
    let final_measurement = measurements.last().unwrap();
    assert!(
        final_measurement["prompt_provenance_bytes"]
            .as_u64()
            .unwrap()
            <= 512
    );
    assert!(final_measurement["provenance_bytes"].as_u64().unwrap() <= 131_072);
    assert!(measurements
        .iter()
        .all(|sample| sample["net_saved_tokens"].as_i64().unwrap() >= 128));
}

#[test]
fn lineage_construction_has_deterministic_replay_digest() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let left = store
        .compact_next_v2(root_request("deterministic-replay"), None)
        .unwrap();
    let right = store
        .compact_next_v2(root_request("deterministic-replay"), None)
        .unwrap();

    assert_ne!(left.receipt.receipt_id, right.receipt.receipt_id);
    assert_eq!(left.receipt.lineage_blake3, right.receipt.lineage_blake3);
    assert_eq!(left.receipt.lineage_sha256, right.receipt.lineage_sha256);
    assert_eq!(
        left.receipt
            .covered_original_sources
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>(),
        right
            .receipt
            .covered_original_sources
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>()
    );

    store.save_v2(&left).unwrap();
    let child_request = next_request(&left, 2);
    let first_child = store
        .compact_next_v2(child_request.clone(), Some(&left.receipt.receipt_id))
        .unwrap();
    let replayed_child = store
        .compact_next_v2(child_request, Some(&left.receipt.receipt_id))
        .unwrap();

    assert_ne!(
        first_child.receipt.receipt_id,
        replayed_child.receipt.receipt_id
    );
    assert_eq!(
        first_child.receipt.lineage_blake3,
        replayed_child.receipt.lineage_blake3
    );
    assert_eq!(
        first_child.receipt.lineage_sha256,
        replayed_child.receipt.lineage_sha256
    );
    assert_eq!(
        first_child
            .receipt
            .covered_original_sources
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>(),
        replayed_child
            .receipt
            .covered_original_sources
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn ancestor_provenance_tampering_fails_closed() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "ancestor-tamper");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);

    mutate_receipt(&tmp, &first.receipt.receipt_id, |value| {
        value["receipt"]["generation"] = Value::from(7);
    });
    assert!(store
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .is_err());
}

#[test]
fn newest_summary_tampering_is_detected_without_blocking_exact_source_recovery() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "newest-summary-tamper");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);

    mutate_receipt(&tmp, &second.receipt.receipt_id, |value| {
        let messages = value["compacted_messages"].as_array_mut().unwrap();
        let summary = messages
            .iter_mut()
            .find(|message| message["name"] == "context_governor")
            .unwrap();
        summary["content"] = Value::String("tampered summary projection".to_string());
        value["receipt"]["compacted_transcript_blake3"] = Value::String("00".repeat(32));
        value["receipt"]["compacted_transcript_sha256"] = Value::String("11".repeat(32));
        value["receipt"]["compacted_approx_tokens"] = Value::from(1);
        value["receipt"]["receipt_identity_blake3"] = Value::String("22".repeat(32));
        value["receipt"]["receipt_identity_sha256"] = Value::String("33".repeat(32));
    });

    assert!(store.load_v2(&second.receipt.receipt_id).is_err());
    let expanded = store
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap();
    assert!(expanded.content.contains(OMITTED_MARKER));
}

#[test]
fn missing_parent_receipt_fails_closed_after_restart() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "missing-parent");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);
    fs::rename(
        receipt_path(&tmp, &first.receipt.receipt_id),
        tmp.path().join("missing-parent.backup"),
    )
    .unwrap();

    let restarted = certified_store(tmp.path());
    assert!(restarted
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .is_err());
    assert!(restarted
        .compact_next_v2(next_request(&second, 3), None)
        .is_err());
}

#[test]
fn missing_or_hash_mismatched_original_source_fails_closed() {
    for mutation in ["missing", "content", "hash"] {
        let tmp = TempDir::new().unwrap();
        let store = certified_store(tmp.path());
        let first = save_root(&store, &format!("source-{mutation}"));
        let source_id = marker_source_id(&first);
        let second = advance(&store, &first, 2);

        mutate_receipt(&tmp, &first.receipt.receipt_id, |value| {
            let sources = value["source_evidence"].as_array_mut().unwrap();
            let index = sources
                .iter()
                .position(|source| source["source_id"] == source_id)
                .unwrap();
            match mutation {
                "missing" => {
                    sources.remove(index);
                }
                "content" => {
                    sources[index]["message"]["content"] =
                        Value::String("corrupt source".to_string());
                }
                "hash" => {
                    sources[index]["content_sha256"] = Value::String("00".repeat(32));
                }
                _ => unreachable!(),
            }
        });

        assert!(store
            .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
            .is_err());
    }
}

#[test]
fn superseded_receipt_remains_directly_recoverable() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "superseded-recovery");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);

    let ancestor = store
        .expand_lineage(&first.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap();
    let newest = store
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap();
    assert_eq!(ancestor.content, newest.content);
    assert_eq!(ancestor.content_blake3, newest.content_blake3);
}

#[test]
fn retention_cannot_remove_ancestry_required_by_a_retained_descendant() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "retention-ancestry");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);

    let prune = store.prune_receipts_keep_last(1).unwrap();
    assert_eq!(prune.removed_receipts, 0);
    assert_eq!(
        prune.protected_receipt_ids,
        vec![first.receipt.receipt_id.clone()]
    );
    assert!(receipt_path(&tmp, &first.receipt.receipt_id).exists());
    assert!(store
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap()
        .content
        .contains(OMITTED_MARKER));
}

#[test]
fn duplicate_child_and_cyclic_parent_graph_fail_closed() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "duplicate-child");
    let left = store
        .compact_next_v2(next_request(&first, 2), None)
        .unwrap();
    let right = store
        .compact_next_v2(next_request(&first, 2), Some(&first.receipt.receipt_id))
        .unwrap();
    store.save_v2(&left).unwrap();
    assert!(store.save_v2(&right).is_err());

    mutate_receipt(&tmp, &first.receipt.receipt_id, |value| {
        value["receipt"]["parent_receipt"] =
            serde_json::to_value(left.receipt.parent_receipt.clone()).unwrap();
        value["receipt"]["supersedes_receipt_id"] = Value::String(first.receipt.receipt_id.clone());
    });
    assert!(store
        .expand_lineage(
            &left.receipt.receipt_id,
            &marker_source_id(&first),
            usize::MAX
        )
        .is_err());
}

#[test]
fn explicit_v1_bridge_preserves_only_proven_legacy_exact_sources() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let key_path = tmp.path().join("legacy-v1.key");
    receipt_index::save_hmac_key(&key_path, &CERTIFIED_KEY).unwrap();
    let mut v1_request = root_request("explicit-v1-bridge");
    v1_request.hmac_key_path = Some(key_path.display().to_string());
    let v1 = compact_context(v1_request).unwrap();
    let v1_exact = v1
        .exact_store
        .iter()
        .find(|item| item.content.contains(OMITTED_MARKER))
        .unwrap()
        .clone();
    store
        .save_with_status_with_hmac_key(&v1, &CERTIFIED_KEY)
        .unwrap();
    let before = fs::read(receipt_path(&tmp, &v1.receipt.receipt_id)).unwrap();

    let mut messages = v1.compacted_messages.clone();
    messages.push(message(
        "user",
        "Continue after the explicit legacy bridge.",
    ));
    let child = store
        .compact_next_v2(
            CompactRequest {
                session_id: v1.receipt.session_id.clone(),
                messages,
                policy: root_request("unused").policy,
                focus: None,
                hmac_key_path: None,
            },
            Some(&v1.receipt.receipt_id),
        )
        .unwrap();
    assert_eq!(child.receipt.generation, 2);
    assert_eq!(
        child
            .receipt
            .parent_receipt
            .as_ref()
            .unwrap()
            .receipt_schema,
        "ContextCompactionReceiptV1"
    );
    assert!(child
        .receipt
        .covered_original_sources
        .iter()
        .any(|source| { source.origin_item_id.as_deref() == Some(v1_exact.item_id.as_str()) }));
    store.save_v2(&child).unwrap();
    assert_eq!(
        fs::read(receipt_path(&tmp, &v1.receipt.receipt_id)).unwrap(),
        before
    );
    assert!(store
        .expand_lineage(&child.receipt.receipt_id, &v1_exact.item_id, usize::MAX)
        .unwrap()
        .content
        .contains(OMITTED_MARKER));
}

#[test]
fn legacy_receipts_are_never_auto_selected_as_v2_parent() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    for suffix in ["one", "two"] {
        let mut request = root_request("legacy-auto-parent");
        request.messages.push(message("assistant", suffix));
        store.save(&compact_context(request).unwrap()).unwrap();
    }

    let root = store
        .compact_next_v2(root_request("legacy-auto-parent"), None)
        .unwrap();
    assert_eq!(root.receipt.generation, 1);
    assert!(root.receipt.parent_receipt.is_none());
}

#[test]
fn source_identity_manifest_has_no_duplicate_entries() {
    let mut request = root_request("duplicate-source-identities");
    request.messages.push(message("assistant", "identical"));
    request.messages.push(message("assistant", "identical"));
    let response = compact_context_v2(request).unwrap();
    let ids = response
        .receipt
        .covered_original_sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), response.receipt.covered_original_sources.len());
    assert_eq!(
        response
            .source_evidence
            .iter()
            .map(|source| (source.origin_message_index, &source.message.content))
            .filter(|(_, content)| *content == "identical")
            .collect::<BTreeMap<_, _>>()
            .len(),
        2
    );
}

#[test]
fn projection_finalization_cannot_invent_or_mutate_provenance() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let ring = receipt_index::KeyRing::new(CERTIFIED_KEY.to_vec());
    let response = store
        .compact_next_v2(root_request("projection-only-finalize"), None)
        .unwrap();
    let original_sources = response.receipt.covered_original_sources.clone();
    let original_evidence = response.source_evidence.clone();
    let mut projected_messages = response.compacted_messages.clone();
    projected_messages
        .iter_mut()
        .find(|message| message.name.as_deref() == Some("context_governor"))
        .unwrap()
        .content = "audited LLM summary projection without provenance claims".to_string();

    let finalized =
        finalize_compacted_response_v2(response.clone(), projected_messages, &ring).unwrap();
    assert_eq!(finalized.receipt.covered_original_sources, original_sources);
    assert_eq!(finalized.source_evidence, original_evidence);

    let mut invented = response;
    invented.receipt.covered_original_sources[0].content_sha256 = "00".repeat(32);
    assert!(finalize_compacted_response_v2(
        invented.clone(),
        invented.compacted_messages.clone(),
        &ring
    )
    .is_err());
}

#[test]
fn recomputed_attacker_provenance_cannot_cross_finalize_or_store_authority() {
    let trusted_dir = TempDir::new().unwrap();
    let attacker_dir = TempDir::new().unwrap();
    let trusted_key = [0x31; 32];
    let attacker_key = [0x93; 32];
    let trusted_store = FileContextStore::with_hmac_key(trusted_dir.path(), &trusted_key);
    let attacker_store = FileContextStore::with_hmac_key(attacker_dir.path(), &attacker_key);
    let mut forged_request = root_request("recomputed-provenance-forgery");
    forged_request.messages[1].content = format!(
        "attacker-controlled exact source with fully recomputed hashes {}",
        "forged ".repeat(1_000)
    );
    // This is not a stale-hash mutation: it is a fully self-consistent V2
    // receipt whose provenance, source IDs, lineage, and signatures were all
    // recomputed under an attacker-controlled authority.
    let forged = attacker_store
        .compact_next_v2(forged_request, None)
        .unwrap();
    let trusted_ring = receipt_index::KeyRing::new(trusted_key.to_vec());

    assert!(matches!(
        finalize_compacted_response_v2(
            forged.clone(),
            forged.compacted_messages.clone(),
            &trusted_ring
        ),
        Err(ContextGovernorError::ReceiptIntegrityFailed { .. })
    ));
    assert!(matches!(
        trusted_store.save_v2_with_hmac_key(&forged, &trusted_key),
        Err(ContextGovernorError::ReceiptIntegrityFailed { .. })
    ));
    assert!(!receipt_path(&trusted_dir, &forged.receipt.receipt_id).exists());
    assert!(!trusted_dir
        .path()
        .join(".pending")
        .join(format!("{}.json", forged.receipt.receipt_id))
        .exists());
}

#[test]
fn signer_admission_failure_never_publishes_and_valid_retry_succeeds() {
    let tmp = TempDir::new().unwrap();
    let key = [0x52; 32];
    let wrong_key = [0xa4; 32];
    let store = FileContextStore::with_hmac_key(tmp.path(), &key);

    let valid = store
        .compact_next_v2(root_request("unsigned-admission-retry"), None)
        .unwrap();
    let mut unsigned = valid.clone();
    unsigned.hmac = None;
    unsigned.evidence_hmac = None;
    assert!(matches!(
        store.save_v2(&unsigned),
        Err(ContextGovernorError::ReceiptIntegrityMissing { .. })
    ));
    assert!(!receipt_path(&tmp, &valid.receipt.receipt_id).exists());
    assert!(!tmp
        .path()
        .join(".pending")
        .join(format!("{}.json", valid.receipt.receipt_id))
        .exists());

    assert!(matches!(
        store.save_v2_with_hmac_key(&valid, &wrong_key),
        Err(ContextGovernorError::WrongConfiguredKeyId { .. })
    ));
    assert!(!receipt_path(&tmp, &valid.receipt.receipt_id).exists());
    store.save_v2_with_hmac_key(&valid, &key).unwrap();
    assert!(receipt_path(&tmp, &valid.receipt.receipt_id).exists());
}

#[test]
fn pending_receipt_is_inert_until_matching_committed_projection_activates() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let response = store
        .compact_next_v2(root_request("pending-two-phase"), None)
        .unwrap();
    let source_id = marker_source_id(&response);
    let prepared = store.prepare_v2(&response).unwrap();

    assert!(prepared.verified);
    assert!(prepared.pending_path.exists());
    assert!(!receipt_path(&tmp, &response.receipt.receipt_id).exists());
    assert_eq!(
        store.resolve_lineage_tip("pending-two-phase").unwrap(),
        None
    );
    assert!(store
        .search(OMITTED_MARKER, 10, SearchScope::All)
        .unwrap()
        .is_empty());
    assert!(matches!(
        store.expand_lineage(&response.receipt.receipt_id, &source_id, usize::MAX),
        Err(ContextGovernorError::ReceiptNotFound(_))
    ));
    let recovered = certified_store(tmp.path())
        .list_pending_v2(Some(&response.receipt.receipt_id))
        .unwrap();
    assert_eq!(recovered, vec![prepared.clone()]);

    let mut mismatched = prepared.expected_compacted_messages.clone();
    mismatched[0].content.push_str(" damaged after host commit");
    assert!(matches!(
        store.activate_v2(ReceiptActivationRequestV2 {
            receipt_id: response.receipt.receipt_id.clone(),
            committed_messages: mismatched,
        }),
        Err(ContextGovernorError::CommittedTranscriptMismatch(_))
    ));
    assert!(prepared.pending_path.exists());
    assert!(!receipt_path(&tmp, &response.receipt.receipt_id).exists());

    let activated = certified_store(tmp.path())
        .activate_v2(ReceiptActivationRequestV2 {
            receipt_id: response.receipt.receipt_id.clone(),
            committed_messages: prepared.expected_compacted_messages,
        })
        .unwrap();
    assert!(activated.activated && activated.verified);
    assert!(!prepared.pending_path.exists());
    assert!(activated.path.exists());
    assert_eq!(
        store.resolve_lineage_tip("pending-two-phase").unwrap(),
        Some(response.receipt.receipt_id)
    );
}

#[test]
fn pending_receipt_can_be_authenticated_and_discarded_after_host_abort() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let response = store
        .compact_next_v2(root_request("pending-discard"), None)
        .unwrap();
    let prepared = store.prepare_v2(&response).unwrap();
    let discarded = certified_store(tmp.path())
        .discard_pending_v2(&prepared.receipt_id)
        .unwrap();
    assert!(discarded.discarded);
    assert!(!prepared.pending_path.exists());
    assert_eq!(store.resolve_lineage_tip("pending-discard").unwrap(), None);
}

#[test]
fn v2_authoritative_operations_fail_closed_without_governed_authority() {
    let tmp = TempDir::new().unwrap();
    let certified = certified_store(tmp.path());
    let response = save_root(&certified, "authority-required");
    let source_id = marker_source_id(&response);
    let unauthenticated = FileContextStore::new(tmp.path());

    for error in [
        unauthenticated
            .load_v2(&response.receipt.receipt_id)
            .unwrap_err(),
        unauthenticated
            .expand_lineage(&response.receipt.receipt_id, &source_id, usize::MAX)
            .unwrap_err(),
        unauthenticated
            .search(OMITTED_MARKER, 10, SearchScope::All)
            .unwrap_err(),
        unauthenticated.prune_receipts_keep_last(0).unwrap_err(),
        unauthenticated
            .compact_next_v2(next_request(&response, 2), None)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            ContextGovernorError::ReceiptIntegrityUnavailable { .. }
        ));
    }
}

#[test]
fn v2_save_is_append_only_and_hmac_covers_issued_bytes() {
    let tmp = TempDir::new().unwrap();
    let key = receipt_index::generate_hmac_key();
    let store = FileContextStore::with_hmac_key(tmp.path(), &key);
    let response = store
        .compact_next_v2(root_request("append-only-hmac"), None)
        .unwrap();
    store.save_v2_with_hmac_key(&response, &key).unwrap();
    assert!(store.save_v2_with_hmac_key(&response, &key).is_err());

    let value: Value = serde_json::from_slice(
        &fs::read(receipt_path(&tmp, &response.receipt.receipt_id)).unwrap(),
    )
    .unwrap();
    assert!(receipt_index::KeyRing::new(key).verify_json(&value, "hmac"));
}

#[test]
fn certified_store_rejects_tampered_ancestry_before_expand_or_parent_selection() {
    let tmp = TempDir::new().unwrap();
    let key = receipt_index::generate_hmac_key();
    let store = FileContextStore::with_hmac_key(tmp.path(), &key);
    let first = store
        .compact_next_v2(root_request("certified-tamper"), None)
        .unwrap();
    store.save_v2_with_hmac_key(&first, &key).unwrap();
    let first = store.load_v2(&first.receipt.receipt_id).unwrap();
    let source_id = marker_source_id(&first);
    let second = store
        .compact_next_v2(next_request(&first, 2), None)
        .expect("verified parent may be selected");
    store.save_v2_with_hmac_key(&second, &key).unwrap();

    // Mutating an ancestor without re-signing it must block every authoritative
    // consumer, including exact fallback and automatic restart parent choice.
    mutate_receipt(&tmp, &first.receipt.receipt_id, |value| {
        value["source_evidence"][0]["message"]["content"] =
            Value::String("attacker controlled source".to_string());
    });
    assert!(matches!(
        store.expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX),
        Err(context_governor::ContextGovernorError::ReceiptIntegrityFailed { .. })
    ));
    assert!(matches!(
        store.compact_next_v2(next_request(&second, 3), None),
        Err(context_governor::ContextGovernorError::ReceiptIntegrityFailed { .. })
    ));
}

#[test]
fn certified_v2_policy_rejects_generation_and_provenance_budget_growth() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let mut root_compact_request = root_request("bounded-v2");
    root_compact_request.policy.max_lineage_generation = Some(1);
    let root = store.compact_next_v2(root_compact_request, None).unwrap();
    store.save_v2(&root).unwrap();
    let mut child_request = next_request(&root, 2);
    child_request.policy.max_lineage_generation = Some(1);
    assert!(matches!(
        store.compact_next_v2(child_request, None),
        Err(
            context_governor::ContextGovernorError::LineageGenerationLimit {
                generation: 2,
                maximum_generation: 1,
            }
        )
    ));

    let mut tiny_budget = root_request("bounded-v2-provenance");
    tiny_budget.policy.max_provenance_bytes = Some(1);
    assert!(matches!(
        compact_context_v2(tiny_budget),
        Err(context_governor::ContextGovernorError::ProvenanceBudgetExceeded { .. })
    ));

    // Small incremental transcripts used to create a larger child receipt
    // than their parent. A certified runtime must decline that compaction,
    // not issue a negative-value receipt and retry forever at pressure.
    let root = save_root(&store, "bounded-v2-net-savings");
    let mut no_benefit = next_request(&root, 2);
    no_benefit.policy.checkpoint.strategy = CheckpointStrategy::Off;
    no_benefit.policy.min_net_savings_tokens = Some(128);
    assert!(matches!(
        store.compact_next_v2(no_benefit, None),
        Err(
            context_governor::ContextGovernorError::CompactionNoNetBenefit {
                minimum_savings: 128,
                ..
            }
        )
    ));
}

#[test]
fn checkpoint_candidate_survives_min_net_gate_for_host_llm() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let root = save_root(&store, "checkpoint-candidate");
    let mut request = next_request(&root, 2);
    request.policy.checkpoint.strategy = CheckpointStrategy::IneffectiveOnly;
    request.policy.min_net_savings_tokens = Some(128);

    let candidate = store.compact_next_v2(request, None).unwrap();

    assert!(candidate.receipt.token_savings_estimate < 128);
    assert_eq!(candidate.receipt.generation, 2);
}

#[test]
fn derived_index_rebuild_does_not_change_lineage_authority() {
    let tmp = TempDir::new().unwrap();
    let store = certified_store(tmp.path());
    let first = save_root(&store, "derived-index-rebuild");
    let source_id = marker_source_id(&first);
    let second = advance(&store, &first, 2);

    let initial = store
        .search(OMITTED_MARKER, 10, SearchScope::ExactStore)
        .unwrap();
    assert!(!initial.is_empty());
    let index = tmp.path().join(".receipt-index.sqlite3");
    assert!(index.exists());
    fs::remove_file(index).unwrap();

    let restarted = certified_store(tmp.path());
    let rebuilt = restarted
        .search(OMITTED_MARKER, 10, SearchScope::ExactStore)
        .unwrap();
    assert_eq!(
        rebuilt
            .iter()
            .map(|hit| (&hit.receipt_id, &hit.hit.content_blake3))
            .collect::<Vec<_>>(),
        initial
            .iter()
            .map(|hit| (&hit.receipt_id, &hit.hit.content_blake3))
            .collect::<Vec<_>>()
    );
    assert!(restarted
        .expand_lineage(&second.receipt.receipt_id, &source_id, usize::MAX)
        .unwrap()
        .content
        .contains(OMITTED_MARKER));
}

use semantic_memory::journal::{
    encode_fact_create_payload, encode_fact_supersede_payload, envelope_digest, payload_digest,
    FactCreatePayloadV1, FactCreateReplicaEnvelopeV1, FactSupersedePayloadV1,
    FactSupersedeReplicaEnvelopeV1, ReplicaApplyOutcome, FACT_CREATE_OPERATION,
    FACT_CREATE_PAYLOAD_SCHEMA, FACT_SUPERSEDE_OPERATION, FACT_SUPERSEDE_PAYLOAD_SCHEMA,
    GENESIS_PREDECESSOR,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use tempfile::TempDir;

fn store(temp: &TempDir) -> MemoryStore {
    MemoryStore::open_with_embedder(
        MemoryConfig {
            base_dir: temp.path().into(),
            ..Default::default()
        },
        Box::new(MockEmbedder::new(768)),
    )
    .unwrap()
}
fn create(id: &str) -> FactCreateReplicaEnvelopeV1 {
    let payload = encode_fact_create_payload(&FactCreatePayloadV1 {
        fact_id: id.into(),
        namespace: "n".into(),
        content: "old".into(),
        source: None,
        metadata: None,
    })
    .unwrap();
    let pd = payload_digest(&payload);
    FactCreateReplicaEnvelopeV1 {
        home_device_id: "d".into(),
        store_id: "s".into(),
        stream_epoch: 1,
        sequence: 1,
        operation_kind: FACT_CREATE_OPERATION.into(),
        payload_schema: FACT_CREATE_PAYLOAD_SCHEMA.into(),
        payload,
        payload_digest: pd,
        predecessor_digest: GENESIS_PREDECESSOR,
        envelope_digest: envelope_digest(
            "d",
            "s",
            1,
            1,
            FACT_CREATE_OPERATION,
            FACT_CREATE_PAYLOAD_SCHEMA,
            &GENESIS_PREDECESSOR,
            &pd,
        ),
    }
}
fn supersede(
    old: &str,
    old_content: &str,
    new: &str,
    sequence: i64,
    predecessor: [u8; 32],
) -> FactSupersedeReplicaEnvelopeV1 {
    let replacement = encode_fact_create_payload(&FactCreatePayloadV1 {
        fact_id: new.into(),
        namespace: "n".into(),
        content: "new".into(),
        source: None,
        metadata: None,
    })
    .unwrap();
    let p = FactSupersedePayloadV1 {
        schema_version: FACT_SUPERSEDE_PAYLOAD_SCHEMA.into(),
        old_fact_id: old.into(),
        new_fact_id: new.into(),
        replacement_payload_digest: payload_digest(&replacement),
        replacement_payload: replacement,
        semantic_predecessor_digest: semantic_memory::journal::semantic_predecessor_digest(
            old_content,
        ),
        current_head_digest: semantic_memory::journal::semantic_head_digest(
            old,
            &semantic_memory::journal::semantic_predecessor_digest(old_content),
        ),
        owner_valid_at: "2026-08-01T00:00:00Z".into(),
        owner_recorded_at: "2026-08-01T00:00:01Z".into(),
        authority_digest: format!("blake3:{}", "3".repeat(64)),
        authority_receipt_id: "00000000-0000-0000-0000-000000000003".into(),
        authority_receipt_digest: "4".repeat(64),
        transition_record_id: "00000000-0000-0000-0000-000000000004".into(),
        transition_digest: "5".repeat(64),
        receipt_id: new.into(),
        receipt_digest: "6".repeat(64),
    };
    let payload = encode_fact_supersede_payload(&p).unwrap();
    let pd = payload_digest(&payload);
    FactSupersedeReplicaEnvelopeV1 {
        home_device_id: "d".into(),
        store_id: "s".into(),
        stream_epoch: 1,
        sequence,
        operation_kind: FACT_SUPERSEDE_OPERATION.into(),
        payload_schema: FACT_SUPERSEDE_PAYLOAD_SCHEMA.into(),
        payload,
        payload_digest: pd,
        predecessor_digest: predecessor,
        envelope_digest: envelope_digest(
            "d",
            "s",
            1,
            sequence,
            FACT_SUPERSEDE_OPERATION,
            FACT_SUPERSEDE_PAYLOAD_SCHEMA,
            &predecessor,
            &pd,
        ),
    }
}

#[tokio::test]
async fn apply_duplicate_stale_and_tamper_are_closed() {
    let temp = TempDir::new().unwrap();
    let s = store(&temp);
    let old = "00000000-0000-0000-0000-000000000001";
    let new = "00000000-0000-0000-0000-000000000002";
    let c = create(old);
    assert!(matches!(
        s.apply_verified_fact_create(c.clone()).await.unwrap(),
        ReplicaApplyOutcome::Applied { .. }
    ));
    let e = supersede(old, "old", new, 2, c.envelope_digest);
    assert_eq!(
        s.apply_verified_fact_supersede(e.clone()).await.unwrap(),
        ReplicaApplyOutcome::Applied {
            sequence: 2,
            fact_id: new.into()
        }
    );
    assert_eq!(
        s.apply_verified_fact_supersede(e.clone()).await.unwrap(),
        ReplicaApplyOutcome::Duplicate { sequence: 2 }
    );
    let stale = supersede(
        old,
        "old",
        "00000000-0000-0000-0000-000000000006",
        3,
        e.envelope_digest,
    );
    assert_eq!(
        s.apply_verified_fact_supersede(stale.clone())
            .await
            .unwrap(),
        ReplicaApplyOutcome::StalePredecessor {
            old_fact_id: old.into()
        }
    );
    assert_eq!(
        s.apply_verified_fact_supersede(stale.clone())
            .await
            .unwrap(),
        ReplicaApplyOutcome::StalePredecessor {
            old_fact_id: old.into()
        },
        "an exact stale retry must remain a conflict"
    );
    let next = supersede(
        new,
        "new",
        "00000000-0000-0000-0000-000000000008",
        3,
        e.envelope_digest,
    );
    assert_eq!(
        s.apply_verified_fact_supersede(next).await.unwrap(),
        ReplicaApplyOutcome::Applied {
            sequence: 3,
            fact_id: "00000000-0000-0000-0000-000000000008".into()
        }
    );
    let mut bad = supersede(
        new,
        "new",
        "00000000-0000-0000-0000-000000000007",
        3,
        e.envelope_digest,
    );
    bad.payload_digest[0] ^= 1;
    assert!(s.apply_verified_fact_supersede(bad).await.is_err());
}

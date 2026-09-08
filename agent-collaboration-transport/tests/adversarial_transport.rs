#![allow(clippy::expect_used)]

use agent_collaboration_transport::{
    read_frame, receive_authenticated_frame, send_authenticated_frame, write_frame, FrameError,
    ReplayCache, StaticIdentity,
};
use chrono::{Duration, Utc};
use stack_ids::{AgentId, ContentDigest};
use tokio::io::{duplex, AsyncWriteExt};

fn identities() -> (StaticIdentity, StaticIdentity, StaticIdentity) {
    (
        StaticIdentity::new(AgentId::new("agent-a"), vec![1_u8; 32]).expect("identity A"),
        StaticIdentity::new(AgentId::new("agent-b"), vec![2_u8; 32]).expect("identity B"),
        StaticIdentity::new(AgentId::new("agent-c"), vec![3_u8; 32]).expect("identity C"),
    )
}

fn valid_frame(
    sender: &StaticIdentity,
    receiver: &StaticIdentity,
    manifest: &ContentDigest,
) -> agent_collaboration_transport::FrameV1 {
    sender
        .issue_frame(
            receiver.agent_id.clone(),
            manifest.clone(),
            (Utc::now() + Duration::minutes(5)).to_rfc3339(),
            b"payload".to_vec(),
        )
        .expect("frame")
}

#[tokio::test]
async fn tampered_payload_and_wrong_key_fail_authentication() {
    let (alice, bob, carol) = identities();
    let manifest = ContentDigest::compute(b"manifest");
    let mut tampered = valid_frame(&alice, &bob, &manifest);
    tampered.payload = b"changed".to_vec();
    let (mut writer, mut reader) = duplex(16 * 1024);
    send_authenticated_frame(&mut writer, &tampered)
        .await
        .expect("send");
    let result =
        receive_authenticated_frame(&mut reader, &bob, &alice, &manifest, &ReplayCache::new())
            .await;
    assert!(matches!(
        result,
        Err(
            agent_collaboration_transport::transport::TransportError::Identity(
                agent_collaboration_transport::IdentityError::AuthenticationFailed
            )
        )
    ));

    let frame = valid_frame(&alice, &bob, &manifest);
    let (mut writer, mut reader) = duplex(16 * 1024);
    send_authenticated_frame(&mut writer, &frame)
        .await
        .expect("send");
    let result =
        receive_authenticated_frame(&mut reader, &bob, &carol, &manifest, &ReplayCache::new())
            .await;
    assert!(matches!(
        result,
        Err(agent_collaboration_transport::transport::TransportError::Identity(_))
    ));
}

#[tokio::test]
async fn expired_and_wrong_audience_frames_fail_closed() {
    let (alice, bob, carol) = identities();
    let manifest = ContentDigest::compute(b"manifest");
    let expired = alice
        .issue_frame(
            bob.agent_id.clone(),
            manifest.clone(),
            (Utc::now() - Duration::minutes(1)).to_rfc3339(),
            b"payload".to_vec(),
        )
        .expect("expired frame");
    assert!(matches!(
        bob.verify_frame(&expired, &alice, &manifest, Utc::now()),
        Err(agent_collaboration_transport::IdentityError::Expired)
    ));

    let wrong_audience = valid_frame(&alice, &carol, &manifest);
    assert!(matches!(
        bob.verify_frame(&wrong_audience, &alice, &manifest, Utc::now()),
        Err(agent_collaboration_transport::IdentityError::Invalid(_))
    ));
    let wrong_manifest = ContentDigest::compute(b"other-manifest");
    let valid = valid_frame(&alice, &bob, &manifest);
    assert!(matches!(
        bob.verify_frame(&valid, &alice, &wrong_manifest, Utc::now()),
        Err(agent_collaboration_transport::IdentityError::Invalid(_))
    ));
}

#[test]
fn replay_cache_is_bounded_and_rejects_duplicates() {
    let cache = ReplayCache::with_capacity(1);
    assert!(cache.admit("nonce-1"));
    assert!(!cache.admit("nonce-1"));
    assert!(!cache.admit("nonce-2"));
    assert_eq!(cache.len(), 1);
}

#[tokio::test]
async fn oversized_length_is_rejected_before_payload_allocation() {
    let (mut writer, mut reader) = duplex(16);
    writer
        .write_u32((agent_collaboration_transport::MAX_FRAME_BYTES as u32) + 1)
        .await
        .expect("length");
    let result = read_frame(&mut reader).await;
    assert!(matches!(result, Err(FrameError::TooLarge)));

    let (mut writer, _reader) = duplex(16 * 1024);
    let mut frame = valid_frame(
        &StaticIdentity::new(AgentId::new("a"), vec![1; 32]).expect("a"),
        &StaticIdentity::new(AgentId::new("b"), vec![2; 32]).expect("b"),
        &ContentDigest::compute(b"manifest"),
    );
    frame.payload = vec![0; agent_collaboration_transport::MAX_FRAME_BYTES];
    assert!(matches!(
        write_frame(&mut writer, &frame).await,
        Err(FrameError::TooLarge)
    ));
}

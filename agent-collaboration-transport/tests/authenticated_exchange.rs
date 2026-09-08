use agent_collaboration_transport::{
    receive_authenticated_frame, send_authenticated_frame, ReplayCache, StaticIdentity,
};
use chrono::{Duration, Utc};
use stack_ids::{AgentId, ContentDigest};
use tokio::io::duplex;

fn identities() -> (StaticIdentity, StaticIdentity) {
    (
        StaticIdentity::new(AgentId::new("agent-a"), vec![1_u8; 32]).expect("identity A"),
        StaticIdentity::new(AgentId::new("agent-b"), vec![2_u8; 32]).expect("identity B"),
    )
}

#[tokio::test]
async fn authenticated_loopback_exchange_and_replay_rejection() {
    let (alice, bob) = identities();
    let manifest = ContentDigest::compute(b"manifest");
    let frame = alice
        .issue_frame(
            bob.agent_id.clone(),
            manifest.clone(),
            (Utc::now() + Duration::minutes(5)).to_rfc3339(),
            b"typed task envelope".to_vec(),
        )
        .expect("frame");
    let (mut writer, mut reader) = duplex(16 * 1024);
    send_authenticated_frame(&mut writer, &frame)
        .await
        .expect("send");
    let replay = ReplayCache::new();
    let received = receive_authenticated_frame(&mut reader, &bob, &alice, &manifest, &replay)
        .await
        .expect("receive");
    assert_eq!(received.payload, b"typed task envelope");
    assert_eq!(replay.len(), 1);

    send_authenticated_frame(&mut writer, &frame)
        .await
        .expect("duplicate send");
    let duplicate =
        receive_authenticated_frame(&mut reader, &bob, &alice, &manifest, &replay).await;
    assert!(matches!(
        duplicate,
        Err(agent_collaboration_transport::transport::TransportError::Replay)
    ));
}

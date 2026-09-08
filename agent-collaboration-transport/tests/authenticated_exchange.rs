use agent_collaboration_transport::{
    accept_one, connect_and_send, receive_authenticated_frame, send_authenticated_frame,
    ReplayCache, StaticIdentity,
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let server_replay = ReplayCache::new();
    let server_bob = bob.clone();
    let server_alice = alice.clone();
    let server_manifest = manifest.clone();
    let server = tokio::spawn(async move {
        let first = accept_one(
            &listener,
            &server_bob,
            &server_alice,
            &server_manifest,
            &server_replay,
        )
        .await;
        let second = accept_one(
            &listener,
            &server_bob,
            &server_alice,
            &server_manifest,
            &server_replay,
        )
        .await;
        (first, second)
    });
    connect_and_send(&address.to_string(), &frame)
        .await
        .expect("TCP send");
    connect_and_send(&address.to_string(), &frame)
        .await
        .expect("duplicate TCP send");
    let (tcp_received, duplicate) = server.await.expect("server task");
    let tcp_received = tcp_received.expect("TCP receive");
    assert_eq!(tcp_received.payload, b"typed task envelope");
    assert!(matches!(
        duplicate,
        Err(agent_collaboration_transport::transport::TransportError::Replay)
    ));
}

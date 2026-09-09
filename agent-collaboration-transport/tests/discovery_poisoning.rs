use agent_collaboration_transport::{
    DiscoveryCacheV1, DiscoveryObservationV1, DiscoveryTrustV1, VerifiedEndpointBookV1,
    DISCOVERY_SERVICE_TYPE, PROTOCOL_VERSION,
};
use chrono::{Duration, Utc};
use stack_ids::{AgentId, ContentDigest};

fn observation(endpoint: &str, nonce: &str, expires_at: String) -> DiscoveryObservationV1 {
    DiscoveryObservationV1 {
        service_type: DISCOVERY_SERVICE_TYPE.to_owned(),
        endpoint: endpoint.to_owned(),
        protocol_version: PROTOCOL_VERSION.to_owned(),
        agent_id_hint: AgentId::new("agent-worker"),
        capability_manifest_digest: ContentDigest::compute_str("worker-capabilities"),
        instance_nonce: nonce.to_owned(),
        expires_at,
    }
}

#[test]
fn discovery_is_untrusted_and_cannot_select_execution_endpoint() {
    let now = Utc::now();
    let mut cache = DiscoveryCacheV1::default();
    cache
        .observe(
            observation(
                "127.0.0.1:43101",
                "nonce-a",
                (now + Duration::minutes(5)).to_rfc3339(),
            ),
            now,
        )
        .expect("valid discovery observation");

    let agent = AgentId::new("agent-worker");
    let records = cache
        .observations_for_agent(&agent, now)
        .expect("active observations");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].trust, DiscoveryTrustV1::UntrustedObservation);
    assert_eq!(cache.execution_candidate(&agent, now), None);

    let mut verified = VerifiedEndpointBookV1::default();
    assert_eq!(verified.resolve_for_execution(&agent), None);
    verified
        .bind_endpoint(agent.clone(), "127.0.0.1:43101".to_owned())
        .expect("explicit caller binding");
    assert_eq!(
        verified.resolve_for_execution(&agent),
        Some("127.0.0.1:43101")
    );
}

#[test]
fn conflicting_observations_remain_visible_and_do_not_choose_first_response() {
    let now = Utc::now();
    let mut cache = DiscoveryCacheV1::default();
    for (endpoint, nonce) in [("10.0.0.2:43101", "nonce-a"), ("10.0.0.3:43101", "nonce-b")] {
        cache
            .observe(
                observation(endpoint, nonce, (now + Duration::minutes(5)).to_rfc3339()),
                now,
            )
            .expect("valid discovery observation");
    }

    let records = cache
        .observations_for_agent(&AgentId::new("agent-worker"), now)
        .expect("active observations");
    assert_eq!(records.len(), 2);
    assert_eq!(
        cache.execution_candidate(&AgentId::new("agent-worker"), now),
        None
    );
}

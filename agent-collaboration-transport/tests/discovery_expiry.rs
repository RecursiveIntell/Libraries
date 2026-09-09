use agent_collaboration_transport::{
    DiscoveryCacheV1, DiscoveryObservationV1, DISCOVERY_SERVICE_TYPE, PROTOCOL_VERSION,
};
use chrono::{Duration, Utc};
use stack_ids::{AgentId, ContentDigest};

fn observation(expires_at: String) -> DiscoveryObservationV1 {
    DiscoveryObservationV1 {
        service_type: DISCOVERY_SERVICE_TYPE.to_owned(),
        endpoint: "127.0.0.1:43101".to_owned(),
        protocol_version: PROTOCOL_VERSION.to_owned(),
        agent_id_hint: AgentId::new("agent-worker"),
        capability_manifest_digest: ContentDigest::compute_str("worker-capabilities"),
        instance_nonce: "nonce-expiring".to_owned(),
        expires_at,
    }
}

#[test]
fn expired_discovery_is_removed_without_mutating_transport_truth() {
    let now = Utc::now();
    let mut cache = DiscoveryCacheV1::default();
    cache
        .observe(observation((now + Duration::seconds(1)).to_rfc3339()), now)
        .expect("valid discovery observation");
    assert_eq!(cache.active(now).expect("active cache").len(), 1);

    let later = now + Duration::seconds(2);
    assert!(cache.active(later).expect("expired cache").is_empty());
    assert_eq!(cache.expire(later).expect("expire cache"), 1);
    assert!(cache.active(later).expect("empty cache").is_empty());
}

#[test]
fn invalid_discovery_is_rejected_before_cache_admission() {
    let now = Utc::now();
    let mut cache = DiscoveryCacheV1::default();
    let error = cache
        .observe(observation((now - Duration::seconds(1)).to_rfc3339()), now)
        .expect_err("expired observations must fail closed");
    assert!(error.to_string().contains("invalid discovery expiry"));
}

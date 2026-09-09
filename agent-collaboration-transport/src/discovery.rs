use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, ContentDigest};
use std::collections::BTreeMap;
use thiserror::Error;

pub const DISCOVERY_SERVICE_TYPE: &str = "_agent-collaboration._tcp";
const MAX_ENDPOINT_BYTES: usize = 512;
const MAX_NONCE_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryObservationV1 {
    pub service_type: String,
    pub endpoint: String,
    pub protocol_version: String,
    pub agent_id_hint: AgentId,
    pub capability_manifest_digest: ContentDigest,
    pub instance_nonce: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryTrustV1 {
    UntrustedObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryRecordV1 {
    pub observation: DiscoveryObservationV1,
    pub trust: DiscoveryTrustV1,
    pub observed_at: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    #[error("discovery field is empty: {0}")]
    EmptyField(&'static str),
    #[error("discovery field exceeds its limit: {field} ({value} > {limit})")]
    ResourceLimit {
        field: &'static str,
        value: usize,
        limit: usize,
    },
    #[error("unsupported discovery service type: {0}")]
    ServiceType(String),
    #[error("unsupported discovery protocol version: {0}")]
    ProtocolVersion(String),
    #[error("invalid discovery endpoint: {0}")]
    Endpoint(String),
    #[error("invalid discovery expiry: {0}")]
    InvalidExpiry(String),
}

impl DiscoveryObservationV1 {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), DiscoveryError> {
        if self.service_type != DISCOVERY_SERVICE_TYPE {
            return Err(DiscoveryError::ServiceType(self.service_type.clone()));
        }
        if self.protocol_version != crate::PROTOCOL_VERSION {
            return Err(DiscoveryError::ProtocolVersion(
                self.protocol_version.clone(),
            ));
        }
        validate_nonempty(&self.endpoint, "endpoint")?;
        validate_nonempty(&self.instance_nonce, "instance_nonce")?;
        validate_limit("endpoint", self.endpoint.len(), MAX_ENDPOINT_BYTES)?;
        validate_limit("instance_nonce", self.instance_nonce.len(), MAX_NONCE_BYTES)?;
        if self
            .endpoint
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
            || self.endpoint.contains("@")
            || self.endpoint.contains("://")
        {
            return Err(DiscoveryError::Endpoint(self.endpoint.clone()));
        }
        let expiry = parse_timestamp(&self.expires_at)?;
        if expiry <= now {
            return Err(DiscoveryError::InvalidExpiry(self.expires_at.clone()));
        }
        Ok(())
    }

    pub fn is_expired_at(&self, now: DateTime<Utc>) -> Result<bool, DiscoveryError> {
        Ok(parse_timestamp(&self.expires_at)? <= now)
    }
}

fn validate_nonempty(value: &str, field: &'static str) -> Result<(), DiscoveryError> {
    if value.trim().is_empty() {
        Err(DiscoveryError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn validate_limit(field: &'static str, value: usize, limit: usize) -> Result<(), DiscoveryError> {
    if value > limit {
        Err(DiscoveryError::ResourceLimit {
            field,
            value,
            limit,
        })
    } else {
        Ok(())
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, DiscoveryError> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| DiscoveryError::InvalidExpiry(value.to_owned()))
}

#[derive(Debug, Default, Clone)]
pub struct DiscoveryCacheV1 {
    observations: BTreeMap<String, DiscoveryRecordV1>,
}

impl DiscoveryCacheV1 {
    pub fn observe(
        &mut self,
        observation: DiscoveryObservationV1,
        now: DateTime<Utc>,
    ) -> Result<(), DiscoveryError> {
        observation.validate(now)?;
        let key = format!("{}|{}", observation.instance_nonce, observation.endpoint);
        self.observations.insert(
            key,
            DiscoveryRecordV1 {
                observation,
                trust: DiscoveryTrustV1::UntrustedObservation,
                observed_at: now.to_rfc3339(),
            },
        );
        Ok(())
    }

    pub fn active(&self, now: DateTime<Utc>) -> Result<Vec<DiscoveryRecordV1>, DiscoveryError> {
        Ok(self
            .observations
            .values()
            .filter(|record| {
                record
                    .observation
                    .is_expired_at(now)
                    .is_ok_and(|expired| !expired)
            })
            .cloned()
            .collect())
    }

    pub fn expire(&mut self, now: DateTime<Utc>) -> Result<usize, DiscoveryError> {
        let before = self.observations.len();
        let mut invalid = None;
        self.observations
            .retain(|_, record| match record.observation.is_expired_at(now) {
                Ok(expired) => !expired,
                Err(error) => {
                    invalid = Some(error);
                    false
                }
            });
        if let Some(error) = invalid {
            return Err(error);
        }
        Ok(before - self.observations.len())
    }

    pub fn observations_for_agent(
        &self,
        agent_id: &AgentId,
        now: DateTime<Utc>,
    ) -> Result<Vec<DiscoveryRecordV1>, DiscoveryError> {
        Ok(self
            .active(now)?
            .into_iter()
            .filter(|record| &record.observation.agent_id_hint == agent_id)
            .collect())
    }

    /// Discovery never selects an execution endpoint. A caller must complete
    /// the authenticated transport handshake and record that fact separately.
    pub fn execution_candidate(&self, _agent_id: &AgentId, _now: DateTime<Utc>) -> Option<String> {
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct VerifiedEndpointBookV1 {
    endpoints: BTreeMap<AgentId, String>,
}

impl VerifiedEndpointBookV1 {
    /// Record an endpoint after an external transport/authentication layer has
    /// made its own decision. This module does not verify that evidence.
    pub fn bind_endpoint(
        &mut self,
        agent_id: AgentId,
        endpoint: String,
    ) -> Result<(), DiscoveryError> {
        validate_nonempty(&endpoint, "endpoint")?;
        validate_limit("endpoint", endpoint.len(), MAX_ENDPOINT_BYTES)?;
        if endpoint
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
            || endpoint.contains("@")
            || endpoint.contains("://")
        {
            return Err(DiscoveryError::Endpoint(endpoint));
        }
        self.endpoints.insert(agent_id, endpoint);
        Ok(())
    }

    pub fn resolve_for_execution(&self, agent_id: &AgentId) -> Option<&str> {
        self.endpoints.get(agent_id).map(String::as_str)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvIdentityV1 {
    pub model_revision: String,
    pub layer: u32,
    pub head: u32,
    pub token_span_digest: String,
    pub position_encoding: String,
    pub kv_format: String,
    pub precision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvReuseDecisionV1 {
    pub exact_kv_reuse: bool,
    pub semantic_artifact_reuse: bool,
    pub mismatched_fields: Vec<String>,
}

pub fn qualify_kv_reuse(
    stored: &KvIdentityV1,
    requested: &KvIdentityV1,
    semantic_artifact_oracle: bool,
) -> KvReuseDecisionV1 {
    let mut mismatched = Vec::new();
    macro_rules! compare {
        ($field:ident) => {
            if stored.$field != requested.$field {
                mismatched.push(stringify!($field).to_string());
            }
        };
    }
    compare!(model_revision);
    compare!(layer);
    compare!(head);
    compare!(token_span_digest);
    compare!(position_encoding);
    compare!(kv_format);
    compare!(precision);
    KvReuseDecisionV1 {
        exact_kv_reuse: mismatched.is_empty(),
        semantic_artifact_reuse: semantic_artifact_oracle,
        mismatched_fields: mismatched,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccelerationMetricsV1 {
    pub storage_bytes: u64,
    pub active_ram_bytes: u64,
    pub active_vram_bytes: u64,
    pub latency_ms: f64,
    pub measured_active_runtime: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FidelityClassV1 {
    Exact,
    LossyBounded,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationQualificationV1 {
    pub codec: String,
    pub fidelity: FidelityClassV1,
    pub exact_evidence_ref: Option<String>,
    pub metrics: AccelerationMetricsV1,
    pub semantic_guarantee_upgraded: bool,
    pub serving_gain_supported: bool,
}

pub fn qualify_representation(
    codec: impl Into<String>,
    fidelity: FidelityClassV1,
    exact_evidence_ref: Option<String>,
    metrics: AccelerationMetricsV1,
    measured_gain: bool,
) -> RepresentationQualificationV1 {
    let serving_gain_supported = metrics.measured_active_runtime && measured_gain;
    RepresentationQualificationV1 {
        codec: codec.into(),
        fidelity,
        exact_evidence_ref,
        metrics,
        semantic_guarantee_upgraded: false,
        serving_gain_supported,
    }
}

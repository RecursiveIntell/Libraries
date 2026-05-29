use quant_codec_core::QuantCodecError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum PolyKvError {
    #[error("invalid shape: {reason}")]
    InvalidShape { reason: String },

    #[error("invalid span: start={start}, end={end}")]
    InvalidSpan { start: u64, end: u64 },

    #[error("shape mismatch: {reason}")]
    ShapeMismatch { reason: String },

    #[error("missing exact fallback")]
    MissingFallback,

    #[error("missing block: {reason}")]
    MissingBlock { reason: String },

    #[error("codec error: {0}")]
    Codec(String),

    #[error("quality gate failed: {0}")]
    QualityGateFailed(String),

    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("unsupported adapter: {adapter} ({reason})")]
    UnsupportedAdapter {
        adapter: &'static str,
        reason: String,
    },
}

impl From<QuantCodecError> for PolyKvError {
    fn from(value: QuantCodecError) -> Self {
        match value {
            QuantCodecError::InvalidShape { reason } => Self::InvalidShape { reason },
            QuantCodecError::InvalidTokenSpan { start, end } => Self::InvalidSpan { start, end },
            QuantCodecError::ShapeMismatch { reason } => Self::ShapeMismatch { reason },
            other => Self::Codec(other.to_string()),
        }
    }
}

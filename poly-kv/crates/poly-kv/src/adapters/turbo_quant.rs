use crate::PolyKvError;

#[derive(Debug, Clone, Copy, Default)]
pub struct TurboQuantValueCodec;

impl TurboQuantValueCodec {
    pub fn new_unsupported() -> Result<Self, PolyKvError> {
        Err(PolyKvError::UnsupportedAdapter {
            adapter: "turbo-quant",
            reason: "external API not inspected in this alpha pass".to_string(),
        })
    }
}

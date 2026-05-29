use crate::PolyKvError;

#[derive(Debug, Clone, Copy, Default)]
pub struct FibQuantValueCodec;

impl FibQuantValueCodec {
    pub fn new_unsupported() -> Result<Self, PolyKvError> {
        Err(PolyKvError::UnsupportedAdapter {
            adapter: "fibquant",
            reason: "external API not inspected in this alpha pass".to_string(),
        })
    }
}

use crate::bootstrap::types::{
    BootstrapCapabilityStatus, BootstrapSymbolCapability, BootstrapSymbolPrecision,
    SYMBOL_CAPABILITY_VERSION,
};

pub(crate) fn symbol_capability(
    language: &str,
    degradation_reason: Option<String>,
) -> BootstrapSymbolCapability {
    let extractor = match language {
        "rust" => "rust_line_scanner_v1",
        "typescript" | "javascript" => "typescript_line_scanner_v1",
        "python" => "python_line_scanner_v1",
        _ => "generic_heading_v1",
    };

    BootstrapSymbolCapability {
        status: if degradation_reason.is_some() {
            BootstrapCapabilityStatus::Degraded
        } else {
            BootstrapCapabilityStatus::Supported
        },
        precision: BootstrapSymbolPrecision::Heuristic,
        extractor: extractor.to_string(),
        policy_version: SYMBOL_CAPABILITY_VERSION.to_string(),
        degradation_reason,
    }
}

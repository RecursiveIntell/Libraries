pub(crate) mod generic;
pub(crate) mod python;
pub(crate) mod rust;
pub(crate) mod typescript;

use crate::bootstrap::types::{ChunkRecord, SourceFileRecord, SymbolRecord, MAX_SYMBOL_SCAN_BYTES};

#[derive(Debug, Clone)]
pub(crate) struct SymbolExtractionResult {
    pub symbols: Vec<SymbolRecord>,
    pub degraded_reason: Option<String>,
}

pub(crate) fn extract_symbols(
    file: &SourceFileRecord,
    chunks: &[ChunkRecord],
) -> SymbolExtractionResult {
    if file.byte_count > MAX_SYMBOL_SCAN_BYTES {
        return SymbolExtractionResult {
            symbols: Vec::new(),
            degraded_reason: Some(format!(
                "symbol extraction skipped above {} byte threshold",
                MAX_SYMBOL_SCAN_BYTES
            )),
        };
    }

    let extraction = match file.language.as_str() {
        "rust" => rust::extract(file),
        "typescript" | "javascript" => SymbolExtractionResult {
            symbols: typescript::extract(file),
            degraded_reason: None,
        },
        "python" => SymbolExtractionResult {
            symbols: python::extract(file),
            degraded_reason: None,
        },
        _ => SymbolExtractionResult {
            symbols: generic::extract(file),
            degraded_reason: None,
        },
    };
    let mut symbols = extraction.symbols;

    for symbol in &mut symbols {
        symbol.parent_chunk_id = chunks
            .iter()
            .find(|chunk| {
                symbol.line_start >= chunk.start_line && symbol.line_start <= chunk.end_line
            })
            .map(|chunk| chunk.chunk_id.clone());
    }

    SymbolExtractionResult {
        symbols,
        degraded_reason: extraction.degraded_reason,
    }
}

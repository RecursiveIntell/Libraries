use crate::bootstrap::types::{
    ChunkRecord, SourceFileRecord, CHUNK_POLICY_VERSION, MAX_CHUNK_BYTES, MAX_CHUNK_LINES,
};
use stack_ids::ContentDigest;

pub(crate) fn chunk_source_file(file: &SourceFileRecord) -> Vec<ChunkRecord> {
    let lines = split_lines(&file.content);
    if lines.is_empty() {
        return vec![build_chunk(file, 0, 1, 1, "")];
    }

    let mut chunks = Vec::new();
    let mut start_index = 0usize;
    let mut current_bytes = 0usize;

    for (index, line) in lines.iter().enumerate() {
        let projected = current_bytes + line.len();
        let line_count = index.saturating_sub(start_index) + 1;
        if index > start_index && (projected > MAX_CHUNK_BYTES || line_count > MAX_CHUNK_LINES) {
            let content = lines[start_index..index].concat();
            chunks.push(build_chunk(
                file,
                chunks.len(),
                start_index + 1,
                index,
                &content,
            ));
            start_index = index;
            current_bytes = 0;
        }
        current_bytes += line.len();
    }

    let content = lines[start_index..].concat();
    chunks.push(build_chunk(
        file,
        chunks.len(),
        start_index + 1,
        lines.len(),
        &content,
    ));
    chunks
}

fn split_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            lines.push(content[start..=index].to_string());
            start = index + 1;
        }
    }
    if start < content.len() {
        lines.push(content[start..].to_string());
    }
    lines
}

fn build_chunk(
    file: &SourceFileRecord,
    chunk_index: usize,
    start_line: usize,
    end_line: usize,
    content: &str,
) -> ChunkRecord {
    let content_digest = ContentDigest::compute(content.as_bytes());
    let anchor = stable_anchor(content);
    let chunk_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}:{}",
        file.relative_path,
        CHUNK_POLICY_VERSION,
        anchor,
        content_digest.hex()
    ));

    ChunkRecord {
        chunk_id: format!("workspace-source-chunk-{chunk_seed}"),
        chunk_index,
        content: content.to_string(),
        content_digest,
        start_line,
        end_line,
        byte_count: content.len(),
        stable_anchor: anchor,
    }
}

fn stable_anchor(content: &str) -> String {
    let mut first = None;
    let mut last = None;
    for line in content.lines() {
        let normalized = line.trim();
        if normalized.is_empty() {
            continue;
        }
        if first.is_none() {
            first = Some(normalized);
        }
        last = Some(normalized);
    }
    let seed = format!(
        "{}:{}:{}",
        first.unwrap_or("empty"),
        last.unwrap_or("empty"),
        content.lines().count()
    );
    crate::bootstrap::manifest::digest_text(&seed)
}

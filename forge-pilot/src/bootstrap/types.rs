use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::ContentDigest;

pub(crate) const BOOTSTRAP_SOURCE_AUTHORITY: &str = "workspace_source";
pub(crate) const BOOTSTRAP_SOURCE_V1_METADATA_KEY: &str = "workspace_source_v1";
pub(crate) const BOOTSTRAP_SOURCE_V2_METADATA_KEY: &str = "workspace_source_v2";
pub(crate) const MAX_SOURCE_FILE_BYTES: usize = 256 * 1024;
pub(crate) const MAX_CHUNK_BYTES: usize = 2_400;
pub(crate) const MAX_CHUNK_LINES: usize = 80;
pub(crate) const MAX_SYMBOL_SCAN_BYTES: usize = 96 * 1024;
pub(crate) const CHUNK_POLICY_VERSION: &str = "workspace_source_chunk_policy_v2";
pub(crate) const SYMBOL_CAPABILITY_VERSION: &str = "workspace_source_symbol_capability_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapSourceRichness {
    #[default]
    Thin,
    Chunked,
    Symbolized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapCapabilityStatus {
    Supported,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapSymbolPrecision {
    Heuristic,
    Structural,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BootstrapSymbolCapability {
    pub status: BootstrapCapabilityStatus,
    pub precision: BootstrapSymbolPrecision,
    pub extractor: String,
    pub policy_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BootstrapChunkPolicyInfo {
    pub policy_version: String,
    pub max_chunk_bytes: usize,
    pub max_chunk_lines: usize,
    pub stable_anchor_strategy: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapSourceDeltaSummary {
    pub new_file_count: usize,
    pub changed_file_count: usize,
    pub unchanged_file_count: usize,
    pub deleted_file_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapDerivedDeltaSummary {
    pub file_count: usize,
    pub chunk_count: usize,
    pub symbol_count: usize,
    pub source_unchanged_file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapSourceSkippedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapSourceObservation {
    pub manifest_id: String,
    pub file_count: usize,
    pub chunk_count: usize,
    pub symbol_count: usize,
    pub degraded_symbol_file_count: usize,
    pub richness: BootstrapSourceRichness,
    pub source_delta: BootstrapSourceDeltaSummary,
    pub derived_delta: BootstrapDerivedDeltaSummary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapRichness {
    #[default]
    Thin,
    Chunked,
    Symbolized,
}

impl From<BootstrapSourceRichness> for BootstrapRichness {
    fn from(value: BootstrapSourceRichness) -> Self {
        match value {
            BootstrapSourceRichness::Thin => Self::Thin,
            BootstrapSourceRichness::Chunked => Self::Chunked,
            BootstrapSourceRichness::Symbolized => Self::Symbolized,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapCurrentState {
    pub manifest_id: Option<String>,
    pub source_envelope_id: Option<String>,
    pub imported_at: Option<String>,
    pub file_count: usize,
    pub chunk_count: usize,
    pub symbol_count: usize,
    pub degraded_symbol_file_count: usize,
    pub richness: BootstrapRichness,
    #[serde(default)]
    pub skipped_files: Vec<BootstrapSourceSkippedFile>,
    #[serde(default)]
    pub large_file_skip_count: usize,
    #[serde(default)]
    pub import_disposition: String,
}

impl BootstrapCurrentState {
    /// Builds the current-state summary from a bootstrap manifest snapshot.
    pub fn from_manifest(manifest: &BootstrapManifestSnapshot) -> Self {
        Self {
            manifest_id: Some(manifest.manifest_id.clone()),
            source_envelope_id: None,
            imported_at: None,
            file_count: manifest.file_count,
            chunk_count: manifest.chunk_count,
            symbol_count: manifest.symbol_count,
            degraded_symbol_file_count: manifest.degraded_symbol_file_count,
            richness: manifest.richness.into(),
            skipped_files: manifest.skipped_files.clone(),
            large_file_skip_count: manifest
                .skipped_files
                .iter()
                .filter(|file| file.reason.contains("bootstrap limit"))
                .count(),
            import_disposition: "manifest_only".into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapDeltaSummary {
    pub new_file_count: usize,
    pub changed_file_count: usize,
    pub unchanged_file_count: usize,
    pub deleted_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapSourceReport {
    pub namespace: String,
    pub workspace_path: String,
    pub scanned_file_count: usize,
    pub imported_file_count: usize,
    pub skipped_file_count: usize,
    pub source_envelope_id: Option<String>,
    pub content_digest: Option<String>,
    pub import_status: Option<String>,
    pub was_duplicate: bool,
    pub skipped_files: Vec<BootstrapSourceSkippedFile>,
    pub manifest_id: Option<String>,
    pub current_manifest_file_count: usize,
    pub current_manifest_chunk_count: usize,
    pub current_manifest_symbol_count: usize,
    pub latest_manifest_only: bool,
    pub source_delta: BootstrapSourceDeltaSummary,
    pub derived_delta: BootstrapDerivedDeltaSummary,
    pub imported_chunk_count: usize,
    pub imported_symbol_count: usize,
    pub degraded_symbol_file_count: usize,
    pub richness: BootstrapSourceRichness,
    #[serde(default)]
    pub skipped_large_file_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceFileRecord {
    pub relative_path: String,
    pub content: String,
    pub content_digest: ContentDigest,
    pub line_count: usize,
    pub byte_count: usize,
    pub file_extension: Option<String>,
    pub language: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ChunkRecord {
    pub chunk_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub content_digest: ContentDigest,
    pub start_line: usize,
    pub end_line: usize,
    pub byte_count: usize,
    pub stable_anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SymbolRecord {
    pub symbol_id: String,
    pub name: String,
    pub kind: String,
    pub language: String,
    pub line_start: usize,
    pub line_end: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_chunk_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedSourceFile {
    pub file: SourceFileRecord,
    pub chunks: Vec<ChunkRecord>,
    pub symbols: Vec<SymbolRecord>,
    pub symbol_capability: BootstrapSymbolCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BootstrapManifestFile {
    pub path: String,
    pub content_digest: String,
    pub byte_count: usize,
    pub line_count: usize,
    pub language: String,
    pub chunk_ids: Vec<String>,
    pub chunk_count: usize,
    pub symbol_ids: Vec<String>,
    pub symbol_count: usize,
    pub symbol_extraction_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_extraction_degradation: Option<String>,
    pub symbol_capability: BootstrapSymbolCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BootstrapManifestSnapshot {
    pub manifest_id: String,
    pub namespace: String,
    pub file_count: usize,
    pub chunk_count: usize,
    pub symbol_count: usize,
    pub degraded_symbol_file_count: usize,
    pub richness: BootstrapSourceRichness,
    pub chunk_policy: BootstrapChunkPolicyInfo,
    #[serde(default)]
    pub skipped_files: Vec<BootstrapSourceSkippedFile>,
    pub files: Vec<BootstrapManifestFile>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BootstrapManifestDelta {
    pub new_files: Vec<BootstrapManifestFile>,
    pub changed_files: Vec<BootstrapManifestFile>,
    pub unchanged_files: Vec<BootstrapManifestFile>,
    pub deleted_files: Vec<BootstrapManifestFile>,
    #[serde(default)]
    pub source_unchanged_derived_changed_files: Vec<BootstrapManifestFile>,
}

impl BootstrapManifestDelta {
    /// Returns the total number of imported files tracked by the delta.
    pub fn imported_file_count(&self) -> usize {
        self.new_files.len() + self.changed_files.len()
    }

    /// Returns true when the manifest delta contains no source or derived changes.
    pub fn is_noop(&self) -> bool {
        self.new_files.is_empty() && self.changed_files.is_empty() && self.deleted_files.is_empty()
    }
}

impl From<&BootstrapManifestDelta> for BootstrapDeltaSummary {
    fn from(value: &BootstrapManifestDelta) -> Self {
        Self {
            new_file_count: value.new_files.len(),
            changed_file_count: value.changed_files.len(),
            unchanged_file_count: value.unchanged_files.len(),
            deleted_file_count: value.deleted_files.len(),
        }
    }
}

impl BootstrapManifestDelta {
    /// Summarizes source-file additions, removals, and changes in this delta.
    pub fn source_delta_summary(&self) -> BootstrapSourceDeltaSummary {
        BootstrapSourceDeltaSummary {
            new_file_count: self.new_files.len(),
            changed_file_count: self.changed_files.len(),
            unchanged_file_count: self.unchanged_files.len()
                + self.source_unchanged_derived_changed_files.len(),
            deleted_file_count: self.deleted_files.len(),
        }
    }

    /// Summarizes derived-artifact additions, removals, and changes in this delta.
    pub fn derived_delta_summary(&self) -> BootstrapDerivedDeltaSummary {
        let changed = self
            .changed_files
            .iter()
            .chain(self.source_unchanged_derived_changed_files.iter());
        BootstrapDerivedDeltaSummary {
            file_count: self.source_unchanged_derived_changed_files.len(),
            chunk_count: changed.clone().map(|file| file.chunk_count).sum(),
            symbol_count: changed.map(|file| file.symbol_count).sum(),
            source_unchanged_file_count: self.source_unchanged_derived_changed_files.len(),
        }
    }
}

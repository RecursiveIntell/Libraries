//! Thin persistence wiring for canonical receipt crates.
//!
//! AiDENs does not define receipt semantics here. This crate only gives the
//! product layer a small JSONL sink for receipts whose schemas are owned by
//! canonical libraries, plus orchestration reports that are explicitly labeled
//! as AiDENs-owned reports rather than stack receipts.

use async_trait::async_trait;
use chrono::Utc;
pub use llm_tool_runtime::{
    ToolError, ToolErrorClass, ToolReceipt as CanonicalRuntimeToolReceipt, ToolReceiptSink,
};
pub use semantic_memory_forge::ForgeToolReceiptV2 as CanonicalForgeToolReceiptV2;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stack_ids::ContentDigest;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
pub use verification_control::ControlReceipt as CanonicalControlReceipt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBundleStoreConfig {
    pub root_path: PathBuf,
    pub bundles_path: PathBuf,
    pub index_path: PathBuf,
}

impl RunBundleStoreConfig {
    pub fn for_receipt_root(root_path: impl Into<PathBuf>) -> Self {
        let root_path = root_path.into();
        let bundles_path = root_path.join("run-bundles");
        Self {
            index_path: bundles_path.join("index.ndjson"),
            bundles_path,
            root_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBundleStoreRecord {
    pub artifact_kind: String,
    pub ownership: String,
    pub support_tier: String,
    pub semantic_status: String,
    pub run_id: String,
    pub bundle_schema: String,
    pub recorded_at: String,
    pub bundle_path: PathBuf,
    pub content_digest: ContentDigest,
    pub canonical_event_log_path: PathBuf,
    pub known_limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunBundleStoreInspection {
    pub record: RunBundleStoreRecord,
    pub bundle: Value,
    pub digest_verified: bool,
}

#[derive(Debug, Error)]
pub enum RunBundleStoreError {
    #[error("run bundle store io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("run bundle store json error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("run bundle digest error: {source}")]
    Digest {
        #[source]
        source: stack_ids::DigestError,
    },
    #[error("run bundle store requires AiDENsRunBundleV3, got {0}")]
    UnsupportedSchema(String),
    #[error("run bundle missing string run_id")]
    MissingRunId,
    #[error("run bundle not found: {0}")]
    NotFound(String),
    #[error("run bundle store has multiple bundles; pass a specific run output directory or bundle path")]
    AmbiguousBundle,
    #[error("run bundle already exists at {0}")]
    AlreadyExists(String),
    #[error("run bundle index chain verification failed at sequence {0}")]
    ChainVerificationFailed(u64),
}

#[derive(Debug, Clone)]
pub struct RunBundleStore {
    config: RunBundleStoreConfig,
}

impl RunBundleStore {
    pub fn open(config: RunBundleStoreConfig) -> Result<Self, RunBundleStoreError> {
        std::fs::create_dir_all(&config.bundles_path).map_err(|source| {
            RunBundleStoreError::Io {
                path: config.bundles_path.clone(),
                source,
            }
        })?;
        ensure_run_bundle_file(&config.index_path)?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &RunBundleStoreConfig {
        &self.config
    }

    pub fn bundle_path_for_run_id(&self, run_id: &str) -> PathBuf {
        self.config
            .bundles_path
            .join(receipt_store_segment(run_id))
            .join("run-bundle.json")
    }

    pub fn bundle_path_for_run_id_and_digest(
        &self,
        run_id: &str,
        content_digest: &ContentDigest,
    ) -> PathBuf {
        self.config
            .bundles_path
            .join(receipt_store_segment(run_id))
            .join(content_digest.hex())
            .join("run-bundle.json")
    }

    pub fn write_bundle_value(
        &self,
        bundle: &Value,
    ) -> Result<RunBundleStoreRecord, RunBundleStoreError> {
        let schema = bundle
            .get("schema")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if schema != "AiDENsRunBundleV3" {
            return Err(RunBundleStoreError::UnsupportedSchema(schema.into()));
        }
        let run_id = bundle
            .get("run_id")
            .and_then(Value::as_str)
            .ok_or(RunBundleStoreError::MissingRunId)?;
        let content_digest = ContentDigest::compute_json(bundle)
            .map_err(|source| RunBundleStoreError::Digest { source })?;
        let bundle_path = self.bundle_path_for_run_id_and_digest(run_id, &content_digest);
        if bundle_path.exists() {
            return Err(RunBundleStoreError::AlreadyExists(
                bundle_path.display().to_string(),
            ));
        }
        if let Some(parent) = bundle_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| RunBundleStoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let body =
            serde_json::to_string_pretty(bundle).map_err(|source| RunBundleStoreError::Json {
                path: bundle_path.clone(),
                source,
            })?;
        write_atomic_file(&bundle_path, &(body + "\n")).map_err(|source| {
            RunBundleStoreError::Io {
                path: bundle_path.clone(),
                source,
            }
        })?;
        let record = RunBundleStoreRecord {
            artifact_kind: "local_operator_run_bundle_store_record".into(),
            ownership:
                "AiDENs-local operator evidence; canonical receipt and trace semantics remain in owner crates.".into(),
            support_tier: bundle
                .pointer("/support/support_tier")
                .and_then(Value::as_str)
                .unwrap_or("partial")
                .into(),
            semantic_status: if bundle
                .pointer("/failure/degraded")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "degraded_exact_check".into()
            } else {
                "exact_check".into()
            },
            run_id: run_id.into(),
            bundle_schema: schema.into(),
            recorded_at: Utc::now().to_rfc3339(),
            bundle_path: bundle_path.clone(),
            content_digest,
            canonical_event_log_path: self.config.root_path.join("canonical-receipts.ndjson"),
            known_limits: vec![
                "Stores AiDENsRunBundleV3 operator evidence only; it is not a canonical memory or verification truth store.".into(),
            ],
        };
        if let Err(error) = self.append_record(&record) {
            let _ = std::fs::remove_file(&bundle_path);
            return Err(error);
        }
        Ok(record)
    }

    pub fn write_bundle<T: Serialize>(
        &self,
        bundle: &T,
    ) -> Result<RunBundleStoreRecord, RunBundleStoreError> {
        let value = serde_json::to_value(bundle).map_err(|source| RunBundleStoreError::Json {
            path: self.config.index_path.clone(),
            source,
        })?;
        self.write_bundle_value(&value)
    }

    pub fn list_records(&self) -> Result<Vec<RunBundleStoreRecord>, RunBundleStoreError> {
        read_run_bundle_records(&self.config.index_path)
    }

    pub fn inspect(&self, run_id: &str) -> Result<RunBundleStoreInspection, RunBundleStoreError> {
        let record = self
            .list_records()?
            .into_iter()
            .rev()
            .find(|record| record.run_id == run_id)
            .ok_or_else(|| RunBundleStoreError::NotFound(run_id.into()))?;
        let bundle_text = std::fs::read_to_string(&record.bundle_path).map_err(|source| {
            RunBundleStoreError::Io {
                path: record.bundle_path.clone(),
                source,
            }
        })?;
        let bundle: Value =
            serde_json::from_str(&bundle_text).map_err(|source| RunBundleStoreError::Json {
                path: record.bundle_path.clone(),
                source,
            })?;
        let digest_verified = ContentDigest::compute_json(&bundle)
            .map(|digest| digest == record.content_digest)
            .unwrap_or(false);
        Ok(RunBundleStoreInspection {
            record,
            bundle,
            digest_verified,
        })
    }

    pub fn single_bundle_path(&self) -> Result<PathBuf, RunBundleStoreError> {
        let records = self.list_records()?;
        let mut paths = records
            .into_iter()
            .map(|record| record.bundle_path)
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        match paths.len() {
            0 => Err(RunBundleStoreError::NotFound(
                "run-bundles/index.ndjson".into(),
            )),
            1 => Ok(paths.remove(0)),
            _ => Err(RunBundleStoreError::AmbiguousBundle),
        }
    }

    fn append_record(&self, record: &RunBundleStoreRecord) -> Result<(), RunBundleStoreError> {
        let _lock = acquire_exclusive_lock(&self.config.index_path).map_err(|source| {
            RunBundleStoreError::Io {
                path: lock_path_for(&self.config.index_path),
                source,
            }
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.index_path)
            .map_err(|source| RunBundleStoreError::Io {
                path: self.config.index_path.clone(),
                source,
            })?;
        serde_json::to_writer(&mut file, record).map_err(|source| RunBundleStoreError::Json {
            path: self.config.index_path.clone(),
            source,
        })?;
        file.write_all(b"\n")
            .map_err(|source| RunBundleStoreError::Io {
                path: self.config.index_path.clone(),
                source,
            })?;
        file.flush().map_err(|source| RunBundleStoreError::Io {
            path: self.config.index_path.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| RunBundleStoreError::Io {
            path: self.config.index_path.clone(),
            source,
        })?;
        Ok(())
    }
}

pub fn forge_tool_receipt_from_runtime(
    receipt: &CanonicalRuntimeToolReceipt,
    raw_payload: Value,
) -> CanonicalForgeToolReceiptV2 {
    receipt.to_forge_tool_receipt_v2(raw_payload)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalEventLogConfig {
    pub root_path: PathBuf,
    pub records_path: PathBuf,
    #[serde(default)]
    pub append_only: bool,
}

impl CanonicalEventLogConfig {
    pub fn for_root(root_path: impl Into<PathBuf>) -> Self {
        let root_path = root_path.into();
        Self {
            records_path: root_path.join("canonical-receipts.ndjson"),
            root_path,
            append_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalEventLogEntry {
    #[serde(default)]
    pub sequence_number: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_record_digest: Option<ContentDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_digest: Option<ContentDigest>,
    pub owner_crate: String,
    pub schema_name: String,
    pub receipt_id: String,
    pub recorded_at: String,
    pub content_digest: ContentDigest,
    pub body: Value,
}

impl CanonicalEventLogEntry {
    pub fn new(
        owner_crate: impl Into<String>,
        schema_name: impl Into<String>,
        receipt_id: impl Into<String>,
        body: Value,
    ) -> Result<Self, CanonicalEventLogError> {
        Self::new_with_chain(owner_crate, schema_name, receipt_id, body, 0, None)
    }

    pub fn new_with_chain(
        owner_crate: impl Into<String>,
        schema_name: impl Into<String>,
        receipt_id: impl Into<String>,
        body: Value,
        sequence_number: u64,
        previous_record_digest: Option<ContentDigest>,
    ) -> Result<Self, CanonicalEventLogError> {
        let content_digest = ContentDigest::compute_json(&body)
            .map_err(|source| CanonicalEventLogError::Digest { source })?;
        let mut entry = Self {
            sequence_number,
            previous_record_digest,
            record_digest: None,
            owner_crate: owner_crate.into(),
            schema_name: schema_name.into(),
            receipt_id: receipt_id.into(),
            recorded_at: Utc::now().to_rfc3339(),
            content_digest,
            body,
        };
        entry.record_digest = Some(entry.compute_record_digest()?);
        Ok(entry)
    }

    pub fn verify_digest(&self) -> bool {
        ContentDigest::compute_json(&self.body)
            .map(|digest| digest == self.content_digest)
            .unwrap_or(false)
    }

    pub fn compute_record_digest(&self) -> Result<ContentDigest, CanonicalEventLogError> {
        let payload = serde_json::json!({
            "sequence_number": self.sequence_number,
            "previous_record_digest": self.previous_record_digest,
            "owner_crate": self.owner_crate,
            "schema_name": self.schema_name,
            "receipt_id": self.receipt_id,
            "recorded_at": self.recorded_at,
            "content_digest": self.content_digest,
            "body": self.body,
        });
        ContentDigest::compute_json(&payload)
            .map_err(|source| CanonicalEventLogError::Digest { source })
    }

    pub fn verify_record_digest(&self) -> bool {
        self.record_digest.as_ref().is_some_and(|expected| {
            self.compute_record_digest()
                .is_ok_and(|actual| actual == *expected)
        })
    }
}

#[derive(Debug, Error)]
pub enum CanonicalEventLogError {
    #[error("canonical receipt log io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("canonical receipt log json error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "canonical receipt log corruption at {path} line {line_number}: {source}; quarantine error: {quarantine_error:?}"
    )]
    CorruptRecord {
        path: PathBuf,
        line_number: usize,
        quarantine_error: Option<String>,
        #[source]
        source: serde_json::Error,
    },
    #[error("canonical receipt digest error: {source}")]
    Digest {
        #[source]
        source: stack_ids::DigestError,
    },
    #[error("canonical receipt record not found: {0}")]
    NotFound(String),
    #[error("canonical receipt log chain verification failed at sequence {0}")]
    ChainVerificationFailed(u64),
    #[error("canonical receipt log duplicate receipt id: {0}")]
    DuplicateReceiptId(String),
}

#[derive(Debug, Clone)]
pub struct CanonicalEventLog {
    config: CanonicalEventLogConfig,
}

impl CanonicalEventLog {
    pub fn open(config: CanonicalEventLogConfig) -> Result<Self, CanonicalEventLogError> {
        std::fs::create_dir_all(&config.root_path).map_err(|source| {
            CanonicalEventLogError::Io {
                path: config.root_path.clone(),
                source,
            }
        })?;
        ensure_file(&config.records_path)?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &CanonicalEventLogConfig {
        &self.config
    }

    pub fn append_runtime_tool_receipt(
        &self,
        receipt: &CanonicalRuntimeToolReceipt,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        self.append_json(
            "llm-tool-runtime",
            "tool-receipt",
            receipt.receipt_id.clone(),
            serde_json::to_value(receipt).map_err(|source| CanonicalEventLogError::Json {
                path: self.config.records_path.clone(),
                source,
            })?,
        )
    }

    pub fn append_forge_tool_receipt(
        &self,
        receipt: &CanonicalForgeToolReceiptV2,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        let event_log_receipt_id = format!(
            "semantic-memory-forge:forge-tool-receipt-v2:{}",
            receipt.receipt_id
        );
        self.append_json(
            "semantic-memory-forge",
            "forge-tool-receipt-v2",
            event_log_receipt_id,
            serde_json::to_value(receipt).map_err(|source| CanonicalEventLogError::Json {
                path: self.config.records_path.clone(),
                source,
            })?,
        )
    }

    pub fn append_control_receipt(
        &self,
        receipt: &CanonicalControlReceipt,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        self.append_json(
            "verification-control",
            "control-receipt",
            receipt.receipt_id.to_string(),
            serde_json::to_value(receipt).map_err(|source| CanonicalEventLogError::Json {
                path: self.config.records_path.clone(),
                source,
            })?,
        )
    }

    pub fn append_orchestration_report(
        &self,
        schema_name: impl Into<String>,
        report_id: impl Into<String>,
        body: Value,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        self.append_json("aidens-orchestration", schema_name, report_id, body)
    }

    pub fn append_json(
        &self,
        owner_crate: impl Into<String>,
        schema_name: impl Into<String>,
        receipt_id: impl Into<String>,
        body: Value,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        let owner_crate = owner_crate.into();
        let schema_name = schema_name.into();
        let receipt_id = receipt_id.into();
        let _lock = acquire_exclusive_lock(&self.config.records_path).map_err(|source| {
            CanonicalEventLogError::Io {
                path: lock_path_for(&self.config.records_path),
                source,
            }
        })?;
        let records = read_records(&self.config.records_path)?;
        validate_record_chain(&records)?;
        if records.iter().any(|record| record.receipt_id == receipt_id) {
            return Err(CanonicalEventLogError::DuplicateReceiptId(receipt_id));
        }
        let sequence_number = records
            .last()
            .map(|record| record.sequence_number + 1)
            .unwrap_or(0);
        let previous_record_digest = records
            .last()
            .and_then(|record| record.record_digest.clone());
        let record = CanonicalEventLogEntry::new_with_chain(
            owner_crate,
            schema_name,
            receipt_id,
            body,
            sequence_number,
            previous_record_digest,
        )?;
        self.append_record_locked(&record)?;
        Ok(record)
    }

    pub fn append_record(
        &self,
        record: &CanonicalEventLogEntry,
    ) -> Result<(), CanonicalEventLogError> {
        let _lock = acquire_exclusive_lock(&self.config.records_path).map_err(|source| {
            CanonicalEventLogError::Io {
                path: lock_path_for(&self.config.records_path),
                source,
            }
        })?;
        let records = read_records(&self.config.records_path)?;
        validate_record_chain(&records)?;
        if records
            .iter()
            .any(|existing| existing.receipt_id == record.receipt_id)
        {
            return Err(CanonicalEventLogError::DuplicateReceiptId(
                record.receipt_id.clone(),
            ));
        }
        let expected_sequence = records
            .last()
            .map(|existing| existing.sequence_number + 1)
            .unwrap_or(0);
        let expected_previous = records
            .last()
            .and_then(|existing| existing.record_digest.clone());
        if record.sequence_number != expected_sequence
            || record.previous_record_digest != expected_previous
            || !record.verify_digest()
            || !record.verify_record_digest()
        {
            return Err(CanonicalEventLogError::ChainVerificationFailed(
                record.sequence_number,
            ));
        }
        self.append_record_locked(record)
    }

    fn append_record_locked(
        &self,
        record: &CanonicalEventLogEntry,
    ) -> Result<(), CanonicalEventLogError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.records_path)
            .map_err(|source| CanonicalEventLogError::Io {
                path: self.config.records_path.clone(),
                source,
            })?;
        serde_json::to_writer(&mut file, record).map_err(|source| {
            CanonicalEventLogError::Json {
                path: self.config.records_path.clone(),
                source,
            }
        })?;
        file.write_all(b"\n")
            .map_err(|source| CanonicalEventLogError::Io {
                path: self.config.records_path.clone(),
                source,
            })?;
        file.flush().map_err(|source| CanonicalEventLogError::Io {
            path: self.config.records_path.clone(),
            source,
        })?;
        file.sync_all()
            .map_err(|source| CanonicalEventLogError::Io {
                path: self.config.records_path.clone(),
                source,
            })?;
        Ok(())
    }

    pub fn list_records(&self) -> Result<Vec<CanonicalEventLogEntry>, CanonicalEventLogError> {
        read_records(&self.config.records_path)
    }

    pub fn inspect(
        &self,
        receipt_id: &str,
    ) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
        self.list_records()?
            .into_iter()
            .find(|record| record.receipt_id == receipt_id)
            .ok_or_else(|| CanonicalEventLogError::NotFound(receipt_id.into()))
    }

    pub fn verify_digest(&self, receipt_id: &str) -> Result<bool, CanonicalEventLogError> {
        Ok(self.inspect(receipt_id)?.verify_digest())
    }

    pub fn verify_chain(&self) -> Result<bool, CanonicalEventLogError> {
        let records = self.list_records()?;
        let mut previous_digest: Option<ContentDigest> = None;
        for (index, record) in records.iter().enumerate() {
            if record.sequence_number != index as u64
                || record.previous_record_digest != previous_digest
                || !record.verify_digest()
                || !record.verify_record_digest()
            {
                return Ok(false);
            }
            previous_digest = record.record_digest.clone();
        }
        Ok(true)
    }
}

#[async_trait]
impl ToolReceiptSink for CanonicalEventLog {
    async fn persist(&self, receipt: &CanonicalRuntimeToolReceipt) -> Result<(), ToolError> {
        self.append_runtime_tool_receipt(receipt)
            .map(|_| ())
            .map_err(|error| {
                ToolError::new(
                    ToolErrorClass::ReceiptPersistence,
                    format!("canonical receipt sink failed: {error}"),
                )
            })
    }
}

fn ensure_file(path: &Path) -> Result<(), CanonicalEventLogError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CanonicalEventLogError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| CanonicalEventLogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn ensure_run_bundle_file(path: &Path) -> Result<(), RunBundleStoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| RunBundleStoreError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| RunBundleStoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn read_run_bundle_records(path: &Path) -> Result<Vec<RunBundleStoreRecord>, RunBundleStoreError> {
    ensure_run_bundle_file(path)?;
    let file = File::open(path).map_err(|source| RunBundleStoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| RunBundleStoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str(&line) {
            Ok(record) => records.push(record),
            Err(source) => {
                quarantine_corrupt_line(path, index + 1, &line, &source).map_err(|source| {
                    RunBundleStoreError::Io {
                        path: quarantine_path_for(path),
                        source,
                    }
                })?;
            }
        }
    }
    Ok(records)
}

fn write_atomic_file(path: &Path, body: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_file_name(format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("receipt"),
        std::process::id(),
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| Utc::now().timestamp_micros())
    ));
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_path)?;
        file.write_all(body.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp_path, path)?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> std::io::Result<()> {
    File::open(path)?.sync_all()
}

fn acquire_exclusive_lock(path: &Path) -> std::io::Result<ExclusiveFileLock> {
    let lock_path = lock_path_for(path);
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let start = Instant::now();
    loop {
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                writeln!(
                    file,
                    "pid={} acquired_at={}",
                    std::process::id(),
                    Utc::now().to_rfc3339()
                )?;
                file.sync_all()?;
                return Ok(ExclusiveFileLock { path: lock_path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if start.elapsed() >= Duration::from_secs(10) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("receipt log lock still held at {}", lock_path.display()),
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
}

fn lock_path_for(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}.lock",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("receipt-log")
    ))
}

struct ExclusiveFileLock {
    path: PathBuf,
}

impl Drop for ExclusiveFileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn quarantine_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("receipt-log");
    path.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("quarantine")
        .join(format!("{file_name}.corrupt.ndjson"))
}

fn quarantine_corrupt_line(
    path: &Path,
    line_number: usize,
    raw_line: &str,
    error: &serde_json::Error,
) -> std::io::Result<()> {
    let quarantine_path = quarantine_path_for(path);
    if let Some(parent) = quarantine_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _lock = acquire_exclusive_lock(&quarantine_path)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&quarantine_path)?;
    let record = serde_json::json!({
        "artifact_kind": "aidens_local_corrupt_receipt_log_line_quarantine",
        "source_path": path.display().to_string(),
        "line_number": line_number,
        "observed_at": Utc::now().to_rfc3339(),
        "error": error.to_string(),
        "raw_line": raw_line,
    });
    serde_json::to_writer(&mut file, &record)?;
    file.write_all(b"\n")?;
    file.flush()?;
    file.sync_all()
}

fn read_records(path: &Path) -> Result<Vec<CanonicalEventLogEntry>, CanonicalEventLogError> {
    ensure_file(path)?;
    let file = File::open(path).map_err(|source| CanonicalEventLogError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| CanonicalEventLogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str(&line) {
            Ok(record) => records.push(record),
            Err(source) => {
                let quarantine_error = quarantine_corrupt_line(path, index + 1, &line, &source)
                    .err()
                    .map(|error| error.to_string());
                return Err(CanonicalEventLogError::CorruptRecord {
                    path: path.to_path_buf(),
                    line_number: index + 1,
                    quarantine_error,
                    source,
                });
            }
        }
    }
    Ok(records)
}

fn validate_record_chain(records: &[CanonicalEventLogEntry]) -> Result<(), CanonicalEventLogError> {
    let mut previous_digest: Option<ContentDigest> = None;
    for (index, record) in records.iter().enumerate() {
        if record.sequence_number != index as u64
            || record.previous_record_digest != previous_digest
            || !record.verify_digest()
            || !record.verify_record_digest()
        {
            return Err(CanonicalEventLogError::ChainVerificationFailed(
                record.sequence_number,
            ));
        }
        previous_digest = record.record_digest.clone();
    }
    Ok(())
}

fn receipt_store_segment(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "run".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_log_persists_owner_labeled_records() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();

        let record = log
            .append_orchestration_report(
                "operator-status-report",
                "report:test",
                serde_json::json!({"status": "ok"}),
            )
            .unwrap();

        assert_eq!(record.owner_crate, "aidens-orchestration");
        assert_eq!(record.sequence_number, 0);
        assert!(record.verify_digest());
        assert!(record.verify_record_digest());
        assert_eq!(log.inspect("report:test").unwrap(), record);
        assert!(log.verify_digest("report:test").unwrap());
        assert!(log.verify_chain().unwrap());
    }

    #[test]
    fn p28_canonical_log_digest_chain_detects_tampering() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-chain-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        let first = log
            .append_orchestration_report(
                "operator-status-report",
                "report:first",
                serde_json::json!({"status": "first"}),
            )
            .unwrap();
        let second = log
            .append_orchestration_report(
                "operator-status-report",
                "report:second",
                serde_json::json!({"status": "second"}),
            )
            .unwrap();

        assert_eq!(second.sequence_number, 1);
        assert_eq!(second.previous_record_digest, first.record_digest);
        assert!(log.verify_chain().unwrap());

        let path = log.config().records_path.clone();
        let mut lines = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut tampered: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        tampered["body"]["status"] = serde_json::json!("tampered");
        lines[0] = serde_json::to_string(&tampered).unwrap();
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();

        let reopened = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        assert!(!reopened.verify_chain().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn append_rejects_tampered_existing_chain_without_writing_a_record() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-append-tampered-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        log.append_orchestration_report(
            "operator-status-report",
            "report:before-tampering",
            serde_json::json!({"status": "valid"}),
        )
        .unwrap();

        let path = log.config().records_path.clone();
        let original = std::fs::read_to_string(&path).unwrap();
        let original_line_count = original.lines().count();
        let mut tampered: serde_json::Value = serde_json::from_str(original.trim()).unwrap();
        tampered["body"]["status"] = serde_json::json!("tampered");
        std::fs::write(&path, serde_json::to_string(&tampered).unwrap() + "\n").unwrap();

        let append_result = log.append_orchestration_report(
            "operator-status-report",
            "report:must-not-append",
            serde_json::json!({"status": "rejected"}),
        );
        let records_after_append = std::fs::read_to_string(&path).unwrap();

        assert!(matches!(
            append_result,
            Err(CanonicalEventLogError::ChainVerificationFailed(0))
        ));
        assert_eq!(records_after_append.lines().count(), original_line_count);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn phase01_concurrent_canonical_appends_keep_single_digest_chain() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-concurrent-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        let mut handles = Vec::new();

        for index in 0..24 {
            let log = log.clone();
            handles.push(std::thread::spawn(move || {
                log.append_orchestration_report(
                    "operator-status-report",
                    format!("report:concurrent:{index}"),
                    serde_json::json!({ "index": index }),
                )
                .unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let records = log.list_records().unwrap();
        assert_eq!(records.len(), 24);
        assert!(log.verify_chain().unwrap());
        for (index, record) in records.iter().enumerate() {
            assert_eq!(record.sequence_number, index as u64);
            assert!(record.verify_record_digest());
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_final_record_is_quarantined_and_fails_closed() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-corrupt-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        log.append_orchestration_report(
            "operator-status-report",
            "report:before-corruption",
            serde_json::json!({"status": "ok"}),
        )
        .unwrap();
        {
            let mut file = OpenOptions::new()
                .append(true)
                .open(log.config().records_path.clone())
                .unwrap();
            file.write_all(b"{not-json}\n").unwrap();
            file.sync_all().unwrap();
        }

        let reopened = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        assert!(matches!(
            reopened.list_records(),
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        assert!(matches!(
            reopened.verify_chain(),
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        let quarantine = quarantine_path_for(&reopened.config().records_path);
        let quarantine_text = std::fs::read_to_string(quarantine).unwrap();
        assert!(quarantine_text.contains("aidens_local_corrupt_receipt_log_line_quarantine"));
        assert!(quarantine_text.contains("not-json"));
        assert!(matches!(
            reopened.append_orchestration_report(
                "operator-status-report",
                "report:after-corruption",
                serde_json::json!({"status": "continued"}),
            ),
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_middle_record_blocks_list_verify_and_append() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-corrupt-middle-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        log.append_orchestration_report(
            "operator-status-report",
            "report:before-corruption",
            serde_json::json!({"status": "before"}),
        )
        .unwrap();
        log.append_orchestration_report(
            "operator-status-report",
            "report:after-corruption",
            serde_json::json!({"status": "after"}),
        )
        .unwrap();

        let path = log.config().records_path.clone();
        let records = std::fs::read_to_string(&path).unwrap();
        let (first, second) = records.split_once('\n').unwrap();
        std::fs::write(&path, format!("{first}\n{{not-json}}\n{second}")).unwrap();

        let list_result = log.list_records();
        let verify_result = log.verify_chain();
        let append_result = log.append_orchestration_report(
            "operator-status-report",
            "report:must-not-append",
            serde_json::json!({"status": "rejected"}),
        );

        assert!(matches!(
            list_result,
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        assert!(matches!(
            verify_result,
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        assert!(matches!(
            append_result,
            Err(CanonicalEventLogError::CorruptRecord { line_number: 2, .. })
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn phase01_duplicate_receipt_ids_are_rejected() {
        let root = std::env::temp_dir().join(format!(
            "aidens-canonical-receipts-duplicate-{}",
            uuid::Uuid::new_v4()
        ));
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        log.append_orchestration_report(
            "operator-status-report",
            "report:duplicate",
            serde_json::json!({"status": "first"}),
        )
        .unwrap();

        let err = log
            .append_orchestration_report(
                "operator-status-report",
                "report:duplicate",
                serde_json::json!({"status": "second"}),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            CanonicalEventLogError::DuplicateReceiptId(id) if id == "report:duplicate"
        ));
        assert!(log.verify_chain().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn run_bundle_store_persists_v3_operator_evidence() {
        let root =
            std::env::temp_dir().join(format!("aidens-run-bundle-store-{}", uuid::Uuid::new_v4()));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({
            "schema": "AiDENsRunBundleV3",
            "run_id": "agent:fixture/run",
            "support": {
                "support_tier": "supported-local"
            },
            "failure": {
                "degraded": false
            }
        });

        let record = store.write_bundle_value(&bundle).unwrap();
        assert_eq!(record.support_tier, "supported-local");
        assert_eq!(record.semantic_status, "exact_check");
        assert!(record.bundle_path.exists());
        assert!(record
            .bundle_path
            .display()
            .to_string()
            .contains(record.content_digest.hex()));
        assert!(matches!(
            store.write_bundle_value(&bundle),
            Err(RunBundleStoreError::AlreadyExists(_))
        ));

        let reopened = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let inspection = reopened.inspect("agent:fixture/run").unwrap();
        assert!(inspection.digest_verified);
        assert_eq!(inspection.bundle["schema"], "AiDENsRunBundleV3");
        assert_eq!(reopened.single_bundle_path().unwrap(), record.bundle_path);
        let _ = std::fs::remove_dir_all(root);
    }
}

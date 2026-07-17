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
    #[serde(default)]
    pub sequence_number: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_record_digest: Option<ContentDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_digest: Option<ContentDigest>,
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

impl RunBundleStoreRecord {
    pub fn compute_record_digest(&self) -> Result<ContentDigest, RunBundleStoreError> {
        let payload = serde_json::json!({
            "sequence_number": self.sequence_number,
            "previous_record_digest": self.previous_record_digest,
            "artifact_kind": self.artifact_kind,
            "ownership": self.ownership,
            "support_tier": self.support_tier,
            "semantic_status": self.semantic_status,
            "run_id": self.run_id,
            "bundle_schema": self.bundle_schema,
            "recorded_at": self.recorded_at,
            "bundle_path": self.bundle_path,
            "content_digest": self.content_digest,
            "canonical_event_log_path": self.canonical_event_log_path,
            "known_limits": self.known_limits,
        });
        ContentDigest::compute_json(&payload)
            .map_err(|source| RunBundleStoreError::Digest { source })
    }

    pub fn verify_record_digest(&self) -> bool {
        self.record_digest.as_ref().is_some_and(|expected| {
            self.compute_record_digest()
                .is_ok_and(|actual| actual == *expected)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunBundleStoreInspection {
    pub record: RunBundleStoreRecord,
    pub bundle: Value,
    pub digest_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunBundleRecoveryState {
    Published,
    PendingIndex,
    Indeterminate,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBundleRecoveryEntry {
    pub run_id: String,
    pub bundle_path: Option<PathBuf>,
    pub state: RunBundleRecoveryState,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunBundleRecoveryReport {
    pub entries: Vec<RunBundleRecoveryEntry>,
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
    #[error("run bundle integrity verification failed: {0}")]
    IntegrityFailed(String),
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
        let store = Self { config };
        let _ = store.reconcile()?;
        Ok(store)
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
        let _publication_lock =
            acquire_exclusive_lock(&self.config.index_path).map_err(|source| {
                RunBundleStoreError::Io {
                    path: lock_path_for(&self.config.index_path),
                    source,
                }
            })?;
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
        let mut record = RunBundleStoreRecord {
            sequence_number: 0,
            previous_record_digest: None,
            record_digest: None,
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
        self.append_record_locked(&mut record)?;
        Ok(record)
    }

    pub fn reconcile(&self) -> Result<RunBundleRecoveryReport, RunBundleStoreError> {
        let records = read_run_bundle_records(&self.config.index_path)?;
        let mut report = RunBundleRecoveryReport::default();
        let mut indexed = std::collections::BTreeSet::new();
        let mut seen_records = std::collections::BTreeSet::new();
        let mut chain_unbroken = true;
        let mut expected_sequence = 0u64;
        let mut expected_previous = None;
        for record in records {
            let duplicate = !seen_records.insert(record.bundle_path.clone());
            indexed.insert(record.bundle_path.clone());
            let record_integrity = chain_unbroken
                && record.sequence_number == expected_sequence
                && record.previous_record_digest == expected_previous
                && record.verify_record_digest();
            if record_integrity {
                expected_sequence += 1;
                expected_previous = record.record_digest.clone();
            } else {
                chain_unbroken = false;
            }
            let (state, reason) = if duplicate {
                (
                    RunBundleRecoveryState::Quarantined,
                    "duplicate_index_record",
                )
            } else if !record_integrity {
                (
                    RunBundleRecoveryState::Quarantined,
                    "index_record_integrity_failed",
                )
            } else if !record.bundle_path.is_file() {
                (RunBundleRecoveryState::Indeterminate, "missing_bundle")
            } else {
                match std::fs::read_to_string(&record.bundle_path) {
                    Err(_) => (RunBundleRecoveryState::Indeterminate, "bundle_read_failed"),
                    Ok(text) => match serde_json::from_str::<Value>(&text) {
                        Err(_) => (RunBundleRecoveryState::Quarantined, "malformed_bundle"),
                        Ok(bundle)
                            if bundle.get("schema").and_then(Value::as_str)
                                != Some("AiDENsRunBundleV3") =>
                        {
                            (
                                RunBundleRecoveryState::Quarantined,
                                "unsupported_bundle_schema",
                            )
                        }
                        Ok(bundle)
                            if ContentDigest::compute_json(&bundle).ok()
                                != Some(record.content_digest.clone()) =>
                        {
                            (
                                RunBundleRecoveryState::Indeterminate,
                                "bundle_digest_mismatch",
                            )
                        }
                        Ok(bundle) if !bundle_matches_record(self, &record, &bundle) => (
                            RunBundleRecoveryState::Quarantined,
                            "bundle_identity_mismatch",
                        ),
                        Ok(bundle) if !verify_child_references(&bundle) => (
                            RunBundleRecoveryState::Quarantined,
                            "child_reference_unverified",
                        ),
                        Ok(_) => (RunBundleRecoveryState::Published, "index_record_published"),
                    },
                }
            };
            report.entries.push(RunBundleRecoveryEntry {
                run_id: record.run_id,
                bundle_path: Some(record.bundle_path),
                state,
                reason: reason.into(),
            });
        }
        for path in scan_bundle_paths(&self.config.bundles_path)? {
            if indexed.contains(&path) {
                continue;
            }
            let (state, reason, run_id) = match std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            {
                Some(bundle) => {
                    let run_id = bundle
                        .get("run_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    if bundle.get("schema").and_then(Value::as_str) != Some("AiDENsRunBundleV3") {
                        (
                            RunBundleRecoveryState::Quarantined,
                            "unsupported_bundle_schema",
                            run_id,
                        )
                    } else {
                        let actual_digest = ContentDigest::compute_json(&bundle).ok();
                        let path_digest = path
                            .parent()
                            .and_then(Path::file_name)
                            .and_then(|name| name.to_str());
                        let digest_matches = actual_digest
                            .as_ref()
                            .is_some_and(|digest| path_digest == Some(&digest.hex()[..]));
                        if !digest_matches {
                            (
                                RunBundleRecoveryState::Indeterminate,
                                "bundle_digest_mismatch",
                                run_id,
                            )
                        } else if !verify_child_references(&bundle) {
                            (
                                RunBundleRecoveryState::Quarantined,
                                "child_reference_unverified",
                                run_id,
                            )
                        } else {
                            (
                                RunBundleRecoveryState::PendingIndex,
                                "bundle_without_index",
                                run_id,
                            )
                        }
                    }
                }
                None => (
                    RunBundleRecoveryState::Quarantined,
                    "malformed_bundle",
                    "unknown".into(),
                ),
            };
            report.entries.push(RunBundleRecoveryEntry {
                run_id,
                bundle_path: Some(path),
                state,
                reason: reason.into(),
            });
        }
        Ok(report)
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
        let published = self.reconcile()?.entries.into_iter().any(|entry| {
            entry.run_id == record.run_id
                && entry.bundle_path.as_ref() == Some(&record.bundle_path)
                && entry.state == RunBundleRecoveryState::Published
        });
        if !published {
            return Err(RunBundleStoreError::IntegrityFailed(format!(
                "run bundle is not in a verified published state: {run_id}"
            )));
        }
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
        if !digest_verified
            || !bundle_matches_record(self, &record, &bundle)
            || !verify_child_references(&bundle)
        {
            return Err(RunBundleStoreError::IntegrityFailed(format!(
                "run bundle content failed identity, digest, or child closure checks: {run_id}"
            )));
        }
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

    fn append_record_locked(
        &self,
        record: &mut RunBundleStoreRecord,
    ) -> Result<(), RunBundleStoreError> {
        let records = read_run_bundle_records(&self.config.index_path)?;
        let mut expected_previous = None;
        for (sequence, existing) in records.iter().enumerate() {
            if existing.sequence_number != sequence as u64
                || existing.previous_record_digest != expected_previous
                || !existing.verify_record_digest()
            {
                return Err(RunBundleStoreError::ChainVerificationFailed(
                    existing.sequence_number,
                ));
            }
            expected_previous = existing.record_digest.clone();
        }
        record.sequence_number = records.len() as u64;
        record.previous_record_digest = expected_previous;
        record.record_digest = Some(record.compute_record_digest()?);
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

fn bundle_matches_record(
    store: &RunBundleStore,
    record: &RunBundleStoreRecord,
    bundle: &Value,
) -> bool {
    let Some(run_id) = bundle.get("run_id").and_then(Value::as_str) else {
        return false;
    };
    let Some(schema) = bundle.get("schema").and_then(Value::as_str) else {
        return false;
    };
    let support_tier = bundle
        .pointer("/support/support_tier")
        .and_then(Value::as_str)
        .unwrap_or("partial");
    let semantic_status = if bundle
        .pointer("/failure/degraded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "degraded_exact_check"
    } else {
        "exact_check"
    };
    record.run_id == run_id
        && record.bundle_schema == schema
        && record.support_tier == support_tier
        && record.semantic_status == semantic_status
        && record.bundle_path
            == store.bundle_path_for_run_id_and_digest(run_id, &record.content_digest)
        && record.canonical_event_log_path
            == store.config.root_path.join("canonical-receipts.ndjson")
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

fn scan_bundle_paths(root: &Path) -> Result<Vec<PathBuf>, RunBundleStoreError> {
    let mut paths = Vec::new();
    if !root.is_dir() {
        return Ok(paths);
    }
    for entry in std::fs::read_dir(root).map_err(|source| RunBundleStoreError::Io {
        path: root.into(),
        source,
    })? {
        let entry = entry.map_err(|source| RunBundleStoreError::Io {
            path: root.into(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            paths.extend(scan_bundle_paths(&path)?);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("run-bundle.json") {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Verify only references whose owner receipt is embedded in the V3 bundle.
/// A reference without a closed, digest-valid owner is never promoted.
fn verify_child_references(bundle: &Value) -> bool {
    let mut observed_children = std::collections::BTreeSet::new();
    for key in ["child_receipts", "children", "child_references"] {
        let Some(children) = bundle.get(key) else {
            continue;
        };
        let Some(children) = children.as_array() else {
            return false;
        };
        for child in children {
            let Some(object) = child.as_object() else {
                return false;
            };
            let Some(receipt) = object
                .get("receipt")
                .or_else(|| object.get("owner_receipt"))
            else {
                return false;
            };
            let Some(owner_id) = object.get("owner_id").and_then(Value::as_str) else {
                return false;
            };
            if owner_id.is_empty() {
                return false;
            }
            let Some(digest) = object
                .get("digest")
                .or_else(|| object.get("content_digest"))
            else {
                return false;
            };
            let Some(digest) = digest.as_str() else {
                return false;
            };
            let Ok(actual) = ContentDigest::compute_json(receipt) else {
                return false;
            };
            if actual.hex() != digest || object.get("closed").and_then(Value::as_bool) != Some(true)
            {
                return false;
            }
            if !observed_children.insert((owner_id.to_string(), digest.to_string())) {
                return false;
            }
        }
    }
    if let Some(required) = bundle.get("required_children") {
        let Some(required) = required.as_array() else {
            return false;
        };
        let mut required_children = std::collections::BTreeSet::new();
        for child in required {
            let Some(object) = child.as_object() else {
                return false;
            };
            let Some(owner_id) = object.get("owner_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(digest) = object
                .get("digest")
                .or_else(|| object.get("content_digest"))
                .and_then(Value::as_str)
            else {
                return false;
            };
            if owner_id.is_empty()
                || digest.is_empty()
                || !required_children.insert((owner_id.to_string(), digest.to_string()))
            {
                return false;
            }
        }
        if required_children != observed_children {
            return false;
        }
    }
    true
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
                quarantine_corrupt_line(path, index + 1, &line, &source).map_err(|source| {
                    CanonicalEventLogError::Io {
                        path: quarantine_path_for(path),
                        source,
                    }
                })?;
            }
        }
    }
    Ok(records)
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
    fn phase01_corrupt_trailing_record_is_quarantined_not_history_poisoning() {
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
        let records = reopened.list_records().unwrap();
        assert_eq!(records.len(), 1);
        assert!(reopened.verify_chain().unwrap());
        let quarantine = quarantine_path_for(&reopened.config().records_path);
        let quarantine_text = std::fs::read_to_string(quarantine).unwrap();
        assert!(quarantine_text.contains("aidens_local_corrupt_receipt_log_line_quarantine"));
        assert!(quarantine_text.contains("not-json"));
        let appended = reopened
            .append_orchestration_report(
                "operator-status-report",
                "report:after-corruption",
                serde_json::json!({"status": "continued"}),
            )
            .unwrap();
        assert_eq!(appended.sequence_number, 1);
        assert!(reopened.verify_chain().unwrap());
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

    #[test]
    fn recovery_scans_orphan_bundle_and_quarantines_unverified_child() {
        let root =
            std::env::temp_dir().join(format!("aidens-recovery-matrix-{}", uuid::Uuid::new_v4()));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({"schema":"AiDENsRunBundleV3","run_id":"orphan","child_receipts":[{"closed":false,"receipt":{"id":"child"},"digest":"00"}]});
        let digest = ContentDigest::compute_json(&bundle).unwrap();
        let path = store.bundle_path_for_run_id_and_digest("orphan", &digest);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string(&bundle).unwrap()).unwrap();
        let report = store.reconcile().unwrap();
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.state == RunBundleRecoveryState::Quarantined
                && entry.reason == "child_reference_unverified"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_quarantines_orphan_whose_path_digest_does_not_match_contents() {
        let root = std::env::temp_dir().join(format!(
            "aidens-recovery-orphan-digest-mismatch-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({"schema":"AiDENsRunBundleV3","run_id":"orphan-digest"});
        let wrong_digest = ContentDigest::compute_json(&serde_json::json!({"different":true})).unwrap();
        let path = store.bundle_path_for_run_id_and_digest("orphan-digest", &wrong_digest);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string(&bundle).unwrap()).unwrap();

        let report = store.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.run_id == "orphan-digest"
                && entry.state == RunBundleRecoveryState::Indeterminate
                && entry.reason == "bundle_digest_mismatch"
        }));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_classifies_valid_orphan_as_pending_index() {
        let root =
            std::env::temp_dir().join(format!("aidens-recovery-pending-{}", uuid::Uuid::new_v4()));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({"schema":"AiDENsRunBundleV3","run_id":"orphan-valid"});
        let digest = ContentDigest::compute_json(&bundle).unwrap();
        let path = store.bundle_path_for_run_id_and_digest("orphan-valid", &digest);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string(&bundle).unwrap()).unwrap();
        let report = store.reconcile().unwrap();
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.state == RunBundleRecoveryState::PendingIndex
                && entry.reason == "bundle_without_index"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_matrix_classifies_index_and_child_failures_with_reason_codes() {
        let root = std::env::temp_dir().join(format!(
            "aidens-recovery-matrix-cases-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let valid = serde_json::json!({"schema":"AiDENsRunBundleV3","run_id":"valid"});
        let digest = ContentDigest::compute_json(&valid).unwrap();
        let path = store.bundle_path_for_run_id_and_digest("valid", &digest);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string(&valid).unwrap()).unwrap();
        let missing = store.bundle_path_for_run_id_and_digest("missing", &digest);
        let mut record = RunBundleStoreRecord {
            sequence_number: 0,
            previous_record_digest: None,
            record_digest: None,
            artifact_kind: "local_operator_run_bundle_store_record".into(),
            ownership: "x".into(),
            support_tier: "partial".into(),
            semantic_status: "exact_check".into(),
            run_id: "missing".into(),
            bundle_schema: "AiDENsRunBundleV3".into(),
            recorded_at: Utc::now().to_rfc3339(),
            bundle_path: missing.clone(),
            content_digest: digest.clone(),
            canonical_event_log_path: root.join("canonical-receipts.ndjson"),
            known_limits: vec![],
        };
        record.record_digest = Some(record.compute_record_digest().unwrap());
        std::fs::write(
            &store.config().index_path,
            format!(
                "{}\n{}\n",
                serde_json::to_string(&record).unwrap(),
                serde_json::to_string(&record).unwrap()
            ),
        )
        .unwrap();
        let report = store.reconcile().unwrap();
        assert!(report
            .entries
            .iter()
            .any(|e| e.reason == "duplicate_index_record"
                && e.state == RunBundleRecoveryState::Quarantined));
        assert!(report
            .entries
            .iter()
            .any(|e| e.reason == "missing_bundle"
                && e.state == RunBundleRecoveryState::Indeterminate));
        assert!(report
            .entries
            .iter()
            .any(|e| e.reason == "bundle_without_index"
                && e.state == RunBundleRecoveryState::PendingIndex));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn required_child_set_requires_owner_id_digest_and_closed_child() {
        let bundle = serde_json::json!({"schema":"AiDENsRunBundleV3","run_id":"child","required_children":[{"owner_id":"owner","digest":"00"}],"child_receipts":[]});
        assert!(!verify_child_references(&bundle));
    }

    #[test]
    fn required_child_set_must_exactly_match_observed_owner_digest_pairs() {
        let receipt_a = serde_json::json!({"receipt_id":"owner:a","status":"closed"});
        let receipt_b = serde_json::json!({"receipt_id":"owner:b","status":"closed"});
        let digest_a = ContentDigest::compute_json(&receipt_a).unwrap();
        let digest_b = ContentDigest::compute_json(&receipt_b).unwrap();
        let child_a = serde_json::json!({
            "owner_id":"owner:a",
            "digest":digest_a.hex(),
            "closed":true,
            "receipt":receipt_a,
        });
        let child_b = serde_json::json!({
            "owner_id":"owner:b",
            "digest":digest_b.hex(),
            "closed":true,
            "receipt":receipt_b,
        });

        let exact = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"child-exact",
            "required_children":[
                {"owner_id":"owner:a","digest":digest_a.hex()},
                {"owner_id":"owner:b","digest":digest_b.hex()},
            ],
            "child_receipts":[child_a.clone(), child_b.clone()],
        });
        assert!(verify_child_references(&exact));

        let missing_required = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"child-missing",
            "required_children":[
                {"owner_id":"owner:a","digest":digest_a.hex()},
                {"owner_id":"owner:b","digest":digest_b.hex()},
            ],
            "child_receipts":[child_a.clone()],
        });
        assert!(!verify_child_references(&missing_required));

        let unexpected_observed = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"child-unexpected",
            "required_children":[{"owner_id":"owner:a","digest":digest_a.hex()}],
            "child_receipts":[child_a, child_b],
        });
        assert!(!verify_child_references(&unexpected_observed));
    }

    #[test]
    fn failed_index_append_retains_bundle_for_reconciliation() {
        let root = std::env::temp_dir().join(format!(
            "aidens-index-append-failure-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"append-failed",
        });
        let digest = ContentDigest::compute_json(&bundle).unwrap();
        let expected_path = store.bundle_path_for_run_id_and_digest("append-failed", &digest);

        std::fs::remove_file(&store.config().index_path).unwrap();
        std::fs::create_dir(&store.config().index_path).unwrap();
        assert!(matches!(
            store.write_bundle_value(&bundle),
            Err(RunBundleStoreError::Io { .. })
        ));
        assert!(expected_path.is_file());

        std::fs::remove_dir(&store.config().index_path).unwrap();
        let reopened = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let report = reopened.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.bundle_path.as_ref() == Some(&expected_path)
                && entry.state == RunBundleRecoveryState::PendingIndex
                && entry.reason == "bundle_without_index"
        }));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn indexed_bundle_recovery_uses_fail_closed_reason_codes() {
        let root = std::env::temp_dir().join(format!(
            "aidens-indexed-bundle-reasons-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let original = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"tampered",
        });
        let record = store.write_bundle_value(&original).unwrap();
        std::fs::write(
            &record.bundle_path,
            serde_json::to_string(&serde_json::json!({
                "schema":"AiDENsRunBundleV3",
                "run_id":"tampered",
                "unexpected":true,
            }))
            .unwrap(),
        )
        .unwrap();
        let report = store.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.run_id == "tampered"
                && entry.state == RunBundleRecoveryState::Indeterminate
                && entry.reason == "bundle_digest_mismatch"
        }));

        let child_root = std::env::temp_dir().join(format!(
            "aidens-indexed-child-reasons-{}",
            uuid::Uuid::new_v4()
        ));
        let child_store =
            RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&child_root)).unwrap();
        let child_receipt = serde_json::json!({"receipt_id":"effect:1"});
        let child_digest = ContentDigest::compute_json(&child_receipt).unwrap();
        let unclosed = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"effect-without-outcome",
            "required_children":[{"owner_id":"effect:1","digest":child_digest.hex()}],
            "child_receipts":[{
                "owner_id":"effect:1",
                "digest":child_digest.hex(),
                "closed":false,
                "receipt":child_receipt,
            }],
        });
        child_store.write_bundle_value(&unclosed).unwrap();
        let report = child_store.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.run_id == "effect-without-outcome"
                && entry.state == RunBundleRecoveryState::Quarantined
                && entry.reason == "child_reference_unverified"
        }));

        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(child_root);
    }

    #[test]
    fn run_bundle_index_chain_and_identity_tampering_fail_closed() {
        let root = std::env::temp_dir().join(format!(
            "aidens-run-bundle-index-integrity-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"bound-run",
            "support":{"support_tier":"supported-local"},
            "failure":{"degraded":false},
        });
        let record = store.write_bundle_value(&bundle).unwrap();
        assert_eq!(record.sequence_number, 0);
        assert!(record.previous_record_digest.is_none());
        assert!(record.verify_record_digest());

        let mut stored: Value = serde_json::from_str(
            std::fs::read_to_string(&store.config().index_path)
                .unwrap()
                .trim(),
        )
        .unwrap();
        stored["run_id"] = Value::String("attacker-selected-run".into());
        std::fs::write(
            &store.config().index_path,
            serde_json::to_string(&stored).unwrap() + "\n",
        )
        .unwrap();

        let report = store.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.run_id == "attacker-selected-run"
                && entry.state == RunBundleRecoveryState::Quarantined
                && entry.reason == "index_record_integrity_failed"
        }));
        assert!(matches!(
            store.inspect("attacker-selected-run"),
            Err(RunBundleStoreError::IntegrityFailed(_))
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn inspection_rejects_bundle_identity_mismatch_even_with_rehashed_index_record() {
        let root = std::env::temp_dir().join(format!(
            "aidens-run-bundle-identity-binding-{}",
            uuid::Uuid::new_v4()
        ));
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap();
        let bundle = serde_json::json!({
            "schema":"AiDENsRunBundleV3",
            "run_id":"original-run",
        });
        let record = store.write_bundle_value(&bundle).unwrap();
        let mut rewritten = record.clone();
        rewritten.run_id = "rewritten-run".into();
        rewritten.record_digest = Some(rewritten.compute_record_digest().unwrap());
        std::fs::write(
            &store.config().index_path,
            serde_json::to_string(&rewritten).unwrap() + "\n",
        )
        .unwrap();

        let report = store.reconcile().unwrap();
        assert!(report.entries.iter().any(|entry| {
            entry.run_id == "rewritten-run"
                && entry.state == RunBundleRecoveryState::Quarantined
                && entry.reason == "bundle_identity_mismatch"
        }));
        assert!(matches!(
            store.inspect("rewritten-run"),
            Err(RunBundleStoreError::IntegrityFailed(_))
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_same_bundle_publication_has_one_winner_and_one_index_record() {
        let root = std::env::temp_dir().join(format!(
            "aidens-run-bundle-concurrent-publication-{}",
            uuid::Uuid::new_v4()
        ));
        let store = std::sync::Arc::new(
            RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&root)).unwrap(),
        );
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(16));
        let mut handles = Vec::new();
        for _ in 0..16 {
            let store = store.clone();
            let barrier = barrier.clone();
            handles.push(std::thread::spawn(move || {
                let bundle = serde_json::json!({
                    "schema":"AiDENsRunBundleV3",
                    "run_id":"same-run",
                });
                barrier.wait();
                store.write_bundle_value(&bundle)
            }));
        }
        let results = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert!(
            results
                .iter()
                .filter(|result| matches!(result, Err(RunBundleStoreError::AlreadyExists(_))))
                .count()
                >= 15
        );
        assert_eq!(store.list_records().unwrap().len(), 1);
        assert_eq!(
            store.reconcile().unwrap().entries[0].state,
            RunBundleRecoveryState::Published
        );
        let _ = std::fs::remove_dir_all(root);
    }
}

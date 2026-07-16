use crate::{
    CompactResponse, ContextGovernorError, ExactFallbackRefV1, ExactRecoveryStateV1, Message,
    RecoveryDurabilityV1, SummaryLossReportV1,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;
use std::path::Path;

/// A candidate produced during rehydration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RetrievalCandidateV1 {
    pub candidate_id: String,
    pub receipt_id: String,
    pub exact_fallback_ref_id: String,
    pub item_source_index: usize,
    pub rank: usize,
    pub retrieval_score: u32,
    pub rejection_reason: Option<String>,
    pub content_blake3: String,
    pub approx_tokens: usize,
}

/// Receipt for a rehydration operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct RetrievalReceiptV1 {
    pub schema: String,
    pub query: RehydrationQueryV1,
    pub candidates: Vec<RetrievalCandidateV1>,
    pub selected_candidate_ids: Vec<String>,
    pub total_approx_tokens: usize,
    pub rejection_reasons: Vec<String>,
    pub search_duration_ms: u64,
}

/// Query for rehydrating context from a store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct RehydrationQueryV1 {
    pub schema: String,
    pub receipt_id: Option<String>,
    pub token_budget: usize,
    pub authority_floor: Option<String>,
    pub lineage: Vec<String>,
    pub keywords: Vec<String>,
    pub top_k: usize,
}

/// Trait for durable context stores.
pub trait ContextStore {
    fn save(&self, response: &CompactResponse) -> Result<StoreSaveReceiptV1, ContextGovernorError>;
    fn load(&self, receipt_id: &str) -> Result<CompactResponse, ContextGovernorError>;
    fn list_receipts(&self) -> Result<Vec<StoreReceiptInfoV1>, ContextGovernorError>;
    fn search(
        &self,
        query: &RehydrationQueryV1,
    ) -> Result<RetrievalReceiptV1, ContextGovernorError>;
    fn prune_receipts_keep_last(
        &self,
        keep_last: usize,
    ) -> Result<StorePruneResultV1, ContextGovernorError>;
    fn status(&self) -> Result<StoreStatusV1, ContextGovernorError>;
}

/// Receipt returned by a save operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct StoreSaveReceiptV1 {
    pub schema: String,
    pub receipt_id: String,
    pub exact_items_stored: usize,
    pub verified: bool,
    pub recovery_durability: RecoveryDurabilityV1,
}

/// Metadata for a stored receipt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct StoreReceiptInfoV1 {
    pub receipt_id: String,
    pub stored_at: String,
    pub compacted_token_count: usize,
    pub exact_item_count: usize,
    pub recovery_durability: RecoveryDurabilityV1,
}

/// Status of a store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct StoreStatusV1 {
    pub schema: String,
    pub path: String,
    pub receipt_count: usize,
    pub exact_item_count: usize,
    pub total_bytes: u64,
    pub searchable: bool,
    pub last_receipt_id: Option<String>,
}

/// Result of a prune operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct StorePruneResultV1 {
    pub schema: String,
    pub kept_receipts: usize,
    pub removed_receipts: usize,
    pub removed_exact_items: usize,
    pub total_bytes: u64,
}

/// SQLite-backed implementation of `ContextStore`.
pub struct SqliteContextStore {
    conn: Connection,
    path: std::path::PathBuf,
}

impl SqliteContextStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ContextGovernorError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)?;
        let store = Self { conn, path };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, ContextGovernorError> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn,
            path: std::path::PathBuf::from(":memory:"),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), ContextGovernorError> {
        self.conn.execute_batch(
            "PRAGMA foreign_keys=ON;
            BEGIN;
            CREATE TABLE IF NOT EXISTS receipts (
                receipt_id TEXT PRIMARY KEY,
                stored_at TEXT NOT NULL,
                compacted_json TEXT NOT NULL,
                compacted_token_count INTEGER NOT NULL,
                recovery_durability TEXT NOT NULL,
                summary_loss_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS exact_items (
                receipt_id TEXT NOT NULL,
                ref_id TEXT NOT NULL,
                content_blake3 TEXT NOT NULL,
                source_index INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                content TEXT NOT NULL,
                content_kind TEXT NOT NULL,
                sensitivity TEXT NOT NULL,
                archived INTEGER NOT NULL,
                PRIMARY KEY (receipt_id, ref_id),
                FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS search_index (
                token TEXT NOT NULL,
                receipt_id TEXT NOT NULL,
                exact_ref_id TEXT NOT NULL,
                source_index INTEGER NOT NULL,
                FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_search_token ON search_index(token);
            CREATE INDEX IF NOT EXISTS idx_exact_receipt ON exact_items(receipt_id);
            COMMIT;",
        )?;
        self.migrate_exact_items_primary_key()?;
        Ok(())
    }

    fn migrate_exact_items_primary_key(&self) -> Result<(), ContextGovernorError> {
        let mut statement = self.conn.prepare("PRAGMA table_info(exact_items)")?;
        let mut primary_key_columns = statement
            .query_map([], |row| {
                Ok((row.get::<_, usize>(5)?, row.get::<_, String>(1)?))
            })?
            .filter_map(Result::ok)
            .filter(|(position, _)| *position > 0)
            .collect::<Vec<_>>();
        primary_key_columns.sort_by_key(|(position, _)| *position);
        let scoped = primary_key_columns
            .iter()
            .map(|(_, name)| name.as_str())
            .eq(["receipt_id", "ref_id"]);
        drop(statement);
        if scoped {
            return Ok(());
        }

        let compacted_json = {
            let mut statement = self
                .conn
                .prepare("SELECT compacted_json FROM receipts ORDER BY receipt_id")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        let transaction = self.conn.unchecked_transaction()?;
        transaction.execute_batch(
            "CREATE TABLE exact_items_v2 (
                receipt_id TEXT NOT NULL,
                ref_id TEXT NOT NULL,
                content_blake3 TEXT NOT NULL,
                source_index INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                content TEXT NOT NULL,
                content_kind TEXT NOT NULL,
                sensitivity TEXT NOT NULL,
                archived INTEGER NOT NULL,
                PRIMARY KEY (receipt_id, ref_id),
                FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE
            );",
        )?;
        for json in compacted_json {
            let response: CompactResponse = serde_json::from_str(&json)?;
            for item in response.exact_store {
                transaction.execute(
                    "INSERT INTO exact_items_v2 (
                        receipt_id, ref_id, content_blake3, source_index,
                        item_type, content, content_kind, sensitivity, archived
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        &response.receipt.receipt_id,
                        &item.item_id,
                        &item.content_blake3,
                        item.source_indices.first().copied().unwrap_or(0) as i64,
                        "exact_stored",
                        &item.content,
                        "plain_text",
                        "internal",
                        0i32,
                    ],
                )?;
            }
        }
        transaction.execute_batch(
            "DROP TABLE exact_items;
             ALTER TABLE exact_items_v2 RENAME TO exact_items;
             CREATE INDEX idx_exact_receipt ON exact_items(receipt_id);",
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .filter(|s| s.len() > 2)
            .map(|s| s.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

impl ContextStore for SqliteContextStore {
    fn save(&self, response: &CompactResponse) -> Result<StoreSaveReceiptV1, ContextGovernorError> {
        let compacted_json = serde_json::to_string(response)?;
        let exact_items_count = response.exact_store.len();
        let stored_at = chrono::Utc::now().to_rfc3339();
        let recovery_durability = if exact_items_count == 0 {
            RecoveryDurabilityV1::Unavailable
        } else {
            RecoveryDurabilityV1::Persisted
        };

        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO receipts (receipt_id, stored_at, compacted_json, compacted_token_count, recovery_durability, summary_loss_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &response.receipt.receipt_id,
                stored_at,
                compacted_json,
                response.receipt.compacted_approx_tokens as i64,
                serde_json::to_string(&recovery_durability)?,
                serde_json::to_string(&response.receipt.summary_loss_report)?,
            ],
        )?;

        // delete old exact items and index for this receipt to re-index cleanly
        tx.execute(
            "DELETE FROM exact_items WHERE receipt_id = ?1",
            params![&response.receipt.receipt_id],
        )?;
        tx.execute(
            "DELETE FROM search_index WHERE receipt_id = ?1",
            params![&response.receipt.receipt_id],
        )?;

        for item in &response.exact_store {
            let source_index = item.source_indices.first().copied().unwrap_or(0);
            tx.execute(
                "INSERT INTO exact_items (content_blake3, receipt_id, ref_id, source_index, item_type, content, content_kind, sensitivity, archived)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &item.content_blake3,
                    &response.receipt.receipt_id,
                    &item.item_id,
                    source_index as i64,
                    "exact_stored".to_string(),
                    &item.content,
                    "plain_text".to_string(),
                    "internal".to_string(),
                    0i32,
                ],
            )?;
            for token in Self::tokenize(&item.content) {
                tx.execute(
                    "INSERT INTO search_index (token, receipt_id, exact_ref_id, source_index)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        token,
                        &response.receipt.receipt_id,
                        &item.item_id,
                        source_index as i64,
                    ],
                )?;
            }
        }
        tx.commit()?;

        let verified = self.load(&response.receipt.receipt_id).map(|loaded| {
            loaded.receipt.receipt_id == response.receipt.receipt_id
                && loaded.exact_store.len() == response.exact_store.len()
        })?;

        Ok(StoreSaveReceiptV1 {
            schema: "StoreSaveReceiptV1".to_string(),
            receipt_id: response.receipt.receipt_id.clone(),
            exact_items_stored: exact_items_count,
            verified,
            recovery_durability: recovery_durability.clone(),
        })
    }

    fn load(&self, receipt_id: &str) -> Result<CompactResponse, ContextGovernorError> {
        let json: String = self
            .conn
            .query_row(
                "SELECT compacted_json FROM receipts WHERE receipt_id = ?1",
                params![receipt_id],
                |row| row.get(0),
            )
            .map_err(|_| ContextGovernorError::ReceiptNotFound(receipt_id.to_string()))?;
        let mut response: CompactResponse = serde_json::from_str(&json)?;
        response.receipt.recovery_durability = RecoveryDurabilityV1::Persisted;
        response.receipt.summary_loss_report.exact_recovery_state = ExactRecoveryStateV1::Persisted;
        Ok(response)
    }

    fn list_receipts(&self) -> Result<Vec<StoreReceiptInfoV1>, ContextGovernorError> {
        let mut stmt = self.conn.prepare(
            "SELECT receipt_id, stored_at, compacted_token_count, recovery_durability
             FROM receipts ORDER BY stored_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let recovery_json: String = row.get(3)?;
            let recovery_durability: RecoveryDurabilityV1 =
                serde_json::from_str(&recovery_json).unwrap_or_default();
            Ok(StoreReceiptInfoV1 {
                receipt_id: row.get(0)?,
                stored_at: row.get(1)?,
                compacted_token_count: row.get(2)?,
                exact_item_count: 0,
                recovery_durability,
            })
        })?;
        let mut out = Vec::new();
        for info in rows {
            let mut info = info?;
            let count: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM exact_items WHERE receipt_id = ?1",
                params![&info.receipt_id],
                |row| row.get(0),
            )?;
            info.exact_item_count = count as usize;
            out.push(info);
        }
        Ok(out)
    }

    fn search(
        &self,
        query: &RehydrationQueryV1,
    ) -> Result<RetrievalReceiptV1, ContextGovernorError> {
        let started = std::time::Instant::now();
        let mut candidates: Vec<RetrievalCandidateV1> = Vec::new();
        let mut rejection_reasons: Vec<String> = Vec::new();

        // Build a scored map of exact_items matching keywords or all receipts for budget
        let keywords_lower: Vec<String> = query
            .keywords
            .iter()
            .map(|k| k.to_lowercase())
            .filter(|k| k.len() > 2)
            .collect();

        let mut stmt = self.conn.prepare(
            "SELECT receipt_id, ref_id, source_index, content_blake3, content
             FROM exact_items
             WHERE (?1 IS NULL OR receipt_id = ?1)
             ORDER BY receipt_id, source_index",
        )?;
        let receipt_filter = query.receipt_id.as_deref();
        let rows = stmt.query_map(params![receipt_filter], |row| {
            Ok((
                row.get::<usize, String>(0)?,
                row.get::<usize, String>(1)?,
                row.get::<usize, usize>(2)?,
                row.get::<usize, String>(3)?,
                row.get::<usize, String>(4)?,
            ))
        })?;

        for (rank, row) in rows.enumerate() {
            let (receipt_id, ref_id, source_index, blake3, content) = row?;
            let mut score: u32 = 0;
            let content_lower = content.to_lowercase();
            for kw in &keywords_lower {
                if content_lower.contains(kw) {
                    score += 10;
                }
            }
            if !query.lineage.is_empty() {
                for ancestor in &query.lineage {
                    if content_lower.contains(&ancestor.to_lowercase()) {
                        score += 5;
                    }
                }
            }

            let approx_tokens = content.split_whitespace().count().max(1);
            let mut rejection: Option<String> = None;
            if query.authority_floor.is_some() && score == 0 && !keywords_lower.is_empty() {
                rejection = Some("below authority/score floor".to_string());
            }
            candidates.push(RetrievalCandidateV1 {
                candidate_id: format!("{}-{}", receipt_id, ref_id),
                receipt_id,
                exact_fallback_ref_id: ref_id,
                item_source_index: source_index,
                rank: rank + 1,
                retrieval_score: score,
                rejection_reason: rejection,
                content_blake3: blake3,
                approx_tokens,
            });
        }

        // sort by score desc, then rank asc
        candidates.sort_by(|a, b| {
            b.retrieval_score
                .cmp(&a.retrieval_score)
                .then_with(|| a.rank.cmp(&b.rank))
        });

        // greedy top-k under token budget
        let mut selected = Vec::new();
        let mut used_tokens = 0usize;
        for c in &candidates {
            if c.rejection_reason.is_some() {
                continue;
            }
            if selected.len() >= query.top_k {
                rejection_reasons.push(format!(
                    "candidate {} excluded by top_k limit",
                    c.candidate_id
                ));
                continue;
            }
            if used_tokens + c.approx_tokens > query.token_budget {
                rejection_reasons.push(format!(
                    "candidate {} excluded by token budget",
                    c.candidate_id
                ));
                continue;
            }
            used_tokens += c.approx_tokens;
            selected.push(c.candidate_id.clone());
        }

        let duration_ms = started.elapsed().as_millis() as u64;
        Ok(RetrievalReceiptV1 {
            schema: "RetrievalReceiptV1".to_string(),
            query: query.clone(),
            candidates: candidates.into_iter().take(query.top_k * 3).collect(),
            selected_candidate_ids: selected,
            total_approx_tokens: used_tokens,
            rejection_reasons,
            search_duration_ms: duration_ms,
        })
    }

    fn prune_receipts_keep_last(
        &self,
        keep_last: usize,
    ) -> Result<StorePruneResultV1, ContextGovernorError> {
        let receipts = self.list_receipts()?;
        let to_remove = receipts.len().saturating_sub(keep_last);
        let mut removed_receipts = 0usize;
        let mut removed_items = 0usize;
        for info in receipts.iter().take(to_remove) {
            let items: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM exact_items WHERE receipt_id = ?1",
                params![&info.receipt_id],
                |row| row.get(0),
            )?;
            self.conn.execute(
                "DELETE FROM receipts WHERE receipt_id = ?1",
                params![&info.receipt_id],
            )?;
            removed_receipts += 1;
            removed_items += items as usize;
        }
        let kept = self.list_receipts()?.len();
        Ok(StorePruneResultV1 {
            schema: "StorePruneResultV1".to_string(),
            kept_receipts: kept,
            removed_receipts,
            removed_exact_items: removed_items,
            total_bytes: 0,
        })
    }

    fn status(&self) -> Result<StoreStatusV1, ContextGovernorError> {
        let receipt_count: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM receipts", [], |row| row.get(0))?;
        let exact_item_count: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM exact_items", [], |row| row.get(0))?;
        let total_bytes: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(compacted_json) + LENGTH(summary_loss_json)), 0) FROM receipts",
                [],
                |row| row.get(0),
            )?;
        let last_receipt_id: Option<String> = self
            .conn
            .query_row(
                "SELECT receipt_id FROM receipts ORDER BY stored_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(StoreStatusV1 {
            schema: "StoreStatusV1".to_string(),
            path: self.path.display().to_string(),
            receipt_count,
            exact_item_count,
            total_bytes: total_bytes as u64,
            searchable: true,
            last_receipt_id,
        })
    }
}

/// High-level rehydration: build a `CompactResponse` from a store by query.
pub fn context_rehydrate(
    store: &dyn ContextStore,
    query: &RehydrationQueryV1,
) -> Result<CompactResponse, ContextGovernorError> {
    let receipt = store.search(query)?;
    if receipt.selected_candidate_ids.is_empty() {
        return Err(ContextGovernorError::ReceiptNotFound(
            "no candidates matched rehydration query".to_string(),
        ));
    }
    // Group by receipt_id, load each, take selected items.
    let mut by_receipt: BTreeMap<String, Vec<&RetrievalCandidateV1>> = BTreeMap::new();
    for cand in &receipt.candidates {
        if receipt.selected_candidate_ids.contains(&cand.candidate_id) {
            by_receipt
                .entry(cand.receipt_id.clone())
                .or_default()
                .push(cand);
        }
    }

    let mut merged_exact_store: Vec<crate::ExactStoredItemV1> = Vec::new();
    let mut merged_compacted_messages: Vec<Message> = Vec::new();
    let mut merged_receipt_ids: Vec<String> = Vec::new();
    let mut merged_fallback_refs: Vec<ExactFallbackRefV1> = Vec::new();
    let mut derived_created_utc: Option<chrono::DateTime<chrono::Utc>> = None;

    for (_receipt_id, cands) in by_receipt {
        let mut loaded = store.load(&_receipt_id)?;
        derived_created_utc = Some(
            derived_created_utc.map_or(loaded.receipt.created_utc, |created| {
                created.max(loaded.receipt.created_utc)
            }),
        );
        merged_receipt_ids.push(_receipt_id);
        merged_compacted_messages.append(&mut loaded.compacted_messages);
        for cand in &cands {
            if let Some(item) = loaded
                .exact_store
                .iter()
                .find(|i| i.item_id == cand.exact_fallback_ref_id)
            {
                merged_exact_store.push(item.clone());
            }
        }
        merged_fallback_refs.extend(cands.iter().filter_map(|candidate| {
            loaded
                .receipt
                .exact_fallback_refs
                .iter()
                .find(|reference| reference.item_id == candidate.exact_fallback_ref_id)
                .cloned()
        }));
    }

    let merged_transcript_blake3 = crate::hash_messages(&merged_compacted_messages)?;
    let merged_transcript_sha256 = crate::hash_messages_sha256(&merged_compacted_messages)?;
    let derivation = serde_json::to_string(&serde_json::json!({
        "schema": "ContextRehydrationDerivationV1",
        "query": query,
        "selected_candidate_ids": receipt.selected_candidate_ids,
        "source_receipt_ids": merged_receipt_ids,
        "compacted_transcript_blake3": merged_transcript_blake3,
        "exact_items": merged_exact_store
            .iter()
            .map(|item| (&item.item_id, &item.content_blake3))
            .collect::<Vec<_>>(),
    }))?;
    let derivation_blake3 = crate::hash_text(&derivation);
    let derived_created_utc = derived_created_utc.ok_or_else(|| {
        ContextGovernorError::ReceiptNotFound(
            "selected rehydration candidates had no source receipt".to_string(),
        )
    })?;

    // A rehydrated derivative is content-addressed: the same governed selection
    // produces the same receipt/plan identity and source-derived timestamp.
    let merged_receipt = crate::ContextCompactionReceiptV1 {
        schema: "ContextCompactionReceiptV1".to_string(),
        receipt_id: format!("rehydrated-{derivation_blake3}"),
        session_id: "rehydrated".to_string(),
        parent_session_id: None,
        created_utc: derived_created_utc,
        engine: "context-governor".to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        original_message_count: merged_compacted_messages.len(),
        compacted_message_count: merged_compacted_messages.len(),
        original_approx_tokens: receipt.total_approx_tokens,
        compacted_approx_tokens: receipt.total_approx_tokens,
        token_savings_estimate: 0,
        token_counter: crate::TokenCounterKind::ApproxChars,
        original_transcript_blake3: merged_transcript_blake3.clone(),
        compacted_transcript_blake3: merged_transcript_blake3,
        original_transcript_sha256: merged_transcript_sha256.clone(),
        compacted_transcript_sha256: merged_transcript_sha256,
        allocation_plan_id: String::new(),
        semantic_memory_fact_ids: vec![],
        semantic_memory_document_ids: vec![],
        exact_fallback_refs: merged_fallback_refs,
        summary_loss_report: SummaryLossReportV1 {
            schema: "SummaryLossReportV1".to_string(),
            ..Default::default()
        },
        warnings: vec![
            "This is a rehydrated derivative; verify against original receipts.".to_string(),
        ],
        recovery_durability: RecoveryDurabilityV1::Persisted,
    };

    Ok(CompactResponse {
        receipt: merged_receipt,
        allocation_plan: crate::ContextAllocationPlanV1 {
            schema: "ContextAllocationPlanV1".to_string(),
            plan_id: format!("rehydrated-plan-{derivation_blake3}"),
            session_id: "rehydrated".to_string(),
            created_utc: derived_created_utc,
            context_budget_tokens: receipt.total_approx_tokens,
            target_output_tokens: 0,
            allocator: "rehydrated".to_string(),
            items: vec![],
            kept_item_ids: vec![],
            summarized_item_ids: vec![],
            archived_item_ids: vec![],
            omitted_item_ids: vec![],
            quarantined_item_ids: vec![],
            selection_evidence: BTreeMap::new(),
            hot_path_operation_counts: crate::HotPathOperationCountsV1::default(),
        },
        compacted_messages: merged_compacted_messages,
        exact_store: merged_exact_store,
        context_steps: vec![],
        plan_state: crate::PlanStateV1::default(),
        structural_floor: crate::StructuralFloorV1::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllocatorMode, CompactRequest, CompactionPolicy, Message};

    fn make_request(target_tokens: usize) -> CompactRequest {
        CompactRequest {
            session_id: "test-session".to_string(),
            messages: vec![
                Message {
                    id: None,
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
                Message {
                    id: None,
                    role: "user".to_string(),
                    content: "What is the capital of France?".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
                Message {
                    id: None,
                    role: "assistant".to_string(),
                    content: "The capital of France is Paris.".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
            ],
            policy: CompactionPolicy {
                target_tokens,
                allocator: AllocatorMode::UtilityV2.as_str().to_string(),
                budget_mode: crate::BudgetMode::SoftWarn,
                ..Default::default()
            },
            focus: Some("France capital".to_string()),
        }
    }

    #[test]
    fn sqlite_store_save_load_roundtrip() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let request = make_request(60);
        let response = crate::compact_context(request).unwrap();
        let receipt_id = response.receipt.receipt_id.clone();
        let save = store.save(&response).unwrap();
        assert_eq!(save.receipt_id, receipt_id);
        assert!(save.verified);
        let loaded = store.load(&receipt_id).unwrap();
        assert_eq!(loaded.receipt.receipt_id, receipt_id);
        assert_eq!(
            loaded.receipt.recovery_durability,
            RecoveryDurabilityV1::Persisted
        );
    }

    #[test]
    fn sqlite_store_lists_receipts() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let request = make_request(60);
        let response = crate::compact_context(request).unwrap();
        store.save(&response).unwrap();
        let list = store.list_receipts().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].receipt_id, response.receipt.receipt_id);
    }

    #[test]
    fn sqlite_store_prune_keeps_last() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let mut ids = Vec::new();
        for i in 0..3 {
            let mut request = make_request(60);
            request.messages[1].content = format!("Question {i}");
            let response = crate::compact_context(request).unwrap();
            ids.push(response.receipt.receipt_id.clone());
            store.save(&response).unwrap();
        }
        let prune = store.prune_receipts_keep_last(1).unwrap();
        assert_eq!(prune.removed_receipts, 2);
        let list = store.list_receipts().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn sqlite_store_search_by_keyword() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let mut response = crate::compact_context(make_request(60)).unwrap();
        // Force an exact-store item so keyword search has indexed content.
        response.exact_store.push(crate::ExactStoredItemV1 {
            item_id: "exact-paris".to_string(),
            source_indices: vec![2],
            content: "The capital of France is Paris.".to_string(),
            content_blake3: blake3::hash("The capital of France is Paris.".as_bytes())
                .to_hex()
                .to_string(),
        });
        response
            .receipt
            .exact_fallback_refs
            .push(crate::ExactFallbackRefV1 {
                item_id: "exact-paris".to_string(),
                start_index: 2,
                end_index: 3,
                content_blake3: response.exact_store[0].content_blake3.clone(),
                content_sha256: crate::hash_text_sha256(&response.exact_store[0].content),
                approx_tokens: 6,
            });
        let receipt_id = response.receipt.receipt_id.clone();
        store.save(&response).unwrap();
        let query = RehydrationQueryV1 {
            schema: "RehydrationQueryV1".to_string(),
            receipt_id: Some(receipt_id),
            token_budget: 100,
            authority_floor: None,
            lineage: vec![],
            keywords: vec!["Paris".to_string()],
            top_k: 5,
        };
        let result = store.search(&query).unwrap();
        assert!(
            !result.candidates.is_empty(),
            "expected at least one candidate from exact store"
        );
    }

    #[test]
    fn context_rehydrate_returns_derivative() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let mut response = crate::compact_context(make_request(60)).unwrap();
        // Force an exact-store item so rehydration has indexed content.
        response.exact_store.push(crate::ExactStoredItemV1 {
            item_id: "exact-paris".to_string(),
            source_indices: vec![2],
            content: "The capital of France is Paris.".to_string(),
            content_blake3: blake3::hash("The capital of France is Paris.".as_bytes())
                .to_hex()
                .to_string(),
        });
        response
            .receipt
            .exact_fallback_refs
            .push(crate::ExactFallbackRefV1 {
                item_id: "exact-paris".to_string(),
                start_index: 2,
                end_index: 3,
                content_blake3: response.exact_store[0].content_blake3.clone(),
                content_sha256: crate::hash_text_sha256(&response.exact_store[0].content),
                approx_tokens: 6,
            });
        let receipt_id = response.receipt.receipt_id.clone();
        store.save(&response).unwrap();
        let query = RehydrationQueryV1 {
            schema: "RehydrationQueryV1".to_string(),
            receipt_id: Some(receipt_id),
            token_budget: 200,
            authority_floor: None,
            lineage: vec![],
            keywords: vec!["Paris".to_string()],
            top_k: 10,
        };
        let rehydrated = context_rehydrate(&store, &query).unwrap();
        assert!(rehydrated.receipt.receipt_id.starts_with("rehydrated-"));
        assert!(!rehydrated.exact_store.is_empty());
        assert!(!rehydrated.compacted_messages.is_empty());
        assert!(!rehydrated.receipt.exact_fallback_refs.is_empty());
        assert_eq!(
            rehydrated.receipt.compacted_transcript_blake3,
            crate::hash_messages(&rehydrated.compacted_messages).unwrap()
        );
        assert_eq!(
            rehydrated.receipt.compacted_transcript_sha256,
            crate::hash_messages_sha256(&rehydrated.compacted_messages).unwrap()
        );
    }

    #[test]
    fn sqlite_store_preserves_identical_exact_content_for_distinct_receipts() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let shared_content = "shared exact evidence";
        let mut receipt_ids = Vec::new();
        for suffix in ["first", "second"] {
            let mut request = make_request(60);
            request.session_id = format!("duplicate-content-{suffix}");
            let mut response = crate::compact_context(request).unwrap();
            let item_id = format!("shared-{suffix}");
            response.exact_store.push(crate::ExactStoredItemV1 {
                item_id: item_id.clone(),
                source_indices: vec![1],
                content: shared_content.to_string(),
                content_blake3: crate::hash_text(shared_content),
            });
            response
                .receipt
                .exact_fallback_refs
                .push(crate::ExactFallbackRefV1 {
                    item_id,
                    start_index: 1,
                    end_index: 2,
                    content_blake3: crate::hash_text(shared_content),
                    content_sha256: crate::hash_text_sha256(shared_content),
                    approx_tokens: 5,
                });
            receipt_ids.push(response.receipt.receipt_id.clone());
            store.save(&response).unwrap();
        }

        for receipt_id in receipt_ids {
            let query = RehydrationQueryV1 {
                schema: "RehydrationQueryV1".to_string(),
                receipt_id: Some(receipt_id.clone()),
                token_budget: 100,
                authority_floor: None,
                lineage: vec![],
                keywords: vec!["shared".to_string()],
                top_k: 10,
            };
            let retrieval = store.search(&query).unwrap();
            assert!(
                retrieval
                    .candidates
                    .iter()
                    .any(|candidate| candidate.receipt_id == receipt_id),
                "duplicate content must remain owned by every receipt"
            );
        }
    }

    #[test]
    fn context_rehydrate_is_deterministic_for_the_same_selection() {
        let store = SqliteContextStore::open_in_memory().unwrap();
        let mut response = crate::compact_context(make_request(60)).unwrap();
        response.exact_store.push(crate::ExactStoredItemV1 {
            item_id: "deterministic-paris".to_string(),
            source_indices: vec![2],
            content: "Paris is deterministic evidence.".to_string(),
            content_blake3: crate::hash_text("Paris is deterministic evidence."),
        });
        response
            .receipt
            .exact_fallback_refs
            .push(crate::ExactFallbackRefV1 {
                item_id: "deterministic-paris".to_string(),
                start_index: 2,
                end_index: 3,
                content_blake3: crate::hash_text("Paris is deterministic evidence."),
                content_sha256: crate::hash_text_sha256("Paris is deterministic evidence."),
                approx_tokens: 8,
            });
        let receipt_id = response.receipt.receipt_id.clone();
        store.save(&response).unwrap();
        let query = RehydrationQueryV1 {
            schema: "RehydrationQueryV1".to_string(),
            receipt_id: Some(receipt_id),
            token_budget: 100,
            authority_floor: None,
            lineage: vec![],
            keywords: vec!["deterministic".to_string()],
            top_k: 10,
        };

        let first = context_rehydrate(&store, &query).unwrap();
        let second = context_rehydrate(&store, &query).unwrap();
        assert_eq!(first.receipt.receipt_id, second.receipt.receipt_id);
        assert_eq!(first.receipt.created_utc, second.receipt.created_utc);
        assert_eq!(
            first.allocation_plan.plan_id,
            second.allocation_plan.plan_id
        );
        assert_eq!(
            first.allocation_plan.created_utc,
            second.allocation_plan.created_utc
        );
        assert_eq!(
            first.receipt.exact_fallback_refs.len(),
            first.exact_store.len()
        );
    }
}

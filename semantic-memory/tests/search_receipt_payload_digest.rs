//! Canonical receipt raw-payload consistency and replay refusal regressions.
use semantic_memory::{
    AuthorityPermit, AuthorityScopeV1, AuthorityScopesV1, ElevationRequirementV1, Embedder,
    ExactnessProfile, MemoryConfig, MemoryError, MemoryStore, MockEmbedder, OriginAuthorityLabelV1,
    OriginClassV1, OriginRiskV1, ReceiptMode, ReplayMode, RevocationStatusV1, SearchContext,
    SearchSourceType,
};
use serde_json::{json, Value};
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tempfile::TempDir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type EmbeddingFuture<'a, T> =
    Pin<Box<dyn Future<Output = std::result::Result<T, MemoryError>> + Send + 'a>>;
const ID: &str = "receipt:payload-consistency-regression";
const QUERY: &str = "Receipt raw bytes naïve 東京 sentinel";
struct Counted {
    calls: Arc<AtomicUsize>,
    inner: MockEmbedder,
}
impl Embedder for Counted {
    fn embed<'a>(&'a self, text: &'a str) -> EmbeddingFuture<'a, Vec<f32>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed(text)
    }
    fn embed_batch<'a>(&'a self, texts: Vec<String>) -> EmbeddingFuture<'a, Vec<Vec<f32>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed_batch(texts)
    }
    fn dimensions(&self) -> usize {
        768
    }
    fn model_name(&self) -> &str {
        self.inner.model_name()
    }
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap()
}
fn digest(raw: &str) -> String {
    format!("blake3:{}", blake3::hash(raw.as_bytes()).to_hex())
}
fn connection(dir: &Path) -> rusqlite::Result<rusqlite::Connection> {
    rusqlite::Connection::open(dir.join("memory.db"))
}
fn raw_row(dir: &Path) -> Result<(String, String)> {
    Ok(connection(dir)?.query_row(
        "SELECT receipt_json,receipt_digest FROM search_receipts WHERE receipt_id=?1",
        [ID],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}
fn row_count(dir: &Path) -> Result<i64> {
    Ok(connection(dir)?.query_row("SELECT count(*) FROM search_receipts", [], |r| r.get(0))?)
}
fn report(name: &str, data: &Value) -> Result<()> {
    if let Ok(root) = std::env::var("MEM_RECEIPT_REPORT_ROOT") {
        let root = Path::new(&root);
        std::fs::create_dir_all(root)?;
        let p = root.join(format!("{name}.json"));
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p)?;
        f.write_all(serde_json::to_string_pretty(data)?.as_bytes())?;
    }
    println!("MEM_RECEIPT_REPORT {}", serde_json::to_string(data)?);
    Ok(())
}
async fn fixture() -> Result<(MemoryStore, TempDir, Arc<AtomicUsize>)> {
    let dir = TempDir::new()?;
    let calls = Arc::new(AtomicUsize::new(0));
    let store = MemoryStore::open_with_embedder(
        MemoryConfig {
            base_dir: dir.path().into(),
            ..Default::default()
        },
        Box::new(Counted {
            calls: calls.clone(),
            inner: MockEmbedder::new(768),
        }),
    )?;
    let label = OriginAuthorityLabelV1::new(
        OriginClassV1::ExternalEvidence,
        "principal:alice",
        "receipt-regression",
        format!("blake3:{}", "a".repeat(64)),
        OriginRiskV1::Low,
        AuthorityScopesV1 {
            recall: AuthorityScopeV1::Audience,
            assertion: AuthorityScopeV1::Denied,
            action: AuthorityScopeV1::Denied,
        },
        ElevationRequirementV1::ExplicitOperatorApproval,
        None,
        RevocationStatusV1::Active,
        vec!["principal:alice".into()],
    )?;
    let issuer =
        semantic_memory::AuthorityIssuer::from_operator_token("receipt-regression-fixture-only")
            .ok_or("fixture issuer")?;
    let evidence = semantic_memory::authority_contracts::ResolvedEvidenceDigest::from_resolver(
        format!("blake3:{}", "b".repeat(64)),
    )
    .ok_or("fixture evidence")?;
    let permit = issuer.mint_with_resolved_evidence(
        "principal:alice",
        "receipt-regression",
        AuthorityPermit::APPEND_CAPABILITY,
        vec![evidence],
        label,
    );
    store
        .authority()
        .append(
            permit,
            "receipt-regression-seed".into(),
            "general".into(),
            QUERY.into(),
            None,
        )
        .await?;
    let mut context = SearchContext::default_now();
    context.request_id = Some(ID.into());
    context.trace_id = Some("trace:original:東京".into());
    context.receipt_mode = ReceiptMode::ReturnReceipt;
    context.replay_mode = ReplayMode::StoreInputs;
    context.exactness_profile = ExactnessProfile::PreferExact;
    let response = store
        .search_with_context(
            QUERY,
            Some(10),
            Some(&["general"]),
            Some(&[SearchSourceType::Facts]),
            context,
        )
        .await?;
    assert_eq!(response.results.len(), 1);
    let (raw, stored) = raw_row(dir.path())?;
    assert_eq!(digest(&raw), stored);
    Ok((store, dir, calls))
}
fn error_outcome(error: MemoryError) -> Value {
    let typed = matches!(&error, MemoryError::CorruptData { table, row_id, .. } if *table == "search_receipts" && row_id == ID);
    json!({"accepted": false, "kind": error.kind(), "typed_corrupt_data": typed})
}
async fn probe(store: &MemoryStore, dir: &Path, calls: &AtomicUsize) -> Result<Value> {
    let before_calls = calls.load(Ordering::SeqCst);
    let before_rows = row_count(dir)?;
    let before_raw = raw_row(dir)?;
    let lookup = match store.get_search_receipt(ID).await {
        Ok(Some(r)) => {
            json!({"accepted": true, "trace_id": r.trace_id, "receipt_digest": r.receipt_digest})
        }
        Ok(None) => return Err("receipt unexpectedly missing".into()),
        Err(e) => error_outcome(e),
    };
    let stored_replay = match store.replay_search_from_stored_inputs(ID).await {
        Ok(r) => {
            json!({"accepted": true, "trace_id": r.original_receipt.trace_id, "replay_trace_id": r.replay_receipt.trace_id})
        }
        Err(e) => error_outcome(e),
    };
    let manual_replay = match store
        .replay_search_receipt(
            ID,
            QUERY,
            Some(10),
            Some(&["general"]),
            Some(&[SearchSourceType::Facts]),
        )
        .await
    {
        Ok(r) => {
            json!({"accepted": true, "trace_id": r.original_receipt.trace_id, "replay_trace_id": r.replay_receipt.trace_id})
        }
        Err(e) => error_outcome(e),
    };
    let after_rows = row_count(dir)?;
    Ok(
        json!({"lookup": lookup, "stored_replay": stored_replay, "manual_replay": manual_replay,
        "embedder_call_delta": calls.load(Ordering::SeqCst)-before_calls, "receipt_row_delta": after_rows-before_rows,
        "original_raw_row_unchanged_by_probe": raw_row(dir)? == before_raw}),
    )
}
fn assert_refused(observed: &Value) {
    for path in ["lookup", "stored_replay", "manual_replay"] {
        assert_eq!(
            observed[path]["typed_corrupt_data"], true,
            "{path} accepted or returned the wrong error: {observed}"
        );
    }
    assert_eq!(observed["embedder_call_delta"], 0);
    assert_eq!(observed["receipt_row_delta"], 0);
    assert_eq!(observed["original_raw_row_unchanged_by_probe"], true);
}

#[test]
fn valid_tamper_lookup_and_both_replays_are_refused() -> Result<()> {
    runtime().block_on(async {
        let (store, dir, calls) = fixture().await?;
        let (raw, old_digest) = raw_row(dir.path())?;
        let mut value: Value = serde_json::from_str(&raw)?; value["trace_id"] = json!("trace:tampered:東京");
        let tampered = serde_json::to_string(&value)?; assert_ne!(digest(&tampered), old_digest);
        connection(dir.path())?.execute("UPDATE search_receipts SET receipt_json=?1 WHERE receipt_id=?2", rusqlite::params![tampered, ID])?;
        let observed = probe(&store, dir.path(), &calls).await?;
        report("valid-tamper", &json!({"case": "valid-trace-tamper", "raw_digest_changed": true, "old_digest": old_digest,
            "altered_raw_digest": digest(&tampered), "observed": observed}))?;
        // Collect all three outcomes before asserting, so the original-owner red run proves both replay paths.
        assert_refused(&observed); Ok(())
    })
}

#[test]
fn malformed_empty_digest_json_and_schema_controls() -> Result<()> {
    runtime().block_on(async {
        let mut rows = Vec::new();
        for case in ["empty-digest", "malformed-digest", "digest-only", "malformed-json-stale", "malformed-json-matching", "empty-json-matching", "outer-schema", "inner-schema-matching"] {
            let (store, dir, calls) = fixture().await?; let (raw, stored) = raw_row(dir.path())?;
            let mut payload = raw.clone(); let mut column = stored.clone(); let mut schema = "vector_search_receipt_v1".to_string();
            match case {
                "empty-digest" => column.clear(), "malformed-digest" => column = "blake3:not-a-digest".into(),
                "digest-only" => column = format!("blake3:{}", "0".repeat(64)),
                "malformed-json-stale" => payload = "{malformed".into(),
                "malformed-json-matching" => { payload = "{malformed".into(); column = digest(&payload); },
                "empty-json-matching" => { payload.clear(); column = digest(&payload); },
                "outer-schema" => schema = "unsupported-task-schema".into(),
                "inner-schema-matching" => { let mut v: Value = serde_json::from_str(&payload)?; v["schema_version"] = json!("unsupported-task-schema"); payload = serde_json::to_string(&v)?; column = digest(&payload); },
                _ => unreachable!(),
            }
            connection(dir.path())?.execute("UPDATE search_receipts SET receipt_json=?1,receipt_digest=?2,schema_version=?3 WHERE receipt_id=?4", rusqlite::params![payload, column, schema, ID])?;
            let observed = probe(&store, dir.path(), &calls).await?; assert_refused(&observed);
            rows.push(json!({"case": case, "observed": observed}));
            drop(store); drop(dir);
        }
        report("controls", &json!({"controls": rows}))?; Ok(())
    })
}

#[test]
fn raw_legacy_whitespace_order_and_default_fields_remain_compatible() -> Result<()> {
    runtime().block_on(async {
        let (store, dir, _) = fixture().await?; let original = store.get_search_receipt(ID).await?.ok_or("missing receipt")?;
        let (raw, _) = raw_row(dir.path())?; let mut v: Value = serde_json::from_str(&raw)?;
        for key in ["schema_version", "receipt_digest", "budget_id"] { v.as_object_mut().ok_or("object required")?.remove(key); }
        let legacy_raw = format!(" \n{}\n ", serde_json::to_string_pretty(&v)?); let hash = digest(&legacy_raw);
        assert_ne!(legacy_raw, raw);
        connection(dir.path())?.execute("UPDATE search_receipts SET receipt_json=?1,receipt_digest=?2 WHERE receipt_id=?3", rusqlite::params![legacy_raw, hash, ID])?;
        let loaded = store.get_search_receipt(ID).await?.ok_or("legacy receipt absent")?;
        assert_eq!(loaded.result_ids, original.result_ids); assert_eq!(loaded.evaluation_time, original.evaluation_time);
        assert_eq!(loaded.trace_id, original.trace_id); assert_eq!(loaded.receipt_digest.as_deref(), Some(hash.as_str()));
        let replay = store.replay_search_from_stored_inputs(ID).await?;
        assert!(replay.query_embedding_digest_matches && replay.result_ids_match);
        assert_eq!(raw_row(dir.path())?, (legacy_raw, hash));
        report("legacy", &json!({"raw_order_whitespace_retained": true, "omitted_default_fields_supported": true, "stored_replay_passed": true}))?; Ok(())
    })
}

#[test]
fn missing_digest_contract_and_missing_receipt_are_preserved() -> Result<()> {
    runtime().block_on(async {
        let (store, dir, calls) = fixture().await?; let original = raw_row(dir.path())?;
        let null = connection(dir.path())?.execute("UPDATE search_receipts SET receipt_digest=NULL WHERE receipt_id=?1", [ID]).unwrap_err();
        assert!(matches!(null, rusqlite::Error::SqliteFailure(ref e, _) if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_NOTNULL));
        assert_eq!(raw_row(dir.path())?, original); assert!(store.get_search_receipt(ID).await?.is_some());
        let before = calls.load(Ordering::SeqCst); let missing = "receipt:definitely-missing";
        assert!(store.get_search_receipt(missing).await?.is_none());
        assert!(!store.search_replay_inputs_available(missing).await?);
        for e in [store.replay_search_from_stored_inputs(missing).await.unwrap_err(), store.replay_search_receipt(missing, QUERY, None, None, None).await.unwrap_err()] {
            assert!(matches!(e, MemoryError::SearchReceiptNotFound { ref receipt_id } if receipt_id == missing));
        }
        assert_eq!(calls.load(Ordering::SeqCst), before);
        report("missing", &json!({"null_digest_refused_by_existing_not_null_constraint": true, "source_receipt_unchanged": true, "missing_row_contract_preserved": true}))?; Ok(())
    })
}

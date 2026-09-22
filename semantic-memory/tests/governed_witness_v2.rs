//! Exact owner-emitted bytes, not authentication or production embedder proof.
use semantic_memory::{
    AuthorityPermit, AuthorityScopeV1, AuthorityScopesV1, ElevationRequirementV1, Embedder,
    GovernedAccessPurposeV1, GovernedAccessRequestV1, GovernedWitnessedSearchErrorV2, MemoryConfig,
    MemoryError, MemoryStore, MockEmbedder, OriginAuthorityLabelV1, OriginClassV1, OriginRiskV1,
    RevocationStatusV1, StateView,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type EmbeddingFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, MemoryError>> + Send + 'a>>;

struct CountingEmbedder {
    calls: Arc<AtomicUsize>,
    inner: MockEmbedder,
}
impl Embedder for CountingEmbedder {
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

type StoreFixture = (MemoryStore, TempDir, Arc<AtomicUsize>);

fn store() -> Result<StoreFixture, Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let calls = Arc::new(AtomicUsize::new(0));
    let store = MemoryStore::open_with_embedder(
        MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        },
        Box::new(CountingEmbedder {
            calls: calls.clone(),
            inner: MockEmbedder::new(768),
        }),
    )?;
    Ok((store, dir, calls))
}
fn request(namespace: &str) -> GovernedAccessRequestV1 {
    GovernedAccessRequestV1::new(
        "principal:alice",
        "principal:alice",
        GovernedAccessPurposeV1::Recall,
        namespace,
    )
}
async fn seed(store: &MemoryStore) -> TestResult {
    let label = OriginAuthorityLabelV1::new(
        OriginClassV1::ExternalEvidence,
        "principal:alice",
        "v2-test",
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
    let permit = AuthorityPermit::with_evidence(
        "principal:alice",
        "v2-test",
        AuthorityPermit::APPEND_CAPABILITY,
        vec![format!("blake3:{}", "b".repeat(64))],
    )
    .with_origin(label);
    store
        .authority()
        .append(
            permit,
            "v2-seed".into(),
            "general".into(),
            "V2 naïve 東京 🦀 sentinel".into(),
            None,
        )
        .await?;
    Ok(())
}

#[tokio::test]
async fn populated_and_empty_bind_complete_request_and_exact_utf8_bytes() -> TestResult {
    let (store, _dir, _) = store()?;
    seed(&store).await?;
    for namespace in ["general", "empty"] {
        let access = request(namespace);
        let query = "V2 naïve 東京 🦀 sentinel\n\"quoted\" \\ slash";
        let id = format!("request:{namespace}");
        let emitted = store
            .authority()
            .search_governed_witnessed_v2(id.clone(), query, 10, access.clone())
            .await?;
        assert_eq!(
            emitted.schema_version,
            "governed_witnessed_search_response_v2"
        );
        assert_eq!(
            emitted.payload_sha256,
            format!(
                "sha256:{:x}",
                Sha256::digest(emitted.payload_json.as_bytes())
            )
        );
        assert!(emitted.payload_json.contains("東京"));
        let payload: Value = serde_json::from_str(&emitted.payload_json)?;
        assert_eq!(
            payload["schema_version"],
            "governed_witnessed_search_payload_v2"
        );
        assert_eq!(
            payload["request"],
            json!({"request_id":id, "query":query, "top_k":10, "access_request":access})
        );
        let response: semantic_memory::GovernedWitnessedSearchResponseV1 =
            serde_json::from_value(payload["response"].clone())?;
        assert_eq!(response.state_view, StateView::Current);
        assert_eq!(
            response.response.results.len(),
            usize::from(namespace == "general")
        );
        assert_eq!(
            response.authority_state,
            store.authority().current_state().await?
        );
        assert_eq!(response.retrieval_witness.request_id, id);
        assert_eq!(
            response.retrieval_witness.authority_snapshot_id,
            response.authority_state.snapshot_id
        );
        for (result, digest) in response
            .response
            .results
            .iter()
            .zip(&response.retrieval_witness.ordered_result_digests)
        {
            let result_id = result.source.result_id();
            let decision = response
                .response
                .decisions
                .iter()
                .find(|d| d.allowed && format!("fact:{}", d.fact_id) == result_id)
                .ok_or("missing decision")?;
            let bytes =
                serde_json::to_vec(&json!({"result":result,"authority_decision":decision}))?;
            assert_eq!(*digest, blake3::hash(&bytes).to_hex().to_string());
        }
        let altered = emitted.payload_json.replace("sentinel", "SUBSTITUTED");
        assert_ne!(emitted.payload_json, altered);
        assert_ne!(
            emitted.payload_sha256,
            format!("sha256:{:x}", Sha256::digest(altered.as_bytes()))
        );
    }
    Ok(())
}

#[tokio::test]
async fn full_request_mutations_fail_before_embedding_even_without_candidates() -> TestResult {
    let (store, _dir, calls) = store()?;
    for populated in [false, true] {
        if populated {
            seed(&store).await?;
        }
        let valid = serde_json::to_value(request("general"))?;
        for (pointer, replacement) in [
            ("/principal", json!("wrong")),
            ("/audience", json!("wrong")),
            ("/namespace", json!("wrong")),
            ("/caller", json!("wrong")),
            ("/subject", json!("wrong")),
            ("/audiences", json!(["wrong"])),
            ("/scope/namespace", json!("wrong")),
            ("/scope/domain", json!("wrong")),
            ("/scope/workspace_id", json!("wrong")),
            ("/scope/repo_id", json!("wrong")),
            ("/policy_version", json!("wrong")),
            ("/policy_digest", json!("wrong")),
            ("/purpose", json!("action")),
        ] {
            let mut mutated = valid.clone();
            *mutated
                .pointer_mut(pointer)
                .ok_or("missing mutation field")? = replacement;
            let before = calls.load(Ordering::SeqCst);
            let error = store
                .authority()
                .search_governed_witnessed_v2(
                    "request:mutant".into(),
                    "sentinel",
                    10,
                    serde_json::from_value(mutated)?,
                )
                .await;
            assert!(
                matches!(
                    error,
                    Err(GovernedWitnessedSearchErrorV2::InvalidAccessRequest
                        | GovernedWitnessedSearchErrorV2::UnsupportedPurpose)
                ),
                "{pointer}, populated={populated}: {error:?}"
            );
            assert_eq!(
                before,
                calls.load(Ordering::SeqCst),
                "retrieval ran for {pointer}"
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn invalid_request_id_zero_limit_and_non_recall_are_refused_before_retrieval() -> TestResult {
    let (store, _dir, calls) = store()?;
    for id in ["", " \t\n"] {
        assert!(matches!(
            store
                .authority()
                .search_governed_witnessed_v2(id.into(), "query", 1, request("general"))
                .await,
            Err(GovernedWitnessedSearchErrorV2::InvalidRequestId)
        ));
    }
    assert!(matches!(
        store
            .authority()
            .search_governed_witnessed_v2("r".into(), "query", 0, request("general"))
            .await,
        Err(GovernedWitnessedSearchErrorV2::InvalidLimit)
    ));
    for purpose in [
        GovernedAccessPurposeV1::Assertion,
        GovernedAccessPurposeV1::Action,
        GovernedAccessPurposeV1::Export,
        GovernedAccessPurposeV1::Replay,
        GovernedAccessPurposeV1::Admin,
    ] {
        assert!(matches!(
            store
                .authority()
                .search_governed_witnessed_v2(
                    "r".into(),
                    "query",
                    1,
                    request("general").with_purpose(purpose)
                )
                .await,
            Err(GovernedWitnessedSearchErrorV2::UnsupportedPurpose)
        ));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn structurally_invalid_requests_with_owner_generated_digests_are_refused() -> TestResult {
    let (store, _dir, calls) = store()?;
    for access in [
        request(" "),
        GovernedAccessRequestV1::new(
            "",
            "principal:alice",
            GovernedAccessPurposeV1::Recall,
            "general",
        ),
        request("general").with_audiences(vec![]),
        GovernedAccessRequestV1::for_principals(
            semantic_memory::CallerPrincipalV1("principal:alice".into()),
            semantic_memory::SubjectPrincipalV1(" ".into()),
            vec!["principal:alice".into()],
            GovernedAccessPurposeV1::Recall,
            semantic_memory::NamespaceScopeV1::exact("general"),
        ),
    ] {
        assert!(matches!(
            store
                .authority()
                .search_governed_witnessed_v2("r".into(), "query", 1, access)
                .await,
            Err(GovernedWitnessedSearchErrorV2::InvalidAccessRequest)
        ));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn denied_content_stays_denied_and_v1_remains_explicitly_available() -> TestResult {
    let (store, _dir, _) = store()?;
    seed(&store).await?;
    let access = GovernedAccessRequestV1::new(
        "principal:bob",
        "principal:bob",
        GovernedAccessPurposeV1::Recall,
        "general",
    );
    let v2 = store
        .authority()
        .search_governed_witnessed_v2(
            "v2-denied".into(),
            "V2 naïve 東京 🦀 sentinel",
            10,
            access.clone(),
        )
        .await?;
    let payload: Value = serde_json::from_str(&v2.payload_json)?;
    assert_eq!(payload["response"]["response"]["results"], json!([]));
    assert!(!payload["response"]["response"]["decisions"]
        .as_array()
        .ok_or("decisions")?
        .is_empty());
    let v1 = store
        .authority()
        .search_governed_witnessed("v1".into(), "sentinel", Some(10), access)
        .await?;
    assert_eq!(v1.schema_version, "governed_witnessed_search_response_v1");
    Ok(())
}

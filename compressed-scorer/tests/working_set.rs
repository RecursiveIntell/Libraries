use compressed_scorer::{
    CacheRuntimePolicy, CompressedPage, CompressedScorer, CompressedWorkingSet, PageRole,
    PreparedQuery, ScorerResult,
};

struct ToyPrepared {
    dim: usize,
}
impl PreparedQuery for ToyPrepared {
    fn dim(&self) -> usize {
        self.dim
    }
}
struct ToyScorer;
impl CompressedScorer for ToyScorer {
    type Prepared = ToyPrepared;
    type Compressed = f32;
    fn prepare_query(&self, query: &[f32]) -> ScorerResult<Self::Prepared> {
        Ok(ToyPrepared { dim: query.len() })
    }
    fn score_prepared(
        &self,
        _prepared: &Self::Prepared,
        compressed: &Self::Compressed,
    ) -> ScorerResult<f32> {
        Ok(*compressed)
    }
    fn decode(&self, compressed: &Self::Compressed) -> ScorerResult<Vec<f32>> {
        Ok(vec![*compressed])
    }
    fn dim(&self) -> usize {
        1
    }
    fn codec_name(&self) -> &'static str {
        "toy"
    }
    fn internal_bytes(&self) -> usize {
        0
    }
}

#[test]
fn working_set_keeps_guards_and_reports_receipt() {
    let mut ws = CompressedWorkingSet::new(
        ToyScorer,
        CacheRuntimePolicy {
            top_k: 2,
            ..Default::default()
        },
    );
    ws.push_page(CompressedPage {
        page_id: "low_guard".into(),
        layer: 0,
        head: 0,
        token_start: 0,
        token_end: 1,
        role: PageRole::SinkGuard,
        codec_profile_digest: "toy".into(),
        payload: -10.0,
        exact_shadow_digest: None,
    });
    ws.push_page(CompressedPage {
        page_id: "high".into(),
        layer: 0,
        head: 0,
        token_start: 1,
        token_end: 2,
        role: PageRole::SharedCold,
        codec_profile_digest: "toy".into(),
        payload: 1.0,
        exact_shadow_digest: None,
    });
    let sel = ws.select(&[1.0]).unwrap();
    assert_eq!(
        sel.receipt.schema_version,
        "compressed_working_set_selection_v1"
    );
    assert_eq!(sel.receipt.guard_count, 1);
    assert!(sel.receipt.refined_count >= sel.candidates.len());
    assert_eq!(sel.candidates[0].idx, 0);
}

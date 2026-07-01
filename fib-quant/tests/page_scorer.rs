use fib_quant::{FibQuantProfileV1, FibQuantizer, FibScorer};

#[test]
fn page_scorer_matches_prepared_scalar_scores() {
    let mut profile = FibQuantProfileV1::paper_default(16, 4, 32, 42).unwrap();
    profile.training_samples = 128;
    profile.lloyd_restarts = 1;
    profile.lloyd_iterations = 2;
    let quantizer = FibQuantizer::new(profile).unwrap();
    let scorer = FibScorer::new(quantizer).unwrap();
    let vectors: Vec<Vec<f32>> = (0..10)
        .map(|i| (0..16).map(|j| ((i + j + 1) as f32).sin()).collect())
        .collect();
    let codes = vectors
        .iter()
        .map(|v| scorer.quantizer().encode(v).unwrap())
        .collect::<Vec<_>>();
    let query = (0..16).map(|j| ((j + 3) as f32).cos()).collect::<Vec<_>>();
    let prepared = scorer.prepare_query(&query).unwrap();

    let scalar = scorer.score_batch_prepared(&prepared, &codes).unwrap();
    let page = scorer
        .score_batch_prepared_pages(&prepared, &codes)
        .unwrap();
    assert_eq!(scalar.len(), page.len());
    for (a, b) in scalar.iter().zip(page.iter()) {
        assert_eq!(a.idx, b.idx);
        assert!(
            (a.score - b.score).abs() < 1.0e-6,
            "{} vs {}",
            a.score,
            b.score
        );
    }
}

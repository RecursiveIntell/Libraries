use gpu_backend::{score_fib_gram_pages_cpu, topk_indices_desc, FibGramPageScoreInput};

#[test]
fn cpu_page_scorer_matches_scalar_reference() {
    let query_indices = vec![0, 1, 2];
    let stored_indices = vec![
        0, 1, 2, // candidate 0
        1, 1, 1, // candidate 1
        2, 2, 2, // candidate 2
        3, 0, 1, // candidate 3
    ];
    let stored_norms = vec![2.0, 1.0, 0.5, 3.0];
    let n_codewords = 4;
    let gram = vec![
        1.0, 0.1, 0.2, 0.3, 0.1, 1.0, 0.4, 0.5, 0.2, 0.4, 1.0, 0.6, 0.3, 0.5, 0.6, 1.0,
    ];
    let input = FibGramPageScoreInput {
        query_indices: &query_indices,
        stored_indices: &stored_indices,
        stored_norms: &stored_norms,
        gram: &gram,
        query_norm: 1.5,
        n_candidates: 4,
        block_count: 3,
        n_codewords,
    };

    let scores = score_fib_gram_pages_cpu(input).unwrap();
    let mut expected = Vec::new();
    for candidate in 0..4 {
        let mut sum = 0.0f32;
        for block in 0..3 {
            let qi = query_indices[block] as usize;
            let si = stored_indices[candidate * 3 + block] as usize;
            sum += gram[qi * n_codewords + si];
        }
        expected.push(sum * 1.5 * stored_norms[candidate]);
    }
    assert_eq!(scores, expected);
}

#[test]
fn topk_indices_are_sorted_and_stable_for_ties() {
    let scores = vec![1.0, 3.0, 3.0, -1.0, 2.0];
    let top = topk_indices_desc(&scores, 3);
    assert_eq!(top, vec![1, 2, 4]);
}

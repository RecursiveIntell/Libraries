use poly_kv::{AttentionType, KvTensorShape, SharedKVPool, CODEC_TURBO_8BIT};

fn shape() -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 1,
        num_heads: 1,
        num_kv_heads: 1,
        head_dim: 8,
        hidden_size: 8,
    }
}

fn corpus(n: usize) -> Vec<(String, Vec<f32>)> {
    let shape = shape();
    let len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let v = (0..len).map(|j| ((i + 1 + j) as f32).sin() * 0.1).collect();
            (format!("tok_{i}"), v)
        })
        .collect()
}

#[test]
fn compressed_attention_path_reports_decode_fallbacks_transparently() {
    let shape = shape();
    let shared = corpus(4);
    let (pool, _) = SharedKVPool::build(&shared, &shape, 42).unwrap();
    let agent = corpus(2);
    let (shell, _) = pool.materialize_shell("agent", &agent, 42).unwrap();
    let query = vec![0.1; shape.head_dim];

    let (hits, receipt) = shell
        .attention_topk_compressed(&pool, 0, &query, 3, &pool.policy.turbo_config)
        .unwrap();

    assert!(!hits.is_empty());
    assert_eq!(receipt.schema_version, "attention_selection_receipt_v1");
    assert_eq!(receipt.decoded_values, hits.len());
    assert!(receipt.candidate_count >= hits.len());
    // Pool-side fib compact pages and shell-side turbo pages must both score
    // without reconstructing full f32 keys.
    assert_eq!(receipt.decoded_keys, 0);

    let exact_hits = shell
        .attention_topk(&pool, 0, &query, 3, &pool.policy.turbo_config)
        .unwrap();
    let overlap = hits
        .iter()
        .filter(|hit| {
            exact_hits.iter().any(|exact| {
                exact.token_index == hit.token_index && exact.from_shell == hit.from_shell
            })
        })
        .count();
    assert!(
        overlap >= 2,
        "expected at least 2/3 top-k overlap, got {overlap}"
    );
}

#[test]
fn shell_uses_compact_turbo_wire_payloads() {
    let shape = shape();
    let shared = corpus(2);
    let (pool, _) = SharedKVPool::build(&shared, &shape, 42).unwrap();
    let agent = corpus(1);
    let (shell, receipt) = pool.materialize_shell("agent", &agent, 42).unwrap();
    assert!(receipt.shell_size_bytes > 0);
    let block = &shell.unique_layers[0].key_blocks[0];
    assert_eq!(block.codec, CODEC_TURBO_8BIT);
    assert!(block.encoded_payload.starts_with(b"TQW1"));
}

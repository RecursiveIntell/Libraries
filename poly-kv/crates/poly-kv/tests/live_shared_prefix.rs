//! Live shared-prefix integration tests for PolyKV.
//!
//! Tests the branch/fork API: shared immutable prefix, isolated writable
//! tails, combined state decoding, and multi-branch isolation.

use poly_kv::{
    BranchConfig, CompressionPolicyV1, DType, ExactFallback, ExactKvBlock, KvLayout, KvRole,
    KvTensorShape, LayerId, ModelFingerprint, PoolBuilder, QualityGateResultV1, SharedKvPool,
    TokenizerFingerprint,
};

fn synthetic_shape() -> KvTensorShape {
    KvTensorShape {
        layers: 2,
        key_heads: 2,
        value_heads: 2,
        seq_len: 4,
        head_dim: 8,
        layout: KvLayout::LayersHeadsTokensDim,
        dtype: DType::F32,
    }
}

fn synthetic_blocks(shape: &KvTensorShape) -> Vec<ExactKvBlock> {
    let mut blocks = Vec::new();
    let per_block =
        (shape.key_heads as usize) * (shape.seq_len as usize) * (shape.head_dim as usize);
    for layer in 0..shape.layers {
        for role in [KvRole::Key, KvRole::Value] {
            let data: Vec<f32> = (0..per_block)
                .map(|i| {
                    (layer as f32 * 1000.0)
                        + match role {
                            KvRole::Key => 0.1,
                            KvRole::Value => 0.2,
                        }
                        + (i as f32 * 0.001)
                })
                .collect();
            blocks.push(ExactKvBlock {
                role,
                layer: LayerId(layer),
                shape: shape.clone(),
                data,
            });
        }
    }
    blocks
}

fn build_pool() -> (SharedKvPool, KvTensorShape) {
    let shape = synthetic_shape();
    let blocks = synthetic_blocks(&shape);
    let fallback = ExactFallback::from_blocks(blocks.clone());
    let pool = PoolBuilder::default()
        .shape(shape.clone())
        .model_fingerprint(ModelFingerprint::new("test-model-v1").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("test-tokenizer-v1").unwrap())
        .exact_fallback(fallback)
        .policy(CompressionPolicyV1 {
            quality_gate: QualityGateResultV1 {
                max_key_mse: 0.01,
                max_value_mse: 0.001,
                passed: true,
                observed_key_mse: None,
                observed_value_mse: None,
                notes: vec!["synthetic relaxed gate".to_string()],
            },
            ..CompressionPolicyV1::alpha_reference()
        })
        .build_from_blocks(blocks)
        .unwrap();
    (pool, shape)
}

#[test]
fn shared_prefix_across_two_branches() {
    let (pool, shape) = build_pool();
    let branch_a = pool.fork(BranchConfig::new("agent-a")).unwrap();
    let branch_b = pool.fork(BranchConfig::new("agent-b")).unwrap();

    assert_eq!(branch_a.shared_prefix_len(), shape.seq_len);
    assert_eq!(branch_b.shared_prefix_len(), shape.seq_len);
    assert_eq!(branch_a.tail_len(), 0);
    assert_eq!(branch_b.tail_len(), 0);

    // Decode shared prefix from both branches — must produce same data.
    let (keys_a, values_a) = branch_a.decode_combined_layer(LayerId(0)).unwrap();
    let (keys_b, values_b) = branch_b.decode_combined_layer(LayerId(0)).unwrap();
    assert_eq!(keys_a, keys_b);
    assert_eq!(values_a, values_b);
}

#[test]
fn branch_isolation_under_mutation() {
    let (pool, shape) = build_pool();
    let mut branch_a = pool.fork(BranchConfig::new("agent-a")).unwrap();
    let mut branch_b = pool.fork(BranchConfig::new("agent-b")).unwrap();

    // Append different data to each branch.
    let tail_data_a: Vec<f32> = vec![1.0; (shape.key_heads as usize) * (shape.head_dim as usize)];
    let tail_data_b: Vec<f32> = vec![2.0; (shape.key_heads as usize) * (shape.head_dim as usize)];

    let tail_blocks_a = vec![ExactKvBlock {
        role: KvRole::Key,
        layer: LayerId(0),
        shape: shape.clone(),
        data: tail_data_a.clone(),
    }];
    let tail_blocks_b = vec![ExactKvBlock {
        role: KvRole::Key,
        layer: LayerId(0),
        shape: shape.clone(),
        data: tail_data_b.clone(),
    }];

    branch_a.append_blocks(tail_blocks_a).unwrap();
    branch_b.append_blocks(tail_blocks_b).unwrap();

    // Verify isolation: each branch's combined state is distinct.
    let (keys_a, _) = branch_a.decode_combined_layer(LayerId(0)).unwrap();
    let (keys_b, _) = branch_b.decode_combined_layer(LayerId(0)).unwrap();
    assert_ne!(keys_a, keys_b, "branches must be isolated");
    // But shared prefix portions should still match.
    let shared_len =
        (shape.key_heads as usize) * (shape.seq_len as usize) * (shape.head_dim as usize);
    assert_eq!(&keys_a[..shared_len], &keys_b[..shared_len]);
}

#[test]
fn interleaved_branch_stability() {
    let (pool, shape) = build_pool();
    let branch_a = pool.fork(BranchConfig::new("agent-a")).unwrap();
    let mut branch_b = pool.fork(BranchConfig::new("agent-b")).unwrap();

    // Capture branch-a state before branch-b mutation.
    let (keys_a_before, _) = branch_a.decode_combined_layer(LayerId(0)).unwrap();

    // Mutate branch-b.
    let tail_data: Vec<f32> = vec![5.0; (shape.key_heads as usize) * (shape.head_dim as usize)];
    branch_b
        .append_blocks(vec![ExactKvBlock {
            role: KvRole::Key,
            layer: LayerId(0),
            shape: shape.clone(),
            data: tail_data,
        }])
        .unwrap();

    // Branch-a state must be unchanged.
    let (keys_a_after, _) = branch_a.decode_combined_layer(LayerId(0)).unwrap();
    assert_eq!(keys_a_before, keys_a_after);
}

#[test]
fn fork_rejects_empty_agent_id() {
    let (pool, _) = build_pool();
    let err = pool.fork(BranchConfig::default()).unwrap_err();
    assert!(err.to_string().contains("agent_id"));
}

#[test]
fn fork_with_initial_tokens() {
    let (pool, _) = build_pool();
    let tokens = vec![101, 102, 103];
    let branch = pool
        .fork(BranchConfig::new("agent-a").with_tokens(tokens.clone()))
        .unwrap();
    assert_eq!(branch.tail_len(), 3);
}

#[test]
fn shared_prefix_len_matches_shape() {
    let (pool, shape) = build_pool();
    let branch = pool.fork(BranchConfig::new("a")).unwrap();
    assert_eq!(branch.shared_prefix_len(), shape.seq_len);
    assert_eq!(branch.current_seq_len(), shape.seq_len);
}

#[test]
fn append_increases_seq_len() {
    let (pool, shape) = build_pool();
    let mut branch = pool.fork(BranchConfig::new("a")).unwrap();
    let tail_data: Vec<f32> = vec![3.0; (shape.key_heads as usize) * (shape.head_dim as usize)];
    branch
        .append(
            &[101],
            vec![ExactKvBlock {
                role: KvRole::Key,
                layer: LayerId(0),
                shape: shape.clone(),
                data: tail_data,
            }],
        )
        .unwrap();
    assert_eq!(branch.current_seq_len(), shape.seq_len + 1);
}

//! `poly_kv_multi_agent_shell` — build a shared pool once, materialize N
//! agent shells, and emit per-agent K/V caches for forward-pass evaluation.
//!
//! Usage:
//!   poly_kv_multi_agent_shell <input.json> <output_dir> [--n-agents N] [--shared-frac F] [--seed S]
//!
//! Input format: same as `poly_kv_fast_roundtrip` (shape + tokens).
//! The script splits the corpus into a shared prefix (first F of the tokens)
//! and N per-agent tails (the remaining tokens, partitioned by agent).
//!
//! Output per agent i in <output_dir>:
//!   - agent_<i>_kv.bin:    length-prefixed K/V tensors (per layer, fp32 LE)
//!   - agent_<i>_receipt.json: per-agent shell receipt (size, digest, etc.)
//!
//! Plus shared outputs in <output_dir>:
//!   - shared_pool_receipt.json: shared pool manifest (id, size, ratio)
//!   - shared_kv.bin:             decompressed shared K/V (per layer, fp32 LE)
//!
//! The K/V binary layout (matching poly_kv_fast_roundtrip):
//!   - manifest: u64 LE length + JSON bytes
//!   - layers:   for each layer in [0, num_layers):
//!     - keys:   u32 num_tokens, then f32[num_tokens * num_kv_heads * head_dim]
//!     - values: u32 num_tokens, then f32[num_tokens * num_kv_heads * head_dim]

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use poly_kv::codec::{CompressedBlock, FibQuantAdapter};
use poly_kv::policy::{CompressionPolicy, FibConfig};
use poly_kv::shape::{AttentionType, KvTensorShape};
use poly_kv::SharedKVPool;

#[derive(Debug, Deserialize)]
struct InputJson {
    shape: ShapeJson,
    tokens: Vec<TokenJson>,
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ShapeJson {
    attention_type: String,
    num_layers: u32,
    num_heads: u32,
    num_kv_heads: u32,
    head_dim: usize,
    hidden_size: usize,
}

#[derive(Debug, Deserialize)]
struct TokenJson {
    id: String,
    vector: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct PoolReceipt {
    pool_id: String,
    num_shared_tokens: u32,
    num_layers: u32,
    num_kv_heads: u32,
    head_dim: usize,
    shared_codec: String,
    compression_ratio: f64,
    pool_size_bytes: u64,
    build_seed: u64,
    built_at_unix: i64,
    build_ms: u64,
}

#[derive(Debug, Serialize)]
struct AgentReceipt {
    agent_id: String,
    pool_digest: String,
    shell_digest: String,
    num_unique_tokens: u32,
    num_layers: u32,
    shell_size_bytes: u64,
    shell_bits: u8,
    shell_projections: usize,
    materialize_ms: u64,
}

fn write_kv_binary(path: &PathBuf, manifest: &serde_json::Value, layers: &[(Vec<f32>, Vec<f32>)]) {
    let manifest_bytes = serde_json::to_vec(manifest).expect("serialize manifest");
    let mut f = fs::File::create(path).expect("create output file");
    f.write_all(&(manifest_bytes.len() as u64).to_le_bytes())
        .unwrap();
    f.write_all(&manifest_bytes).unwrap();
    for (k, v) in layers {
        f.write_all(&(k.len() as u32).to_le_bytes()).unwrap();
        let k_bytes: Vec<u8> = k.iter().flat_map(|f_| f_.to_le_bytes()).collect();
        f.write_all(&k_bytes).unwrap();
        f.write_all(&(v.len() as u32).to_le_bytes()).unwrap();
        let v_bytes: Vec<u8> = v.iter().flat_map(|f_| f_.to_le_bytes()).collect();
        f.write_all(&v_bytes).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: {} <input.json> <output_dir> [--n-agents N] [--shared-frac F] \
             [--seed S] [--shell-bits B] [--shell-projections P]",
            args[0]
        );
        std::process::exit(1);
    }
    let input_path = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    fs::create_dir_all(&output_dir).expect("create output dir");

    // Parse optional flags
    let mut n_agents: usize = 2;
    let mut shared_frac: f64 = 0.8;
    let mut override_seed: Option<u64> = None;
    let mut shell_bits_override: Option<u8> = None;
    let mut shell_projections_override: Option<usize> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--n-agents" => {
                n_agents = args[i + 1].parse().expect("--n-agents must be int");
                i += 2;
            }
            "--shared-frac" => {
                shared_frac = args[i + 1].parse().expect("--shared-frac must be float");
                i += 2;
            }
            "--seed" => {
                override_seed = Some(args[i + 1].parse().expect("--seed must be int"));
                i += 2;
            }
            "--shell-bits" => {
                shell_bits_override = Some(args[i + 1].parse().expect("--shell-bits must be int"));
                i += 2;
            }
            "--shell-projections" => {
                shell_projections_override = Some(
                    args[i + 1]
                        .parse()
                        .expect("--shell-projections must be int"),
                );
                i += 2;
            }
            _ => panic!("unknown arg: {}", args[i]),
        }
    }

    let input_bytes = fs::read(&input_path).expect("read input");
    let input: InputJson = serde_json::from_slice(&input_bytes).expect("parse input json");
    let attn_type = match input.shape.attention_type.as_str() {
        "MHA" => AttentionType::MHA,
        "GQA" => AttentionType::GQA,
        "MQA" => AttentionType::MQA,
        other => panic!("unknown attention_type: {other}"),
    };
    let shape = KvTensorShape {
        attention_type: attn_type,
        num_layers: input.shape.num_layers,
        num_heads: input.shape.num_heads,
        num_kv_heads: input.shape.num_kv_heads,
        head_dim: input.shape.head_dim,
        hidden_size: input.shape.hidden_size,
    };
    let seed = override_seed.or(input.seed).unwrap_or(42);
    let total_tokens = input.tokens.len();
    let n_shared = ((total_tokens as f64) * shared_frac).floor() as usize;
    let n_tail_total = total_tokens - n_shared;
    if n_tail_total < n_agents {
        panic!(
            "tail has {} tokens but {} agents requested (need at least 1 per agent)",
            n_tail_total, n_agents
        );
    }
    let per_agent = n_tail_total / n_agents;

    eprintln!(
        "[multi-agent] total={} shared={} n_agents={} per_agent_tail={} seed={}",
        total_tokens, n_shared, n_agents, per_agent, seed
    );

    let shared_tokens: Vec<(String, Vec<f32>)> = input.tokens[..n_shared]
        .iter()
        .map(|t| (t.id.clone(), t.vector.clone()))
        .collect();
    let agent_tokens: Vec<Vec<(String, Vec<f32>)>> = (0..n_agents)
        .map(|a| {
            let start = n_shared + a * per_agent;
            let end = if a == n_agents - 1 {
                total_tokens
            } else {
                start + per_agent
            };
            input.tokens[start..end]
                .iter()
                .map(|t| (t.id.clone(), t.vector.clone()))
                .collect()
        })
        .collect();

    // Build the shared pool
    let fib_cfg: FibConfig = CompressionPolicy::default_two_tier().fib_config.clone();
    eprintln!(
        "[multi-agent] building shared pool ({} tokens)...",
        shared_tokens.len()
    );
    let t_build = Instant::now();
    let (mut pool, pool_receipt) =
        SharedKVPool::build(&shared_tokens, &shape, seed).expect("build shared pool");
    let build_ms = t_build.elapsed().as_millis() as u64;
    eprintln!(
        "[multi-agent] shared pool built in {}ms: pool_id={} ratio={:.2}x size={}",
        build_ms,
        &pool.manifest.pool_id.hex()[..12],
        pool.manifest.compression_ratio,
        pool.manifest.pool_size_bytes
    );

    // Mutate the pool's policy to apply shell-bits/projections overrides AFTER
    // build (SharedKVPool::build always uses default_two_tier internally; we
    // override here so materialize_shell uses the requested shell config).
    if let Some(b) = shell_bits_override {
        pool.policy.turbo_config.bits = b;
        eprintln!("[multi-agent] applied shell bits={} to pool policy", b);
    }
    if let Some(p) = shell_projections_override {
        pool.policy.turbo_config.projections = p;
        eprintln!(
            "[multi-agent] applied shell projections={} to pool policy",
            p
        );
    }

    let shared_pool_receipt = PoolReceipt {
        pool_id: pool.manifest.pool_id.hex().to_string(),
        num_shared_tokens: pool.manifest.num_shared_tokens,
        num_layers: pool.manifest.num_layers,
        num_kv_heads: pool.manifest.shape.num_kv_heads,
        head_dim: pool.manifest.shape.head_dim,
        shared_codec: format!("{:?}", pool.manifest.shared_codec),
        compression_ratio: pool.manifest.compression_ratio,
        pool_size_bytes: pool.manifest.pool_size_bytes,
        build_seed: pool.manifest.build_seed,
        built_at_unix: pool.manifest.built_at_unix,
        build_ms,
    };
    fs::write(
        output_dir.join("shared_pool_receipt.json"),
        serde_json::to_string_pretty(&shared_pool_receipt).unwrap(),
    )
    .unwrap();

    // Decompress the shared pool (per-layer K+V as f32) in parallel.
    eprintln!("[multi-agent] decompressing shared pool (fast path)...");
    let t_dec = Instant::now();
    let num_layers = pool.manifest.num_layers as usize;
    let num_shared_tokens = shared_tokens.len();
    let num_kv_heads = pool.manifest.shape.num_kv_heads as usize;
    let head_dim = pool.manifest.shape.head_dim;
    let adapter = FibQuantAdapter::new(
        head_dim,
        fib_cfg.k,
        fib_cfg.n,
        fib_cfg.training_samples,
        fib_cfg.lloyd_restarts,
        fib_cfg.lloyd_iterations,
    )
    .expect("create adapter");
    let shared_layers: Vec<(Vec<f32>, Vec<f32>)> = (0..num_layers)
        .into_par_iter()
        .map(|layer_idx| {
            let quantizer = adapter.build_quantizer(seed).expect("create quantizer");
            let layer = &pool.layers[layer_idx];
            let decode_one = |b: &CompressedBlock| -> fib_quant::FibCodeV1 {
                let bytes = &b.encoded_payload;
                if bytes.len() >= 3 && bytes[0..3] == fib_quant::COMPACT_MAGIC {
                    fib_quant::FibCodeV1::from_compact_bytes(bytes, quantizer.profile())
                        .expect("decode compact")
                } else {
                    serde_json::from_slice(bytes).expect("decode json fallback")
                }
            };
            let k_codes: Vec<fib_quant::FibCodeV1> =
                layer.key_blocks.iter().map(decode_one).collect();
            let v_codes: Vec<fib_quant::FibCodeV1> =
                layer.value_blocks.iter().map(decode_one).collect();
            let k_decoded = quantizer.decode_batch_fast(&k_codes).expect("decode K");
            let v_decoded = quantizer.decode_batch_fast(&v_codes).expect("decode V");
            let mut k_flat: Vec<f32> =
                Vec::with_capacity(num_shared_tokens * num_kv_heads * head_dim);
            for chunk in &k_decoded {
                k_flat.extend_from_slice(chunk);
            }
            let mut v_flat: Vec<f32> =
                Vec::with_capacity(num_shared_tokens * num_kv_heads * head_dim);
            for chunk in &v_decoded {
                v_flat.extend_from_slice(chunk);
            }
            (k_flat, v_flat)
        })
        .collect();
    let dec_ms = t_dec.elapsed().as_millis() as u64;
    eprintln!("[multi-agent] shared pool decompressed in {}ms", dec_ms);

    // Write shared K/V to disk (per layer, fp32 LE).
    let shared_manifest = serde_json::json!({
        "kind": "shared_kv",
        "num_layers": num_layers,
        "num_kv_heads": num_kv_heads,
        "head_dim": head_dim,
        "num_tokens": num_shared_tokens,
    });
    write_kv_binary(
        &output_dir.join("shared_kv.bin"),
        &shared_manifest,
        &shared_layers,
    );
    eprintln!("[multi-agent] wrote shared_kv.bin");

    // For each agent, materialize a shell and write per-agent K/V.
    let mut agent_receipts: Vec<AgentReceipt> = Vec::with_capacity(n_agents);
    for agent_idx in 0..n_agents {
        let agent_id = format!("agent_{}", agent_idx);
        let tokens = &agent_tokens[agent_idx];
        eprintln!(
            "[multi-agent] materializing shell for {} ({} unique tokens)...",
            agent_id,
            tokens.len()
        );
        let t_mat = Instant::now();
        let (shell, mat_receipt) =
            poly_kv::shell::materialize_shell(&pool, &agent_id, tokens, seed)
                .expect("materialize shell");
        let mat_ms = t_mat.elapsed().as_millis() as u64;
        eprintln!(
            "[multi-agent] {} shell materialized in {}ms: digest={} size={} bytes",
            agent_id,
            mat_ms,
            &mat_receipt.shell_digest.hex()[..12],
            mat_receipt.shell_size_bytes
        );

        // Decompress the shell to per-layer K/V.
        let shell_layers: Vec<(Vec<f32>, Vec<f32>)> = (0..num_layers)
            .into_par_iter()
            .map(|layer_idx| {
                let sl = &shell.unique_layers[layer_idx];
                // The shell uses turbo at the pool policy's bits/projections.
                use poly_kv::codec::KVecCodec;
                let turbo = poly_kv::codec::TurboQuantAdapter::new(
                    head_dim,
                    pool.policy.turbo_config.bits,
                    pool.policy.turbo_config.projections,
                )
                .expect("turbo adapter");
                let mut k_flat: Vec<f32> =
                    Vec::with_capacity(tokens.len() * num_kv_heads * head_dim);
                let mut v_flat: Vec<f32> =
                    Vec::with_capacity(tokens.len() * num_kv_heads * head_dim);
                for b in &sl.key_blocks {
                    let v = turbo.decode(&b.encoded_payload, seed).expect("decode K");
                    k_flat.extend_from_slice(&v);
                }
                for b in &sl.value_blocks {
                    let v = turbo.decode(&b.encoded_payload, seed).expect("decode V");
                    v_flat.extend_from_slice(&v);
                }
                (k_flat, v_flat)
            })
            .collect();

        // Write per-agent K/V: the shared + the shell, in the same binary format.
        // The Python side knows to read both and concatenate them when building
        // the DynamicCache.
        let agent_manifest = serde_json::json!({
            "kind": "agent_kv",
            "agent_id": agent_id,
            "num_layers": num_layers,
            "num_kv_heads": num_kv_heads,
            "head_dim": head_dim,
            "num_shared_tokens": num_shared_tokens,
            "num_unique_tokens": tokens.len(),
            "shell_size_bytes": mat_receipt.shell_size_bytes,
        });
        write_kv_binary(
            &output_dir.join(format!("agent_{}_kv.bin", agent_idx)),
            &agent_manifest,
            &shell_layers,
        );
        eprintln!("[multi-agent] wrote agent_{}_kv.bin", agent_idx);

        agent_receipts.push(AgentReceipt {
            agent_id: agent_id.clone(),
            pool_digest: pool.manifest.pool_id.hex().to_string(),
            shell_digest: mat_receipt.shell_digest.hex().to_string(),
            num_unique_tokens: mat_receipt.num_unique_tokens,
            num_layers: shell.unique_layers.len() as u32,
            shell_size_bytes: mat_receipt.shell_size_bytes,
            shell_bits: pool.policy.turbo_config.bits,
            shell_projections: pool.policy.turbo_config.projections,
            materialize_ms: mat_ms,
        });
    }

    fs::write(
        output_dir.join("agents_receipt.json"),
        serde_json::to_string_pretty(&agent_receipts).unwrap(),
    )
    .unwrap();

    eprintln!(
        "[multi-agent] DONE: 1 shared pool ({} bytes, {:.2}x compression) + {} agent shells",
        pool.manifest.pool_size_bytes, pool.manifest.compression_ratio, n_agents
    );
    eprintln!(
        "[multi-agent] total wall: shared build {}ms + decompress {}ms + {} agent shell materializations",
        build_ms,
        dec_ms,
        n_agents
    );
}

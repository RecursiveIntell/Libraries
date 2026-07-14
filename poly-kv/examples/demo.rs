//! poly-kv demo: PoolManifest → codec → compress/decompress KV block → receipt.
//!
//! Run from the poly-kv directory:
//!   cargo run --example demo

use poly_kv::{
    create_codec,
    policy::{CompressionPolicy, FibConfig, CODEC_FIB_K4_N32},
    receipt::{now_unix, PoolBuildReceipt},
    shape::{AttentionType, KvTensorShape},
    CompressedBlock, Digest, PoolManifest,
};

fn main() {
    println!("┌─────────────────────────────────────────────┐");
    println!("│  poly-kv demo: manifest + codec + roundtrip  │");
    println!("└─────────────────────────────────────────────┘");

    // ── 1. Create a PoolManifest ───────────────────────────────────
    //
    // A PoolManifest describes a built shared KV pool: its shape, policy,
    // compression, and provenance. We construct one with a small MHA shape
    // suitable for a demo.

    let shape = KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 4,
        num_heads: 8,
        num_kv_heads: 8,
        head_dim: 64,
        hidden_size: 512,
    };

    let policy = CompressionPolicy::default_two_tier();

    // Compute a pool_id (blake3 of a canonical label).
    let pool_id = Digest::compute_str("demo-pool-v1");

    let num_tokens: u32 = 16;
    let raw_size_bytes = shape.total_kv_bytes(num_tokens as usize) as u64;
    // Approximate compressed size (fib-quant nominally 50×).
    let pool_size_bytes = raw_size_bytes / 50;

    let manifest = PoolManifest::new(
        pool_id,
        shape.clone(),
        policy.clone(),
        num_tokens,
        shape.num_layers,
        pool_size_bytes,
        raw_size_bytes,
        42, // build seed
        now_unix(),
    )
    .expect("create PoolManifest");

    manifest.validate().expect("manifest validates");

    println!("\n── 1. PoolManifest ──");
    println!("  schema_version : {}", manifest.schema_version);
    println!("  pool_id        : {}…", &manifest.pool_id.hex()[..16]);
    println!(
        "  shape          : {} layers, {} heads, head_dim={}",
        manifest.num_layers, manifest.shape.num_heads, manifest.shape.head_dim,
    );
    println!("  shared_codec   : {}", manifest.shared_codec);
    println!("  num_tokens     : {}", manifest.num_shared_tokens);
    println!("  pool_size      : {} bytes", manifest.pool_size_bytes);
    println!("  raw_size       : {} bytes", raw_size_bytes);
    println!("  compression    : {:.1}×", manifest.compression_ratio,);
    println!("  build_seed     : {}", manifest.build_seed);
    println!("  validation     : OK");

    // ── 2. Create a codec via create_codec ─────────────────────────
    //
    // create_codec dispatches on the codec_id string and returns a
    // boxed trait object (Box<dyn KVecCodec>). We use the default
    // fib_k4_n32 codec — the cold-tier shared-pool codec.

    let fib_cfg = FibConfig::default_k4_n32();

    let codec = create_codec(
        CODEC_FIB_K4_N32,
        shape.head_dim,
        Some(&fib_cfg),
        Some(&policy.turbo_config),
    )
    .expect("create codec");

    println!("\n── 2. Codec ──");
    println!("  codec_id       : {}", codec.codec_id());
    println!("  dim            : {}", codec.dim());
    println!("  nominal_ratio  : {:.1}×", codec.compression_ratio());

    // ── 3. Compress and decompress a KV block ──────────────────────
    //
    // A "KV block" is a single head vector (key or value) of dimension
    // head_dim. We synthesize one with a sine pattern, encode it, wrap
    // it in a CompressedBlock (which computes the blake3 payload digest),
    // then decode and measure reconstruction error.

    let seed: u64 = 42;
    let vector: Vec<f32> = (0..shape.head_dim)
        .map(|i| ((i as f32) * 0.1).sin())
        .collect();

    let raw_bytes = vector.len() * 4; // f32 = 4 bytes

    println!("\n── 3. Compress / Decompress KV block ──");
    println!("  input dim      : {}", vector.len());
    println!("  raw bytes      : {}", raw_bytes);

    // Encode (compress)
    let encoded = codec.encode(&vector, seed).expect("encode");
    println!("  compressed     : {} bytes", encoded.len());

    // Wrap in CompressedBlock — computes blake3 digest and metadata
    let block = CompressedBlock::new(codec.codec_id(), encoded.clone(), vector.len());

    println!("\n  CompressedBlock:");
    println!("    codec        : {}", block.codec);
    println!("    payload_hash : {}…", &block.payload_digest.hex()[..16]);
    println!("    original_dim : {}", block.original_dim);
    println!("    comp_bytes   : {}", block.compressed_bytes);
    println!("    block_ratio  : {:.1}×", block.compression_ratio(),);

    // Decode (decompress)
    let decoded = codec.decode(&encoded, seed).expect("decode");

    // Measure reconstruction quality
    let max_err = vector
        .iter()
        .zip(decoded.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let mean_err = vector
        .iter()
        .zip(decoded.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / vector.len() as f32;

    println!("\n  Decompression:");
    println!("    decoded dim  : {}", decoded.len());
    println!("    max_error    : {:.6}", max_err);
    println!("    mean_error   : {:.6}", mean_err);

    // ── 4. Show the receipt ────────────────────────────────────────
    //
    // A PoolBuildReceipt is the content-addressed provenance receipt
    // for a pool build. It records digests, sizes, policy snapshot,
    // and timing. We construct one from the manifest + codec metadata.

    let layer_digests: Vec<Digest> = (0..shape.num_layers)
        .map(|i| Digest::compute_str(&format!("demo-layer-{i}")))
        .collect();

    // The codec can provide codebook & rotation digests for provenance.
    let codebook_digest = codec
        .codebook_digest(seed)
        .map(Digest::from_hex_unchecked)
        .unwrap_or_else(|| Digest::from_hex_unchecked("0"));
    let rotation_digest = codec
        .rotation_digest(seed)
        .map(Digest::from_hex_unchecked)
        .unwrap_or_else(|| Digest::from_hex_unchecked("0"));

    let receipt = PoolBuildReceipt::new(
        manifest.pool_id.clone(),
        layer_digests,
        codebook_digest,
        rotation_digest,
        num_tokens,
        0, // fib_build_ms — not timed in this demo
        pool_size_bytes,
        raw_size_bytes,
        policy.clone(),
        seed,
        now_unix(),
    );

    receipt.validate().expect("receipt validates");
    let receipt_digest = receipt.digest().expect("compute receipt digest");

    println!("\n── 4. PoolBuildReceipt ──");
    println!("  schema_version   : {}", receipt.schema_version);
    println!("  pool_digest      : {}…", &receipt.pool_digest.hex()[..16]);
    println!(
        "  layer_digests    : {} entries",
        receipt.layer_digests.len()
    );
    println!(
        "  codebook_digest  : {}…",
        &receipt.codebook_digest.hex()[..16]
    );
    println!(
        "  rotation_digest  : {}…",
        &receipt.rotation_digest.hex()[..16]
    );
    println!("  total_tokens     : {}", receipt.total_tokens);
    println!("  pool_size_bytes  : {}", receipt.pool_size_bytes);
    println!("  raw_size_bytes   : {}", receipt.raw_size_bytes);
    println!("  compression_ratio: {:.1}×", receipt.compression_ratio);
    println!("  backend          : {}", receipt.backend);
    println!("  seeded_with      : {}", receipt.seeded_with);
    println!("  validation       : OK");
    println!("  receipt_digest   : {}…", &receipt_digest.hex()[..16]);

    println!("\n┌─────────────────────────────────────────────┐");
    println!("│  demo complete — all steps succeeded         │");
    println!("└─────────────────────────────────────────────┘");
}

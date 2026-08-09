// proveKV immutable cold pool — builder, exporter, importer.
//
// The cold pool is a read-only 8-bit compressed key vector store.
// It's portable across machines — same format on Vega iGPU (Vulkan),
// GTX 1070 (Vulkan), and CPU. The hot shell (f16 attention cache)
// is machine-local and never transferred.
//
// Commands:
//   provekv-pool build <text-file> [--dims 768]       → create pool from text
//   provekv-pool export <pool-file>                    → verify + stats
//   provekv-pool import <pool-file> <target-dir>       → reconstruct context
//   provekv-pool info <pool-file>                      → compression stats

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

const POOL_MAGIC: &[u8; 4] = b"PVKP";
const POOL_VERSION: u32 = 1;
const MAX_HEADER_BYTES: usize = 1024 * 1024;

/// proveKV immutable compressed pool
#[derive(Debug, Serialize, Deserialize)]
struct PoolHeader {
    magic: [u8; 4],
    version: u32,
    dims: u32,
    n_vectors: u64,
    /// Original text byte count (for compression ratio)
    original_bytes: u64,
    /// Timestamp of pool creation
    created_at: u64,
    /// SHA-256 of original text (for integrity)
    source_hash: [u8; 32],
    /// Compression flags
    flags: u32,
}

impl PoolHeader {
    fn new(dims: u32, n_vectors: u64, original_bytes: u64, source_hash: [u8; 32]) -> Self {
        Self {
            magic: *POOL_MAGIC,
            version: POOL_VERSION,
            dims,
            n_vectors,
            original_bytes,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source_hash,
            flags: 0,
        }
    }

    fn pool_bytes(&self) -> u64 {
        self.n_vectors * self.dims as u64
    }

    fn expected_pool_bytes(&self) -> io::Result<usize> {
        let bytes = self.expected_pool_bytes_u64()?;
        usize::try_from(bytes).map_err(|_| invalid_data("pool payload size does not fit usize"))
    }

    fn expected_pool_bytes_u64(&self) -> io::Result<u64> {
        self.n_vectors
            .checked_mul(u64::from(self.dims))
            .ok_or_else(|| invalid_data("pool payload size overflows u64"))
    }

    fn validate(&self) -> io::Result<()> {
        if self.magic != *POOL_MAGIC {
            return Err(invalid_data("invalid pool magic"));
        }
        if self.version != POOL_VERSION {
            return Err(invalid_data(format!(
                "unsupported pool version {}",
                self.version
            )));
        }
        if self.dims == 0 {
            return Err(invalid_data("pool dimensions must be greater than zero"));
        }
        if self.n_vectors == 0 {
            return Err(invalid_data("pool must contain at least one vector"));
        }
        self.expected_pool_bytes_u64()?;
        Ok(())
    }

    fn compression_ratio(&self) -> f64 {
        if self.original_bytes == 0 {
            return 0.0;
        }
        self.original_bytes as f64 / self.pool_bytes() as f64
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Embedder {
    Hash,
    Ollama,
}

impl FromStr for Embedder {
    type Err = io::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "hash" => Ok(Self::Hash),
            "ollama" => Ok(Self::Ollama),
            _ => Err(invalid_input(format!(
                "unsupported embedder {value:?}; expected one of: hash, ollama"
            ))),
        }
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[derive(Parser)]
#[command(
    name = "provekv-pool",
    about = "proveKV immutable compressed pool tool"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a compressed pool from a text file
    Build {
        /// Input text file
        #[arg(long)]
        input: PathBuf,
        /// Output pool file
        #[arg(long, default_value = "pool.pvkp")]
        output: PathBuf,
        /// Vector dimensions (default: 768 for nomic-embed-text)
        #[arg(long, default_value_t = 768)]
        dims: u32,
        /// Embedding backend: hash (fast, pseudo-random) or ollama (real nomic-embed-text)
        #[arg(long, default_value = "hash")]
        embedder: String,
        /// Ollama server URL (for --embedder ollama)
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
        /// Ollama model for embeddings
        #[arg(long, default_value = "nomic-embed-text:q8")]
        ollama_model: String,
    },
    /// Show compression stats for a pool
    Info {
        /// Pool file
        pool: PathBuf,
    },
    /// Export pool as portable JSON for transfer
    Export {
        /// Pool file
        pool: PathBuf,
        /// Output JSON file
        #[arg(long, default_value = "pool.json")]
        output: PathBuf,
    },
    /// Import pool from JSON
    Import {
        /// Input JSON file
        json: PathBuf,
        /// Output pool file
        #[arg(long, default_value = "imported.pvkp")]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {
            input,
            output,
            dims,
            embedder,
            ollama_url,
            ollama_model,
        } => cmd_build(input, output, dims, embedder, ollama_url, ollama_model),
        Command::Info { pool } => cmd_info(pool),
        Command::Export { pool, output } => cmd_export(pool, output),
        Command::Import { json, output } => cmd_import(json, output),
    }
}

/// Encode a float to 8-bit uniform quantized value.
/// Range: [-1.0, 1.0] → [0, 255]
fn f32_to_u8(v: f32) -> u8 {
    let clamped = v.clamp(-1.0, 1.0);
    ((clamped + 1.0) * 127.5).round() as u8
}

/// Decode 8-bit quantized value back to float.
#[cfg(test)]
fn u8_to_f32(v: u8) -> f32 {
    (v as f32 / 127.5) - 1.0
}

/// Build a compressed pool from text using simple hash-based embedding.
/// In production, this would use nomic-embed-text via Ollama.
/// For the pool proof-of-concept, we use a deterministic hash-based
/// embedding. It is not a semantic embedding backend.
fn embed_text(text: &str, dims: u32) -> Vec<Vec<f32>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![vec![0.0; dims as usize]];
    }

    // Simple fixed-size chunking: every ~128 words = 1 vector
    let chunk_size = 128;
    let n_vectors = words.len().div_ceil(chunk_size).max(1);

    let mut vectors = Vec::with_capacity(n_vectors);
    let mut rng: u64 = 42;

    for chunk_idx in 0..n_vectors {
        let start = chunk_idx * chunk_size;
        let end = (start + chunk_size).min(words.len());
        let chunk = &words[start..end];

        let mut vec = vec![0.0f32; dims as usize];
        for &word in chunk {
            // Hash word to a pseudo-random vector direction
            let word_hash = word
                .bytes()
                .fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let seed = word_hash ^ rng;

            // Generate a pseudo-random unit vector component for this word
            for (d, value) in vec.iter_mut().enumerate() {
                let idx = (seed as usize).wrapping_mul(d + 1) % 1024;
                let phase = (idx as f32 / 1024.0) * std::f32::consts::TAU;
                *value += phase.sin() * 0.01; // Small contribution per word
            }
        }

        // Normalize to [-1, 1] range
        let max_abs = vec.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
        if max_abs > 1.0 {
            for v in &mut vec {
                *v /= max_abs;
            }
        }

        vectors.push(vec);
    }

    vectors
}

fn embed_text_ollama(
    text: &str,
    dims: u32,
    url: &str,
    model: &str,
) -> Result<Vec<Vec<f32>>, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let chunk_size = 128;
    let n_vectors = words.len().div_ceil(chunk_size).max(1);
    let mut vectors = Vec::with_capacity(n_vectors);

    let client = reqwest::blocking::Client::new();
    for chunk_idx in 0..n_vectors {
        let start = chunk_idx * chunk_size;
        let end = (start + chunk_size).min(words.len());
        let chunk_text = words[start..end].join(" ");

        let body = serde_json::json!({
            "model": model,
            "prompt": chunk_text,
        });

        let resp = client
            .post(format!("{}/api/embeddings", url))
            .json(&body)
            .send()
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned {}", resp.status()));
        }

        let data: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Ollama JSON parse: {}", e))?;

        let embedding: Vec<f32> = data["embedding"]
            .as_array()
            .ok_or("No embedding in response")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        if embedding.len() != dims as usize {
            return Err(format!("Expected {} dims, got {}", dims, embedding.len()));
        }
        vectors.push(embedding);
    }
    Ok(vectors)
}

fn source_hash(input: &[u8]) -> [u8; 32] {
    Sha256::digest(input).into()
}

fn validate_pool_file_len(file_len: u64, header_len: usize, header: &PoolHeader) -> io::Result<()> {
    let header_len = u64::try_from(header_len)
        .map_err(|_| invalid_data("pool header length does not fit u64"))?;
    let expected_pool_bytes = header.expected_pool_bytes_u64()?;
    let expected_file_len = 4u64
        .checked_add(header_len)
        .and_then(|len| len.checked_add(expected_pool_bytes))
        .ok_or_else(|| invalid_data("pool file size overflows u64"))?;
    if file_len != expected_file_len {
        return Err(invalid_data(format!(
            "pool file length mismatch: expected {expected_file_len} bytes, got {file_len}"
        )));
    }
    Ok(())
}

fn read_pool(pool: &Path) -> Result<(PoolHeader, Vec<u8>), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(pool)?;
    let file_len = file.metadata()?.len();
    let mut len_buf = [0u8; 4];
    file.read_exact(&mut len_buf)?;
    let header_len = u32::from_le_bytes(len_buf) as usize;
    if header_len == 0 || header_len > MAX_HEADER_BYTES {
        return Err(invalid_data(format!(
            "invalid pool header length {header_len}; maximum is {MAX_HEADER_BYTES}"
        ))
        .into());
    }
    let mut header_json = vec![0u8; header_len];
    file.read_exact(&mut header_json)?;
    let header: PoolHeader = serde_json::from_slice(&header_json)?;
    header.validate()?;
    validate_pool_file_len(file_len, header_len, &header)?;
    let expected_pool_bytes = header.expected_pool_bytes()?;
    let mut pool_data = Vec::new();
    pool_data
        .try_reserve_exact(expected_pool_bytes)
        .map_err(|_| invalid_data("pool payload allocation failed"))?;
    pool_data.resize(expected_pool_bytes, 0);
    file.read_exact(&mut pool_data)?;
    let mut trailing = [0u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(invalid_data("pool contains trailing bytes").into());
    }
    Ok((header, pool_data))
}

fn validate_import_payload(header: &PoolHeader, pool_data: &[u8]) -> io::Result<()> {
    header.validate()?;
    let expected = header.expected_pool_bytes()?;
    if pool_data.len() != expected {
        return Err(invalid_data(format!(
            "pool payload length mismatch: expected {expected} bytes, got {}",
            pool_data.len()
        )));
    }
    Ok(())
}

fn expected_base64_len(payload_len: usize) -> io::Result<usize> {
    payload_len
        .checked_add(2)
        .and_then(|len| len.checked_div(3))
        .and_then(|chunks| chunks.checked_mul(4))
        .ok_or_else(|| invalid_data("base64 payload length overflows usize"))
}

fn cmd_build(
    input: PathBuf,
    output: PathBuf,
    dims: u32,
    embedder: String,
    ollama_url: String,
    ollama_model: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let embedder: Embedder = embedder.parse()?;
    if dims == 0 {
        return Err(invalid_input("dimensions must be greater than zero").into());
    }
    println!("=== proveKV Pool Builder ===");
    println!(
        "input: {}  dims: {}  embedder: {:?}  output: {}",
        input.display(),
        dims,
        embedder,
        output.display()
    );

    let t0 = Instant::now();

    // Read input text
    let mut text = String::new();
    let mut file = fs::File::open(&input)?;
    let original_bytes = file.metadata()?.len();
    file.read_to_string(&mut text)?;

    // Compute the documented SHA-256 source identity.
    let source_hash = source_hash(text.as_bytes());

    // Embed text into vectors using the explicitly selected backend.
    let vectors_f32 = match embedder {
        Embedder::Hash => embed_text(&text, dims),
        Embedder::Ollama => {
            println!("Embedding via Ollama: {} @ {}", ollama_model, ollama_url);
            embed_text_ollama(&text, dims, &ollama_url, &ollama_model)
                .map_err(Box::<dyn std::error::Error>::from)?
        }
    };
    let n_vectors = vectors_f32.len() as u64;
    let read_time = t0.elapsed();

    // Quantize to 8-bit
    let t1 = Instant::now();
    let pool_bytes = usize::try_from(
        n_vectors
            .checked_mul(u64::from(dims))
            .ok_or_else(|| invalid_input("pool payload size overflows u64"))?,
    )
    .map_err(|_| invalid_input("pool payload size does not fit usize"))?;
    let mut pool = Vec::new();
    pool.try_reserve_exact(pool_bytes)
        .map_err(|_| invalid_input("pool payload allocation failed"))?;
    pool.resize(pool_bytes, 0);
    for (i, vec) in vectors_f32.iter().enumerate() {
        for (d, &v) in vec.iter().enumerate() {
            pool[i * dims as usize + d] = f32_to_u8(v);
        }
    }
    let quant_time = t1.elapsed();

    // Write header
    let header = PoolHeader::new(dims, n_vectors, original_bytes, source_hash);
    let header_json = serde_json::to_string(&header)?;
    let header_bytes = header_json.as_bytes();

    // Write pool file: [header_len: u32][JSON header][pool bytes]
    let mut out = fs::File::create(&output)?;
    out.write_all(&(header_bytes.len() as u32).to_le_bytes())?;
    out.write_all(header_bytes)?;
    out.write_all(&pool)?;
    out.flush()?;

    let total_time = t0.elapsed();
    let file_size = 4 + header_bytes.len() + pool_bytes;

    // Stats
    println!();
    println!(
        "Source: {} ({:.1} KB text, {} words)",
        input.display(),
        original_bytes as f64 / 1024.0,
        text.split_whitespace().count()
    );
    println!(
        "Vectors: {} × {} dims = {:.1} KB f32 → {:.1} KB u8",
        n_vectors,
        dims,
        (n_vectors * dims as u64 * 4) as f64 / 1024.0,
        pool_bytes as f64 / 1024.0
    );
    println!(
        "Compression: {:.1}x (original text → pool)",
        header.compression_ratio()
    );
    println!(
        "Pool file: {:.1} KB ({:.1} KB header + {:.1} KB data)",
        file_size as f64 / 1024.0,
        header_bytes.len() as f64 / 1024.0,
        pool_bytes as f64 / 1024.0
    );
    println!(
        "Timing: {:?} read + {:?} quant = {:?} total",
        read_time, quant_time, total_time
    );
    println!(
        "Transfer size: {:.1} KB ({:.2} MB)",
        file_size as f64 / 1024.0,
        file_size as f64 / 1024.0 / 1024.0
    );
    println!();
    println!("Export with: provekv-pool export {}", output.display());
    println!("Import with: provekv-pool import <json> --output <file>");
    println!("Pool ready for cross-machine transfer via scp/rsync.");

    Ok(())
}

fn cmd_info(pool: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let (header, _) = read_pool(&pool)?;

    let file_size = fs::metadata(&pool)?.len();

    println!("=== proveKV Pool: {} ===", pool.display());
    println!("Version:     v{}", header.version);
    println!("Dimensions:  {}", header.dims);
    println!(
        "Vectors:     {} ({} chunks)",
        header.n_vectors, header.n_vectors
    );
    println!(
        "Source:      {:.1} KB → pool {:.1} KB ({:.1}x compression)",
        header.original_bytes as f64 / 1024.0,
        header.pool_bytes() as f64 / 1024.0,
        header.compression_ratio()
    );
    println!("File:        {:.1} KB total", file_size as f64 / 1024.0);
    println!("Created:     {}", header.created_at);
    println!("Per-vector:  {:.1} bytes", header.dims as f64);
    println!("Per-chunk:   ~128 words → 768 bytes (8-bit quantized)");
    println!();
    println!("Cross-machine transfer:");
    println!("  scp {} msi:/tmp/pool.pvkp", pool.display());
    println!("  # Then on target: provekv-pool info /tmp/pool.pvkp");

    Ok(())
}

fn cmd_export(pool: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let (header, pool_data) = read_pool(&pool)?;

    // Export as base64-encoded JSON for portability
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let pool_b64 = B64.encode(&pool_data);

    #[derive(Serialize)]
    struct Export {
        header: PoolHeader,
        pool_base64: String,
    }

    let export = Export {
        header,
        pool_base64: pool_b64,
    };

    let json = serde_json::to_string_pretty(&export)?;
    fs::write(&output, &json)?;

    println!(
        "Exported {} → {} ({:.1} KB JSON)",
        pool.display(),
        output.display(),
        json.len() as f64 / 1024.0
    );
    println!("Transfer via scp or paste into any machine with provekv-pool.");

    Ok(())
}

fn cmd_import(json: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string(&json)?;
    let export: Export = serde_json::from_str(&data)?;

    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    export.header.validate()?;
    let expected_pool_bytes = export.header.expected_pool_bytes()?;
    let expected_base64_len = expected_base64_len(expected_pool_bytes)?;
    if export.pool_base64.len() != expected_base64_len {
        return Err(invalid_data(format!(
            "base64 payload length mismatch: expected {expected_base64_len} bytes, got {}",
            export.pool_base64.len()
        ))
        .into());
    }
    let pool_data = B64.decode(&export.pool_base64)?;
    validate_import_payload(&export.header, &pool_data)?;

    let header_json = serde_json::to_string(&export.header)?;
    let header_bytes = header_json.as_bytes();

    let mut out = fs::File::create(&output)?;
    out.write_all(&(header_bytes.len() as u32).to_le_bytes())?;
    out.write_all(header_bytes)?;
    out.write_all(&pool_data)?;

    println!(
        "Imported {} → {} ({:.1} KB pool, {} vectors)",
        json.display(),
        output.display(),
        pool_data.len() as f64 / 1024.0,
        export.header.n_vectors
    );

    Ok(())
}

// Need these for export/import
#[derive(Serialize, Deserialize)]
struct Export {
    header: PoolHeader,
    pool_base64: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_header() -> PoolHeader {
        PoolHeader::new(8, 2, 16, [0; 32])
    }

    #[test]
    fn source_hash_is_documented_sha256() {
        let actual = source_hash(b"abc");
        let actual_hex = actual
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            actual_hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn embedder_admission_is_explicit() {
        assert_eq!("hash".parse::<Embedder>().unwrap(), Embedder::Hash);
        assert_eq!("ollama".parse::<Embedder>().unwrap(), Embedder::Ollama);
        assert!("no-such-embedder".parse::<Embedder>().is_err());
    }

    #[test]
    fn import_payload_rejects_invalid_header_and_length() {
        let mut wrong_magic = valid_header();
        wrong_magic.magic = *b"NOPE";
        assert!(validate_import_payload(&wrong_magic, &[0; 16]).is_err());

        let valid = valid_header();
        assert!(validate_import_payload(&valid, &[0; 15]).is_err());
        assert!(validate_import_payload(&valid, &[0; 17]).is_err());
        assert!(validate_import_payload(&valid, &[0; 16]).is_ok());
    }

    #[test]
    fn base64_length_is_checked_before_decode() {
        assert_eq!(expected_base64_len(1).unwrap(), 4);
        assert_eq!(expected_base64_len(16).unwrap(), 24);
    }

    #[test]
    fn pool_header_rejects_empty_or_wrong_sized_artifacts() {
        let mut empty = valid_header();
        empty.n_vectors = 0;
        assert!(empty.validate().is_err());

        let valid = valid_header();
        assert!(validate_pool_file_len(4 + 16 + 16, 16, &valid).is_ok());
        assert!(validate_pool_file_len(4 + 16 + 15, 16, &valid).is_err());
    }

    #[test]
    fn quantization_roundtrip_stays_within_one_step() {
        for value in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let decoded = u8_to_f32(f32_to_u8(value));
            assert!((decoded - value).abs() <= (1.0 / 127.5));
        }
    }
}

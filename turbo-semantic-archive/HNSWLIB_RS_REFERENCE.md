# hnswlib-rs API Reference

## Source

Derived from crates.io and GitHub for `hnswlib-rs` as of January 2026. **Always verify against the actual published crate** — this is new and may have evolved.

- Crates.io: https://crates.io/crates/hnswlib-rs
- GitHub: https://github.com/jean-pierreBoth/hnswlib-rs
- Fallback crate: `hnsw_rs` (same author, older, different API)

## Core Concept

`hnswlib-rs` **decouples the graph from vector storage**:

- `Hnsw<K, M>` — graph topology + key→NodeId mapping
- `VectorStore` (you provide) — supplies vectors keyed by NodeId
- Vectors are NOT inside the graph — persist them separately

This means different precisions (f32, f16, bf16, qi8) without changing graph code.

## Imports

```rust
use hnswlib_rs::{
    Hnsw,                      // Graph. Generic over K (key), M (metric)
    HnswConfig,                // Builder: dim, max_nodes, m, ef_construction, ef_search
    InMemoryVectorStore,       // f32 vector storage
    InMemoryQi8VectorStore,    // Quantized int8 vector storage
    L2,                        // L2 distance (f32)
    Cosine,                    // Cosine distance (f32)
    CosineQi8,                 // Cosine distance (Qi8)
    L2Qi8,                     // L2 distance (Qi8)
    InnerProductQi8,           // Inner product (Qi8)
    Qi8Ref,                    // Borrowed quantized vector
    Hit,                       // Search result
    Result,                    // Library error type
};
```

## HnswConfig

```rust
let cfg = HnswConfig::new(dim, max_nodes)
    .m(16)                 // Connections per node (16-64)
    .ef_construction(200)  // Build search width (200-800)
    .ef_search(50);        // Query search width (>= top_k)
```

## Qi8Ref

```rust
pub struct Qi8Ref<'a> {
    pub data: &'a [i8],    // Quantized int8 values
    pub scale: f32,         // Scale factor
    pub zero_point: i8,     // Asymmetric zero point
}
// Reconstruct: original[i] ≈ (data[i] - zero_point) * scale
```

## Hit<K>

```rust
pub struct Hit<K> {
    pub key: K,         // External key (e.g., "fact:42")
    pub distance: f32,  // Lower = more similar
}
// For cosine: similarity = 1.0 - distance
```

## Operations

### Construction
```rust
let hnsw: Hnsw<String, CosineQi8> = Hnsw::new(CosineQi8::new(), cfg);
let store = InMemoryQi8VectorStore::new(dim, max_nodes);
```

### Insert
```rust
let qi8 = Qi8Ref { data: &quantized_i8_vec, scale: 0.02, zero_point: 0 };
hnsw.insert(&store, "fact:42".to_string(), qi8)?;
```

### Search
```rust
let hits: Vec<Hit<String>> = hnsw.search(&store, query_qi8, top_k, filter)?;
// filter: Option<&dyn Fn(&String) -> bool>
```

### Delete
```rust
hnsw.delete("fact:42")?;  // Tombstones node, key mapping retained
```

### Set (Insert-or-Update)
```rust
hnsw.set(&store, "fact:42".to_string(), new_qi8)?;
// Exists → update + repair connections
// Deleted → resurrect
// New → insert
```

### Persistence (Separate Files)
```rust
// Save
hnsw.save_to(&mut File::create("memory.hnsw")?)?;
store.save_to(&mut File::create("memory.vectors")?, hnsw.len())?;

// Load
let hnsw = Hnsw::load_from(CosineQi8::new(), &mut File::open("memory.hnsw")?)?;
let (store, count) = InMemoryQi8VectorStore::load_from(&mut File::open("memory.vectors")?)?;
```

### Other
```rust
hnsw.len()                 // Active (non-deleted) count
hnsw.is_empty()
hnsw.node_id(&key)         // Option<NodeId>
```

## Concurrency

From docs: "Hnsw is designed for concurrent search + concurrent mutation. InMemoryVectorStore supports lock-free reads and parallel updates (per-NodeId atomic swap)."

→ Wrap in `Arc` for sharing. No `Mutex`/`RwLock` needed (verify empirically).

## Available Metrics

| Metric | Vector Type | Our Choice |
|---|---|---|
| `CosineQi8` | Qi8 | ✅ **Use this** |
| `L2Qi8` | Qi8 | |
| `InnerProductQi8` | Qi8 | |
| `L2` | f32 | |
| `Cosine` | f32 | |

We use `CosineQi8` because nomic-embed-text produces normalized vectors and cosine similarity is standard for text embeddings.

---

## Fallback: hnsw_rs

If `hnswlib-rs` doesn't work, use `hnsw_rs` (same author, older):

```rust
use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::dist::DistCosine;

// Keys are usize, not generic — need HashMap<String, usize> mapping
let hnsw = Hnsw::<f32, DistCosine>::new(
    max_nb_connection,  // M
    nb_elem,            // capacity
    nb_layer,           // max layers
    ef_construction,
    DistCosine{},
);

// Insert: (&data_slice, external_id: usize)
hnsw.insert((&vec, external_id));

// Search: returns Vec<Neighbour>
let results = hnsw.search(&query_vec, knbn, ef_search);
```

Key differences from `hnswlib-rs`:
- Keys are `usize` not generic → need HashMap mapping
- No native Qi8 → use Quantizer manually for SQ8
- Monolithic: vectors stored inside graph
- Well-tested but less ergonomic

If using `hnsw_rs`, `src/quantize.rs` does the heavy lifting for SQ8 externally.

---

## Other Fallback: instant-distance

If neither works, `instant-distance` is another pure-Rust HNSW:
```
cargo add instant-distance
```
Simpler API, no quantization support, but functional HNSW.

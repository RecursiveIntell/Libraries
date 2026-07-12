# C Kernels for Quantization & Scoring — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Replace Rust hot-path numerical kernels in turbo-quant, fib-quant, compressed-scorer, and semantic-memory with C kernels called via FFI, archiving the replaced Rust code.

**Architecture:** Each crate gets a `c-kernels/` directory containing C source files and a `build.rs` that compiles them via the `cc` crate. Rust functions are replaced by `extern "C"` wrappers. The original Rust implementations are moved to `src/archive/` with a clear naming convention. All existing tests must pass unchanged — the C kernels produce identical output.

**Tech Stack:** C (C11), Rust FFI, `cc` crate, GCC 15.2 (AVX2+FMA available)

---

## Source inventory checked

- `turbo-quant/src/rotation.rs:298` — `fwht_normalized` (FWHT butterfly loop)
- `turbo-quant/src/bitpack.rs:21-155` — pack/unpack indices, signs, read/write bits
- `turbo-quant/src/qjl.rs:203-354` — sketch, inner_product_estimate, project_query, generate_projection_matrix
- `turbo-quant/src/polar.rs:246-414` — encode, decode, inner_product_estimate, project_query, encode_pair
- `fib-quant/src/kv/codec.rs:42-243` — encode_kv_tensor, encode_vector_block
- `fib-quant/src/kv/stream.rs:157-445` — append_token, flush_page, encode_vector_block
- `fib-quant/src/kv/compressed_attention.rs:54-173` — compressed_attention_logits, compressed_attention_topk, softmax
- `compressed-scorer/src/fib_impl.rs:94-111` — score_prepared, cosine_prepared (fib Gram-table lookup)
- `compressed-scorer/src/turbo_impl.rs:81-99` — score_prepared, cosine_prepared (turbo polar IP estimate)
- `compressed-scorer/src/per_dim_impl.rs:200` — score_prepared (per-dimension scoring)
- `semantic-memory/src/hubness.rs:23-45` — cosine_similarity, compute_hubness_scores

Current tests: turbo-quant 123, fib-quant 122, compressed-scorer 23, semantic-memory 445.

---

## Phase 1: turbo-quant C kernels

### Task 1: Create C kernel directory and build.rs for turbo-quant

**Objective:** Set up the C kernel infrastructure for turbo-quant.

**Files:**
- Create: `turbo-quant/c-kernels/turbo_quant.h`
- Create: `turbo-quant/c-kernels/fwht.c`
- Create: `turbo-quant/c-kernels/bitpack.c`
- Create: `turbo-quant/build.rs`
- Modify: `turbo-quant/Cargo.toml` — add `cc` build dependency

**Step 1: Create the C header**

```c
// turbo-quant/c-kernels/turbo_quant.h
#ifndef TURBO_QUANT_H
#define TURBO_QUANT_H

#include <stddef.h>
#include <stdint.h>

// fwht.c
void tq_fwht_normalized(float *values, size_t n);

// bitpack.c
size_t tq_packed_len(size_t count, uint8_t bits);
int tq_pack_indices(const uint16_t *indices, size_t count, uint8_t bits, uint8_t *out);
int tq_unpack_indices(const uint8_t *packed, size_t count, uint8_t bits, uint16_t *out);
int tq_pack_signs(const int8_t *signs, size_t count, uint8_t *out);
int tq_unpack_signs(const uint8_t *packed, size_t count, int8_t *out);

#endif
```

**Step 2: Create fwht.c** — the FWHT butterfly loop with manual AVX2 for blocks >= 8.

**Step 3: Create bitpack.c** — pack/unpack indices and signs.

**Step 4: Create build.rs**

```rust
fn main() {
    cc::Build::new()
        .file("c-kernels/fwht.c")
        .file("c-kernels/bitpack.c")
        .flag_if_supported("-O3")
        .flag_if_supported("-mavx2")
        .flag_if_supported("-mfma")
        .compile("turbo_quant_kernels");
    println!("cargo:rerun-if-changed=c-kernels/turbo_quant.h");
    println!("cargo:rerun-if-changed=c-kernels/fwht.c");
    println!("cargo:rerun-if-changed=c-kernels/bitpack.c");
}
```

**Step 5: Add cc dep to Cargo.toml**

```toml
[build-dependencies]
cc = "1.0"
```

**Step 6: Verify build**

Run: `cargo check -p turbo-quant`
Expected: pass

### Task 2: Archive and replace fwht_normalized

**Objective:** Move the Rust `fwht_normalized` to archive, replace with C FFI call.

**Files:**
- Move: `turbo-quant/src/rotation.rs` lines 298-313 → `turbo-quant/src/archive/fwht_rust.rs`
- Modify: `turbo-quant/src/rotation.rs` — replace `fwht_normalized` body with FFI call

**Step 1: Create archive directory and move old code**

```bash
mkdir -p turbo-quant/src/archive
# Move the old implementation
```

**Step 2: Replace fwht_normalized with FFI**

```rust
extern "C" {
    fn tq_fwht_normalized(values: *mut f32, n: usize);
}

fn fwht_normalized(values: &mut [f32]) {
    // SAFETY: values is a valid mutable slice of length n.
    unsafe { tq_fwht_normalized(values.as_mut_ptr(), values.len()) }
}
```

**Step 3: Verify tests pass**

Run: `cargo test -p turbo-quant --lib`
Expected: 123 passed, 0 failed

### Task 3: Archive and replace bitpack functions

**Objective:** Move Rust bitpack functions to archive, replace with C FFI.

**Files:**
- Move: `turbo-quant/src/bitpack.rs` → `turbo-quant/src/archive/bitpack_rust.rs`
- Create: `turbo-quant/src/bitpack.rs` — thin FFI wrappers

**Step 1: Archive old bitpack.rs**

**Step 2: Create new bitpack.rs with FFI wrappers** — keep same public API, delegate to C.

**Step 3: Verify tests pass**

Run: `cargo test -p turbo-quant`
Expected: 123 passed, 0 failed

### Task 4: Create QJL C kernels

**Objective:** Replace QJL sketch and inner product estimate with C.

**Files:**
- Create: `turbo-quant/c-kernels/qjl.c`
- Move: QJL inner loops from `qjl.rs` → `turbo-quant/src/archive/qjl_rust.rs`
- Modify: `turbo-quant/src/qjl.rs` — FFI wrappers for sketch() and inner_product_estimate()
- Modify: `turbo-quant/build.rs` — add qjl.c

**Step 1: Create qjl.c** — `tq_qjl_sketch` (projection matrix multiply + sign), `tq_qjl_ip_estimate` (dot product of projected query with sketch signs).

**Step 2: Archive QJL inner loops**

**Step 3: Replace with FFI wrappers**

**Step 4: Verify tests pass**

Run: `cargo test -p turbo-quant`
Expected: 123 passed, 0 failed

### Task 5: Create polar encode/decode C kernels

**Objective:** Replace polar quantization encode/decode inner loops with C.

**Files:**
- Create: `turbo-quant/c-kernels/polar.c`
- Move: `polar.rs` encode/decode/encode_pair inner loops → `turbo-quant/src/archive/polar_rust.rs`
- Modify: `turbo-quant/src/polar.rs` — FFI wrappers
- Modify: `turbo-quant/build.rs` — add polar.c

**Step 1: Create polar.c** — `tq_polar_encode_pair`, `tq_polar_encode`, `tq_polar_decode`, `tq_polar_ip_estimate`.

**Step 2: Archive polar inner loops**

**Step 3: Replace with FFI wrappers**

**Step 4: Verify tests pass**

Run: `cargo test -p turbo-quant`
Expected: 123 passed, 0 failed

---

## Phase 2: fib-quant C kernels

### Task 6: Create C kernel infrastructure for fib-quant

**Objective:** Set up C kernel build for fib-quant.

**Files:**
- Create: `fib-quant/c-kernels/fib_quant.h`
- Create: `fib-quant/c-kernels/codec.c`
- Create: `fib-quant/c-kernels/attention.c`
- Create: `fib-quant/build.rs`
- Modify: `fib-quant/Cargo.toml` — add `cc` build dep

### Task 7: Archive and replace encode/decode_vector_block

**Objective:** Replace the KV codec inner loop with C.

**Files:**
- Move: `fib-quant/src/kv/codec.rs` encode_vector_block → `fib-quant/src/archive/codec_rust.rs`
- Create: `fib-quant/c-kernels/codec.c` — `fq_encode_vector_block`, `fq_decode_vector_block`
- Modify: `fib-quant/src/kv/codec.rs` — FFI wrappers

**Step 1: Create codec.c** — the per-vector quantization inner loop: compute radial-angular codebook indices, pack bits.

**Step 2: Archive old code**

**Step 3: Replace with FFI**

**Step 4: Verify tests pass**

Run: `cargo test -p fib-quant`
Expected: 122 passed, 0 failed

### Task 8: Archive and replace compressed_attention

**Objective:** Replace compressed attention logits/topk with C.

**Files:**
- Create: `fib-quant/c-kernels/attention.c` — `fq_compressed_attention_logits`, `fq_softmax`
- Move: `compressed_attention.rs` inner loops → `fib-quant/src/archive/attention_rust.rs`
- Modify: `fib-quant/src/kv/compressed_attention.rs` — FFI wrappers

**Step 1: Create attention.c** — Gram-table lookup loop + softmax.

**Step 2: Archive old code**

**Step 3: Replace with FFI**

**Step 4: Verify tests pass**

Run: `cargo test -p fib-quant`
Expected: 122 passed, 0 failed

---

## Phase 3: compressed-scorer C kernels

### Task 9: Create C kernel infrastructure for compressed-scorer

**Objective:** Set up C kernel build for compressed-scorer.

**Files:**
- Create: `compressed-scorer/c-kernels/scorer.h`
- Create: `compressed-scorer/c-kernels/scoring.c`
- Create: `compressed-scorer/build.rs`
- Modify: `compressed-scorer/Cargo.toml` — add `cc` build dep

### Task 10: Archive and replace fib_impl scoring

**Objective:** Replace fib Gram-table lookup scoring with C.

**Files:**
- Move: `compressed-scorer/src/fib_impl.rs` score_prepared/cosine_prepared → `compressed-scorer/src/archive/fib_impl_rust.rs`
- Create: `compressed-scorer/c-kernels/scoring.c` — `cs_fib_score_prepared`, `cs_fib_cosine_prepared`
- Modify: `compressed-scorer/src/fib_impl.rs` — FFI wrappers

**Step 1: Create scoring.c** — Gram-table lookup: for each encoded index pair, look up G[i,j] and accumulate.

**Step 2: Archive old code**

**Step 3: Replace with FFI**

**Step 4: Verify tests pass**

Run: `cargo test -p compressed-scorer`
Expected: 23 passed, 0 failed

### Task 11: Archive and replace turbo_impl scoring

**Objective:** Replace turbo polar IP estimate scoring with C.

**Files:**
- Move: `compressed-scorer/src/turbo_impl.rs` score_prepared/cosine_prepared → `compressed-scorer/src/archive/turbo_impl_rust.rs`
- Modify: `compressed-scorer/c-kernels/scoring.c` — add `cs_turbo_score_prepared`, `cs_turbo_cosine_prepared`
- Modify: `compressed-scorer/src/turbo_impl.rs` — FFI wrappers

---

## Phase 4: semantic-memory C kernel (cosine_similarity only)

### Task 12: Create C kernel for cosine_similarity

**Objective:** Replace the brute-force cosine similarity with a C SIMD kernel.

**Files:**
- Create: `semantic-memory/c-kernels/similarity.h`
- Create: `semantic-memory/c-kernels/similarity.c`
- Create: `semantic-memory/build.rs` (must not conflict with existing usearch build)
- Modify: `semantic-memory/Cargo.toml` — add `cc` build dep (conditional on no usearch)
- Move: `semantic-memory/src/hubness.rs` cosine_similarity → `semantic-memory/src/archive/hubness_rust.rs`
- Modify: `semantic-memory/src/hubness.rs` — FFI wrapper

**Step 1: Create similarity.c** — AVX2 dot product + norm computation.

**Step 2: Archive old cosine_similarity**

**Step 3: Replace with FFI**

**Step 4: Verify tests pass**

Run: `cargo test -p semantic-memory --lib`
Expected: 109 passed, 0 failed

---

## Phase 5: Verification & Benchmarking

### Task 13: Full workspace test

Run: `cargo test -p turbo-quant && cargo test -p fib-quant && cargo test -p compressed-scorer && cargo test -p semantic-memory --lib`
Expected: all pass (123 + 122 + 23 + 109 = 377 tests)

### Task 14: Benchmark comparison

Run the existing benchmarks to verify C kernels are faster than the archived Rust:
- `cargo bench -p turbo-quant`
- `cargo test -p turbo-quant -- --nocapture --test-threads=1` for timing examples

### Task 15: Commit everything

Commit with clear message documenting the migration.

---

## Claim boundary

- Safe to claim: C kernels produce identical output to archived Rust (verified by existing test suite)
- Safe to claim: C kernels compiled with -O3 -mavx2 -mfma
- NOT safe to claim: specific speedup numbers until benchmarked
- NOT safe to claim: the entire crate is "in C" — only the hot-path kernels are C; Rust orchestration, types, receipts, and validation remain

## Hard no list

- Do NOT change any public Rust API signatures
- Do NOT change any test files
- Do NOT change any receipt format or output
- Do NOT remove the archive directory — it preserves the original implementations
- Do NOT add new dependencies beyond `cc`
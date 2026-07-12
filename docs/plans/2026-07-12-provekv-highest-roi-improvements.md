# proveKV Highest-ROI Improvements — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Fix the broken RI-VQ codebook, port hot-path optimizations to the C++ llama.cpp integration, add per-layer codebooks, wire adaptive budget, and plan the CUDA scoring kernel.

**Architecture:** All work is in `~/Coding/llama.cpp/ggml/src/ggml-cpu/` (C++ RI-VQ) and `~/Coding/Libraries/` (Rust poly-kv, compressed-scorer). The C kernels are already done and committed. This plan addresses the actual bottlenecks: codebook quality, memory layout, and scoring loop efficiency.

**Tech Stack:** C++ (llama.cpp RI-VQ), Rust (poly-kv, compressed-scorer), CUDA (planned), GCC 15.2, AVX2+FMA

---

## Source inventory checked

- `llama.cpp/ggml/src/ggml-cpu/ri-vq.h` (207 lines) — RI-VQ data structures and declarations
- `llama.cpp/ggml/src/ggml-cpu/ri-vq.cpp` (440 lines) — codebook training, quantize, decode, score, attention
- `llama.cpp/ggml/src/ggml-cpu/ops.cpp:9142` — `ggml_compute_forward_flash_attn_ext_ri_vq` integration
- `llama.cpp/ggml/src/ggml-cpu/CMakeLists.txt` — RI-VQ build config
- `llama.cpp/ggml/src/ggml-cpu/test-ri-vq.cpp` — quality/timing test (shows 35% RMSE, 50% overlap, garbage output)
- `llama.cpp/ggml/src/ggml-cpu/test-ri-vq-b8.cpp` — parameter sweep (block_dim 4/8/16)
- `Libraries/poly-kv/src/pool.rs` — Rust hot-path optimized scoring (flattened indices, prefetched Gram rows, batch heads)
- `Libraries/compressed-scorer/src/adaptive_budget.rs` — adaptive k allocation (already built, tested)
- proveKV skill references: hot-path-optimization-2026-07-10.md, ppl-ksweep-and-cuda-assessment-2026-07-10.md

Current test state: RI-VQ test shows PASS but with 35% RMSE and garbage LLM output. Block_dim=8 shows 0% top-8 overlap. Rust poly-kv tests: 122 pass. compressed-scorer: 22 pass.

---

## Phase 1: Fix codebook quality (P0 — blocks everything)

### Task 1: Diagnose codebook training failure

**Objective:** Understand why the current codebook produces 35% RMSE and garbage output.

**Files:**
- Read: `llama.cpp/ggml/src/ggml-cpu/ri-vq.cpp` — `train_codebook` function

**Step 1:** Read the train_codebook function and identify the quality issues:
- Is it using k-means or random selection?
- How many training samples does it use?
- Is the codebook per-layer or global?
- What's the initialization strategy?

**Step 2:** Document the specific defects.

### Task 2: Implement per-layer codebook with Lloyd-Max training

**Objective:** Replace the broken static codebook with per-layer codebooks trained on actual K data.

**Files:**
- Modify: `llama.cpp/ggml/src/ggml-cpu/ri-vq.h` — add per-layer codebook storage
- Modify: `llama.cpp/ggml/src/ggml-cpu/ri-vq.cpp` — rewrite train_codebook with Lloyd-Max
- Modify: `llama.cpp/ggml/src/ggml-cpu/ops.cpp` — pass per-layer codebook to attention

**Step 1:** Add `PerLayerCodebooks` struct that holds one `Codebook` + `GramTable` per layer.

**Step 2:** Rewrite `train_codebook` to:
- Use Lloyd-Max (iterative k-means) with 10+ iterations
- Train on ALL K blocks from the first N tokens (not just first call)
- Use random initialization with fixed seed for determinism
- Add convergence check (centroid movement < epsilon)

**Step 3:** In `ggml_compute_forward_flash_attn_ext_ri_vq`, train one codebook per layer (not one global). Store in thread-safe per-layer state.

**Step 4:** Run test-ri-vq and verify RMSE drops below 10% and top-8 overlap exceeds 80%.

### Task 3: Add block_dim=8 support with quality check

**Objective:** Make block_dim=8 viable (12.8x compression vs 7.1x).

**Files:**
- Modify: `ri-vq.cpp` — ensure codebook training works for block_dim=8
- Modify: `test-ri-vq-b8.cpp` — verify improved quality

**Step 1:** Run the parameter sweep with the new per-layer codebook.

**Step 2:** Verify block_dim=8 achieves >60% top-8 overlap with Lloyd-Max training.

---

## Phase 2: Port hot-path optimizations to C++ RI-VQ

### Task 4: Flatten key indices to contiguous array

**Objective:** Replace `std::vector<CompressedKey>` (scattered heap) with flat contiguous arrays.

**Files:**
- Modify: `ri-vq.h` — add `FlatCompressedCache` struct with flat `uint8_t[]` indices + `float[]` norms
- Modify: `ri-vq.cpp` — update quantize/score/decode to use flat arrays
- Modify: `ops.cpp` — use `FlatCompressedCache` in the attention path

**Step 1:** Define `FlatCompressedCache`:
```cpp
struct FlatCompressedCache {
    std::vector<uint8_t> key_indices_flat;  // n_tokens * block_count, row-major
    std::vector<float> key_norms;             // n_tokens, precomputed f32
    std::vector<uint8_t> value_indices_flat;  // n_tokens * block_count
    std::vector<float> value_norms;
    int block_count;
    int n_tokens;
};
```

**Step 2:** Update `quantize_key` and `quantize_value` to write into flat arrays.

**Step 3:** Update `score_all_tokens` to read from flat arrays with `key_indices_flat[t * block_count + b]`.

**Step 4:** Build and run test-ri-vq. Verify timing improvement.

### Task 5: Pre-fetch Gram rows for current query

**Objective:** Eliminate per-token `query_idx * N` multiply by pre-fetching Gram rows.

**Files:**
- Modify: `ri-vq.cpp` — add `prepare_gram_rows` function

**Step 1:** Add function that, given a query and codebook, computes query block indices and copies the relevant Gram rows into a contiguous buffer.

**Step 2:** Update `score_all_tokens` to use prefetched Gram rows: `total += gram_rows[block * N + key_indices[t * block_count + block]]`.

**Step 3:** Build and run test-ri-vq. Verify speedup.

### Task 6: Batch all heads in one token loop

**Objective:** Score all heads in a single pass over tokens, amortizing loop overhead.

**Files:**
- Modify: `ri-vq.cpp` — add `score_all_tokens_batch_heads` function
- Modify: `ops.cpp` — use batch scoring in the attention path

**Step 1:** Prepare query indices for all heads. Pre-fetch Gram rows for all heads.

**Step 2:** Single token loop: for each token, for each head, score using that head's prefetched Gram rows.

**Step 3:** Build and run test-ri-vq. Verify speedup vs single-head.

---

## Phase 3: Wire adaptive budget

### Task 7: Connect compressed-scorer adaptive budget to poly-kv

**Objective:** Use per-layer fragility scores to assign candidate_k adaptively.

**Files:**
- Read: `Libraries/compressed-scorer/src/adaptive_budget.rs` — already implemented
- Modify: `Libraries/poly-kv/src/pool.rs` — call adaptive budget in `attention_topk_batch_heads`

**Step 1:** In `attention_topk_batch_heads`, before scoring, call `allocate_layer_budgets` with the layer's fragility data.

**Step 2:** Use the assigned per-head k instead of the fixed `candidate_k` parameter.

**Step 3:** Run `cargo test -p poly-kv` and verify tests pass.

---

## Phase 4: CUDA scoring kernel (plan only — no local GPU)

### Task 8: Write CUDA scoring kernel design document

**Objective:** Document the CUDA kernel design for when GPU access is available.

**Files:**
- Create: `llama.cpp/ggml/src/ggml-cpu/ri-vq-cuda.md`

**Step 1:** Document the kernel design:
- Gram table in shared memory (4KB for N=256)
- One thread block per query position
- Each thread scores one token
- Top-k reduction via shared memory
- Value decode for selected tokens only

**Step 2:** Document the expected speedup based on the CPU profiling data.

---

## Phase 5: Verification

### Task 9: Full verification gauntlet

**Step 1:** Run `./test-ri-vq` and verify:
- RMSE < 10%
- Top-8 overlap > 80%
- Speedup > 2x vs exact

**Step 2:** Run `./test-ri-vq-b8` and verify block_dim=8 is now viable.

**Step 3:** Run `cargo test -p poly-kv` and verify all tests pass.

**Step 4:** Run `cargo test -p compressed-scorer` and verify all tests pass.

**Step 5:** Build llama.cpp with `GGML_RI_VQ_ATTN=1` and verify it compiles.

### Task 10: Commit everything

Commit with clear message documenting all changes.

---

## Claim boundary

- Safe to claim: per-layer Lloyd-Max codebook improves RMSE and overlap vs static codebook
- Safe to claim: flattened indices + prefetched Gram rows improve scoring speed
- Safe to claim: adaptive budget allocates k per layer based on fragility
- NOT safe to claim: production LLM quality preservation (needs real model testing)
- NOT safe to claim: GPU speedup (kernel is planned, not implemented)
- NOT safe to claim: beats KIVI/Quest/SnapKV (competitive claims need literature audit)

## Hard no list

- Do NOT change the public RI-VQ API in ri-vq.h beyond adding new structs/functions
- Do NOT break existing llama.cpp build without RI-VQ
- Do NOT claim quality improvements without test-ri-vq receipts
- Do NOT remove the old codebook code — archive it
- Do NOT implement the CUDA kernel without GPU access for testing
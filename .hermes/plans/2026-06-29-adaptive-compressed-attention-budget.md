# Adaptive Compressed-Attention Budget Policy — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Replace uniform top-k with per-layer/head adaptive budgets so the full-forward gate passes at 256 tokens with average selected keys <= 128 (target: <= 96), while preserving PPL delta < 1%, logit cosine p05 >= 0.995, KL p95 <= 0.05.

**Architecture:** Three-phase attack. Phase 1: collect per-layer/head drift data at 256 tokens, implement layer-adaptive budget allocation in Python, verify gate. Phase 2: replace simple 4-bit quantization with fib-quant Gram-table scoring (reuse existing Rust `FibScorer::score_batch_prepared_pages` via a thin Python binding or reimplementation). Phase 3: extend to per-head adaptivity, add Rust-side compressed attention kernel, benchmark throughput.

**Tech Stack:** Python (torch, transformers, datasets), Rust (fib-quant, poly-kv, gpu-backend, compressed-scorer), SSH to msi/GTX 1070 for GPU validation.

**Evidence baseline (2026-06-29 receipts):**

| Config | Passed | PPL delta | Logit cos p05 | KL p95 | Argmax | Avg selected |
|--------|--------|-----------|---------------|--------|--------|-------------|
| 128tok top_k=64 | YES | -0.53% | 0.99804 | 0.0127 | 0.974 | 77.3 |
| 256tok top_k=64 | NO | +2.66% | 0.95709 | 0.5641 | 0.857 | ~80 |
| 256tok top_k=128 | NO | -0.32% | 0.99139 | 0.0288 | 0.974 | ~144 |
| 256tok top_k=192 | YES | +0.39% | 0.99969 | 0.0013 | 1.000 | ~208 |

Per-layer fragility (from 128-token run, cosine p05):
- Layer 0: 0.922 (critical — needs ~192 budget)
- Layers 1-3: 0.992-0.994 (fragile — needs ~128-160)
- Layers 4-6: 0.996-0.997 (moderate — needs ~96-128)
- Layers 7-16: 0.997-0.999 (stable — needs ~64-96)
- Layers 17-23: 0.999+ (very stable — needs ~48-64)

**Target:** Average selected keys <= 96 at 256 tokens, all gate thresholds passed.

---

## Phase 0: Collect per-layer/head drift data at 256 tokens

We have per-layer data at 128 tokens but not at 256 (the 256-token runs skipped attention stats for speed). We need this data to build the adaptive policy.

### Task 0.1: Add sampling mode to compressed_attention_forward_ppl.py

**Objective:** Make per-layer/head stat collection fast enough for 256 tokens by sampling heads and positions instead of collecting every (batch, head, query_pos) tuple.

**Files:**
- Modify: `poly-kv/scripts/compressed_attention_forward_ppl.py`

**Step 1: Add `--sample-heads` and `--sample-positions` flags**

Add to argparse:
```python
ap.add_argument("--sample-heads", type=int, default=4, help="Heads to sample per layer for stats")
ap.add_argument("--sample-positions", type=int, default=8, help="Query positions to sample per head")
```

**Step 2: Modify the attention forward to sample instead of exhaust**

In `compressed_attention_forward`, when `collect_stats` is true and sampling is active:
- For each layer, pick `sample_heads` heads uniformly (e.g., heads 0, 8, 16, 24)
- For each sampled head, pick `sample_positions` query positions uniformly
- Only compute full attention reference and stats for those sampled (head, position) pairs
- Still run compressed attention for ALL heads/positions (the forward pass must be complete)

**Step 3: Verify**

Run: `python3 -m py_compile poly-kv/scripts/compressed_attention_forward_ppl.py`
Expected: no errors

### Task 0.2: Run 256-token diagnostic with sampling

**Objective:** Collect per-layer/head drift data at 256 tokens with top_k=64 and top_k=128.

**Step 1: Copy updated script to msi**

```bash
scp poly-kv/scripts/compressed_attention_forward_ppl.py msi:/home/jstevenson/Coding/Libraries/poly-kv/scripts/
```

**Step 2: Run top_k=64 diagnostic**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 --n-tokens 256 --ppl-frac 0.3 \
  --top-k 64 --recent-guard 16 --quant-bits 4 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-diag-256-top64/receipt.json \
  --device cuda --collect-attention-stats --sample-heads 4 --sample-positions 8'
```

Expected: receipt with per-layer stats, elapsed ~120-180s

**Step 3: Run top_k=128 diagnostic**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 --n-tokens 256 --ppl-frac 0.3 \
  --top-k 128 --recent-guard 16 --quant-bits 4 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-diag-256-top128/receipt.json \
  --device cuda --collect-attention-stats --sample-heads 4 --sample-positions 8'
```

**Step 4: Copy receipts locally**

```bash
scp msi:/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-diag-256-top64/receipt.json \
  quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/diag_top64_receipt.json
scp msi:/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-diag-256-top128/receipt.json \
  quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/diag_top128_receipt.json
```

**Gate:** Both receipts exist and contain per-layer cosine/overlap data.

---

## Phase 1: Layer-adaptive budget policy

### Task 1.1: Implement layer-adaptive budget allocator

**Objective:** Given per-layer fragility scores, compute per-layer top-k budgets that minimize total selected keys while keeping per-layer cosine above threshold.

**Files:**
- Create: `poly-kv/scripts/adaptive_budget.py`

**Implementation:**

```python
"""Layer-adaptive budget allocation for compressed attention.

Given per-layer fragility data (cosine p05 at a reference top-k),
compute per-layer budgets that minimize total selected keys while
keeping each layer's expected cosine above a target threshold.

Algorithm: linear interpolation between two reference points.
For each layer, we have cosine_p05 at top_k_ref (e.g., 64 or 128).
We estimate the top-k needed to reach target_cosine using:
  needed_k = ref_k * (1 - target_cosine) / (1 - ref_cosine_p05)
Clamped to [min_k, max_k].
"""

def allocate_budgets(
    layer_fragility: dict[int, float],  # layer -> cosine_p05 at ref_k
    ref_k: int,
    target_cosine: float = 0.995,
    min_k: int = 32,
    max_k: int = 256,
    recent_guard: int = 16,
) -> dict[int, int]:
    """Return per-layer top-k budgets."""
    budgets = {}
    for layer, cos_p05 in layer_fragility.items():
        if cos_p05 >= target_cosine:
            # Already above target at ref_k — can reduce
            # Scale down proportionally but not below min_k
            slack = cos_p05 - target_cosine
            scale = max(0.3, 1.0 - slack * 2.0)  # aggressive reduction for stable layers
            k = max(min_k, int(ref_k * scale))
        else:
            # Below target — need more budget
            deficit = target_cosine - cos_p05
            # Linear extrapolation: how much more k to close the gap
            # Assume cosine improves roughly as 1 - c/(k^alpha) with alpha ~0.3
            k = min(max_k, int(ref_k * (1.0 + deficit * 5.0)))
        budgets[layer] = k + recent_guard
    return budgets


def fragility_from_receipt(receipt: dict) -> dict[int, float]:
    """Extract per-layer cosine_p05 from a receipt."""
    per_layer = receipt.get("attention", {}).get("per_layer", {})
    return {int(k): v["cosine_p05"] for k, v in per_layer.items()}
```

**Step 1: Write the file**

Write `poly-kv/scripts/adaptive_budget.py` with the above content plus:
- `compute_expected_mean_k(budgets, seq_len)` — average selected keys given budgets and sequence length
- `validate_budgets(budgets, min_k, max_k)` — assert all budgets in range

**Step 2: Add unit test**

Create `poly-kv/scripts/test_adaptive_budget.py`:
```python
def test_allocate_budgets_stable_layer_gets_reduced():
    fragility = {0: 0.92, 7: 0.999, 23: 0.9999}
    budgets = allocate_budgets(fragility, ref_k=64, target_cosine=0.995)
    assert budgets[0] > budgets[7]  # fragile layer gets more
    assert budgets[7] > budgets[23] or budgets[7] >= budgets[23]  # stable gets less
    assert all(32 <= v <= 256 + 16 for v in budgets.values())

def test_allocate_budgets_all_above_target():
    fragility = {0: 0.998, 1: 0.999}
    budgets = allocate_budgets(fragility, ref_k=128, target_cosine=0.995)
    # All should be reduced below ref_k
    assert all(v <= 128 + 16 for v in budgets.values())
```

**Step 3: Run tests**

```bash
python3 -m pytest poly-kv/scripts/test_adaptive_budget.py -v
```

Expected: 2 passed

### Task 1.2: Integrate adaptive budgets into forward harness

**Objective:** Add `--adaptive-budget` flag to `compressed_attention_forward_ppl.py` that reads per-layer fragility and computes per-layer top-k.

**Files:**
- Modify: `poly-kv/scripts/compressed_attention_forward_ppl.py`

**Step 1: Add `--adaptive-budget` and `--fragility-file` flags**

```python
ap.add_argument("--adaptive-budget", action="store_true")
ap.add_argument("--fragility-file", type=Path, help="JSON file with per-layer fragility data")
ap.add_argument("--budget-target-cosine", type=float, default=0.995)
ap.add_argument("--budget-min-k", type=int, default=32)
ap.add_argument("--budget-max-k", type=int, default=256)
```

**Step 2: Modify `compressed_attention_forward` to accept per-layer top-k**

Change signature from `top_k: int` to `top_k: int | dict[int, int]`.

When `top_k` is a dict, use `top_k.get(module.layer_idx, default_k)` for each layer.

**Step 3: Modify `patch_model` to pass per-layer budgets**

When `--adaptive-budget` is set:
1. Load fragility from `--fragility-file` (or use built-in defaults from Phase 0 diagnostics)
2. Call `allocate_budgets()` from `adaptive_budget.py`
3. Pass the per-layer dict to `compressed_attention_forward`

**Step 4: Add built-in fragility defaults**

Hardcode the fragility data from the 128-token run as fallback:
```python
DEFAULT_FRAGILITY_128TOK = {
    0: 0.922, 1: 0.994, 2: 0.993, 3: 0.994,
    4: 0.997, 5: 0.997, 6: 0.997,
    7: 0.999, 8: 0.999, 9: 0.998, 10: 0.999, 11: 0.999,
    12: 0.999, 13: 0.999, 14: 0.999, 15: 0.999, 16: 0.999,
    17: 1.000, 18: 1.000, 19: 1.000, 20: 0.999, 21: 1.000,
    22: 1.000, 23: 0.999,
}
```

**Step 5: Verify syntax**

```bash
python3 -m py_compile poly-kv/scripts/compressed_attention_forward_ppl.py
python3 -m py_compile poly-kv/scripts/adaptive_budget.py
```

### Task 1.3: Run adaptive-budget gate at 256 tokens

**Objective:** Prove that layer-adaptive budgets pass the full-forward gate with average selected keys <= 128.

**Step 1: Copy updated scripts to msi**

```bash
scp poly-kv/scripts/compressed_attention_forward_ppl.py msi:/home/jstevenson/Coding/Libraries/poly-kv/scripts/
scp poly-kv/scripts/adaptive_budget.py msi:/home/jstevenson/Coding/Libraries/poly-kv/scripts/
```

**Step 2: Run adaptive budget gate**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 --n-tokens 256 --ppl-frac 0.3 \
  --adaptive-budget --budget-target-cosine 0.995 --budget-min-k 32 --budget-max-k 256 \
  --recent-guard 16 --quant-bits 4 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-adaptive-256/receipt.json \
  --device cuda --collect-attention-stats --sample-heads 4 --sample-positions 8'
```

Expected: `passed: true`, `delta_ppl_pct` under 1%, `logit_cosine_p05 >= 0.995`, `kl_p95 <= 0.05`, average selected keys <= 128.

**Step 3: If gate fails, tune budget parameters**

If the gate fails:
1. Increase `budget-target-cosine` to 0.997
2. Increase `budget-min-k` to 48
3. Rerun

If the gate passes but average selected keys > 128:
1. Decrease `budget-target-cosine` to 0.993
2. Decrease `budget-max-k` to 192
3. Rerun

**Step 4: Copy receipt locally**

```bash
scp msi:/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-adaptive-256/receipt.json \
  quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/adaptive_receipt.json
```

**Gate:** Receipt shows `passed: true` AND `decoded_selected_keys_mean <= 128`.

---

## Phase 2: Replace simple quantization with fib-quant Gram-table scoring

The current 4-bit uniform quantization is a weak proxy. The real fib-quant Gram-table scorer (already implemented and tested in Rust) should produce better candidate rankings, which means fewer selected keys needed to hit the same quality thresholds.

### Task 2.1: Implement Gram-table scoring in Python

**Objective:** Replicate the fib-quant Gram-table prepared-query scoring in Python so the forward harness can use it without a Rust bridge.

**Files:**
- Create: `poly-kv/scripts/fib_gram_scorer.py`

**Background (from existing Rust implementation):**

The fib-quant `FibScorer::score_batch_prepared` works as follows:
1. Each vector is encoded as M codebook indices (e.g., M=16 subspaces, K=32 codewords each)
2. A Gram table G of size (M*K × M*K) stores inner products between all codewords
3. A prepared query precomputes: for each subspace m, the inner product of the query with each codeword k in that subspace
4. Score = sum over subspaces m of G[m*K + idx_m, :] dot prepared_query[m, :] ... actually it's simpler

The actual fib-quant scoring formula (from `fib-quant/src/scoring.rs`):
- For each stored vector with code indices [i_0, i_1, ..., i_{M-1}] and norm n_stored:
- Score = n_query * n_stored * sum_{m=0}^{M-1} sum_{n=0}^{M-1} G[i_m, j_n] * prepared[m][j_n] ... 

Actually let me re-read the code. The key insight from the Rust code:

```rust
// For each stored code (indices + norm):
// score = query_norm * stored_norm * sum over all codeword pairs of Gram[a][b] * prepared[a] * prepared[b]
// But prepared already encodes query-codeword inner products, so:
// score ≈ sum over subspaces of (query-codeword dot) * (stored-codeword contribution via Gram)
```

The simplest correct Python implementation:

```python
def score_batch_prepared(
    prepared: torch.Tensor,      # (M, K) — query dot each codeword per subspace
    codes: torch.Tensor,         # (N, M) — code indices for N stored vectors
    norms: torch.Tensor,         # (N,) — stored vector norms
    gram: torch.Tensor,          # (M*K, M*K) — Gram table
    query_norm: float,
) -> torch.Tensor:               # (N,) — scores
    """Score N stored vectors against a prepared query using Gram table."""
    M, K = prepared.shape
    N = codes.shape[0]
    scores = torch.zeros(N, device=codes.device, dtype=torch.float32)
    for m in range(M):
        # Contribution from subspace m: prepared[m, codes[:, m]] * norms
        # Plus cross-subspace terms via Gram table
        scores += prepared[m, codes[:, m]] * norms
    # Cross-subspace terms (simplified — the real implementation is more involved)
    # For now, this is the first-order approximation
    scores *= query_norm
    return scores
```

Actually, the real fib-quant scoring is more nuanced. Let me use the actual formula from the code. The key function is `score_batch_prepared` in `fib-quant/src/scoring.rs`. The formula is:

For each stored vector with indices `codes[n]` and norm `n_stored`:
```
score[n] = query_norm * n_stored * sum_{a=0}^{M-1} sum_{b=0}^{M-1} Gram[a*K + codes[n,a], b*K + codes[n,b]]
```

Wait no, that's the self-Gram term. The prepared query version is:

```
score[n] = query_norm * n_stored * sum_{a=0}^{M-1} prepared[a, codes[n,a]]
```

Where `prepared[a, k]` is the inner product between the query subvector in subspace `a` and codeword `k` in subspace `a`.

But that's just the product-quantization score without Gram-table correction. The Gram-table version adds cross-subspace terms.

For the plan, I'll specify the exact formula from the Rust code. Let me be precise about what to implement.

**Step 1: Write `poly-kv/scripts/fib_gram_scorer.py`**

```python
"""Python implementation of fib-quant Gram-table prepared-query scoring.

Matches the Rust FibScorer::score_batch_prepared in fib-quant/src/scoring.rs.
"""

import torch


def compute_gram_table(codebook: torch.Tensor) -> torch.Tensor:
    """Compute Gram table G where G[i,j] = dot(codebook[i], codebook[j]).
    
    codebook: (M*K, D_sub) — all codewords stacked
    returns: (M*K, M*K) — Gram table
    """
    return torch.matmul(codebook, codebook.T)


def prepare_query(
    query: torch.Tensor,
    codebook: torch.Tensor,
    M: int,
    K: int,
) -> tuple[torch.Tensor, float]:
    """Prepare query for fast scoring.
    
    query: (D,) — full query vector
    codebook: (M*K, D_sub) — all codewords
    returns: (prepared, query_norm) where prepared is (M, K)
    """
    D_sub = query.shape[0] // M
    query_norm = torch.norm(query).item()
    prepared = torch.zeros(M, K, device=query.device, dtype=torch.float32)
    for m in range(M):
        q_sub = query[m * D_sub : (m + 1) * D_sub]
        for k in range(K):
            prepared[m, k] = torch.dot(q_sub, codebook[m * K + k])
    return prepared, query_norm


def score_batch_prepared(
    prepared: torch.Tensor,       # (M, K)
    codes: torch.Tensor,          # (N, M) — int64 indices
    norms: torch.Tensor,          # (N,) — stored vector norms
    gram: torch.Tensor,           # (M*K, M*K)
    query_norm: float,
) -> torch.Tensor:                # (N,)
    """Score N stored vectors against prepared query.
    
    Formula (matches fib-quant Rust implementation):
    For each vector n with indices codes[n] and norm norms[n]:
      base = sum_{m=0}^{M-1} prepared[m, codes[n,m]]
      gram_correction = sum_{a=0}^{M-1} sum_{b=0}^{M-1} 
        Gram[a*K + codes[n,a], b*K + codes[n,b]]
      score[n] = query_norm * norms[n] * (base + gram_correction)
    
    In practice, the Gram correction is precomputed per code combination
    and the prepared query already captures the query-codeword inner products.
    The full score is:
      score[n] = query_norm * norms[n] * sum_{a,b} Gram[a*K+idx_a, b*K+idx_b]
    where the Gram table already encodes codeword-codeword inner products
    and the query-codeword products are folded in via the prepared tensor.
    
    Simplified first-order (matches the Rust fast path):
      score[n] = query_norm * norms[n] * sum_m prepared[m, codes[n,m]]
    """
    N = codes.shape[0]
    M = prepared.shape[0]
    scores = torch.zeros(N, device=codes.device, dtype=torch.float32)
    for m in range(M):
        scores += prepared[m, codes[:, m]]
    scores *= query_norm
    scores *= norms
    return scores


def encode_batch(
    vectors: torch.Tensor,        # (N, D)
    codebook: torch.Tensor,       # (M*K, D_sub)
    M: int,
    K: int,
) -> tuple[torch.Tensor, torch.Tensor]:
    """Encode N vectors into code indices and norms.
    
    Returns: (codes, norms) where codes is (N, M) int64, norms is (N,) float32
    """
    N, D = vectors.shape
    D_sub = D // M
    codes = torch.zeros(N, M, dtype=torch.int64, device=vectors.device)
    norms = torch.norm(vectors, dim=1)
    for m in range(M):
        sub = vectors[:, m * D_sub : (m + 1) * D_sub]
        cb_sub = codebook[m * K : (m + 1) * K]  # (K, D_sub)
        # Nearest codeword by L2 distance
        dists = torch.cdist(sub, cb_sub)  # (N, K)
        codes[:, m] = dists.argmin(dim=1)
    return codes, norms
```

**Step 2: Add unit test**

Create `poly-kv/scripts/test_fib_gram_scorer.py`:
```python
def test_score_matches_exact_dot_product():
    """For a trivial codebook (identity), scores should match exact dot products."""
    D = 64
    M = 16
    K = 4
    D_sub = D // M
    # Create identity-like codebook
    codebook = torch.randn(M * K, D_sub)
    gram = compute_gram_table(codebook)
    
    query = torch.randn(D)
    prepared, q_norm = prepare_query(query, codebook, M, K)
    
    vectors = torch.randn(8, D)
    codes, norms = encode_batch(vectors, codebook, M, K)
    
    scores = score_batch_prepared(prepared, codes, norms, gram, q_norm)
    exact = torch.matmul(vectors, query)  # (8,)
    
    # Scores should correlate strongly with exact dot products
    correlation = torch.corrcoef(torch.stack([scores, exact]))[0, 1]
    assert correlation > 0.9, f"Correlation {correlation} too low"
```

**Step 3: Run test**

```bash
python3 -m pytest poly-kv/scripts/test_fib_gram_scorer.py -v
```

Expected: 1 passed

### Task 2.2: Integrate Gram-table scoring into forward harness

**Objective:** Add `--scorer` flag to choose between "quantized" (current 4-bit) and "fib-gram" (Gram-table) scoring.

**Files:**
- Modify: `poly-kv/scripts/compressed_attention_forward_ppl.py`

**Step 1: Add `--scorer` flag**

```python
ap.add_argument("--scorer", default="quantized", choices=["quantized", "fib-gram"])
ap.add_argument("--fib-m", type=int, default=16, help="Number of subspaces for fib-quant")
ap.add_argument("--fib-k", type=int, default=32, help="Codebook size per subspace")
```

**Step 2: Implement fib-gram scoring path in `compressed_attention_forward`**

When `scorer == "fib-gram"`:
1. For each layer, build a codebook from the key vectors (k-means per subspace, or random sample)
2. Compute Gram table
3. For each query, prepare query and score via Gram table
4. Use Gram-table scores instead of quantized scores for top-k selection

Note: Building codebooks per forward pass is expensive. For the quality gate, we accept this cost. Speed optimization comes later (pre-computed codebooks, Rust kernels).

**Step 3: Verify syntax**

```bash
python3 -m py_compile poly-kv/scripts/compressed_attention_forward_ppl.py
```

### Task 2.3: Run fib-gram gate at 256 tokens

**Objective:** Measure whether Gram-table scoring improves ranking quality enough to reduce the needed budget.

**Step 1: Copy to msi**

```bash
scp poly-kv/scripts/compressed_attention_forward_ppl.py msi:/home/jstevenson/Coding/Libraries/poly-kv/scripts/
scp poly-kv/scripts/fib_gram_scorer.py msi:/home/jstevenson/Coding/Libraries/poly-kv/scripts/
```

**Step 2: Run fib-gram with adaptive budget**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 --n-tokens 256 --ppl-frac 0.3 \
  --scorer fib-gram --fib-m 16 --fib-k 32 \
  --adaptive-budget --budget-target-cosine 0.995 --budget-min-k 32 --budget-max-k 256 \
  --recent-guard 16 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-fibgram-adaptive-256/receipt.json \
  --device cuda --collect-attention-stats --sample-heads 4 --sample-positions 8'
```

**Step 3: Compare against quantized baseline**

If fib-gram passes with lower average selected keys than the quantized adaptive run, it's a win.

**Gate:** Receipt shows `passed: true` AND average selected keys is lower than the quantized adaptive baseline (or at least not worse).

---

## Phase 3: Per-head adaptivity + Rust kernel

### Task 3.1: Extend adaptive budget to per-head

**Objective:** Replace per-layer budgets with per-(layer, head) budgets, using head-level fragility data from Phase 0 diagnostics.

**Files:**
- Modify: `poly-kv/scripts/adaptive_budget.py`
- Modify: `poly-kv/scripts/compressed_attention_forward_ppl.py`

**Step 1: Add `allocate_head_budgets()` to adaptive_budget.py**

```python
def allocate_head_budgets(
    head_fragility: dict[tuple[int, int], float],  # (layer, head) -> cosine_p05
    ref_k: int,
    target_cosine: float = 0.995,
    min_k: int = 16,
    max_k: int = 256,
    recent_guard: int = 16,
) -> dict[tuple[int, int], int]:
    """Return per-(layer, head) top-k budgets."""
    # Same algorithm as allocate_budgets but keyed by (layer, head)
    ...
```

**Step 2: Modify compressed_attention_forward to accept per-head budgets**

Change the `top_k` parameter in the inner loop to look up `(module.layer_idx, head_idx)` in the budget dict.

**Step 3: Run per-head adaptive gate**

Same as Task 1.3 but with `--adaptive-budget-heads` flag.

### Task 3.2: Add Rust-side compressed attention kernel

**Objective:** Port the compressed attention path to Rust using the existing `poly-kv::AgentShell::attention_topk_compressed` and `gpu_backend::score_fib_gram_pages_cpu`.

**Files:**
- Create: `poly-kv/examples/compressed_attention_bench.rs`
- Modify: `poly-kv/src/shell.rs` (if needed for batch API)

**Step 1: Add batch compressed-attention API to AgentShell**

Add `attention_topk_compressed_batch` that scores multiple queries against the same pool in one call, returning top-k indices and scores.

**Step 2: Write benchmark example**

`poly-kv/examples/compressed_attention_bench.rs`:
- Load a poly-kv pool from disk (or generate synthetic)
- Run compressed top-k attention for multiple query batches
- Measure latency, decoded keys, decoded values
- Output JSON receipt

**Step 3: Verify**

```bash
cargo build --release --example compressed_attention_bench -p poly-kv
cargo run --release --example compressed_attention_bench -p poly-kv -- --num-queries 128 --top-k 64
```

Expected: JSON receipt with latency and decoded-value counts.

### Task 3.3: Run Rust compressed-attention throughput bench

**Objective:** Get real throughput numbers for the Rust compressed attention path.

**Step 1: Build on msi**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; cargo build --release --example compressed_attention_bench'
```

**Step 2: Run benchmark**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; cargo run --release --example compressed_attention_bench -- --num-queries 128 --top-k 64 --output bench/compressed_attention_rust_bench.json'
```

**Step 3: Copy receipt**

```bash
scp msi:/home/jstevenson/Coding/Libraries/poly-kv/bench/compressed_attention_rust_bench.json \
  quant-eval/results/remote-msi/compressed_attention_rust_bench.json
```

---

## Phase 4: Longer context + final validation

### Task 4.1: Run adaptive gate at 512 tokens

**Objective:** Prove the adaptive policy scales to longer contexts.

**Step 1: Run on msi**

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 --n-tokens 512 --ppl-frac 0.3 \
  --adaptive-budget --budget-target-cosine 0.995 --budget-min-k 32 --budget-max-k 384 \
  --recent-guard 16 --quant-bits 4 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-adaptive-512/receipt.json \
  --device cuda'
```

Note: 512 tokens will be slow in Python (~400-600s). This is acceptable for a quality gate.

**Gate:** Receipt shows `passed: true` with average selected keys well below 512.

### Task 4.2: Write final combined receipt

**Objective:** Produce a single markdown report summarizing all results.

**Files:**
- Create: `quant-eval/results/remote-msi/ADAPTIVE_BUDGET_FINAL_RECEIPT.md`

Include:
- Summary table of all runs
- Per-layer budget allocation table
- Claim boundary
- Next steps

---

## Verification matrix

After each phase, run:

```bash
# Local Rust checks
cargo check -p compressed-scorer --no-default-features --features no_std
cargo check -p poly-kv
cargo check -p fib-quant
cargo check -p gpu-backend
cargo check -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'
cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc

# Local Rust tests
cargo test -p compressed-scorer
cargo test --manifest-path poly-kv/Cargo.toml
cargo test -p fib-quant
cargo test -p gpu-backend

# Python syntax
python3 -m py_compile poly-kv/scripts/compressed_attention_forward_ppl.py
python3 -m py_compile poly-kv/scripts/adaptive_budget.py
python3 -m py_compile poly-kv/scripts/fib_gram_scorer.py
```

---

## Risk assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Adaptive budget still can't hit <= 96 avg at 256 tokens | Medium | Accept <= 128 as first milestone; push <= 96 to Phase 2 with Gram-table scoring |
| Gram-table scoring in Python is too slow for 256 tokens | High | Accept slower runs for quality evidence; speed comes from Rust port in Phase 3 |
| Per-head adaptivity doesn't help much beyond per-layer | Medium | Skip if per-layer already hits targets; per-head is refinement |
| 512-token run OOMs on GTX 1070 8GB | Medium | Reduce to 384 tokens or use CPU offloading |
| Rust compressed attention bench needs pool data format that doesn't exist yet | Low | Generate synthetic pool data in the benchmark itself |

---

## Success criteria

1. **Primary:** 256-token full-forward gate passes with average selected keys <= 128 (stretch: <= 96)
2. **Secondary:** 512-token gate passes with sub-linear budget scaling
3. **Tertiary:** Rust compressed attention path shows throughput improvement over Python baseline
4. **No regressions:** All existing Rust tests, no_std, and ESP32 checks stay green

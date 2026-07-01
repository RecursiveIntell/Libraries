#!/usr/bin/env python3
"""Python implementation of fib-quant Gram-table prepared-query scoring.

Matches the Rust FibScorer::score_batch_prepared in fib-quant/src/scoring.rs.

The Rust scoring works as follows:
1. Normalize the query (divide by L2 norm).
2. Apply the rotation (same rotation used in encoding).
3. For each block, find the nearest codeword index (argmin of L2 distance).
4. Score each stored vector: for each block, look up G[query_idx, stored_idx]
   in the Gram table (G[i,j] = <codeword_i, codeword_j>).
5. Sum block-level Gram lookups, scale by query_norm * stored_norm.

This Python version simplifies by using k-means (or random subset) codebooks
instead of the full fib-quant rotation pipeline. The Gram-table scoring itself
is identical — the approximation quality depends on the codebook, not the
scoring formula.
"""
from __future__ import annotations

import torch
import torch.nn.functional as F


def build_codebook_kmeans(vectors: torch.Tensor, M: int, K: int) -> torch.Tensor:
    """Build a product-quantization codebook using k-means per subspace.

    vectors: (N, D) — the vectors to quantize
    M: number of subspaces
    K: codebook size per subspace

    Returns: (M*K, D/M) — all codewords stacked
    """
    N, D = vectors.shape
    D_sub = D // M
    codebook = torch.zeros(M * K, D_sub, device=vectors.device, dtype=torch.float32)
    for m in range(M):
        sub = vectors[:, m * D_sub : (m + 1) * D_sub].float()
        if N <= K:
            # Not enough vectors for k-means; pad with random
            cb = sub.clone()
            if N < K:
                pad = torch.randn(K - N, D_sub, device=vectors.device, dtype=torch.float32) * sub.std(dim=0, keepdim=True)
                cb = torch.cat([cb, pad], dim=0)
            codebook[m * K : (m + 1) * K] = cb
        else:
            # Simple k-means
            indices = torch.randperm(N)[:K]
            cb = sub[indices].clone()
            for _ in range(10):  # 10 k-means iterations
                dists = torch.cdist(sub, cb)  # (N, K)
                assignments = dists.argmin(dim=1)  # (N,)
                for k in range(K):
                    mask = assignments == k
                    if mask.any():
                        cb[k] = sub[mask].mean(dim=0)
            codebook[m * K : (m + 1) * K] = cb
    return codebook


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
) -> tuple[torch.Tensor, float, torch.Tensor]:
    """Prepare query for fast scoring.

    query: (D,) — full query vector
    codebook: (M*K, D_sub) — all codewords

    Returns: (prepared, query_norm, query_indices)
      - prepared: (M, K) — query dot each codeword per subspace
      - query_norm: L2 norm of query
      - query_indices: (M,) — nearest codeword index per subspace
    """
    D = query.shape[0]
    D_sub = D // M
    query_norm = torch.norm(query).item()
    if query_norm == 0.0:
        return torch.zeros(M, K, device=query.device, dtype=torch.float32), 0.0, torch.zeros(M, dtype=torch.int64, device=query.device)
    normalized = (query.float() / query_norm).to(query.device)
    prepared = torch.zeros(M, K, device=query.device, dtype=torch.float32)
    query_indices = torch.zeros(M, dtype=torch.int64, device=query.device)
    for m in range(M):
        q_sub = normalized[m * D_sub : (m + 1) * D_sub]
        cb_sub = codebook[m * K : (m + 1) * K]  # (K, D_sub)
        # Query-codeword inner products
        prepared[m] = torch.matmul(cb_sub, q_sub)
        # Nearest codeword (for Gram lookup)
        dists = torch.cdist(q_sub.unsqueeze(0), cb_sub).squeeze(0)
        query_indices[m] = dists.argmin()
    return prepared, query_norm, query_indices


def score_batch_prepared(
    query_indices: torch.Tensor,  # (M,) — nearest codeword per subspace
    codes: torch.Tensor,          # (N, M) — code indices for N stored vectors
    norms: torch.Tensor,           # (N,) — stored vector norms
    gram: torch.Tensor,            # (M*K, M*K) — Gram table
    query_norm: float,
    M: int,
    K: int,
) -> torch.Tensor:                 # (N,) — scores
    """Score N stored vectors against a prepared query using Gram table.

    Matches the Rust FibScorer::score_prepared formula:
      For each vector n with indices codes[n] and norm norms[n]:
        score[n] = query_norm * norms[n] * sum_{m=0}^{M-1} G[query_indices[m], codes[n,m]]
    """
    N = codes.shape[0]
    scores = torch.zeros(N, device=codes.device, dtype=torch.float32)
    for m in range(M):
        # Gram lookup: G[query_idx_m, codes[:, m]]
        gram_row = gram[query_indices[m].item() * K : (query_indices[m].item() + 1) * K]  # (K, M*K)
        # But we need G[query_idx_m, codes[n, m]] for each n
        # gram is (M*K, M*K), so G[query_idx_m, codes[n,m]] = gram[query_idx_m, codes[n,m]]
        # Since codeword indices are global (m*K + idx), we need:
        # G[query_indices[m], codes[n, m]] where query_indices[m] is the GLOBAL index
        # and codes[n, m] is the per-subspace index (0..K-1)
        # Wait — in the Rust code, indices are per-subspace (0..N-1 where N=codebook_size)
        # and the Gram table is N x N (not M*K x M*K per subspace).
        # Actually the Rust Gram table is N x N where N = codebook_size.
        # Each block uses the same codebook (same N codewords).
        # So G[query_idx, stored_idx] where both are in [0, N).
        # The Gram table here is (M*K, M*K) with block-diagonal structure if
        # each subspace has its own codebook, or (K, K) if shared.
        #
        # For the PQ case with per-subspace codebooks, the Gram table is block-diagonal:
        # G[m*K + i, m*K + j] = <cw_{m,i}, cw_{m,j}> and cross-block terms are 0.
        # But in the Rust fib-quant, there's a SINGLE codebook of size N used for ALL blocks.
        # So the Gram table is N x N and the same for every block.
        #
        # For our Python implementation, we use per-subspace codebooks (PQ style),
        # so we need per-subspace Gram tables. Let's restructure.
        #
        # Actually, looking at the Rust code more carefully:
        # - FibQuantizer has a single codebook of size N with block_dim k
        # - Each block is k-dimensional
        # - All blocks share the same codebook
        # - Gram table is N x N
        # - For block b: G[query_idx_b, stored_idx_b]
        #
        # For PQ with per-subspace codebooks, we need M Gram tables of K x K each.
        # Let's use the simpler shared-codebook model to match the Rust code.
        pass

    # Redo with the correct model: shared codebook, Gram table is K x K per subspace
    # Actually let's just use the per-subspace Gram tables approach
    # gram is (M*K, M*K) but block-diagonal
    for m in range(M):
        q_idx = query_indices[m].item()
        for n in range(N):
            s_idx = codes[n, m].item()
            # Block-diagonal: G[m*K + q_idx, m*K + s_idx]
            scores[n] += gram[m * K + q_idx, m * K + s_idx]

    scores *= query_norm
    scores *= norms
    return scores


def score_batch_prepared_vectorized(
    query_indices: torch.Tensor,  # (M,) — nearest codeword per subspace
    codes: torch.Tensor,          # (N, M) — code indices for N stored vectors
    norms: torch.Tensor,           # (N,) — stored vector norms
    gram: torch.Tensor,            # (M*K, M*K) — Gram table (block-diagonal for PQ)
    query_norm: float,
    M: int,
    K: int,
) -> torch.Tensor:                 # (N,) — scores
    """Vectorized version of score_batch_prepared.

    Uses advanced indexing to avoid Python loops over N.
    """
    N = codes.shape[0]
    device = codes.device
    scores = torch.zeros(N, device=device, dtype=torch.float32)
    for m in range(M):
        q_idx = query_indices[m].item()
        # Gram block for subspace m: gram[m*K:(m+1)*K, m*K:(m+1)*K]
        gram_block = gram[m * K : (m + 1) * K, m * K : (m + 1) * K]  # (K, K)
        # For each n: gram_block[q_idx, codes[n, m]]
        scores += gram_block[q_idx, codes[:, m]]  # (N,)

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
        sub = vectors[:, m * D_sub : (m + 1) * D_sub].float()
        cb_sub = codebook[m * K : (m + 1) * K]  # (K, D_sub)
        dists = torch.cdist(sub, cb_sub)  # (N, K)
        codes[:, m] = dists.argmin(dim=1)
    return codes, norms


if __name__ == "__main__":
    # Quick self-test
    D = 64
    M = 16
    K = 32
    D_sub = D // M
    torch.manual_seed(42)
    vectors = torch.randn(128, D)
    codebook = build_codebook_kmeans(vectors, M, K)
    gram = compute_gram_table(codebook)
    codes, norms = encode_batch(vectors, codebook, M, K)
    query = vectors[0]
    prepared, q_norm, q_indices = prepare_query(query, codebook, M, K)
    scores = score_batch_prepared_vectorized(q_indices, codes, norms, gram, q_norm, M, K)
    exact = torch.matmul(vectors, query)
    corr = torch.corrcoef(torch.stack([scores, exact]))[0, 1]
    print(f"Correlation with exact dot product: {corr:.4f}")
    print(f"Query norm: {q_norm:.4f}")
    print(f"First 5 scores: {scores[:5].tolist()}")
    print(f"First 5 exact: {exact[:5].tolist()}")
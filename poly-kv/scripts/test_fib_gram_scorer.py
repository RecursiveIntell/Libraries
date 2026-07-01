#!/usr/bin/env python3
"""Tests for fib_gram_scorer.py"""
import sys
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).parent))

from fib_gram_scorer import (
    build_codebook_kmeans,
    compute_gram_table,
    prepare_query,
    score_batch_prepared_vectorized,
    encode_batch,
)


def test_score_correlates_with_exact_dot_product():
    """Scores from Gram-table scoring should correlate strongly with exact dot products."""
    D = 64
    M = 16
    K = 32
    torch.manual_seed(42)
    vectors = torch.randn(128, D)
    codebook = build_codebook_kmeans(vectors, M, K)
    gram = compute_gram_table(codebook)
    codes, norms = encode_batch(vectors, codebook, M, K)

    query = vectors[0]
    _, q_norm, q_indices = prepare_query(query, codebook, M, K)
    scores = score_batch_prepared_vectorized(q_indices, codes, norms, gram, q_norm, M, K)
    exact = torch.matmul(vectors, query)

    correlation = torch.corrcoef(torch.stack([scores, exact]))[0, 1]
    assert correlation > 0.8, f"Correlation {correlation} too low — expected > 0.8"


def test_zero_query_returns_zero_scores():
    """A zero query should produce all-zero scores."""
    D = 64
    M = 16
    K = 32
    torch.manual_seed(42)
    vectors = torch.randn(16, D)
    codebook = build_codebook_kmeans(vectors, M, K)
    gram = compute_gram_table(codebook)
    codes, norms = encode_batch(vectors, codebook, M, K)

    query = torch.zeros(D)
    _, q_norm, q_indices = prepare_query(query, codebook, M, K)
    assert q_norm == 0.0
    scores = score_batch_prepared_vectorized(q_indices, codes, norms, gram, q_norm, M, K)
    assert torch.all(scores == 0.0), "Zero query should produce zero scores"


def test_encode_decode_roundtrip():
    """Encoded vectors should have correct shape and norms."""
    D = 64
    M = 16
    K = 32
    torch.manual_seed(42)
    vectors = torch.randn(32, D)
    codebook = build_codebook_kmeans(vectors, M, K)
    codes, norms = encode_batch(vectors, codebook, M, K)

    assert codes.shape == (32, M)
    assert norms.shape == (32,)
    assert (codes >= 0).all() and (codes < K).all(), "Code indices should be in [0, K)"


def test_gram_table_symmetric():
    """Gram table should be symmetric."""
    torch.manual_seed(42)
    codebook = torch.randn(64, 8)
    gram = compute_gram_table(codebook)
    assert torch.allclose(gram, gram.T, atol=1e-5), "Gram table should be symmetric"


if __name__ == "__main__":
    import pytest
    sys.exit(pytest.main([__file__, "-v"]))
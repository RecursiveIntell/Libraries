/*
 * attention.c — C implementation of compressed attention logits and softmax.
 *
 * The logits function computes, for each key, the sum of Gram table
 * lookups G[query_idx, key_idx] across all blocks, then scales by
 * query_norm * stored_norm / sqrt(head_dim). This matches the Rust
 * `FibScorer::score_prepared` + `compressed_attention_logits` inner loop.
 *
 * Softmax is numerically stable with max-subtraction and f64 accumulator,
 * matching the Rust `softmax` function in compressed_attention.rs.
 */
#include "fib_quant.h"
#include <math.h>

void fq_compressed_attention_logits(const uint16_t *key_indices,
                                    size_t n_keys,
                                    const float *key_norms,
                                    const float *gram_table,
                                    size_t gram_size,
                                    const uint16_t *query_indices,
                                    size_t query_count,
                                    float query_norm,
                                    float scale,
                                    float *out_logits) {
    /* For each key, sum Gram[query_idx[b], key_idx[b]] over all blocks. */
    for (size_t k_i = 0; k_i < n_keys; k_i++) {
        const uint16_t *k_indices = key_indices + k_i * query_count;
        float total = 0.0f;
        for (size_t b = 0; b < query_count; b++) {
            size_t qi = (size_t)query_indices[b];
            size_t ki = (size_t)k_indices[b];
            /* Gram table is row-major N×N: G[qi * N + ki] */
            total += gram_table[qi * gram_size + ki];
        }
        /* Scale by query_norm * stored_norm, then by 1/sqrt(head_dim). */
        float stored_norm = key_norms[k_i];
        out_logits[k_i] = total * query_norm * stored_norm / scale;
    }
}

int fq_softmax(float *logits, size_t n) {
    if (n == 0) return -1;

    /* Find max (f32). */
    float max = logits[0];
    for (size_t i = 1; i < n; i++) {
        if (logits[i] > max) max = logits[i];
    }

    /* Compute exp(x - max) with f64 accumulator for sum. */
    double sum = 0.0;
    for (size_t i = 0; i < n; i++) {
        double e = exp((double)(logits[i] - max));
        sum += e;
        logits[i] = (float)e;
    }

    if (!isfinite(sum) || sum <= 0.0) return -1;

    /* Normalize. */
    for (size_t i = 0; i < n; i++) {
        logits[i] = (float)((double)logits[i] / sum);
    }

    return 0;
}
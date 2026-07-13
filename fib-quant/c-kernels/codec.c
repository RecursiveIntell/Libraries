/*
 * codec.c — C implementation of fib-quant encode/decode vector blocks.
 *
 * The encode function performs nearest-codeword lookup (squared L2
 * distance argmin) for each block of `block_dim` floats. This matches
 * the Rust `gpu_backend::nearest_codeword_f32_scalar` algorithm exactly.
 *
 * The decode function gathers codewords by index.
 */
#include "fib_quant.h"
#include <math.h>

/*
 * Nearest-codeword index lookup: find the codeword in `codebook`
 * (row-major N×k) that minimizes squared L2 distance to `sample`.
 *
 * This is the scalar f32 reference, matching
 * gpu_backend::nearest_codeword_f32_scalar exactly:
 *   - f32 accumulation of squared distance
 *   - strict `<` comparison (first minimum wins on ties)
 */
static size_t nearest_codeword_f32_scalar(const float *sample,
                                          const float *codebook,
                                          size_t n_codewords,
                                          size_t k) {
    size_t best_idx = 0;
    float best_dist = INFINITY;
    for (size_t idx = 0; idx < n_codewords; idx++) {
        const float *cw = codebook + idx * k;
        float dist = 0.0f;
        for (size_t j = 0; j < k; j++) {
            float delta = sample[j] - cw[j];
            dist += delta * delta;
        }
        if (dist < best_dist) {
            best_dist = dist;
            best_idx = idx;
        }
    }
    return best_idx;
}

size_t fq_encode_vector_block(const float *vec, size_t dim,
                              const float *codebook, size_t codebook_size,
                              size_t block_dim,
                              uint16_t *out_indices) {
    size_t block_count = dim / block_dim;
    for (size_t b = 0; b < block_count; b++) {
        const float *block = vec + b * block_dim;
        size_t idx = nearest_codeword_f32_scalar(block, codebook,
                                                  codebook_size, block_dim);
        out_indices[b] = (uint16_t)idx;
    }
    return block_count;
}

void fq_decode_vector_block(const uint16_t *indices, size_t block_count,
                            const float *codebook, size_t codebook_size,
                            size_t block_dim,
                            float *out_vec) {
    (void)codebook_size; /* Validated by caller; not needed in gather loop. */
    for (size_t b = 0; b < block_count; b++) {
        size_t idx = (size_t)indices[b];
        /* Caller is responsible for bounds-checking idx < codebook_size
           in the Rust FFI wrapper. We assume valid indices here. */
        const float *cw = codebook + idx * block_dim;
        float *out = out_vec + b * block_dim;
        for (size_t j = 0; j < block_dim; j++) {
            out[j] = cw[j];
        }
    }
}
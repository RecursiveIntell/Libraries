/*
 * fib_quant.h — C kernel declarations for fib-quant hot paths.
 *
 * These functions replace the inner numerical loops of the Rust
 * encode/decode and compressed-attention paths with compiled C,
 * compiled with -O3 -mavx2 -mfma via the `cc` crate in build.rs.
 *
 * The Rust side retains all validation, receipt generation, and
 * orchestration — only the tight numerical inner loops delegate here.
 */
#ifndef FIB_QUANT_H
#define FIB_QUANT_H

#include <stddef.h>
#include <stdint.h>

/*
 * Encode a vector block: for each block of `block_dim` consecutive floats
 * in `vec`, find the nearest codeword in `codebook` (row-major N×k) and
 * store the index in `out_indices`.
 *
 *   vec          — input vector, length = dim
 *   dim          — total dimension (must be divisible by block_dim)
 *   codebook     — row-major codewords, length = codebook_size * block_dim
 *   codebook_size— number of codewords (N)
 *   block_dim    — dimension of each block (k)
 *   out_indices  — output array, length = dim / block_dim
 *
 * Returns the number of blocks encoded.
 */
size_t fq_encode_vector_block(const float *vec, size_t dim,
                              const float *codebook, size_t codebook_size,
                              size_t block_dim,
                              uint16_t *out_indices);

/*
 * Decode a vector block: for each index in `indices`, gather the
 * corresponding codeword from `codebook` and write it to `out_vec`.
 *
 *   indices      — codeword indices, length = block_count
 *   block_count  — number of blocks (= dim / block_dim)
 *   codebook     — row-major codewords, length = codebook_size * block_dim
 *   codebook_size— number of codewords (N)
 *   block_dim    — dimension of each block (k)
 *   out_vec      — output vector, length = block_count * block_dim
 */
void fq_decode_vector_block(const uint16_t *indices, size_t block_count,
                            const float *codebook, size_t codebook_size,
                            size_t block_dim,
                            float *out_vec);

/*
 * Compute compressed attention logits: for each key, sum the Gram table
 * lookups for the (key_index, query_index) pair across all blocks, then
 * scale by query_norm * stored_norm / sqrt(head_dim).
 *
 *   key_indices    — flattened key codeword indices (n_keys * n_blocks)
 *   n_keys         — number of keys
 *   key_norms      — stored norms for each key, length = n_keys
 *   gram_table     — flat N×N f32 matrix, row-major
 *   gram_size      — N (codebook size)
 *   query_indices  — precomputed query codeword indices, length = n_blocks
 *   query_norm     — L2 norm of the query
 *   n_blocks       — number of blocks per vector
 *   out_logits     — output, length = n_keys
 */
void fq_compressed_attention_logits(const uint16_t *key_indices,
                                    size_t n_keys,
                                    const float *key_norms,
                                    const float *gram_table,
                                    size_t gram_size,
                                    const uint16_t *query_indices,
                                    size_t query_count,
                                    float query_norm,
                                    float scale,
                                    float *out_logits);

/*
 * Numerically stable softmax with max-subtraction (f64 accumulator).
 * Modifies `logits` in-place, replacing each element with exp(x-max)/sum.
 *
 *   logits — array to softmax, length = n
 *   n      — number of elements
 *
 * Returns 0 on success, -1 on underflow/empty.
 */
int fq_softmax(float *logits, size_t n);

#endif /* FIB_QUANT_H */
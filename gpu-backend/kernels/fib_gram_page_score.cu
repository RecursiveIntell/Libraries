extern "C" __global__ void fib_gram_page_score(
    const unsigned int* query_indices,
    const unsigned int* stored_indices,
    const float* stored_norms,
    const float* gram,
    float* scores,
    int n_candidates,
    int block_count,
    int n_codewords,
    float query_norm
) {
    int candidate = blockIdx.x * blockDim.x + threadIdx.x;
    if (candidate >= n_candidates) return;
    float sum = 0.0f;
    int base = candidate * block_count;
    for (int block = 0; block < block_count; ++block) {
        unsigned int qi = query_indices[block];
        unsigned int si = stored_indices[base + block];
        sum += gram[((int)qi) * n_codewords + ((int)si)];
    }
    scores[candidate] = sum * query_norm * stored_norms[candidate];
}

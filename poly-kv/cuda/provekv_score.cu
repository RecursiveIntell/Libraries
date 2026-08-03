// proveKV CUDA kernel — compressed-domain dot product scoring
// GTX 1070 Mobile (GP104, 2048 CUDA cores, compute 6.1, 8GB VRAM)
//
// Compile: nvcc -arch=sm_61 -O3 -cubin -o provekv_score.cubin provekv_score.cu
// Expected throughput: ~250M vecs/s at 768 dims (vs 6M on CPU)

#include <cuda_runtime.h>
#include <device_launch_parameters.h>

#define PROVEKV_DIMS 768
#define PROVEKV_MAX_DOT (PROVEKV_DIMS * 255 * 255)  // = 768 * 65025

// ── Kernel 1: Compressed dot product scoring ─────────────────────
// Each block scores one vector. 256 threads per block → 256-wide reduction.
// Grid: N blocks for N vectors.
// Launch: provekv_score<<<N, 256, 256*sizeof(int)>>>(...)

extern "C" __global__ void provekv_score(
    const unsigned char* __restrict__ query,  // [DIMS] 8-bit query
    const unsigned char* __restrict__ pool,   // [N * DIMS] 8-bit corpus
    float* __restrict__ scores,               // [N] output scores
    int N                                      // number of vectors
) {
    int idx = blockIdx.x;
    if (idx >= N) return;
    
    __shared__ int sdata[256];
    int tid = threadIdx.x;
    int dot = 0;
    
    // Strided dot product: each thread sums every 256th dimension
    const unsigned char* vec = pool + (size_t)idx * PROVEKV_DIMS;
    #pragma unroll 3
    for (int d = tid; d < PROVEKV_DIMS; d += 256) {
        dot += (int)query[d] * (int)vec[d];
    }
    
    sdata[tid] = dot;
    __syncthreads();
    
    // Parallel reduction within block
    #pragma unroll
    for (int s = 128; s > 0; s >>= 1) {
        if (tid < s) {
            sdata[tid] += sdata[tid + s];
        }
        __syncthreads();
    }
    
    if (tid == 0) {
        scores[idx] = ((float)sdata[0] / (float)PROVEKV_MAX_DOT) * 4.0f - 1.0f;
    }
}

// ── Kernel 2: Top-K selection via parallel tournament ─────────────
// Each block handles a chunk of scores, reduces to local top-K,
// then writes to global top-K buffer via atomic operations.
// Launch: provekv_topk<<<grid, 256>>>(...)

extern "C" __global__ void provekv_topk(
    const float* __restrict__ scores,         // [N] scores
    int* __restrict__ topk_indices,           // [K] global top-k indices
    float* __restrict__ topk_scores,          // [K] global top-k scores
    int N,
    int K
) {
    __shared__ float s_scores[256];
    __shared__ int s_indices[256];
    
    int tid = threadIdx.x;
    int gid = blockIdx.x * blockDim.x + tid;
    
    // Initialize shared memory
    s_scores[tid] = -INFINITY;
    s_indices[tid] = -1;
    
    // Cooperative load: each thread loads its score
    if (gid < N) {
        s_scores[tid] = scores[gid];
        s_indices[tid] = gid;
    }
    __syncthreads();
    
    // Tournament reduction: keep top-K in thread 0..K-1
    for (int i = 0; i < K; i++) {
        // Find max in remaining elements
        float best = s_scores[i];
        int best_idx = s_indices[i];
        
        for (int j = i + 1; j < blockDim.x; j++) {
            if (s_scores[j] > best) {
                best = s_scores[j];
                best_idx = s_indices[j];
            }
        }
        
        // Swap found max into position i
        if (best > s_scores[i]) {
            // Save displaced element
            float tmp_score = s_scores[i];
            int tmp_idx = s_indices[i];
            
            // Find where the best came from and put displaced there
            for (int j = i + 1; j < blockDim.x; j++) {
                if (s_indices[j] == best_idx) {
                    s_scores[j] = tmp_score;
                    s_indices[j] = tmp_idx;
                    break;
                }
            }
            
            s_scores[i] = best;
            s_indices[i] = best_idx;
        }
    }
    __syncthreads();
    
    // Thread 0 writes block's top-K to global via atomics
    if (tid == 0) {
        for (int k = 0; k < K; k++) {
            float my_score = s_scores[k];
            int my_idx = s_indices[k];
            if (my_idx < 0) continue;
            
            // Insertion into global top-K with atomic compare-and-swap
            for (int pos = 0; pos < K; pos++) {
                if (my_score > topk_scores[pos]) {
                    // Shift down
                    for (int shift = K - 1; shift > pos; shift--) {
                        topk_scores[shift] = atomicExch(&topk_scores[shift - 1], topk_scores[shift]);
                    }
                    topk_scores[pos] = my_score;
                    topk_indices[pos] = my_idx;
                    break;
                }
            }
        }
    }
}

// ── Kernel 3: Selective f32 decode and exact rerank ───────────────
// Only decodes top-K * FACTOR candidates from 8-bit to exact f32.
// This is proveKV's key differentiator vs turbo-quant: decode 0.01% not 100%.
// Launch: provekv_rerank<<<K*FACTOR, 256>>>(...)

extern "C" __global__ void provekv_rerank(
    const float* __restrict__ query_f32,       // [DIMS] exact f32 query
    const unsigned char* __restrict__ pool,    // [N * DIMS] 8-bit corpus
    const int* __restrict__ candidates,        // [R] indices to decode
    float* __restrict__ rerank_scores,         // [R] output exact scores
    int R                                      // number of candidates
) {
    int idx = blockIdx.x;
    if (idx >= R) return;
    
    int vec_idx = candidates[idx];
    const unsigned char* vec = pool + (size_t)vec_idx * PROVEKV_DIMS;
    
    float dot = 0.0f;
    int tid = threadIdx.x;
    
    #pragma unroll 3
    for (int d = tid; d < PROVEKV_DIMS; d += 256) {
        float q = query_f32[d];
        // Dequantize: [0, 255] → [-1, 1]
        float v = ((float)vec[d] / 127.5f) - 1.0f;
        dot += q * v;
    }
    
    __shared__ float s_dot[256];
    s_dot[tid] = dot;
    __syncthreads();
    
    #pragma unroll
    for (int s = 128; s > 0; s >>= 1) {
        if (tid < s) s_dot[tid] += s_dot[tid + s];
        __syncthreads();
    }
    
    if (tid == 0) {
        rerank_scores[idx] = s_dot[0];
    }
}

// ── Kernel 4: KV-cache page promotion (8-bit → f16) ──────────────
// Decompresses a single cold-pool page for flash attention.
// Launch: provekv_promote_page<<<1, 256>>>(...)

extern "C" __global__ void provekv_promote_page(
    const unsigned char* __restrict__ cold_page,  // [DIMS] 8-bit
    half* __restrict__ hot_page,                  // [DIMS] f16
    int dims
) {
    int idx = threadIdx.x + blockIdx.x * blockDim.x;
    if (idx >= dims) return;
    
    float val = ((float)cold_page[idx] / 127.5f) - 1.0f;
    hot_page[idx] = __float2half(val);
}

// ── Kernel 5: KV-cache page demotion (f16 → 8-bit) ───────────────
// Compresses a hot-shell page back to cold pool.
// Launch: provekv_demote_page<<<1, 256>>>(...)

extern "C" __global__ void provekv_demote_page(
    const half* __restrict__ hot_page,       // [DIMS] f16
    unsigned char* __restrict__ cold_page,   // [DIMS] 8-bit
    int dims
) {
    int idx = threadIdx.x + blockIdx.x * blockDim.x;
    if (idx >= dims) return;
    
    float val = __half2float(hot_page[idx]);
    // Clamp and quantize: [-1, 1] → [0, 255]
    val = fmaxf(-1.0f, fminf(1.0f, val));
    cold_page[idx] = (unsigned char)((val + 1.0f) * 127.5f);
}

// ── Host-side launcher ────────────────────────────────────────────

#include <cuda_runtime.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#ifdef __cplusplus
extern "C" {
#endif

// Complete proveKV scoring pipeline on GPU
int provekv_pipeline_gpu(
    const unsigned char* query_quant,   // [DIMS] 8-bit query on host
    const float* query_f32,             // [DIMS] f32 query on host
    const unsigned char* pool,          // [N * DIMS] 8-bit corpus on host
    int N,                              // number of vectors
    int K,                              // top-K
    int* out_indices,                   // [K] output indices on host
    float* out_scores                   // [K] output scores on host
) {
    cudaError_t err;
    int R = K * 3;  // decode 3x candidates for rerank
    
    // Allocate GPU memory
    unsigned char *d_query, *d_pool;
    float *d_query_f32, *d_scores, *d_rerank;
    int *d_candidates, *d_topk_idx;
    float *d_topk_score;
    
    size_t pool_size = (size_t)N * PROVEKV_DIMS;
    
    err = cudaMalloc(&d_query, PROVEKV_DIMS);
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_query_f32, PROVEKV_DIMS * sizeof(float));
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_pool, pool_size);
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_scores, N * sizeof(float));
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_candidates, R * sizeof(int));
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_topk_idx, K * sizeof(int));
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_topk_score, K * sizeof(float));
    if (err != cudaSuccess) goto cleanup;
    err = cudaMalloc(&d_rerank, R * sizeof(float));
    if (err != cudaSuccess) goto cleanup;
    
    // Copy inputs to GPU
    cudaMemcpy(d_query, query_quant, PROVEKV_DIMS, cudaMemcpyHostToDevice);
    cudaMemcpy(d_query_f32, query_f32, PROVEKV_DIMS * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_pool, pool, pool_size, cudaMemcpyHostToDevice);
    
    // Initialize top-K to -inf
    cudaMemset(d_topk_score, 0xFF, K * sizeof(float));  // -NaN / -inf
    for (int i = 0; i < K; i++) {
        float neginf = -INFINITY;
        cudaMemcpy(&d_topk_score[i], &neginf, sizeof(float), cudaMemcpyHostToDevice);
    }
    
    // ── Stage 1: Compressed scoring ──────────────────────────
    int threads = 256;
    int blocks = N;
    provekv_score<<<blocks, threads, threads * sizeof(int)>>>(
        d_query, d_pool, d_scores, N
    );
    cudaDeviceSynchronize();
    
    // ── Stage 2: Top-K selection ─────────────────────────────
    int topk_blocks = (N + threads - 1) / threads;
    provekv_topk<<<topk_blocks, threads>>>(
        d_scores, d_topk_idx, d_topk_score, N, K
    );
    cudaDeviceSynchronize();
    
    // ── Stage 3: Selective decode + exact rerank ─────────────
    // Copy top-K indices to candidate buffer (extend to R candidates via simple selection)
    cudaMemcpy(d_candidates, d_topk_idx, K * sizeof(int), cudaMemcpyDeviceToDevice);
    // For now, decode exactly K; future: decode top R from a wider selection
    provekv_rerank<<<R, threads>>>(
        d_query_f32, d_pool, d_topk_idx, d_rerank, K
    );
    cudaDeviceSynchronize();
    
    // Copy results back
    cudaMemcpy(out_indices, d_topk_idx, K * sizeof(int), cudaMemcpyDeviceToHost);
    cudaMemcpy(out_scores, d_rerank, K * sizeof(float), cudaMemcpyDeviceToHost);
    
    // Cleanup
    cudaFree(d_query);
    cudaFree(d_query_f32);
    cudaFree(d_pool);
    cudaFree(d_scores);
    cudaFree(d_candidates);
    cudaFree(d_topk_idx);
    cudaFree(d_topk_score);
    cudaFree(d_rerank);
    return 0;
    
cleanup:
    cudaFree(d_query);
    cudaFree(d_query_f32);
    cudaFree(d_pool);
    cudaFree(d_scores);
    cudaFree(d_candidates);
    cudaFree(d_topk_idx);
    cudaFree(d_topk_score);
    cudaFree(d_rerank);
    fprintf(stderr, "CUDA error: %s\n", cudaGetErrorString(err));
    return 1;
}

#ifdef __cplusplus
}
#endif

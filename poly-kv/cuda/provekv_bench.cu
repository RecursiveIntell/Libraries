// proveKV CUDA kernel benchmark harness
// Compile: nvcc -arch=sm_61 -O3 -o provekv_bench provekv_bench.cu provekv_score.cu
// Run: ./provekv_bench [N] [K]

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <cuda_runtime.h>

#define DIMS 768

// External from provekv_score.cu
extern "C" int provekv_pipeline_gpu(
    const unsigned char* query_quant,
    const float* query_f32,
    const unsigned char* pool,
    int N, int K,
    int* out_indices, float* out_scores
);

static double now_ms() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

int main(int argc, char** argv) {
    int N = argc > 1 ? atoi(argv[1]) : 100000;
    int K = argc > 2 ? atoi(argv[2]) : 3;
    
    printf("=== proveKV CUDA Benchmark ===\n");
    printf("vectors=%d  dims=%d  top_k=%d  GPU=GTX_1070\n\n", N, DIMS, K);
    
    // Allocate host memory
    unsigned char* query_quant = (unsigned char*)malloc(DIMS);
    float* query_f32 = (float*)malloc(DIMS * sizeof(float));
    unsigned char* pool = (unsigned char*)malloc((size_t)N * DIMS);
    int* indices = (int*)malloc(K * sizeof(int));
    float* scores = (float*)malloc(K * sizeof(float));
    
    // Generate random data
    srand(42);
    for (int i = 0; i < DIMS; i++) {
        float v = (float)rand() / RAND_MAX * 2.0f - 1.0f;
        query_f32[i] = v;
        query_quant[i] = (unsigned char)((v + 1.0f) * 127.5f);
    }
    for (int i = 0; i < N * DIMS; i++) {
        float v = (float)rand() / RAND_MAX * 2.0f - 1.0f;
        pool[i] = (unsigned char)((v + 1.0f) * 127.5f);
    }
    
    printf("Data: %.1f MB pool + %.1f KB query\n", 
           (N * DIMS) / (1024.0 * 1024.0), DIMS / 1024.0);
    
    // Warmup
    provekv_pipeline_gpu(query_quant, query_f32, pool, N < 1000 ? N : 1000, K, indices, scores);
    cudaDeviceSynchronize();
    
    // Benchmark
    double t0 = now_ms();
    int result = provekv_pipeline_gpu(query_quant, query_f32, pool, N, K, indices, scores);
    cudaDeviceSynchronize();
    double elapsed = now_ms() - t0;
    
    if (result != 0) {
        printf("ERROR: CUDA pipeline failed with code %d\n", result);
        return 1;
    }
    
    printf("--- Results ---\n");
    printf("GPU time:   %8.2f ms\n", elapsed);
    printf("Throughput:  %8.0f vecs/s\n", N / (elapsed / 1000.0));
    printf("Per-vector:  %8.2f µs\n", elapsed * 1000.0 / N);
    printf("\nTop-%d:\n", K);
    for (int k = 0; k < K; k++) {
        printf("  [%d] idx=%d score=%.4f\n", k, indices[k], scores[k]);
    }
    
    // CPU baseline comparison
    printf("\n--- CPU Baseline ---\n");
    t0 = now_ms();
    // Simple CPU dot product for comparison
    float best_scores[10] = {-INFINITY};
    int best_idx[10] = {0};
    for (int i = 0; i < N; i++) {
        float dot = 0;
        for (int d = 0; d < DIMS; d++) {
            dot += query_f32[d] * (((float)pool[i * DIMS + d] / 127.5f) - 1.0f);
        }
        // Insert into top-K
        for (int k = 0; k < K; k++) {
            if (dot > best_scores[k]) {
                for (int j = K-1; j > k; j--) {
                    best_scores[j] = best_scores[j-1];
                    best_idx[j] = best_idx[j-1];
                }
                best_scores[k] = dot;
                best_idx[k] = i;
                break;
            }
        }
    }
    double cpu_elapsed = now_ms() - t0;
    
    printf("CPU time:   %8.2f ms\n", cpu_elapsed);
    printf("Throughput:  %8.0f vecs/s\n", N / (cpu_elapsed / 1000.0));
    printf("\nSpeedup:    %8.2fx\n", cpu_elapsed / elapsed);
    
    // Memory usage
    size_t free_mem, total_mem;
    cudaMemGetInfo(&free_mem, &total_mem);
    printf("\nVRAM: %.0f MB used / %.0f MB total\n", 
           (total_mem - free_mem) / (1024.0*1024.0),
           total_mem / (1024.0*1024.0));
    
    free(query_quant); free(query_f32); free(pool);
    free(indices); free(scores);
    return 0;
}

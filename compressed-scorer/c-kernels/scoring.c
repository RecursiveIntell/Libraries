#include "scoring.h"

/* Per-dimension scoring: for each dimension i, look up
 * lut[i * levels + codes[i]] and accumulate.
 *
 * Rust original archived in src/archive/per_dim_rust.rs
 */
float cs_per_dim_score(const uint8_t *codes, size_t dim,
                       const float *lut, size_t levels) {
    double sum = 0.0;
    for (size_t i = 0; i < dim; i++) {
        size_t idx = (size_t)codes[i];
        sum += (double)lut[i * levels + idx];
    }
    return (float)sum;
}

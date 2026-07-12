#ifndef CS_SCORING_H
#define CS_SCORING_H

#include <stddef.h>
#include <stdint.h>

/* per_dim scoring: accumulate LUT values for each code index */
float cs_per_dim_score(const uint8_t *codes, size_t dim,
                       const float *lut, size_t levels);

#endif /* CS_SCORING_H */

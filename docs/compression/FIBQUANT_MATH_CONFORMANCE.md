# FibQuant Math Conformance

Created: 2026-05-16

Implemented conformance targets:

- canonical spherical-Beta block source, including normalized-Gaussian reference sampling;
- `R^2 ~ Beta(k/2, (d-k)/2)` for `k < d`, with explicit unit-sphere degeneracy for `k = d`;
- Bennett-Gersho radius shape `beta_{d,k}`;
- `k = 2` closed-form radius path;
- `k = 2`, `k = 3`, and `k >= 4` direction families;
- radial-angular row-major codebook initialization;
- mandatory multi-restart Lloyd-Max refinement;
- fixed-rate index packing using `ceil(log2(N))`;
- separate paper and wire rate fields;
- digest-bound profile, codebook, and encoded artifacts.

Known deviations and boundaries:

- The first pass uses exhaustive nearest-codeword search, not the paper appendix's optional hierarchical encoder acceleration.
- The first pass uses Euclidean Lloyd-Max only. Mahalanobis/task-weighted Lloyd objectives remain out of scope.
- No local benchmark wins are claimed. Paper benchmark numbers remain source-reported only.

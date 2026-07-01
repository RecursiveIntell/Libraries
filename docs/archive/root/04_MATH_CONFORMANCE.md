# 04 — Math Conformance Spec

## Paper-faithful target

FibQuant is not scalar quantization with a new name. The implementation must respect the spherical-Beta block source induced by normalize + Haar rotation.

## Source law

For `x ∈ R^d`, define:

```text
nu = ||x||_2
u_vec = x / nu
y = Pi nu_vec
```

For a k-coordinate block of `y`, the paper's canonical law is spherical-Beta on the unit ball:

```text
R² ~ Beta(k/2, (d-k)/2)
U ~ Uniform(S^{k-1})
B = R U
```

Required tests:

- Empirical `E[R²] ≈ k/d`.
- Coordinate variance `≈ 1/d`.
- Samples lie inside unit ball.
- Reference Gaussian projection sampler agrees with canonical direct sampler on coarse moments.

## Radius law

Let `q_n = (n - 1/2) / N`.

Required implementation:

```text
r_n = sqrt(BetaInv(q_n; k/2, beta_{d,k}))
```

where the paper's radial companding shape must be implemented from the source basis. If the exact expression is ambiguous in extracted math text, Codex must stop and read the PDF/HTML source rather than guess.

For k=2, implement the closed-form path from the paper and test it against the general path.

## Direction law

- k=2: planar Fibonacci spiral using golden-angle sequence.
- k=3: Fibonacci sphere with equal-area latitude bands and golden-angle azimuth.
- k>=4: Roberts-Kronecker rank-one low-discrepancy sequence, inverse-normal mapped, projected to unit sphere.

## Lloyd-Max law

The initialization alone is insufficient.

Required loop:

1. Build initial codebook.
2. For each restart, apply a deterministic random orthogonal rotation to the initial codebook.
3. Sample training blocks from spherical-Beta source.
4. Assign every sample to nearest codeword by squared Euclidean distance.
5. Update occupied centroids.
6. Repair empty cells by splitting a high-distortion occupied cell.
7. Keep lowest-MSE restart.

## Rate law

Track both:

```text
paper_rate_bits_per_coord = log2(N) / k
wire_index_bits = ceil(log2(N))
wire_bits_per_coord = wire_index_bits / k
```

Do not collapse these fields. Dense mathematical rate does not imply compact practical wire encoding unless enumerative/fractional coding is implemented, which is out of scope for the first pass.

## Random-access law

The encoded artifact must be fixed-size for a given profile and vector dimension:

```text
norm header + block_count * wire_index_bits
```

Variable-length coding is forbidden in the paper-faithful path.

## Numerical precision law

- f64 during codebook construction and metric evaluation.
- f32 for persisted codewords unless a reference path explicitly chooses f64.
- fp16 norm header by default.
- F32 norm only in reference/test mode.

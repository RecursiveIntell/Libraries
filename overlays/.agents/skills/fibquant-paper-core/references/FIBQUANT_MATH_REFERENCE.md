# FibQuant Math Reference

Implementation target:

- Normalize input vector.
- Apply deterministic Haar-like orthogonal rotation.
- Split rotated unit vector into k-blocks.
- Each block follows the spherical-Beta law on the unit ball.
- Use Beta-quantile radii.
- Use Fibonacci/Roberts-Kronecker quasi-uniform directions.
- Polish with multi-restart Lloyd-Max.
- Store fixed-rate indices and fp16 norm.
- Decode each vector independently.

Reject any implementation that only does scalar quantization, product quantization, k=2-only codebooks, or variable-length payloads.

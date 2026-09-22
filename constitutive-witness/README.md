# constitutive-witness (experimental)

Dependency-free exact rational checks for `A b = r`, `T b = 0`, `b >= 0`.
The supplied matrices represent a finite problem, not a certified continuum PDE.

This first version is an explicit nested Cargo workspace, not a root default
member. That keeps the experimental dependency/build lane isolated and avoids
changing the existing root lockfile or production behavior. Promotion into the
root workspace is a separate integration decision.

```sh
cargo test --manifest-path constitutive-witness/Cargo.toml --locked --offline
printf 'CW1 1 1 0 100\n-1/1\n1/1\n' | cargo run --quiet --manifest-path constitutive-witness/Cargo.toml --locked --offline
```

Native input is ASCII whitespace-separated: `CW1 m n k budget`, followed by
row-major A (m*n canonical rational tokens), r (m tokens), and T (k*n tokens).
Every rational is reduced `numerator/positive_denominator`; integers use `/1`.
No trailing fields, ambiguous numeric spelling, or unchecked overflow is allowed.
Limits: 64 KiB input, 32 combined rows, 16 columns, 10,000 candidate attempts.

A checked primal satisfies all equalities and nonnegativity. A checked dual
satisfies `[A;T]^T y <= 0` and `[r;0]^T y > 0`. The latter is a standard Farkas
certificate: it contradicts the existence of any nonnegative primal vector.
Reference: https://docs.mosek.com/modeling-cookbook/linear.html#farkas-lemma .
No novelty claim is made for linear programming, rational arithmetic, or duality.

Search tries cheap one/two-row dual candidates and exact basic-column primal
candidates within a finite attempt budget. It is not an optimized or complete
LP solver. Failure/exhaustion returns `unresolved`, never infeasible. Arithmetic
failure exits nonzero; no approximate certificate is substituted.

The binary emits `cw.candidate.v1`, not an authenticated receipt or claim
judgment. ClaimLedger must bind the input/statement and independently recheck
the candidate. No pressure projection, transport discretization accuracy,
smoothness, physical calibration, or continuum theorem is certified here.

Related work: RecursiveIntell/ClaimLedger#1 and RecursiveIntell/Ares#59.
See `docs/research/constitutive-witness-plan.md` for the broader tracked gates.

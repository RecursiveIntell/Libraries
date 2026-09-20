# Constitutive witness validation checkpoint

Recorded 2026-09-20. Experimental implementation, not a continuum certificate or publication claim.

Code pin: `6b73cc69076c7db0047316eca54684f2d6858e23`.
Base: `5de5a070fbf0e4d96b2949e34feeabc600c1889d`.
PR: https://github.com/RecursiveIntell/Libraries/pull/26 .

## Actual result

GitHub Actions run https://github.com/RecursiveIntell/Libraries/actions/runs/35540287469 , job `106156583795`, passed:

- `cargo test --manifest-path constitutive-witness/Cargo.toml --locked --offline`: **14 tests passed**.
- `cargo build --manifest-path constitutive-witness/Cargo.toml --locked --offline`.
- `python3 receipt-bench/fixtures/falsify/check_corpus.py --binary constitutive-witness/target/debug/constitutive-witness`: **all seven expected finite-problem outcomes passed independent rational rechecking**.

Rust/Cargo: 1.98.1, Ubuntu 24.04. The tested PR merge tree was `580feb979604c9e77edb699b7b94750232bd175f`, combining the code pin and base above. This is not an actual merge into main.

| Case | Result | Candidate attempts |
|---|---|---:|
| energy-sign | dual | 4 |
| nonnegative-primal-control | primal | 6 |
| range-obstruction | dual | 12 |
| transport-coupling | dual | 51 |
| budget-is-not-refutation | unresolved | 1 |
| periodic-shear-primal | primal | 54 |
| periodic-shear-energy | dual | 9 |

Corpus SHA256: `bb38cfb08e05db0fc797382a2a0d6148ad83454313cea0106efb690170b69484`.
These attempt counts are observations on this seed, not a performance or algorithmic novelty benchmark.

The latest bounded search also includes a small ternary dual grid for at most six augmented rows after basic-column primal search. All candidates are rechecked before returning a result. Lack of a found certificate remains unresolved.

## Cross-repo integration

Related PRs: RecursiveIntell/ClaimLedger#1 and RecursiveIntell/Ares#59.
The authorized ClaimLedger CI run `35540345772` pinned this kernel and Ares `05b013f2bdb26486271c54ea43ca32ebe89d5e81`, and exercised all seven cases through the actual kernel, independent evidence checker, and bounded runner. No provider call or live agent installation was needed.

Canonical implementation/auditor handoff and remaining program tracker:
https://github.com/RecursiveIntell/Ares/blob/research/falsify-skill-20260920/docs/research/scientific-falsification-handoff.md .

## Scope, missing gates, and rollback

The crate is deliberately an isolated nested Cargo workspace with `publish = false`, not a root default member. It does not alter production quantization, memory, root dependencies, or runtime activation. A later root-workspace integration needs its own lockfile/release decision.

No local Rust compiler was available in the implementation container; Rust evidence above comes from CI. Full repository hardening/release completion, compiler minimum-version testing, formatting/MSRV qualification, platform matrix, independent mathematical/code review and continuum-adapter correctness remain separate gates. Root hardening run `35540287984` was still in progress at the checkpoint; do not infer whole-repo readiness from the isolated job.

Primal/dual checks prove facts only about supplied exact finite matrices. Material-label equations are not automatically an accurate physical-flow discretization. Pressure projection, constitutive model equivalence, continuum convergence, novel mathematics and the historical conditional theorem are not certified.

Rollback: close/revert this additive PR. No source manuscript, durable evidence store, live runtime, or existing default dependency graph requires restoration.

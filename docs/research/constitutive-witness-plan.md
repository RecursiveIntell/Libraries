# Constitutive witness implementation ledger

Status: experimental implementation in progress; no theorem, novelty, or speedup claim.
Base: `5de5a070fbf0e4d96b2949e34feeabc600c1889d`.

## Ownership and boundaries

`constitutive-witness` will check explicitly supplied finite-dimensional problems and witnesses. It will not own claims, support admission, permits, or execution authority. ClaimLedger owns scientific evidence interpretation; Ares owns optional orchestration. No default-on integration, runtime activation, merge, release, or external researcher contact is part of this work.

The September 18 research packet is motivation, not a proof oracle. The selected transported-scalar extension remains an unreviewed conditional argument with retuned forcing. No implementation here depends on a claim that the classical Navier–Stokes problem has been solved. Continuum certificates require separate discretization-error and theorem-contract evidence.

## Incremental delivery

1. Implement a dependency-light exact rational finite-dimensional witness kernel, checked arithmetic, strict input/resource limits, and adversarial tests.
2. Implement nonnegative-coefficient primal verification and Farkas-style dual separation for a space-time operator with explicit transport equalities. Bounded search may return unresolved; exhaustion is never proof of infeasibility.
3. Add periodic shear/operator fixtures and a small falsification corpus, including energy-sign, range, and transport obstructions. Preserve the distinction between exact finite-dimensional certificates and numerical diagnostics.
4. Add reproducible validation, source-bound result records, cross-repo adapter contract, and an independent-auditor handoff.

## Acceptance gates

- Reject malformed dimensions, noncanonical numbers, overflow, excessive work, and unknown protocol fields.
- A primal certificate must satisfy every supplied equality and coefficient nonnegativity exactly.
- A dual certificate must satisfy the full augmented operator transpose inequality and strict separating margin exactly.
- Certificates bind the exact problem bytes/contract at the orchestration/evidence layer; changed inputs invalidate receipts.
- Search failure is unresolved, not a negative mathematical result.
- Tests cover sign errors, pressure/range limitations, transport constraints, invalid witnesses, zero cases, arithmetic overflow, and resource exhaustion.
- Do not imply a generic PDE solver or complete inverse-problem algorithm.

## Tracked follow-on work (not completed by this checkpoint)

- Rigorous continuum lifting and pressure-projection adapter with discretization bounds.
- Symmetric-stress adapter, kept distinct from componentwise diffusion.
- Larger primal/dual search backends and benchmark comparisons.
- Formal proof of small obstruction lemmas and independent review of the conditional research theorem.
- Public novelty/literature review and any separate mathematics repository: gated on independent review, not automatic.

## Rollback

This is additive experimental source on a separate branch. Close the PR or revert its commits; do not alter source manuscripts, existing claim stores, or live runtime state.

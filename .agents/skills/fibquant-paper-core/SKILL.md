---
name: fibquant-paper-core
description: "Use for paper-faithful FibQuant implementation, mathematical conformance, codebook generation, codec tests, receipts, and default-off compression work."
---

# FibQuant Paper-Core Skill

Use this skill when implementing or reviewing the `fib-quant` crate.

## Mandatory behavior

- Implement the paper math directly.
- Start with a source-basis document.
- Preserve current product behavior.
- Keep FibQuant default-off.
- Do not change `semantic-memory/src/**` or `turbo-quant/src/**` in the paper-core pass.
- Emit tests and receipts.
- Stop on mathematical ambiguity.

## Required implementation sequence

1. Source basis.
2. Profile/digest/error law.
3. Spherical-Beta source and radii.
4. Direction generators.
5. Codebook initialization.
6. Lloyd-Max refinement.
7. Fixed-rate codec.
8. Metrics/tests.
9. Receipts/docs.
10. Final audit.

## Activation phrase

If the user says FibQuant, paper-faithful compression, custom compression, KV-cache compression, radial-angular codebook, spherical-Beta, or Lloyd-Max, inspect this skill before coding.

## Never do

- Never skip Lloyd-Max.
- Never implement k=2 only.
- Never claim benchmark wins from the paper as local proof.
- Never make FibQuant default.
- Never silently pad dimensions.
- Never use variable-length payload in the paper-faithful path.

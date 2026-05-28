# FibQuant Rollback Plan

Created: 2026-05-16

FibQuant is default-off. Rollback is therefore limited to removing standalone artifacts:

1. Remove `fib-quant` from root workspace `members`.
2. Delete the `fib-quant/` crate directory.
3. Delete FibQuant docs under `docs/compression/` if the source-basis record is no longer needed.
4. Regenerate `Cargo.lock` with Cargo.

No `semantic-memory/src/**` or `turbo-quant/src/**` rollback steps are required because this pass does not modify those surfaces.

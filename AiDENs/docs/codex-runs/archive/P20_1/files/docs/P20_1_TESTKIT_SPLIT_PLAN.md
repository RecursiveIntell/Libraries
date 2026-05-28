# P20.1 Testkit Split Plan

## Current problem

`aidens-testkit` is described as a reference/contract-support crate but currently normal-depends on production crates such as runner, provider, boundary, memory, kernel, governance, repair, tool, and CLI crates. Several production crates also dev-depend on `aidens-testkit`.

## Target

`aidens-testkit` should contain only:

- fixture loading helpers;
- pure reference interpreters;
- static scanner helpers;
- JSON case validation helpers;
- shared assertion utilities that do not import production crates.

Production behavior tests should move to:

- `crates/aidens-integration-tests/tests/`, or
- root `tests/`, or
- package-local tests.

## Acceptance

- `aidens-testkit/Cargo.toml` has no normal dependencies on `aidens-runner`, `aidens-provider-kit`, `aidens-boundary-kit`, `aidens-tool-kit`, `aidens-cli`, or other production crates.
- production crates may dev-depend on `aidens-testkit` only after it is pure.
- cargo test passes.

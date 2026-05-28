# P20.2 Testkit Split Plan

## Problem

`aidens-testkit` currently imports production crates, making it an integration crate disguised as a reference crate. That can create dependency cycles and corrupt the meaning of "reference" tests.

## Target topology

```text
crates/aidens-testkit/
  pure reference models
  fixture loaders
  expected-output comparators
  no production runtime dependencies

crates/aidens-integration-tests/
  depends on aidens-runner/provider/tool/agency/etc.
  owns vertical-slice tests
  owns test-agent tests
  owns package/integration smoke tests
```

## Required moves

- Move tests that import production crates from `aidens-testkit/tests` to `aidens-integration-tests/tests`.
- Leave pure reference tests in `aidens-testkit`.
- Add workspace member for `crates/aidens-integration-tests`.
- Add scanner gate preventing testkit regression.

## Acceptance

```bash
python3 scripts/p20_2_scan_testkit_purity.py . --require-integration-crate
cargo test -p aidens-testkit --all-targets
cargo test -p aidens-integration-tests --all-targets
```

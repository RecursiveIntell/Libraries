# P25 Flagship Demo Acceptance Fixtures

## Required fixture repository

```text
fixtures/p25/coding-agent-repo/
  README.md
  Cargo.toml or simple metadata file where applicable
  src/lib.rs
```

## Required task

The task should be small and deterministic, such as:

> Inspect the fixture repository and propose a minimal patch that changes `add_one` to handle overflow safely, or abstain if unsupported.

## Required outputs

- patch proposal or abstention,
- tool receipts,
- permit request,
- permit use receipt if apply path is tested,
- run bundle,
- replay report.

## Failure modes that must be tested

- no permit for write -> must not write;
- invalid patch -> fail closed;
- missing file -> abstain or error with receipt;
- replay mismatch -> fail gate.

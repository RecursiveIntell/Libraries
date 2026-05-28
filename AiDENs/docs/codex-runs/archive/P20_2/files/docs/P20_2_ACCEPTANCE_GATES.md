# P20.2 Acceptance Gates

## P0 gates — must pass

- [ ] `evals/p20_agency_eval_cases.jsonl` exists.
- [ ] `python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl` passes.
- [ ] `python3 scripts/p20_2_scan_package_integrity.py .` reports no missing `include_str!` / `include_bytes!` targets.
- [ ] `MANIFEST.txt` and `MANIFEST.json`, if present, reference existing files or are updated honestly.
- [ ] `scripts/p20_2_verify.sh` exists and runs from repo root.
- [ ] `aidens-testkit` depends only on reference-safe crates.
- [ ] production integration tests are moved to `aidens-integration-tests` or a clearly equivalent integration crate.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo check --workspace --all-targets --all-features` passes.
- [ ] `cargo test --workspace --all-targets --all-features` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.

## P1 gates — v0.1 proof

- [ ] canonical test-agent fixture exists.
- [ ] integration test proves test agent vertical slice.
- [ ] CLI or script entry exists for running the test agent.
- [ ] event log / receipt output is deterministic enough for assertions.
- [ ] provider capability matrix is honest: mock supported, Ollama chat-only/partial, cloud providers unavailable unless tested.
- [ ] agency evals assert expected policy outcome, required receipts, and forbidden behavior.
- [ ] final audit bundle generated under source-controlled handoff or release artifact path.
- [ ] release zip is unpacked and rechecked.

## P2 stretch gates

- [ ] profile crates either have smoke tests or are explicitly deferred.
- [ ] examples are runnable and documented.
- [ ] P21 plan generated from actual remaining risks.

## Failure rule

A gate cannot be marked pass from documentation alone. It must be backed by command output, tests, scanner output, or explicit code evidence.

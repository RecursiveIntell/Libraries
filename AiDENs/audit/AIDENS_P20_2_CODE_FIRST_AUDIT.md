# AiDENs P20.2 Code-First Audit Basis

## Snapshot basis

Archive inspected: `libraries-source-clean-20260430.zip` extracted as `aidens_latest`.
Review mode: static source/package inspection. Cargo/rustc were unavailable in this environment, so this is not green-build certification.

## Code-level findings that drive this pass

### P0: package artifact is still internally inconsistent as uploaded

The source references `evals/p20_agency_eval_cases.jsonl` from actual code and scripts:

- `crates/aidens-agency-kit/src/lib.rs`
- `crates/aidens-testkit/tests/phase_09_reference_hostile_tests.rs`
- `scripts/p20_validate_agency_cases.py`
- `scripts/p20_verify.sh`

The uploaded source tree does not contain `evals/`. This is a code/package failure, not a documentation problem.

### P0: `aidens-testkit` is still production-dependent in the uploaded archive

`crates/aidens-testkit/Cargo.toml` depends on production crates such as:

- `aidens-agency-kit`
- `aidens-boundary-kit`
- `aidens-cli`
- `aidens-daemon-kit`
- `aidens-governance-kit`
- `aidens-kernel-kit`
- `aidens-memory-kit`
- `aidens-provider-kit`
- `aidens-runner`
- `aidens-tool-kit`

This violates the intended test topology. `aidens-testkit` must become pure/reference-only. Production integration tests must move to `aidens-integration-tests` or equivalent.

### P1: fixture/script restoration improved but needs verification

The source tree now contains `tests/fixtures`, `fixtures/runner`, `scripts`, and `prompts/phase_injections`. That is good. It must be verified by scanner rather than trusted.

### P1: implementation direction remains strong

Actual code surfaces remain promising:

- `aidens-runner` has real provider/tool/permit/boundary/agency/receipt paths.
- `aidens-provider-kit` is honest about mock/Ollama/unavailable providers.
- `aidens-agency-kit` is substantial and runner-connected.
- `aidens-memory-kit`, `aidens-kernel-kit`, `aidens-governance-kit`, and `aidens-repair-kit` are correctly thin canonical adapters.

### Conclusion

The next pass should not be broad feature expansion. It should close code/package integrity, split tests correctly, prove the canonical test agent, then certify v0.1. A guarded stretch lane may run only after all gates are green.

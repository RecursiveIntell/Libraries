# P20.1 Acceptance Gates

## G0 — package integrity

- `scripts/p20_1_hard_code_audit.py --fail-on-blocking` passes.
- `MANIFEST.txt` has zero missing paths.
- every `include_str!` / `include_bytes!` target exists.
- `evals/p20_agency_eval_cases.jsonl` exists and validates.

## G1 — testkit topology

- `aidens-testkit` is pure reference/fixture/helper code, or renamed/split.
- no production crate both dev-depends on `aidens-testkit` while `aidens-testkit` normal-depends on it.
- production integration tests live in `aidens-integration-tests`, root tests, or package-local tests without reference-testkit impurity.

## G2 — canonical ownership scanner

- ownership scanner refuses to certify if canonical sibling crate baseline is missing.
- scanner output records whether canonical inventory is present.
- no duplicate canonical concepts are introduced in `aidens-contracts`.

## G3 — cargo gates

Run in the real workspace:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All must pass or final report must say FAIL.

## G4 — runner/provider/agency revalidation

- mock runner vertical slice still passes.
- Ollama remains chat-only unless native tool-loop tests exist.
- cloud/native tool-loop providers remain unavailable unless executable tests exist.
- agency eval cases run and produce expected policy/receipt outcomes.

## G5 — final archive integrity

- final archive includes everything referenced by manifest, code, tests, and install scripts.
- final audit bundle is copied to a source-visible handoff directory or explicitly included in the release artifact.

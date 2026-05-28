# AiDENs P20.1 code-first hard audit summary

Basis: actual source/package inspection of `libraries-source-clean-20260430.zip`. Documentation is not treated as proof; docs matter only when source, manifests, tests, scripts, or release packaging depend on them.

## Actual source shape

- Workspace crates: 32
- Rust files: 54
- Approximate Rust LOC: 32,134
- `aidens-contracts`: ~10,181 LOC; still the largest local risk surface.
- `aidens-testkit`: ~4,656 LOC; must be split or purified.
- `aidens-runner`, `aidens-tool-kit`, `aidens-provider-kit`, `aidens-agency-kit`, and canonical adapter crates are real enough to preserve.

## Blocking code/package findings

1. `evals/p20_agency_eval_cases.jsonl` is required by Rust test code and missing from the submitted archive.
2. `MANIFEST.txt` names missing paths: agency eval JSONL, runner expected event log, and three support matrices.
3. `aidens-testkit` normal-depends on production crates while production crates dev-depend on it. This risks Cargo dependency cycles and violates the reference-testkit concept.
4. Ownership scanning must not return false confidence when canonical sibling crates are absent/unavailable.
5. Real cargo gates could not be run in this environment; they must run in the complete sibling-workspace layout.

## Positive code findings

- Runner vertical slice exists and is worth preserving.
- Provider capability model is much more honest: mock executable, Ollama partial chat, cloud/unavailable providers explicitly unavailable unless implemented.
- Agency governance exists in source and runner path, though it is heuristic v0.1 policy.
- Canonical adapter crates are properly thin.
- Boundary kit contains real parser/repair/treatment-integrity behavior.

## Next pass type

Run a **code/package repair pass**, not an architecture pass. The primary objective is getting the current architecture build-certified and auditor-safe.

# P29 Phase 07 Report

## Phase

Phase 07 - Search ranking dedup and classifier repair.

## Scope

Focused on FTS query quoting, RRF recency denominator, message dedup identity, vector sort NaN handling, query classifier extraction, and merge normalization in `semantic-memory` and `knowledge-runtime`.

## Files changed

- `../semantic-memory/src/search.rs`
- `../semantic-memory/tests/search_tests.rs`
- `../knowledge-runtime/src/query/classify.rs`
- `../knowledge-runtime/src/query/merge.rs`
- `handoffs/p29/PHASE_07_REPORT.md`

## Issue IDs addressed

- Fixed: `BUG-021`, `BUG-022`, `BUG-023`, `BUG-026`, `BUG-053`, `BUG-054`, `BUG-055`, `BUG-056`, `BUG-058`, `BUG-059`
- Quarantined: `BUG-024`, `BUG-025`, `BUG-027`, `BUG-028`, `BUG-029`, `BUG-030`, `BUG-057`

## Tests/checks run

| Command | Result | Log |
|---|---|---|
| `cargo test --test search_tests -- --nocapture` in `../semantic-memory` | pass | `target/p29/audit/phase07_semantic_search_tests.log` |
| `cargo check --all-targets` in `../knowledge-runtime` | pass | `target/p29/audit/phase07_knowledge_cargo_check.log` |
| `cargo test --lib query::classify -- --nocapture` in `../knowledge-runtime` | pass | `target/p29/audit/phase07_knowledge_classify_tests.log` |
| `cargo test --lib query::merge -- --nocapture` in `../knowledge-runtime` | pass | `target/p29/audit/phase07_knowledge_merge_tests.log` |
| `cargo test --test invariant_tests -- --nocapture` in `../knowledge-runtime` | pass | `target/p29/audit/phase07_knowledge_invariant_tests.log` |
| `cargo check --workspace --all-targets` in `AiDENs/` | pass | `target/p29/audit/phase07_aidens_cargo_check.log` |

## Evidence produced

- FTS tokens are individually quoted after sanitization.
- Empty sanitized FTS queries still skip FTS safely and keep vector paths available.
- Recency contribution uses the best available rank denominator.
- Message dedup keys include session scope.
- Classifier preserves multiple entity mentions and prefers more specific temporal phrases.
- Merge handles zero limits, non-finite boosts, deterministic provenance ordering, finite min/max initialization, and clamps MinMax boosted scores to `[0, 1]`.

## Claims changed

No v11A/v11B support claim was advanced.

## Risks / limitations

Quarantined search items require broader API or scoring-contract changes and remain known limitations.

## Gate status

- [x] pass
- [ ] fail

## Next phase notes

Stop for the required Phase 07 manual injection before Phase 08.

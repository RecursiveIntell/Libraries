# Changelog

## Unreleased

### Added

- Explicit `TokenCounterKind` recorded in compaction receipts.
- `BudgetMode::{SoftWarn, HardCascade, FailClosed}` for honest budget behavior.
- `ContentKind` classification for JSON, diffs, Rust, Markdown, Cargo output, shell logs, search results, and prose.
- Structured context summary anchors in `SummaryLossReportV1`.
- Content-aware previews for JSON, diffs, Cargo output, and shell logs.
- `MemoryArchiveRecordV1`, `MemorySink`, and `archive_response_to_memory` for honest adapter-owned archival.
- `FileContextStore::expand` and `FileContextStore::search` across saved receipts.
- CLI subcommands: `compact`, `store`, `expand`, `search`, and `diff`.
- Deterministic replay/eval example and reusable replay fixture evaluator.
- Local Hermes SQLite replay evaluation script with privacy-preserving aggregate markdown report.
- Aggressive replay compaction (`aggressive_v1`) that demotes long path/evidence/tool-heavy context to receipt-backed exact fallback instead of preserving it verbatim.
- Whitespace-normalized replay visibility/search scoring so multiline active tasks and exact fallback records match literal probes.
- Hard-cascade budget enforcement now degrades to the protected minimum instead of failing when the requested target is below the minimum viable prompt.
- Architecture, eval, Hermes integration, and complete implementation-plan docs.

### Changed

- Compaction receipts now disclose token counter kind.
- Summary text now includes structured anchors before lossy previews.
- Hard budget behavior can now refuse rather than silently overflowing.

### Claim boundary

- No KV-cache compression claim.
- No learned prompt-compression claim.
- No built-in semantic-memory network write claim; memory archival is exposed through an adapter trait.

## 0.1.0

- Initial governed context compaction crate with receipts, exact fallback store, recall filtering, CLI compact mode, and performance harness.

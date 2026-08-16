# Changelog

## Unreleased

### Added

- `ContextCompactionReceiptV2` with one-parent recursive lineage, monotonic
  generations, explicit supersession, deterministic lineage digests, and
  transitive original-source identities.
- Append-only V2 file-store admission, restart-safe tip recovery, ancestry-aware
  retention, and exact expansion through verified V1/V2 parent chains.
- Governed CLI `compact-v2`, authenticated `finalize-v2`, and two-phase
  `prepare-v2`/`pending-v2`/`activate-v2`/`discard-v2` surfaces for thin host
  adapters; pending receipts are inert until the matching host transcript is
  durably committed.
- Independent V2 evidence HMACs keep exact provenance/source recovery
  authenticated even when only the rebuildable summary projection is damaged.
- Explicit `TokenCounterKind` recorded in compaction receipts.
- `TokenCounterKind::ProviderChatApprox` for provider-style chat overhead estimates without pretending to be a native tokenizer.
- `BudgetMode::{SoftWarn, HardCascade, FailClosed}` for honest budget behavior.
- `ContentKind` classification for JSON, diffs, Rust, Markdown, Cargo output, shell logs, search results, and prose.
- Structured context summary anchors in `SummaryLossReportV1`.
- Content-aware previews for JSON, diffs, Cargo output, and shell logs.
- `MemoryArchiveRecordV1`, `MemorySink`, and `archive_response_to_memory` for honest adapter-owned archival.
- `FileContextStore::expand` and `FileContextStore::search` across saved receipts.
- `FileContextStore::status` and CLI `status --dir` for receipt count, store bytes, tmp cleanup, and index lifecycle visibility.
- CLI subcommands: `compact`, `store`, `expand`, `search`, `status`, and `diff`.
- Deterministic replay/eval example and reusable replay fixture evaluator.
- Local Hermes SQLite replay evaluation script with privacy-preserving aggregate markdown report.
- Same-transcript comparison script with full/head-tail/context-governor baselines and explicit unsupported receipts for Hermes built-in/offline external adapters.
- Historical Hermes coding-task answerability replay with aggregate/hash-only reporting.
- Certification gates now include same-transcript comparison and optional historical answerability.
- `TokenCounterKind::TiktokenCl100k` as a fail-loud native-tokenizer surface; current default build falls back to provider chat approximation with a warning.
- Search-result reducers preserve path/match lines from read/search-style tool output instead of generic shell-log noise.
- Aggressive replay compaction (`aggressive_v1`) that demotes long path/evidence/tool-heavy context to receipt-backed exact fallback instead of preserving it verbatim.
- Whitespace-normalized replay visibility/search scoring so multiline active tasks and exact fallback records match literal probes.
- Hard-cascade budget enforcement now degrades to the protected minimum instead of failing when the requested target is below the minimum viable prompt.
- Architecture, eval, Hermes integration, and complete implementation-plan docs.

### Changed

- Mixed receipt stores now discriminate on `receipt.schema`; V2 cannot be
  silently deserialized and rewritten as V1.
- Existing V1 receipts remain unchanged and require an explicit parent locator
  before their proven exact leaves can seed a V2 bridge.
- V2 authoritative operations require inherited governed descriptors. V1
  verification accepts historical 8-hex fingerprints and legacy key lengths
  without weakening V2's 32-byte key/full-ID requirement.
- Compaction receipts now disclose token counter kind.
- Summary text now includes structured anchors before lossy previews.
- Receipt-only omitted items can still contribute content-kind-aware previews to the recovery summary while exact fallback remains authoritative.
- Hard budget behavior can now refuse rather than silently overflowing.

### Claim boundary

- No KV-cache compression claim.
- No learned prompt-compression claim.
- No built-in semantic-memory network write claim; memory archival is exposed through an adapter trait.

## 0.1.0

- Initial governed context compaction crate with receipts, exact fallback store, recall filtering, CLI compact mode, and performance harness.

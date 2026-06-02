# Master Issue Matrix — TurboQuant × Semantic-Memory Super-Pass

| Phase | ID | Priority | Area | Problem | Required action | Acceptance proof |
|---|---|---:|---|---|---|---|
| 0 | PRE-001 | P0 | Source basis | TurboQuant may not be available as a sibling crate | Resolve repo layout; no absolute paths; stop if canonical crate unavailable | `SOURCE_BASIS.md` generated; cargo metadata succeeds |
| 1 | TQ-001 | P0 | Storage accounting | Current encoded bytes understate/overstate real layout | Implement explicit `encoded_bytes` and storage tests for current and compact forms | Tests show bytes for f32/SQ8/Turbo profiles |
| 1 | TQ-002 | P0 | QJL | Signs stored as `i8` instead of packed bits | Add bitpacked sign encoding while preserving deterministic sketch semantics | Round-trip + corruption tests |
| 1 | TQ-003 | P0 | Profile | No canonical codec profile/digest | Add `TurboQuantCodecProfileV1` and deterministic digest | Golden fixture |
| 1 | TQ-004 | P1 | Query cost | Query scoring regenerates expensive state per candidate | Add query workspace/prepared scoring | Benchmark or test proves reused workspace |
| 1 | TQ-005 | P1 | Cosine | semantic-memory wants cosine-like ranking | Add norm-aware cosine estimate | Tests against f32 baseline |
| 2 | SM-001 | P0 | Abstraction | semantic-memory has codec behavior coupled to current quantization | Add `VectorCodec` abstraction without breaking SQ8 | Existing quantization tests pass |
| 2 | SM-002 | P0 | Authority | approximate codec could silently affect search | Keep raw/SQ8 path authoritative by default | Config defaults and tests |
| 3 | INT-001 | P0 | Cargo | Need optional dependency without brittle path | Add feature-gated dependency only if real canonical crate path exists | `cargo metadata`, no absolute path grep |
| 3 | INT-002 | P0 | No shadow | Risk of copied TurboQuant math in semantic-memory | Adapter calls `turbo-quant` crate only | grep/assert script |
| 4 | SHADOW-001 | P0 | Ingestion | Need encode sidecar with receipts | Persist or emit encoded artifacts and encode receipts | DB/JSON artifact tests |
| 4 | SHADOW-002 | P1 | Failure mode | Shadow encode failure could break writes | Non-strict shadow failures degrade, do not fail authoritative write | Failure injection test |
| 5 | SEARCH-001 | P0 | Disclosure | Approximate search must not be invisible | Add score provenance/degradation fields | Search disclosure tests |
| 5 | SEARCH-002 | P1 | Rerank | Turbo candidate scoring needs f32 rerank option | Add optional f32 rerank path | Top-k/rerank tests |
| 6 | EVAL-001 | P0 | Evidence | Need recall/top-k/latency evidence | Add evaluation harness and report artifacts | JSON report or DB eval row |
| 6 | EVAL-002 | P1 | Regression | Turbo should not regress existing search | Existing HNSW/vector/hybrid tests pass | Cargo test output |
| 7 | DOC-001 | P0 | Documentation | Users need safe enablement instructions | Update README/docs with feature-gated, non-default behavior | Docs explain defaults and risk |
| 7 | AUDIT-001 | P0 | Final proof | Need hostile audit before declaring done | Run assertion scripts and final checklist | Final report with pass/fail |

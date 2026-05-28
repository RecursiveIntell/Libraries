# P29 Claude Audit Absorption

The uploaded Claude audit is treated as a primary defect source for P29.

## Audit scale

The audit reports:

- 200 confirmed bugs.
- Estimated 100–300 additional bugs in unaudited components.
- Critical/high issues in HNSW, SQLite, search/ranking, memory store, graph/chunker, knowledge-runtime, stack IDs, AiDENs contracts, living-memory, and high-risk unaudited components.

## P29 absorption rule

Not every issue can be deeply fixed in one pass. P29 must:

1. Fix P0/P1 bugs that directly threaten v11A local release.
2. Fix package/evidence/verifier bugs first.
3. Fix HNSW/SQLite/search bugs that can corrupt retrieval truth.
4. Fix AiDENs contract/receipt issues that block material-operation evidence.
5. Quarantine unaudited high-risk components.
6. Add tests/assertions so fixed bug classes cannot regress.

## Top bug families

### HNSW integrity/concurrency

BUG-001 through BUG-010.

Target phase: Phase 05.

### SQLite/migration/database integrity

BUG-011 through BUG-020 and BUG-076 through BUG-085.

Target phase: Phase 06.

### Search/ranking/dedup

BUG-021 through BUG-030 and BUG-053 through BUG-059.

Target phase: Phase 07.

### Quantization/vector disclosure

BUG-031 through BUG-034 and related vector path issues.

Target phase: Phase 08.

### Pool/concurrency/reembed/drop

BUG-035 through BUG-042 and selected medium issues.

Target phase: Phase 09.

### Graph/chunker/knowledge-runtime

BUG-043 through BUG-059 and BUG-086 through BUG-100.

Target phase: Phase 10.

### Stack IDs / AiDENs contracts / living-memory evidence

BUG-060 through BUG-075 and BUG-130 through BUG-149.

Target phase: Phase 11.

### Unaudited high-risk layers

BUG-190 through BUG-200.

Target phase: Phase 04 and quarantine in final known limitations.

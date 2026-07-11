# Memory System Completion Plan
Date: 2026-07-11
Status: Active

## Completed (this session)
- [x] Tier 1: Graph SQL LIMIT, MCP receipts on 16 tools
- [x] Tier 2: Trust enrichment in search, proof-debt tool, contradiction recording
- [x] D1-D4: JSONL persistence, hash-chained ledger, real proof-debt weights, auto-linking
- [x] Fixes: Eliminated drift, verify chain on startup, fuzzy trigram matching
- [x] ROI #1: Semantic contradiction detection using embeddings
- [x] ROI #2: Incremental ClaimTrustIndex rebuild
- [x] ROI #3: sm_benchmark_trust tool

## Remaining work (ranked by ROI)

### Phase 1: Quick wins + high-impact wiring (today)
1. **Subgraph pruning MCP tool** — 2-3 hrs, logic exists in integration.rs, just needs tool exposure
2. **Matryoshka in production search** — 4-6 hrs, multi_resolution_search() exists and tested, needs search.rs wiring
3. **Factor graph as reranking stage** — 5-8 hrs, FactorGraph::propagate() exists, needs search pipeline integration

### Phase 2: Infrastructure (next session)
4. **Complete replay capability** — 6-10 hrs, replay_search_receipt exists, needs replay_inputs table + ReplayMode
5. **RL routing production wiring** — 6-8 hrs, RoutingPolicy exists, needs feedback loop + persistence

### Phase 3: Advanced (future)
6. **SPLADE sparse vectors** — 8-12 hrs, needs BGE-M3 model + storage + fusion
7. **Ledger compaction/rotation** — 8-10 hrs, needs LedgerSnapshot + verify_compaction

### Phase 4: Test fixes
8. **Integration-mode hardening_semantics test** — pre-existing, RRF math differs when integration features enabled

## Execution strategy
- Phase 1 items 1-2: Do directly (controller has enough context)
- Phase 1 item 3 + Phase 2: Delegate to Codex when available (~12 min)
- Phase 3-4: Future sessions
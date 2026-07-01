# MCP Integration ROI Analysis

## Executive Summary

The semantic-memory-mcp server currently exposes 53 MCP tools but under-exposes the underlying semantic-memory library. Several categories of high-ROI additions exist:

1. **Direct library wrappers** - trivial `#[tool]` additions for existing async functions
2. **AiDENs autonomous integration** - gap detection and evaluation tools
3. **Knowledge-runtime query pipeline** - classification, routing, merge functions
4. **Proof-debt budget tools** - risk-gated work accounting

## 1. Missing Direct Wrappers (Trivial Additions)

These functions exist in `semantic-memory/src/lib.rs` but lack MCP tool wrappers:

| Tool Name | Function | Category |
|-----------|----------|----------|
| sm_rebuild_vector_artifacts | rebuild_vector_artifacts | Maintenance |
| sm_rebuild_hnsw_index | rebuild_hnsw_index | Maintenance |
| sm_compact_hnsw | compact_hnsw | Maintenance |
| sm_verify_integrity | verify_integrity | Maintenance |
| sm_list_graph_edges_for_node | list_graph_edges_for_node | Graph |
| sm_list_all_graph_edges | list_all_graph_edges | Graph |
| sm_list_graph_edges_for_neighborhood | list_graph_edges_for_neighborhood | Graph |
| sm_count_graph_edges | count_graph_edges | Graph |
| sm_search_with_context | search_with_context | Search |
| sm_search_fts_only | search_fts_only | Search |
| sm_search_vector_only | search_vector_only | Search |
| sm_embedding_displacement | embedding_displacement | Debug |
| sm_embed | embed | Debug |
| sm_embed_batch | embed_batch | Debug |
| sm_list_scope_domains | list_scope_domains | Bitemporal |
| sm_save_routing_policy | save_routing_policy | Routing |
| sm_load_routing_policy | load_routing_policy | Routing |
| sm_last_import_at | last_import_at | Import |

## 2. AiDENs Autonomous Integration (High ROI)

The `gap_detector` module provides autonomous gap detection. It currently uses HTTP to call MCP tools, but could be exposed directly:

| Tool Name | Function | Description |
|-----------|----------|-------------|
| sm_detect_gaps | gap_detector::detect_gaps() | Detect knowledge gaps (isolated nodes, missing links, contradictions, duplicates, stale facts) |
| sm_detect_gaps_in_namespace | gap_detector::detect_gaps_in_namespace() | Targeted gap detection for specific namespaces |
| sm_generate_tasks | task_generator::generate_tasks() | Convert detected gaps into actionable tasks |
| sm_evaluate_facts | evaluation::evaluate() | Evaluate captured facts for promotion/quarantine |
| sm_hostile_audit | hostile_audit::audit() | Hardened audit pass |
| sm_run_autonomous_loop | loop_driver::run() | Full detect→execute→capture→evaluate loop |

## 3. Knowledge-Runtime Query Pipeline

Query classification and routing functions that add significant value:

| Tool Name | Function | Description |
|-----------|----------|-------------|
| sm_classify_query | query::classify() | Intent classification (semantic/entity/temporal/mixed) |
| sm_route_query_plan | query::plan() | Route planning for multi-stage retrieval |
| sm_merge_results | query::merge() | Result fusion with provenance tracking |
| sm_query_pipeline | Full pipeline | End-to-end query processing with degradation reporting |

## 4. Proof-Debt / Verification Integration

Risk-gated work accounting from `proof_debt.rs`:

| Tool Name | Function | Description |
|-----------|----------|-------------|
| sm_proof_debt_status | proof_debt::outstanding_by_risk() | Current debt by risk class |
| sm_pay_proof_debt | proof_debt::pay_for_claim() | Pay debt for a specific claim |
| sm_viscosity_signal | viscosity::compute_signal() | Compute viscosity signal for strictness adjustment |

## 5. HTTP Endpoint Parity Issues

Some HTTP endpoints lack corresponding MCP tools or are incomplete:

| HTTP Endpoint | MCP Status | Issue |
|---------------|------------|-------|
| /search-routed | sm_search_with_routing exists but verify implementation |
| /rerank | No MCP equivalent | Need sm_rerank_results |
| /maintenance/compact-hnsw | Missing |
| /integrity/verify | Missing (but verify_integrity exists in lib) |
| /embedding-displacement | Missing |

## 6. Recommendations by Priority

**P0 - Immediate (trivial wrappers):**
1. sm_verify_integrity - critical for maintenance
2. sm_list_all_graph_edges - useful for debugging
3. sm_compact_hnsw - optimization tool

**P1 - High value (cross-crate):**
1. sm_detect_gaps - autonomous knowledge improvement
2. sm_classify_query - intent awareness for agents
3. sm_proof_debt_status - budget transparency

**P2 - Enhancement:**
1. sm_rerank_results - LLM reranking via MCP
2. sm_query_pipeline - full query lifecycle exposure

## Implementation Notes

MCP tools follow the pattern:
```rust
#[tool(description = "...", annotations(read_only_hint = true))]
fn sm_<function_name>(&self, Parameters(params): Parameters<ParamStruct>) -> Result<String, ErrorData> {
    let store = &self.bridge.store;
    let result = tokio::task::block_in_place(|| {
        Handle::current().block_on(store.<function>(...))
    });
    // JSON serialization...
}
```

Parameter structs go in `tools.rs` with `schemars::JsonSchema` derivation.
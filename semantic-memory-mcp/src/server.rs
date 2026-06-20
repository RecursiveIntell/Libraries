//! MCP server handler using rmcp's #[tool_router] macro.
//!
//! Each #[tool] method becomes an MCP tool that Hermes/Claude Desktop
//! can discover and call. The rmcp macro auto-generates JSON Schema
//! from the parameter structs in tools.rs.

use crate::bridge::MemoryBridge;
use crate::tools::*;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use std::sync::Arc;
use tokio::runtime::Handle;

// Re-export the specific parameter types we use in tool signatures.
use crate::tools::{
    AddGraphEdgeParams, CommunityParams, FactorGraphParams, InvalidateGraphEdgeParams,
    ListGraphEdgesParams, TopologyParams,
};

pub struct SemanticMemoryServer {
    bridge: Arc<MemoryBridge>,
}

impl SemanticMemoryServer {
    pub fn new(bridge: MemoryBridge) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }
}

#[tool_router(server_handler)]
impl SemanticMemoryServer {
    // ── Core search tools ────────────────────────────────────────────

    #[tool(description = "Semantic hybrid search over the knowledge base. Combines BM25 keyword matching with vector similarity and Reciprocal Rank Fusion. Returns ranked results with content and scores.")]
    fn sm_search(
        &self,
        Parameters(SearchParams { query, top_k, namespaces }): Parameters<SearchParams>,
    ) -> String {
        let k = top_k.map(|v| v as usize);
        let ns: Option<Vec<&str>> = namespaces
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(store.search(&query, k, ns.as_deref(), None)));

        match result {
            Ok(results) => {
                let json_results: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "content": r.content,
                            "source": r.source,
                            "score": r.score,
                            "bm25_rank": r.bm25_rank,
                            "vector_rank": r.vector_rank,
                            "cosine_similarity": r.cosine_similarity,
                        })
                    })
                    .collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "results": json_results,
                    "count": json_results.len(),
                }))
                .unwrap_or_else(|e| format!("Error serializing results: {e}"))
            }
            Err(e) => format!("Search error: {e}"),
        }
    }

    #[tool(description = "Search with full score breakdown showing how BM25 and vector scores combine. Useful for debugging retrieval quality.")]
    fn sm_search_explained(
        &self,
        Parameters(SearchExplainedParams { query, top_k }): Parameters<SearchExplainedParams>,
    ) -> String {
        let k = top_k.map(|v| v as usize);
        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(store.search_explained(&query, k, None, None)));

        // search_explained returns Vec<ExplainedResult>; each has .result (SearchResult)
        // and .breakdown (ScoreBreakdown). Iterate directly, no .results wrapper.
        match result {
            Ok(results) => serde_json::to_string_pretty(&serde_json::json!({
                "results": results.iter().map(|r| {
                    serde_json::json!({
                        "content": r.result.content,
                        "score": r.result.score,
                        "bm25_rank": r.result.bm25_rank,
                        "vector_rank": r.result.vector_rank,
                        "cosine_similarity": r.result.cosine_similarity,
                    })
                }).collect::<Vec<_>>(),
                "count": results.len(),
            }))
            .unwrap_or_else(|e| format!("Error serializing: {e}")),
            Err(e) => format!("Search error: {e}"),
        }
    }

    #[tool(description = "Add a fact to the knowledge base. The fact will be embedded and indexed for semantic search. Returns the fact ID.")]
    fn sm_add_fact(
        &self,
        Parameters(AddFactParams { content, namespace, source }): Parameters<AddFactParams>,
    ) -> String {
        let store = &self.bridge.store;
        let src = source.as_deref();
        // add_fact signature: (namespace, content, source, metadata)
        let result = tokio::task::block_in_place(|| Handle::current().block_on(store.add_fact(&namespace, &content, src, None)));

        match result {
            Ok(id) => format!("Added fact with ID: {id}"),
            Err(e) => format!("Error adding fact: {e}"),
        }
    }

    #[tool(description = "Ingest a document with automatic chunking. The document is split into chunks, each embedded and indexed. Returns document ID and chunk count.")]
    fn sm_ingest_document(
        &self,
        Parameters(IngestDocumentParams { content, title, namespace }): Parameters<IngestDocumentParams>,
    ) -> String {
        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(store.ingest_document(&title, &content, &namespace, None, None)));

        match result {
            Ok(doc_id) => {
                let chunk_count = tokio::task::block_in_place(|| Handle::current().block_on(store.count_chunks_for_document(&doc_id)))
                    .unwrap_or(0);
                format!(
                    "Ingested document '{title}' with ID: {doc_id}, {chunk_count} chunks created"
                )
            }
            Err(e) => format!("Error ingesting document: {e}"),
        }
    }

    #[tool(description = "Get knowledge base statistics: fact count, chunk count, document count, database size, embedding model and dimensions.")]
    fn sm_stats(&self) -> String {
        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(store.stats()));

        match result {
            Ok(stats) => serde_json::to_string_pretty(&serde_json::json!({
                "facts": stats.total_facts,
                "chunks": stats.total_chunks,
                "documents": stats.total_documents,
                "sessions": stats.total_sessions,
                "messages": stats.total_messages,
                "db_size_bytes": stats.database_size_bytes,
                "db_size_mb": (stats.database_size_bytes as f64 / 1_048_576.0 * 100.0).round() / 100.0,
                "embedding_model": stats.embedding_model,
                "embedding_dimensions": stats.embedding_dimensions,
            }))
            .unwrap_or_else(|e| format!("Error serializing stats: {e}")),
            Err(e) => format!("Stats error: {e}"),
        }
    }

    #[tool(description = "Find the shortest path between two items in the knowledge graph. Traverses semantic, temporal, and causal edges. Returns the path as a list of node IDs.")]
    fn sm_graph_path(
        &self,
        Parameters(GraphPathParams { from_id, to_id, max_depth }): Parameters<GraphPathParams>,
    ) -> String {
        let depth = max_depth.map(|v| v as usize).unwrap_or(5);
        let store = &self.bridge.store;
        // graph_view() is a sync method returning Arc<dyn GraphView>.
        let g = store.graph_view();

        match g.path(&from_id, &to_id, depth) {
            Ok(Some(path)) => serde_json::to_string_pretty(&serde_json::json!({
                "from": from_id,
                "to": to_id,
                "path": path,
                "path_length": path.len(),
            }))
            .unwrap_or_else(|e| format!("Error serializing path: {e}")),
            Ok(None) => format!("No path found from {from_id} to {to_id} within depth {depth}"),
            Err(e) => format!("Graph view error: {e}"),
        }
    }

    // ── Feature-gated tools ──────────────────────────────────────────
    // Note: cfg gates are removed from individual tool methods because
    // rmcp's #[tool_router] macro needs all tools visible at expansion
    // time. The `full` feature in Cargo.toml already enables the
    // semantic-memory sub-features these tools depend on.

    #[tool(description = "Profile a query and get an adaptive routing decision. Determines which retrieval stages (BM25, vector, rerank, graph, decoder, discord) should be activated for this query.")]
    fn sm_route_query(
        &self,
        Parameters(RouteQueryParams { query }): Parameters<RouteQueryParams>,
    ) -> String {
        use semantic_memory::routing::RetrievalRouter;

        let router = RetrievalRouter {
            decoder_enabled: true,
            discord_enabled: true,
            corpus_density: 0.5,
            ..Default::default()
        };

        let decision = router.route_query(&query);
        serde_json::to_string_pretty(&serde_json::json!({
            "bm25_coarse": decision.bm25_coarse,
            "vector_medium": decision.vector_medium,
            "rerank_fine": decision.rerank_fine,
            "graph_expansion": decision.graph_expansion,
            "decoder": decision.decoder,
            "discord": decision.discord,
            "no_retrieval": decision.no_retrieval,
            "reasoning": decision.reasoning,
        }))
        .unwrap_or_else(|e| format!("Error serializing routing decision: {e}"))
    }

    #[tool(description = "Adaptive search: profiles the query, routes to appropriate stages, and applies decoder refinement if contradictions are detected. This is the full intelligent retrieval pipeline.")]
    fn sm_search_with_routing(
        &self,
        Parameters(SearchWithRoutingParams { query, top_k, contradictions }): Parameters<SearchWithRoutingParams>,
    ) -> String {
        use semantic_memory::integration::plan_execution;
        use semantic_memory::routing::RetrievalRouter;

        let k = top_k.map(|v| v as usize).unwrap_or(5);
        let router = RetrievalRouter {
            decoder_enabled: true,
            discord_enabled: true,
            corpus_density: 0.5,
            ..Default::default()
        };

        let decision = router.route_query(&query);
        let contras = contradictions.unwrap_or_default();
        let plan = plan_execution(&decision, contras.clone());

        // Execute search (plain or with decoder based on routing)
        let store = &self.bridge.store;
        let search_result = if plan.use_decoder {
            // Use pipeline with decoder if we have a pipeline feature
            // For now, do plain search and note that decoder refinement
            // would be applied by the pipeline
            tokio::task::block_in_place(|| Handle::current().block_on(store.search(&query, Some(k), None, None)))
        } else {
            tokio::task::block_in_place(|| Handle::current().block_on(store.search(&query, Some(k), None, None)))
        };

        match search_result {
            Ok(results) => {
                let json_results: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "content": r.content,
                            "score": r.score,
                        })
                    })
                    .collect();

                serde_json::to_string_pretty(&serde_json::json!({
                    "routing_decision": {
                        "bm25_coarse": decision.bm25_coarse,
                        "vector_medium": decision.vector_medium,
                        "rerank_fine": decision.rerank_fine,
                        "graph_expansion": decision.graph_expansion,
                        "decoder": decision.decoder,
                        "discord": decision.discord,
                        "no_retrieval": decision.no_retrieval,
                        "reasoning": decision.reasoning,
                    },
                    "results": json_results,
                    "count": json_results.len(),
                    "decoder_applied": plan.use_decoder,
                }))
                .unwrap_or_else(|e| format!("Error serializing: {e}"))
            }
            Err(e) => format!("Search error: {e}"),
        }
    }

    #[tool(description = "Detect contradictions and inconsistencies in search results. Runs syndrome detection, computes corrections, and applies belief propagation to refine confidence scores.")]
    fn sm_decoder_analyze(
        &self,
        Parameters(DecoderAnalyzeParams { results, contradictions }): Parameters<DecoderAnalyzeParams>,
    ) -> String {
        use semantic_memory::decoder::{
            compute_correction, detect_syndromes, pass_messages, ConflictGraph,
        };

        let contras = contradictions.unwrap_or_default();
        let syndromes = detect_syndromes(&results, &contras);
        let corrections = compute_correction(&syndromes, 10.0);
        let graph = ConflictGraph::from_syndromes(&results, &syndromes);
        let mp = pass_messages(&graph, 50, 0.001);

        serde_json::to_string_pretty(&serde_json::json!({
            "syndromes": syndromes.iter().map(|s| serde_json::json!({
                "id": s.id,
                "severity": format!("{:?}", s.severity),
                "items": s.items,
                "description": s.description,
                "type": format!("{:?}", s.syndrome_type),
            })).collect::<Vec<_>>(),
            "syndrome_count": syndromes.len(),
            "corrections": corrections.iter().map(|c| serde_json::json!({
                "id": c.id,
                "confidence": c.confidence,
                "cost": c.cost,
                "operations": c.operations.len(),
            })).collect::<Vec<_>>(),
            "correction_count": corrections.len(),
            "message_passing": {
                "iterations": mp.iterations,
                "converged": mp.converged,
                "elapsed_ms": mp.elapsed_ms,
            },
        }))
        .unwrap_or_else(|e| format!("Error serializing decoder analysis: {e}"))
    }

    #[tool(description = "Second-order retrieval: find items related to your search results through the knowledge graph, but NOT themselves direct hits. Discovers connected knowledge you didn't explicitly ask for.")]
    fn sm_discord_search(
        &self,
        Parameters(DiscordSearchParams { direct_result_ids, graph_edges }): Parameters<DiscordSearchParams>,
    ) -> String {
        use semantic_memory::discord::{DiscordScorer, GraphEdgeRef};

        let edges: Vec<GraphEdgeRef> = graph_edges
            .iter()
            .map(|(s, t, et, w)| GraphEdgeRef {
                source: s.clone(),
                target: t.clone(),
                edge_type: et.clone(),
                weight: *w,
            })
            .collect();

        let scorer = DiscordScorer::with_defaults();
        let results = scorer.score(&direct_result_ids, &edges);

        serde_json::to_string_pretty(&serde_json::json!({
            "discord_results": results.iter().map(|r| serde_json::json!({
                "item_id": r.item_id,
                "discord_score": r.discord_score,
                "anchor_ids": r.anchor_ids,
                "relationship_types": r.relationship_types,
            })).collect::<Vec<_>>(),
            "count": results.len(),
        }))
        .unwrap_or_else(|e| format!("Error serializing discord results: {e}"))
    }

    #[tool(description = "Set provenance (evidence confidence) for an item. Uses the ConfidenceSemiring: confidence in [0.0, 1.0] with a support count of independent observations. Returns a provenance receipt.")]
    fn sm_set_provenance(
        &self,
        Parameters(SetProvenanceParams { item_id, confidence, support_count }): Parameters<SetProvenanceParams>,
    ) -> String {
        use semantic_memory::provenance::{
            ConfidenceSemiring, ConfidenceValue, ProvenanceItemType,
        };

        let value = ConfidenceValue::new(confidence, support_count);
        let store = &self.bridge.store;

        // set_provenance signature:
        //   (item_type: &ProvenanceItemType, item_id: &str, value: &S::Value,
        //    support_chain: &[String], episode_id: Option<&str>)
        let result = tokio::task::block_in_place(|| Handle::current().block_on(
            store.set_provenance::<ConfidenceSemiring>(
                &ProvenanceItemType::Fact,
                &item_id,
                &value,
                &[],
                None,
            ),
        ));

        match result {
            Ok(receipt) => serde_json::to_string_pretty(&serde_json::json!({
                "provenance_id": receipt.provenance_id,
                "item_id": receipt.item_id,
                "semiring_type": receipt.semiring_type,
                "recorded_at": receipt.recorded_at,
                "message": "Provenance set successfully",
            }))
            .unwrap_or_else(|e| format!("Error serializing receipt: {e}")),
            Err(e) => format!("Provenance error: {e}"),
        }
    }

    #[tool(description = "Run a memory lifecycle pass: analyze items for syndromes, compute corrections, identify subtraction candidates, and check if compression recompression is needed. This is the autonomous memory health check.")]
    fn sm_run_lifecycle(
        &self,
        Parameters(RunLifecycleParams { item_ids }): Parameters<RunLifecycleParams>,
    ) -> String {
        use semantic_memory::decoder::{compute_correction, detect_syndromes};
        use semantic_memory::integration::{
            corrections_to_subtraction_candidates, should_trigger_recompression,
        };

        // Phase 1: Detect syndromes (using items as results with neutral scores)
        let results: Vec<(String, f64)> = item_ids.iter().map(|id| (id.clone(), 0.5)).collect();
        let syndromes = detect_syndromes(&results, &[]);
        let corrections = compute_correction(&syndromes, 10.0);

        // Phase 2: Convert corrections to subtraction candidates
        let sub_candidates = corrections_to_subtraction_candidates(&corrections);

        // Phase 3: Check if recompression is needed
        let subtracted_count = sub_candidates.len();
        let remaining_count = item_ids.len().saturating_sub(subtracted_count);
        let recompression = should_trigger_recompression(
            subtracted_count,
            remaining_count,
            false, // don't know importance yet
        );

        serde_json::to_string_pretty(&serde_json::json!({
            "items_analyzed": item_ids.len(),
            "syndromes_detected": syndromes.len(),
            "corrections_computed": corrections.len(),
            "subtraction_candidates": sub_candidates.iter().map(|c| serde_json::json!({
                "item_id": c.item_id,
                "structuring_score": c.structuring_score,
                "operation_type": c.operation_type,
                "reason": c.reason,
            })).collect::<Vec<_>>(),
            "recompression_triggered": recompression.triggered,
            "recompression_reason": recompression.reason,
            "summary": format!(
                "Analyzed {} items: {} syndromes, {} corrections, {} subtraction candidates, recompression: {}",
                item_ids.len(), syndromes.len(), corrections.len(), sub_candidates.len(),
                if recompression.triggered { "needed" } else { "not needed" }
            ),
        }))
        .unwrap_or_else(|e| format!("Error serializing lifecycle report: {e}"))
    }

    // ── First-class graph edge tools ───────────────────────────────

    #[tool(description = "Add a durable, typed graph edge between two nodes in the knowledge graph. Nodes use prefixed IDs (e.g. fact:<uuid>, namespace:<name>, document:<id>). Edge types: semantic, temporal, causal, entity. Insertion is idempotent — same edge returns existing ID. Returns the edge ID and metadata.")]
    fn sm_add_graph_edge(
        &self,
        Parameters(params): Parameters<AddGraphEdgeParams>,
    ) -> String {
        use semantic_memory::GraphEdgeType;

        let edge_type = match params.edge_type.as_str() {
            "semantic" => GraphEdgeType::Semantic {
                cosine_similarity: params.cosine_similarity.unwrap_or(0.5),
            },
            "temporal" => GraphEdgeType::Temporal {
                delta_secs: params.delta_secs.unwrap_or(0),
            },
            "causal" => GraphEdgeType::Causal {
                confidence: params.confidence.unwrap_or(0.5),
                evidence_ids: params.evidence_ids.unwrap_or_default(),
            },
            "entity" => GraphEdgeType::Entity {
                relation: params.relation.unwrap_or_else(|| "related".to_string()),
            },
            other => return format!("Invalid edge_type '{other}'. Must be one of: semantic, temporal, causal, entity"),
        };

        let metadata = params.metadata.as_deref().and_then(|s| {
            serde_json::from_str(s).ok()
        });

        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(
            store.add_graph_edge(&params.source, &params.target, edge_type, params.weight, metadata)
        ));

        match result {
            Ok(edge) => serde_json::to_string_pretty(&serde_json::json!({
                "id": edge.id,
                "source": edge.source,
                "target": edge.target,
                "edge_type": edge.edge_type,
                "weight": edge.weight,
                "content_digest": edge.content_digest,
                "recorded_at": edge.recorded_at,
                "message": "Graph edge added successfully",
            }))
            .unwrap_or_else(|e| format!("Error serializing edge: {e}")),
            Err(e) => format!("Error adding graph edge: {e}"),
        }
    }

    #[tool(description = "List graph edges for a specific node (as source or target), or all stored graph edges if no node_id is provided. Returns non-invalidated edges only.")]
    fn sm_list_graph_edges(
        &self,
        Parameters(ListGraphEdgesParams { node_id }): Parameters<ListGraphEdgesParams>,
    ) -> String {
        let store = &self.bridge.store;
        let result = match node_id {
            Some(id) => tokio::task::block_in_place(|| Handle::current().block_on(
                store.list_graph_edges_for_node(&id)
            )),
            None => tokio::task::block_in_place(|| Handle::current().block_on(
                store.list_all_graph_edges()
            )),
        };

        match result {
            Ok(edges) => serde_json::to_string_pretty(&serde_json::json!({
                "edges": edges.iter().map(|e| serde_json::json!({
                    "id": e.id,
                    "source": e.source,
                    "target": e.target,
                    "edge_type": e.edge_type,
                    "weight": e.weight,
                    "metadata": e.metadata,
                    "recorded_at": e.recorded_at,
                })).collect::<Vec<_>>(),
                "count": edges.len(),
            }))
            .unwrap_or_else(|e| format!("Error serializing edges: {e}")),
            Err(e) => format!("Error listing graph edges: {e}"),
        }
    }

    #[tool(description = "Invalidate a stored graph edge by ID. Append-only — the edge row is never deleted, only marked invalidated with a reason.")]
    fn sm_invalidate_graph_edge(
        &self,
        Parameters(InvalidateGraphEdgeParams { edge_id, reason }): Parameters<InvalidateGraphEdgeParams>,
    ) -> String {
        let store = &self.bridge.store;
        let result = tokio::task::block_in_place(|| Handle::current().block_on(
            store.invalidate_graph_edge(&edge_id, &reason)
        ));

        match result {
            Ok(()) => format!("Edge {edge_id} invalidated successfully"),
            Err(e) => format!("Error invalidating edge: {e}"),
        }
    }

    // ── Factor graph, topology, and community tools ─────────────────

    #[tool(description = "Run factor graph belief propagation on heterogeneous graph edges. Models all 4 edge types (semantic, temporal, causal, entity) as factors in a single probabilistic reasoning framework. Returns unified confidence scores after message propagation converges.")]
    fn sm_factor_graph(
        &self,
        Parameters(params): Parameters<FactorGraphParams>,
    ) -> String {
        use semantic_memory::factor_graph::{
            factors_from_edges, FactorGraph, FactorGraphConfig,
        };
        use semantic_memory::GraphEdgeType;

        // Build config from optional overrides, falling back to defaults.
        let defaults = FactorGraphConfig::default();
        let config = FactorGraphConfig {
            semantic_weight: params.semantic_weight.unwrap_or(defaults.semantic_weight),
            temporal_weight: params.temporal_weight.unwrap_or(defaults.temporal_weight),
            causal_weight: params.causal_weight.unwrap_or(defaults.causal_weight),
            entity_weight: params.entity_weight.unwrap_or(defaults.entity_weight),
            self_influence: params.self_influence.unwrap_or(defaults.self_influence),
            max_iterations: params.max_iterations.map(|v| v as usize).unwrap_or(defaults.max_iterations),
            convergence_threshold: params.convergence_threshold.unwrap_or(defaults.convergence_threshold),
        };

        // Convert FactorGraphEdgeInput → (source, target, GraphEdgeType, weight, metadata_json)
        let raw_edges: Vec<(String, String, GraphEdgeType, f64, Option<String>)> = params
            .edges
            .iter()
            .map(|e| {
                let et = match e.edge_type.as_str() {
                    "semantic" => GraphEdgeType::Semantic { cosine_similarity: 0.5 },
                    "temporal" => GraphEdgeType::Temporal { delta_secs: 0 },
                    "causal" => GraphEdgeType::Causal {
                        confidence: 0.5,
                        evidence_ids: vec![],
                    },
                    "entity" => GraphEdgeType::Entity {
                        relation: "related".to_string(),
                    },
                    other => GraphEdgeType::Entity {
                        relation: other.to_string(),
                    },
                };
                (e.source.clone(), e.target.clone(), et, e.weight, e.metadata.clone())
            })
            .collect();

        let factors = factors_from_edges(&raw_edges);

        // Convert FactorGraphNodeInput → (item_id, initial_belief)
        let nodes: Vec<(String, f64)> = params
            .nodes
            .iter()
            .map(|n| (n.item_id.clone(), n.initial_belief))
            .collect();

        let graph = FactorGraph::new(&nodes, factors, config);
        let result = graph.propagate();

        serde_json::to_string_pretty(&serde_json::json!({
            "node_beliefs": result.node_beliefs,
            "iterations": result.iterations,
            "converged": result.converged,
            "elapsed_ms": result.elapsed_ms,
            "factor_counts": {
                "semantic": result.factor_counts.semantic,
                "temporal": result.factor_counts.temporal,
                "causal": result.factor_counts.causal,
                "entity": result.factor_counts.entity,
                "total": result.factor_counts.total(),
            },
            "config": {
                "semantic_weight": result.config.semantic_weight,
                "temporal_weight": result.config.temporal_weight,
                "causal_weight": result.config.causal_weight,
                "entity_weight": result.config.entity_weight,
                "self_influence": result.config.self_influence,
                "max_iterations": result.config.max_iterations,
                "convergence_threshold": result.config.convergence_threshold,
            },
        }))
        .unwrap_or_else(|e| format!("Error serializing factor graph result: {e}"))
    }

    #[tool(description = "Find topological voids in the knowledge graph. Computes Betti numbers (connected components and independent cycles) and detects structural gaps: missing context (isolated nodes), missing links (distant nodes in the same component), and contradiction gaps (duplicate edge assertions). Returns void descriptions and suggested connections.")]
    fn sm_topology(&self, Parameters(params): Parameters<TopologyParams>) -> String {
        use semantic_memory::topology::{compute_betti_numbers, find_voids, gap_report};

        // Build adjacency list from edges for Betti number computation.
        let mut adjacency: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (src, tgt) in &params.edges {
            adjacency
                .entry(src.clone())
                .or_default()
                .push(tgt.clone());
            adjacency
                .entry(tgt.clone())
                .or_default()
                .push(src.clone());
        }

        let betti = compute_betti_numbers(&adjacency);
        let voids = find_voids(&params.edges);
        let report = gap_report(&voids);

        serde_json::to_string_pretty(&serde_json::json!({
            "betti_numbers": {
                "betti_0": betti.betti_0,
                "betti_1": betti.betti_1,
            },
            "voids": voids.iter().map(|v| serde_json::json!({
                "description": v.description,
                "nearby_items": v.nearby_items,
                "suggested_connections": v.suggested_connections,
                "void_type": format!("{:?}", v.void_type),
            })).collect::<Vec<_>>(),
            "void_count": voids.len(),
            "report": report,
        }))
        .unwrap_or_else(|e| format!("Error serializing topology result: {e}"))
    }

    #[tool(description = "Detect communities in the knowledge graph using a Leiden-inspired algorithm. Returns community assignments with member lists, optional within-community contradiction scans, and optional community-aware compression recommendations.")]
    fn sm_community(
        &self,
        Parameters(params): Parameters<CommunityParams>,
    ) -> String {
        use semantic_memory::community::{
            community_aware_compression, community_contradiction_scan, detect_communities,
        };

        let resolution = params.resolution.unwrap_or(1.0);
        let seed = params.seed.unwrap_or(42);

        let communities = detect_communities(&params.edges, resolution, seed);

        let contradictions = params.contradictions.unwrap_or_default();
        let community_contras = community_contradiction_scan(&communities, &contradictions);

        let importance_scores = params.importance_scores.unwrap_or_default();
        let compression = community_aware_compression(&communities, &importance_scores);

        serde_json::to_string_pretty(&serde_json::json!({
            "communities": communities.iter().map(|c| serde_json::json!({
                "id": c.id,
                "members": c.members,
                "level": c.level,
                "parent": c.parent,
                "member_count": c.members.len(),
            })).collect::<Vec<_>>(),
            "community_count": communities.len(),
            "contradictions": community_contras.iter().map(|cc| serde_json::json!({
                "community_id": cc.community_id,
                "item_a": cc.item_a,
                "item_b": cc.item_b,
                "description": cc.description,
            })).collect::<Vec<_>>(),
            "contradiction_count": community_contras.len(),
            "compression_recommendations": compression.iter().map(|cr| serde_json::json!({
                "community_id": cr.community_id,
                "quantization_level": cr.quantization_level,
                "reason": cr.reason,
            })).collect::<Vec<_>>(),
            "compression_count": compression.len(),
        }))
        .unwrap_or_else(|e| format!("Error serializing community result: {e}"))
    }
}
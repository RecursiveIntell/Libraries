//! # cea-store
//!
//! Storage contract for causal edit attribution graphs.
//!
//! This Tier 3 crate defines the narrow persistence boundary used by
//! `cea-core`. It owns adapter-facing row shapes and idempotent graph update
//! semantics, but not attribution policy or SQLite implementation details.

#[derive(Debug, thiserror::Error)]
pub enum CeaStoreError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Backend(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateResult {
    Applied {
        edges_added: usize,
        edges_updated: usize,
    },
    AlreadyProcessed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeaNodeRow {
    pub node_id: String,
    pub node_kind: String,
    pub sig_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CeaEdgeRow {
    pub edge_id: String,
    pub cause_node_id: String,
    pub effect_node_id: String,
    pub weight: f64,
    /// Mirrors `cea_core::EdgeStats::alpha` for persisted edges.
    pub alpha: f64,
    /// Mirrors `cea_core::EdgeStats::beta` for persisted edges.
    pub beta: f64,
    /// Mirrors `cea_core::EdgeStats::observations`.
    pub count: i64,
    pub confidence: f64,
    pub version_id: String,
    pub last_seen: String,
}

pub trait CeaStoreWriteTx {
    fn has_run(&self, run_hash: &str) -> Result<bool, CeaStoreError>;
    fn upsert_node(
        &self,
        node_id: &str,
        node_kind: &str,
        sig_json: &str,
    ) -> Result<(), CeaStoreError>;
    /// Persist one positive observation for the edge identified by
    /// `(cause_node_id, effect_node_id, version_id)`.
    fn upsert_edge(
        &self,
        edge_id: &str,
        cause_node_id: &str,
        effect_node_id: &str,
        weight_delta: f64,
        version_id: &str,
    ) -> Result<bool, CeaStoreError>;
    /// Return the persisted effect ids previously observed for one
    /// `(cause_node_id, version_id)` pair.
    fn load_effect_ids_for_cause(
        &self,
        cause_node_id: &str,
        version_id: &str,
    ) -> Result<Vec<String>, CeaStoreError>;
    /// Persist one negative observation for an existing edge when the cause
    /// reappears but the effect is absent in the new run.
    fn reinforce_negative_edge(
        &self,
        cause_node_id: &str,
        effect_node_id: &str,
        amount: f64,
        version_id: &str,
    ) -> Result<(), CeaStoreError>;
    fn insert_run_log(
        &self,
        run_hash: &str,
        eval_id: &str,
        edges_added: i64,
        edges_updated: i64,
    ) -> Result<(), CeaStoreError>;
}

pub trait CeaStore {
    fn with_write_tx<T, F>(&self, f: F) -> Result<T, CeaStoreError>
    where
        F: FnOnce(&dyn CeaStoreWriteTx) -> Result<T, CeaStoreError>;

    fn load_nodes(&self) -> Result<Vec<CeaNodeRow>, CeaStoreError>;
    fn load_edges(&self, version_id: Option<&str>) -> Result<Vec<CeaEdgeRow>, CeaStoreError>;
}

pub fn update_graph<S: CeaStore>(
    store: &S,
    result: &cea_core::AttributedRunResult,
    eval_id: &str,
    version_id: &str,
    decay_factor: f64,
) -> Result<UpdateResult, CeaStoreError> {
    use std::collections::{BTreeMap, BTreeSet};

    store.with_write_tx(|tx| {
        if tx.has_run(&result.run_hash)? {
            return Ok(UpdateResult::AlreadyProcessed);
        }

        let mut edges_added = 0_usize;
        let mut edges_updated = 0_usize;
        let mut observed_effects_by_cause = BTreeMap::<String, BTreeSet<String>>::new();

        for triple in &result.triples {
            let cause_id = cea_core::edit_op_node_id(&triple.cause);
            let effect_id = cea_core::effect_node_id(&triple.effect);

            tx.upsert_node(&cause_id, "cause", &serde_json::to_string(&triple.cause)?)?;
            tx.upsert_node(
                &effect_id,
                "effect",
                &serde_json::to_string(&triple.effect)?,
            )?;

            let score =
                cea_core::attribution_score(triple.distance, &triple.effect.severity, decay_factor);
            let edge_id = format!("{cause_id}_{effect_id}_{version_id}");
            if tx.upsert_edge(&edge_id, &cause_id, &effect_id, score, version_id)? {
                edges_added += 1;
            } else {
                edges_updated += 1;
            }

            observed_effects_by_cause
                .entry(cause_id)
                .or_default()
                .insert(effect_id);
        }

        for (cause_id, observed_effects) in &observed_effects_by_cause {
            for effect_id in tx.load_effect_ids_for_cause(cause_id, version_id)? {
                if observed_effects.contains(&effect_id) {
                    continue;
                }

                tx.reinforce_negative_edge(cause_id, &effect_id, 1.0, version_id)?;
                edges_updated += 1;
            }
        }

        tx.insert_run_log(
            &result.run_hash,
            eval_id,
            edges_added as i64,
            edges_updated as i64,
        )?;

        Ok(UpdateResult::Applied {
            edges_added,
            edges_updated,
        })
    })
}

pub fn load_graph<S: CeaStore>(
    store: &S,
    version_id: Option<&str>,
) -> Result<cea_core::CausalGraph, CeaStoreError> {
    let nodes = store.load_nodes()?;
    let edges = store.load_edges(version_id)?;
    let mut graph = cea_core::CausalGraph::new();

    for node in &nodes {
        match node.node_kind.as_str() {
            "cause" => {
                if let Ok(signature) =
                    serde_json::from_str::<cea_core::EditOpSignature>(&node.sig_json)
                {
                    let index = graph.graph.add_node(cea_core::CausalNode::Cause(signature));
                    graph.node_index_map.insert(node.node_id.clone(), index);
                }
            }
            "effect" => {
                if let Ok(signature) =
                    serde_json::from_str::<check_runner::EffectSignature>(&node.sig_json)
                {
                    let index = graph
                        .graph
                        .add_node(cea_core::CausalNode::Effect(signature));
                    graph.node_index_map.insert(node.node_id.clone(), index);
                }
            }
            _ => {}
        }
    }

    for edge in &edges {
        let cause_index = graph.node_index_map.get(&edge.cause_node_id);
        let effect_index = graph.node_index_map.get(&edge.effect_node_id);
        if let (Some(cause_index), Some(effect_index)) = (cause_index, effect_index) {
            let stats = cea_core::EdgeStats {
                alpha: edge.alpha,
                beta: edge.beta,
                observations: edge.count.max(0) as u64,
            };
            graph.graph.add_edge(
                *cause_index,
                *effect_index,
                cea_core::CausalEdge {
                    weight: edge.weight,
                    count: edge.count as u64,
                    confidence: stats.confidence(),
                    stats,
                },
            );
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    use cea_core::{
        AnchorKind, AttributedRunResult, AttributionTriple, EditOpKind, EditOpSignature, FileIndex,
        OpIndex, ScopeTag,
    };
    use check_runner::{CheckResult, EffectSignature, ParsedCheckOutput};

    use super::*;

    #[derive(Clone, Default)]
    struct FakeState {
        run_hashes: BTreeSet<String>,
        nodes: BTreeMap<String, CeaNodeRow>,
        edges: BTreeMap<(String, String, String), CeaEdgeRow>,
    }

    struct FakeStore {
        state: RefCell<FakeState>,
        fail_run_log: bool,
    }

    struct FakeWriteTx<'a> {
        state: &'a RefCell<FakeState>,
        fail_run_log: bool,
    }

    impl CeaStoreWriteTx for FakeWriteTx<'_> {
        fn has_run(&self, run_hash: &str) -> Result<bool, CeaStoreError> {
            Ok(self.state.borrow().run_hashes.contains(run_hash))
        }

        fn upsert_node(
            &self,
            node_id: &str,
            node_kind: &str,
            sig_json: &str,
        ) -> Result<(), CeaStoreError> {
            let mut state = self.state.borrow_mut();
            state
                .nodes
                .entry(node_id.to_string())
                .or_insert(CeaNodeRow {
                    node_id: node_id.to_string(),
                    node_kind: node_kind.to_string(),
                    sig_json: sig_json.to_string(),
                });
            Ok(())
        }

        fn upsert_edge(
            &self,
            edge_id: &str,
            cause_node_id: &str,
            effect_node_id: &str,
            weight_delta: f64,
            version_id: &str,
        ) -> Result<bool, CeaStoreError> {
            let mut state = self.state.borrow_mut();
            let key = (
                cause_node_id.to_string(),
                effect_node_id.to_string(),
                version_id.to_string(),
            );
            match state.edges.get_mut(&key) {
                Some(edge) => {
                    let mut stats = cea_core::EdgeStats {
                        alpha: edge.alpha,
                        beta: edge.beta,
                        observations: edge.count.max(0) as u64,
                    };
                    stats.observe_positive(weight_delta);
                    edge.weight += weight_delta;
                    edge.alpha = stats.alpha;
                    edge.beta = stats.beta;
                    edge.count = stats.observations as i64;
                    edge.confidence = stats.confidence();
                    Ok(false)
                }
                None => {
                    let mut stats = cea_core::EdgeStats::default();
                    stats.observe_positive(weight_delta);
                    state.edges.insert(
                        key,
                        CeaEdgeRow {
                            edge_id: edge_id.to_string(),
                            cause_node_id: cause_node_id.to_string(),
                            effect_node_id: effect_node_id.to_string(),
                            weight: weight_delta,
                            alpha: stats.alpha,
                            beta: stats.beta,
                            count: stats.observations as i64,
                            confidence: stats.confidence(),
                            version_id: version_id.to_string(),
                            last_seen: "now".to_string(),
                        },
                    );
                    Ok(true)
                }
            }
        }

        fn insert_run_log(
            &self,
            run_hash: &str,
            _eval_id: &str,
            _edges_added: i64,
            _edges_updated: i64,
        ) -> Result<(), CeaStoreError> {
            if self.fail_run_log {
                return Err(CeaStoreError::Backend(
                    "injected run log failure".to_string(),
                ));
            }

            self.state
                .borrow_mut()
                .run_hashes
                .insert(run_hash.to_string());
            Ok(())
        }

        fn load_effect_ids_for_cause(
            &self,
            cause_node_id: &str,
            version_id: &str,
        ) -> Result<Vec<String>, CeaStoreError> {
            let state = self.state.borrow();
            let mut effect_ids = state
                .edges
                .iter()
                .filter_map(|((stored_cause_id, effect_id, stored_version_id), _)| {
                    (stored_cause_id == cause_node_id && stored_version_id == version_id)
                        .then_some(effect_id.clone())
                })
                .collect::<Vec<_>>();
            effect_ids.sort();
            Ok(effect_ids)
        }

        fn reinforce_negative_edge(
            &self,
            cause_node_id: &str,
            effect_node_id: &str,
            amount: f64,
            version_id: &str,
        ) -> Result<(), CeaStoreError> {
            let mut state = self.state.borrow_mut();
            let key = (
                cause_node_id.to_string(),
                effect_node_id.to_string(),
                version_id.to_string(),
            );
            let edge = state.edges.get_mut(&key).ok_or_else(|| {
                CeaStoreError::Backend(format!(
                    "missing edge for negative observation: {cause_node_id} -> {effect_node_id} ({version_id})"
                ))
            })?;
            let mut stats = cea_core::EdgeStats {
                alpha: edge.alpha,
                beta: edge.beta,
                observations: edge.count.max(0) as u64,
            };
            stats.observe_negative(amount);
            edge.alpha = stats.alpha;
            edge.beta = stats.beta;
            edge.count = stats.observations as i64;
            edge.confidence = stats.confidence();
            edge.last_seen = "now".to_string();
            Ok(())
        }
    }

    impl CeaStore for FakeStore {
        fn with_write_tx<T, F>(&self, f: F) -> Result<T, CeaStoreError>
        where
            F: FnOnce(&dyn CeaStoreWriteTx) -> Result<T, CeaStoreError>,
        {
            let working = RefCell::new(self.state.borrow().clone());
            let tx = FakeWriteTx {
                state: &working,
                fail_run_log: self.fail_run_log,
            };
            let result = f(&tx);
            if result.is_ok() {
                *self.state.borrow_mut() = working.into_inner();
            }
            result
        }

        fn load_nodes(&self) -> Result<Vec<CeaNodeRow>, CeaStoreError> {
            Ok(self.state.borrow().nodes.values().cloned().collect())
        }

        fn load_edges(&self, version_id: Option<&str>) -> Result<Vec<CeaEdgeRow>, CeaStoreError> {
            Ok(self
                .state
                .borrow()
                .edges
                .values()
                .filter(|edge| match version_id {
                    Some(version) => edge.version_id == version,
                    None => true,
                })
                .cloned()
                .collect())
        }
    }

    fn sample_signature(seed: &str) -> EditOpSignature {
        EditOpSignature {
            op_kind: EditOpKind::Insert,
            anchor_kind: AnchorKind::AfterLine,
            lines_added: 2,
            lines_removed: 0,
            context_hash: format!("{seed:0>8}"),
            file_extension: "rs".to_string(),
            scope_tag: ScopeTag::Function,
            op_index: OpIndex(0),
            file_index: FileIndex(0),
        }
    }

    fn sample_effect(seed: &str) -> EffectSignature {
        EffectSignature {
            check_kind: "clippy".to_string(),
            outcome: "warning".to_string(),
            severity: "warning".to_string(),
            message_class: format!("unused_{seed}"),
            line_offset_from_edit: Some(1),
        }
    }

    fn sample_result(seed: &str) -> AttributedRunResult {
        sample_result_with(seed, seed)
    }

    fn sample_result_with(cause_seed: &str, effect_seed: &str) -> AttributedRunResult {
        let triple = AttributionTriple {
            cause: sample_signature(cause_seed),
            effect: sample_effect(effect_seed),
            distance: 2,
            weight: 1.0,
        };

        AttributedRunResult::new(
            vec![triple],
            CheckResult {
                fmt_pass: true,
                clippy_pass: false,
                test_pass: true,
                fmt_output: ParsedCheckOutput::default(),
                clippy_output: ParsedCheckOutput::default(),
                test_output: ParsedCheckOutput::default(),
                total_duration_ms: 1,
            },
        )
    }

    #[test]
    fn update_graph_rolls_back_when_run_log_insert_fails() {
        let store = FakeStore {
            state: RefCell::new(FakeState::default()),
            fail_run_log: true,
        };
        let result = sample_result("rollback");

        let err = update_graph(&store, &result, "eval-fail", "v1", 0.95).unwrap_err();
        assert!(matches!(err, CeaStoreError::Backend(_)));
        assert!(store.load_nodes().unwrap().is_empty());
        assert!(store.load_edges(None).unwrap().is_empty());
    }

    #[test]
    fn update_graph_is_idempotent_after_commit() {
        let store = FakeStore {
            state: RefCell::new(FakeState::default()),
            fail_run_log: false,
        };
        let result = sample_result("once");

        let first = update_graph(&store, &result, "eval-ok", "v1", 0.95).unwrap();
        assert_eq!(
            first,
            UpdateResult::Applied {
                edges_added: 1,
                edges_updated: 0,
            }
        );
        let second = update_graph(&store, &result, "eval-ok", "v1", 0.95).unwrap();
        assert_eq!(second, UpdateResult::AlreadyProcessed);
        assert_eq!(store.load_nodes().unwrap().len(), 2);
        assert_eq!(store.load_edges(None).unwrap().len(), 1);
    }

    #[test]
    fn update_graph_persists_negative_evidence_for_absent_effects() {
        let store = FakeStore {
            state: RefCell::new(FakeState::default()),
            fail_run_log: false,
        };

        update_graph(
            &store,
            &sample_result_with("shared", "one"),
            "eval-1",
            "v1",
            0.95,
        )
        .unwrap();
        let second = update_graph(
            &store,
            &sample_result_with("shared", "two"),
            "eval-2",
            "v1",
            0.95,
        )
        .unwrap();
        assert_eq!(
            second,
            UpdateResult::Applied {
                edges_added: 1,
                edges_updated: 1,
            }
        );

        let cause_id = cea_core::edit_op_node_id(&sample_signature("shared"));
        let first_effect_id = cea_core::effect_node_id(&sample_effect("one"));
        let second_effect_id = cea_core::effect_node_id(&sample_effect("two"));
        let edges = store.load_edges(Some("v1")).unwrap();
        let first_edge = edges
            .iter()
            .find(|edge| edge.cause_node_id == cause_id && edge.effect_node_id == first_effect_id)
            .unwrap();
        let second_edge = edges
            .iter()
            .find(|edge| edge.cause_node_id == cause_id && edge.effect_node_id == second_effect_id)
            .unwrap();

        assert_eq!(first_edge.beta, 2.0);
        assert_eq!(first_edge.count, 2);
        assert_eq!(second_edge.beta, 1.0);
        assert_eq!(second_edge.count, 1);
    }

    #[test]
    fn load_graph_filters_edges_by_version() {
        let store = FakeStore {
            state: RefCell::new(FakeState::default()),
            fail_run_log: false,
        };

        let result_v1 = sample_result("v1");
        let result_v2 = sample_result("v2");
        update_graph(&store, &result_v1, "eval-1", "v1", 0.95).unwrap();
        update_graph(&store, &result_v2, "eval-2", "v2", 0.95).unwrap();

        let filtered = load_graph(&store, Some("v1")).unwrap();
        let all = load_graph(&store, None).unwrap();
        assert_eq!(filtered.graph.edge_count(), 1);
        assert_eq!(all.graph.edge_count(), 2);
    }

    #[test]
    fn load_graph_restores_persisted_edge_stats() {
        let cause = sample_signature("stats");
        let effect = sample_effect("stats");
        let cause_id = cea_core::edit_op_node_id(&cause);
        let effect_id = cea_core::effect_node_id(&effect);
        let stats = cea_core::EdgeStats {
            alpha: 3.5,
            beta: 2.25,
            observations: 4,
        };
        let store = FakeStore {
            state: RefCell::new(FakeState {
                run_hashes: BTreeSet::new(),
                nodes: BTreeMap::from([
                    (
                        cause_id.clone(),
                        CeaNodeRow {
                            node_id: cause_id.clone(),
                            node_kind: "cause".to_string(),
                            sig_json: serde_json::to_string(&cause).unwrap(),
                        },
                    ),
                    (
                        effect_id.clone(),
                        CeaNodeRow {
                            node_id: effect_id.clone(),
                            node_kind: "effect".to_string(),
                            sig_json: serde_json::to_string(&effect).unwrap(),
                        },
                    ),
                ]),
                edges: BTreeMap::from([(
                    (cause_id.clone(), effect_id.clone(), "v1".to_string()),
                    CeaEdgeRow {
                        edge_id: format!("{cause_id}_{effect_id}_v1"),
                        cause_node_id: cause_id.clone(),
                        effect_node_id: effect_id.clone(),
                        weight: 2.5,
                        alpha: stats.alpha,
                        beta: stats.beta,
                        count: stats.observations as i64,
                        confidence: 0.0,
                        version_id: "v1".to_string(),
                        last_seen: "now".to_string(),
                    },
                )]),
            }),
            fail_run_log: false,
        };

        let graph = load_graph(&store, Some("v1")).unwrap();
        let cause_index = graph.node_index_map[&cause_id];
        let effect_index = graph.node_index_map[&effect_id];
        let edge = graph.graph.find_edge(cause_index, effect_index).unwrap();
        let edge = graph.graph.edge_weight(edge).unwrap();

        assert_eq!(edge.stats, stats);
        assert_eq!(edge.count, stats.observations);
        assert!((edge.confidence - stats.confidence()).abs() < 1e-12);
    }
}

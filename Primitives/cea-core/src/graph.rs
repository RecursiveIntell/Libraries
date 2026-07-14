use std::collections::{HashMap, HashSet};
use std::path::Path;

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::calibration;
use check_runner::EffectSignature;

use crate::attribution::{edit_op_node_id, effect_node_id, AttributedRunResult, AttributionConfig};
use crate::error::CeaCoreError;
use crate::types::EditOpSignature;

const GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeStats {
    /// Positive evidence (effect observed when the cause appeared).
    pub alpha: f64,
    /// Negative evidence (cause observed while this effect was absent).
    pub beta: f64,
    /// Total update events applied to the edge.
    pub observations: u64,
}

impl Default for EdgeStats {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            observations: 0,
        }
    }
}

impl EdgeStats {
    pub fn observe_positive(&mut self, amount: f64) {
        let amount = amount.max(0.0);
        self.alpha += amount;
        self.observations += 1;
    }

    pub fn observe_negative(&mut self, amount: f64) {
        let amount = amount.max(0.0);
        self.beta += amount;
        self.observations += 1;
    }

    pub fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta).max(f64::EPSILON)
    }

    pub fn variance(&self) -> f64 {
        let total = self.alpha + self.beta;
        (self.alpha * self.beta) / ((total * total) * (total + 1.0)).max(f64::EPSILON)
    }

    pub fn confidence(&self) -> f64 {
        let reliability = calibration::conservative_reliability(self.alpha, self.beta);
        calibration::advisory_confidence(
            reliability,
            1.0,
            self.observations as f64,
            1,
            calibration::MIN_SAMPLES_PER_SIGNATURE,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CausalNode {
    Cause(EditOpSignature),
    Effect(EffectSignature),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalEdge {
    pub weight: f64,
    pub count: u64,
    pub confidence: f64,
    pub stats: EdgeStats,
}

impl CausalEdge {
    fn new(initial_weight: f64) -> Self {
        let mut edge = Self {
            weight: initial_weight.max(0.0),
            count: 0,
            confidence: 0.0,
            stats: EdgeStats::default(),
        };
        edge.stats.observe_positive(initial_weight.max(0.0));
        edge.sync_legacy_fields();
        edge
    }

    fn reinforce_positive(&mut self, amount: f64) {
        self.weight += amount.max(0.0);
        self.stats.observe_positive(amount);
        self.sync_legacy_fields();
    }

    fn reinforce_negative(&mut self, amount: f64) {
        self.stats.observe_negative(amount);
        self.sync_legacy_fields();
    }

    fn decay(&mut self, factor: f64) {
        self.weight *= factor.clamp(0.0, 1.0);
    }

    fn sync_legacy_fields(&mut self) {
        self.count = self.stats.observations;
        self.confidence = self.stats.confidence();
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoverageSummary {
    pub total_cause_nodes: usize,
    pub total_effect_nodes: usize,
    pub total_edges: usize,
    pub mean_confidence: f64,
}

#[derive(Debug)]
pub struct CausalGraph {
    pub graph: DiGraph<CausalNode, CausalEdge>,
    pub node_index_map: HashMap<String, NodeIndex>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index_map: HashMap::new(),
        }
    }

    pub fn ensure_cause_node(&mut self, signature: &EditOpSignature) -> NodeIndex {
        let node_id = edit_op_node_id(signature);
        if let Some(index) = self.node_index_map.get(&node_id) {
            return *index;
        }

        let index = self.graph.add_node(CausalNode::Cause(signature.clone()));
        self.node_index_map.insert(node_id, index);
        index
    }

    pub fn ensure_effect_node(&mut self, signature: &EffectSignature) -> NodeIndex {
        let node_id = effect_node_id(signature);
        if let Some(index) = self.node_index_map.get(&node_id) {
            return *index;
        }

        let index = self.graph.add_node(CausalNode::Effect(signature.clone()));
        self.node_index_map.insert(node_id, index);
        index
    }

    pub fn update_edge(&mut self, cause_index: NodeIndex, effect_index: NodeIndex, score: f64) {
        self.reinforce_edge(cause_index, effect_index, score, true);
    }

    pub fn ingest_run(
        &mut self,
        run: &AttributedRunResult,
        config: &AttributionConfig,
    ) -> Result<(), CeaCoreError> {
        config.validate()?;
        self.apply_decay(config.decay_factor);

        let mut observed_effects_by_cause: HashMap<NodeIndex, HashSet<String>> = HashMap::new();
        for triple in &run.triples {
            let cause_index = self.ensure_cause_node(&triple.cause);
            let effect_index = self.ensure_effect_node(&triple.effect);
            let effect_id = effect_node_id(&triple.effect);
            self.reinforce_edge(cause_index, effect_index, triple.weight, true);
            observed_effects_by_cause
                .entry(cause_index)
                .or_default()
                .insert(effect_id);
        }

        for (cause_index, observed_effects) in observed_effects_by_cause {
            let edges = self
                .graph
                .edges(cause_index)
                .map(|edge| (edge.id(), edge.target()))
                .collect::<Vec<_>>();
            for (edge_index, target_index) in edges {
                let effect_id = match self.graph.node_weight(target_index) {
                    Some(CausalNode::Effect(signature)) => effect_node_id(signature),
                    _ => {
                        return Err(CeaCoreError::GraphCorruption(
                            "cause node points to a non-effect node".to_string(),
                        ))
                    }
                };
                if !observed_effects.contains(&effect_id) {
                    self.reinforce_edge_by_index(edge_index, 1.0, false)?;
                }
            }
        }

        Ok(())
    }

    pub fn outgoing_edges(&self, node_index: NodeIndex) -> Vec<(NodeIndex, &CausalEdge)> {
        self.graph
            .edges(node_index)
            .map(|edge| (edge.target(), edge.weight()))
            .collect()
    }

    pub fn apply_decay(&mut self, factor: f64) {
        for edge in self.graph.edge_weights_mut() {
            edge.decay(factor);
        }
    }

    pub fn coverage_summary(&self) -> CoverageSummary {
        let mut total_cause_nodes = 0_usize;
        let mut total_effect_nodes = 0_usize;
        let mut confidence_sum = 0.0;
        let mut edge_count = 0_usize;

        for node in self.graph.node_weights() {
            match node {
                CausalNode::Cause(_) => total_cause_nodes += 1,
                CausalNode::Effect(_) => total_effect_nodes += 1,
            }
        }

        for edge in self.graph.edge_weights() {
            edge_count += 1;
            confidence_sum += edge.confidence;
        }

        CoverageSummary {
            total_cause_nodes,
            total_effect_nodes,
            total_edges: edge_count,
            mean_confidence: if edge_count == 0 {
                0.0
            } else {
                confidence_sum / edge_count as f64
            },
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), CeaCoreError> {
        let snapshot = GraphSnapshot::from_graph(self)?;
        // postcard 1.1 with the use-std feature: to_stdvec replaces
        // bincode 1.x's `bincode::serialize`. The error type is
        // postcard::Error (single enum, vs bincode 2.x's split
        // EncodeError/DecodeError), which converts via #[from] in
        // CeaCoreError::Persistence.
        let bytes = postcard::to_stdvec(&snapshot)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, CeaCoreError> {
        let bytes = std::fs::read(path)?;
        // postcard 1.1: from_bytes replaces bincode 1.x's
        // `bincode::deserialize`. Decode failures propagate as
        // postcard::Error → CeaCoreError::Persistence via #[from].
        // Prior code (bincode 1.x) surfaced decode failures as
        // CeaCoreError::GraphCorruption. That was a domain-level
        // decision (load failures mean the file is corrupt or
        // wrong-version) — preserved here by mapping Persistence →
        // GraphCorruption in the call site.
        let snapshot: GraphSnapshot = postcard::from_bytes(&bytes)
            .map_err(|e| CeaCoreError::GraphCorruption(e.to_string()))?;
        snapshot.into_graph()
    }

    pub(crate) fn cause_nodes(&self) -> Vec<(NodeIndex, &EditOpSignature)> {
        self.graph
            .node_indices()
            .filter_map(|index| match self.graph.node_weight(index) {
                Some(CausalNode::Cause(signature)) => Some((index, signature)),
                _ => None,
            })
            .collect()
    }

    fn reinforce_edge(
        &mut self,
        cause_index: NodeIndex,
        effect_index: NodeIndex,
        score: f64,
        positive: bool,
    ) {
        if let Some(edge_index) = self.graph.find_edge(cause_index, effect_index) {
            let _ = self.reinforce_edge_by_index(edge_index, score, positive);
            return;
        }

        let mut edge = CausalEdge::new(score);
        if !positive {
            edge.reinforce_negative(score);
        }
        self.graph.add_edge(cause_index, effect_index, edge);
    }

    fn reinforce_edge_by_index(
        &mut self,
        edge_index: EdgeIndex,
        score: f64,
        positive: bool,
    ) -> Result<(), CeaCoreError> {
        let edge = self.graph.edge_weight_mut(edge_index).ok_or_else(|| {
            CeaCoreError::GraphCorruption(format!("missing edge weight for edge {edge_index:?}"))
        })?;
        if positive {
            edge.reinforce_positive(score);
        } else {
            edge.reinforce_negative(score);
        }
        Ok(())
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphSnapshot {
    version: u32,
    nodes: Vec<NodeRecord>,
    edges: Vec<EdgeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeRecord {
    id: String,
    node: CausalNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeRecord {
    source_id: String,
    target_id: String,
    edge: CausalEdge,
}

impl GraphSnapshot {
    fn from_graph(graph: &CausalGraph) -> Result<Self, CeaCoreError> {
        let mut seen_ids = HashSet::new();
        let mut nodes = Vec::new();
        for index in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(index).cloned() {
                let id = node_identifier(&node);
                if !seen_ids.insert(id.clone()) {
                    return Err(CeaCoreError::GraphCorruption(format!(
                        "duplicate node id while saving graph: {id}"
                    )));
                }
                nodes.push(NodeRecord { id, node });
            }
        }

        let edges = graph
            .graph
            .edge_references()
            .map(|edge| -> Result<EdgeRecord, CeaCoreError> {
                let source_id = graph
                    .graph
                    .node_weight(edge.source())
                    .map(node_identifier)
                    .ok_or_else(|| {
                        CeaCoreError::GraphCorruption(
                            "edge references a missing source node while saving graph".to_string(),
                        )
                    })?;
                let target_id = graph
                    .graph
                    .node_weight(edge.target())
                    .map(node_identifier)
                    .ok_or_else(|| {
                        CeaCoreError::GraphCorruption(
                            "edge references a missing target node while saving graph".to_string(),
                        )
                    })?;
                Ok(EdgeRecord {
                    source_id,
                    target_id,
                    edge: edge.weight().clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            version: GRAPH_SCHEMA_VERSION,
            nodes,
            edges,
        })
    }

    fn into_graph(self) -> Result<CausalGraph, CeaCoreError> {
        if self.version != GRAPH_SCHEMA_VERSION {
            return Err(CeaCoreError::UnsupportedSchemaVersion {
                expected: GRAPH_SCHEMA_VERSION,
                found: self.version,
            });
        }

        let mut graph = CausalGraph::new();
        for record in self.nodes {
            let index = graph.graph.add_node(record.node);
            graph.node_index_map.insert(record.id, index);
        }

        for edge in self.edges {
            let source = graph
                .node_index_map
                .get(&edge.source_id)
                .copied()
                .ok_or_else(|| {
                    CeaCoreError::GraphCorruption(format!(
                        "missing source node during load: {}",
                        edge.source_id
                    ))
                })?;
            let target = graph
                .node_index_map
                .get(&edge.target_id)
                .copied()
                .ok_or_else(|| {
                    CeaCoreError::GraphCorruption(format!(
                        "missing target node during load: {}",
                        edge.target_id
                    ))
                })?;
            graph.graph.add_edge(source, target, edge.edge);
        }

        Ok(graph)
    }
}

fn node_identifier(node: &CausalNode) -> String {
    match node {
        CausalNode::Cause(signature) => edit_op_node_id(signature),
        CausalNode::Effect(signature) => effect_node_id(signature),
    }
}

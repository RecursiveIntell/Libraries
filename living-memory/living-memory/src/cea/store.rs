use crate::cea::graph::CausalGraph;
use crate::cea::instrumentation::AttributedRunResult;
use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::store::ForgeStore;
use rusqlite::Transaction;

pub use cea_store::UpdateResult;

/// Update the causal graph from an attributed run result.
///
/// Invariant I10: Idempotent — same run_hash produces no graph change on replay.
pub fn update_graph(
    store: &ForgeStore,
    result: &AttributedRunResult,
    eval_id: &str,
    version_id: &str,
    config: &ForgeConfig,
) -> ForgeResult<UpdateResult> {
    store.with_transaction_conn(|tx| update_graph_with_tx(tx, result, eval_id, version_id, config))
}

/// Update the causal graph using an already-open transaction handle.
///
/// This avoids nested write transactions and keeps CEA writes inside the
/// owning store transaction boundary.
pub fn update_graph_with_tx(
    tx: &Transaction,
    result: &AttributedRunResult,
    eval_id: &str,
    version_id: &str,
    config: &ForgeConfig,
) -> ForgeResult<UpdateResult> {
    let sqlite = cea_sqlite::SqliteCeaStoreTx::new(tx);
    Ok(cea_store::update_graph(
        &sqlite,
        result,
        eval_id,
        version_id,
        config.cea.attribution_decay_factor,
    )?)
}

/// Load the causal graph from the store into memory.
///
/// Reconstructs the in-memory `CausalGraph` from persisted `cea_nodes` and `cea_edges`.
/// If `version_id` is `Some`, only edges for that version are loaded; otherwise all edges.
pub fn load_graph(store: &ForgeStore, version_id: Option<&str>) -> ForgeResult<CausalGraph> {
    store.with_conn(|conn| {
        let sqlite = cea_sqlite::SqliteCeaStoreConn::new(conn);
        Ok(cea_store::load_graph(&sqlite, version_id)?)
    })
}

/// Load the causal graph from an already-open transaction handle.
///
/// Reuses the caller-provided transaction for transaction-local graph reads.
pub fn load_graph_with_tx(tx: &Transaction, version_id: Option<&str>) -> ForgeResult<CausalGraph> {
    let sqlite = cea_sqlite::SqliteCeaStoreTx::new(tx);
    Ok(cea_store::load_graph(&sqlite, version_id)?)
}

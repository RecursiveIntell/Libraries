use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_graph::config::GraphConfig;
use agent_graph::event_sink::GraphEvent;
use agent_graph::state::{AgentState, StateLimits};
use serde_json::Value;

use crate::compiler::{compile, CompileContext};
use crate::evidence::{bundle, digest, redact};
use crate::spec::{ensure_size, GraphSpec, MAX_OUTPUT_BYTES, MAX_STATE_BYTES};

const MAX_RUNS: usize = 100;
const MAX_ACTIVE_RUNS: usize = 8;

fn is_terminal(status: &str) -> bool {
    matches!(status, "completed" | "failed" | "cancelled")
}

#[derive(Clone)]
pub struct RunRecord {
    pub run_id: String,
    pub trace: String,
    pub graph_id: String,
    pub graph_version: String,
    pub status: String,
    pub success: Option<bool>,
    pub input: Value,
    pub state: Value,
    pub final_state: Value,
    pub steps: Vec<Value>,
    pub error: Option<String>,
    pub events: VecDeque<Value>,
    pub next_cursor: u64,
    pub dropped_events: u64,
    pub receipt: Value,
    pub bundle: Value,
    pub cancelled: Arc<AtomicBool>,
}

impl RunRecord {
    pub fn public(&self) -> Value {
        serde_json::json!({
            "run_id":self.run_id,"trace":self.trace,"graph_id":self.graph_id,"graph_version":self.graph_version,
            "storage_class":"volatile","status":self.status,"success":self.success,"final_state":self.final_state,
            "state":self.state,"steps":self.steps,"error":self.error,"receipt":self.receipt,
            "replay_capability":"integrity_verified"
        })
    }
}

#[derive(Clone)]
pub struct RunManager {
    inner: Arc<Mutex<Inner>>,
    counter: Arc<AtomicU64>,
}
struct Inner {
    runs: HashMap<String, RunRecord>,
    order: VecDeque<String>,
}

impl Default for RunManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                runs: HashMap::new(),
                order: VecDeque::new(),
            })),
            counter: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl RunManager {
    pub fn allocate(
        &self,
        graph_id: &str,
        graph_version: &str,
        input: Value,
    ) -> Result<String, String> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let run_id = format!("run-{millis:x}-{n:x}");
        let record = RunRecord {
            run_id: run_id.clone(),
            trace: format!("trace-{millis:x}-{n:x}"),
            graph_id: graph_id.into(),
            graph_version: graph_version.into(),
            status: "accepted".into(),
            success: None,
            input,
            state: Value::Null,
            final_state: Value::Null,
            steps: vec![],
            error: None,
            events: VecDeque::new(),
            next_cursor: 0,
            dropped_events: 0,
            receipt: Value::Null,
            bundle: Value::Null,
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let mut inner = self.inner.lock().expect("run registry poisoned");
        if inner.order.len() == MAX_RUNS {
            let Some(old) = inner.order.iter().find_map(|id| {
                inner
                    .runs
                    .get(id)
                    .filter(|run| is_terminal(&run.status))
                    .map(|_| id.clone())
            }) else {
                return Err(format!(
                    "run retention capacity reached: {MAX_RUNS} live runs cannot be evicted"
                ));
            };
            inner.order.retain(|id| id != &old);
            inner.runs.remove(&old);
        }
        inner.order.push_back(run_id.clone());
        inner.runs.insert(run_id.clone(), record);
        Ok(run_id)
    }

    /// Atomically reserves one of the bounded execution slots for an accepted run.
    pub fn admit_async(&self, id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().map_err(|_| "run registry poisoned")?;
        let active_runs = inner
            .runs
            .values()
            .filter(|run| run.status == "running")
            .count();
        if active_runs >= MAX_ACTIVE_RUNS {
            return Err(format!(
                "active run capacity reached: {MAX_ACTIVE_RUNS} concurrent runs"
            ));
        }
        let run = inner.runs.get_mut(id).ok_or("run not found")?;
        if run.status != "accepted" {
            return Err(format!("run '{id}' is not awaiting admission"));
        }
        run.status = "running".into();
        Ok(())
    }

    pub fn execute(
        &self,
        run_id: &str,
        spec: GraphSpec,
        base_url: String,
        default_model: String,
    ) -> Result<Value, String> {
        self.update(run_id, |r| r.status = "running".into())?;
        let (input, cancelled) = {
            let inner = self.inner.lock().map_err(|_| "run registry poisoned")?;
            let r = inner.runs.get(run_id).ok_or("run not found")?;
            (r.input.clone(), r.cancelled.clone())
        };
        let events = Arc::new(Mutex::new(Vec::<GraphEvent>::new()));
        let graph = compile(
            &spec,
            CompileContext {
                base_url,
                default_model,
                cancelled,
                events: events.clone(),
            },
        )?;
        let mut initial = HashMap::new();
        initial.insert("__input__".into(), input.clone());
        if let Value::Object(map) = &input {
            initial.extend(map.clone());
        }
        let state = AgentState::with_data_and_limits(
            initial,
            StateLimits {
                max_keys: 1000,
                max_value_bytes: 256 * 1024,
                max_history_len: 100,
                lock_timeout: std::time::Duration::from_secs(5),
            },
        );
        let snapshot = state.clone();
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let config = GraphConfig::new()
            .with_recursion_limit(spec.max_iterations.unwrap_or(64))
            .with_max_parallelism(spec.max_parallelism.unwrap_or(8));
        let (result, core_receipt) =
            rt.block_on(graph.execute_with_receipt(&spec.entry, state, config));
        let exported = rt.block_on(snapshot.export());
        let state_value = serde_json::to_value(exported).map_err(|e| e.to_string())?;
        ensure_size(&state_value, MAX_STATE_BYTES, "total state")?;
        let final_state = state_value.get("__input__").cloned().unwrap_or(Value::Null);
        ensure_size(&final_state, MAX_OUTPUT_BYTES, "execution output")?;
        let graph_events = events
            .lock()
            .map_err(|_| "event registry poisoned")?
            .clone();
        let mut outputs_by_node: HashMap<String, VecDeque<Value>> = HashMap::new();
        for event in &graph_events {
            if let GraphEvent::StateUpdate {
                node_id, updates, ..
            } = event
            {
                let output = updates
                    .get("__input__")
                    .or_else(|| updates.get("__route__"))
                    .cloned()
                    .unwrap_or_else(|| serde_json::to_value(updates).unwrap_or(Value::Null));
                outputs_by_node
                    .entry(node_id.clone())
                    .or_default()
                    .push_back(output);
            }
        }
        let steps: Vec<Value> = core_receipt
            .steps
            .iter()
            .map(|step| {
                let output = outputs_by_node
                    .get_mut(&step.agent_id)
                    .and_then(VecDeque::pop_front);
                serde_json::json!({
                    "node_id": step.agent_id,
                    "status": if step.error.is_some() { "failed" } else { "success" },
                    "output": output
                })
            })
            .collect();
        let error = result.err().map(|e| e.to_string());
        let success = error.is_none();
        let trace = self.get(run_id).ok_or("run not found")?.trace;
        let graph_version = self.get(run_id).ok_or("run not found")?.graph_version;
        let models: Vec<Value> = spec.nodes.iter().filter(|node| matches!(node.node_type, crate::spec::NodeType::Llm)).map(|node| serde_json::json!({
            "node_id":node.id,"model_alias":node.model.as_deref().unwrap_or("server_default"),"prompt_digest":digest(&Value::String(node.prompt.clone().unwrap_or_else(||"{input}".into())))
        })).collect();
        let receipt = serde_json::json!({"schema":"agent-graph-mcp-receipt-v1","run_id":run_id,"trace":trace,"graph_version":graph_version,
            "input_digest":digest(&input),"output_digest":digest(&state_value),"step_count":steps.len(),"models":models,
            "core":core_receipt,"dependency_envelopes_complete":false,"replay_capability":"integrity_verified"});
        let artifact = bundle(run_id, &graph_version, &input, &state_value, &receipt);
        self.update(run_id, |r| {
            r.status = if success {
                "completed"
            } else if r.cancelled.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "failed"
            }
            .into();
            r.success = Some(success);
            r.state = state_value.clone();
            r.final_state = final_state.clone();
            r.steps = steps.clone();
            r.error = error.clone();
            r.receipt = receipt.clone();
            r.bundle = artifact.clone();
            for event in graph_events {
                push_event(r, serde_json::to_value(event).unwrap_or(Value::Null));
            }
        })?;
        Ok(self.get(run_id).expect("updated run").public())
    }

    pub fn start(&self, run_id: String, spec: GraphSpec, base_url: String, model: String) {
        let manager = self.clone();
        std::thread::spawn(move || {
            if let Err(error) = manager.execute(&run_id, spec, base_url, model) {
                let _ = manager.update(&run_id, |r| {
                    r.status = "failed".into();
                    r.success = Some(false);
                    r.error = Some(error.clone());
                });
            }
        });
    }
    pub fn cancel(&self, id: &str) -> Result<Value, String> {
        self.update(id, |r| r.cancelled.store(true, Ordering::SeqCst))?;
        Ok(
            serde_json::json!({"run_id":id,"status":"cancellation_requested","interrupts_blocked_provider_call":false}),
        )
    }
    pub fn get(&self, id: &str) -> Option<RunRecord> {
        self.inner.lock().ok()?.runs.get(id).cloned()
    }
    pub(crate) fn remove(&self, id: &str) -> Option<RunRecord> {
        let mut inner = self.inner.lock().ok()?;
        if inner
            .runs
            .get(id)
            .is_some_and(|run| run.status == "running")
        {
            return None;
        }
        inner.order.retain(|run_id| run_id != id);
        inner.runs.remove(id)
    }
    pub fn list(&self) -> Vec<Value> {
        self.inner
            .lock()
            .map(|i| {
                i.order
                    .iter()
                    .filter_map(|id| i.runs.get(id).map(RunRecord::public))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn events(&self, id: &str, cursor: u64, limit: usize) -> Result<Value, String> {
        let r = self.get(id).ok_or("run not found")?;
        let first = r
            .events
            .front()
            .and_then(|v| v.get("cursor"))
            .and_then(Value::as_u64)
            .unwrap_or(r.next_cursor);
        let events: Vec<_> = r
            .events
            .iter()
            .filter(|v| v["cursor"].as_u64().unwrap_or(0) >= cursor)
            .take(limit.min(200))
            .cloned()
            .collect();
        Ok(
            serde_json::json!({"run_id":id,"events":events,"next_cursor":r.next_cursor,"gap":cursor<first,"truncated":r.dropped_events>0,"dropped":r.dropped_events}),
        )
    }
    fn update(&self, id: &str, f: impl FnOnce(&mut RunRecord)) -> Result<(), String> {
        let mut inner = self.inner.lock().map_err(|_| "run registry poisoned")?;
        let r = inner.runs.get_mut(id).ok_or("run not found")?;
        f(r);
        Ok(())
    }
}

fn push_event(run: &mut RunRecord, event: Value) {
    if run.events.len() == 512 {
        run.events.pop_front();
        run.dropped_events += 1;
    }
    let cursor = run.next_cursor;
    run.next_cursor += 1;
    run.events
        .push_back(serde_json::json!({"cursor":cursor,"event":redact(&event)}));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_admission_is_bounded_atomically() {
        let manager = RunManager::default();
        for index in 0..MAX_ACTIVE_RUNS {
            let id = manager
                .allocate("graph", "version", serde_json::json!({"index": index}))
                .expect("allocate admitted run");
            manager.admit_async(&id).expect("admit within cap");
        }
        let overflow = manager
            .allocate("graph", "version", Value::Null)
            .expect("registry still has room");
        assert!(manager.admit_async(&overflow).is_err());
        manager.remove(&overflow);
        assert!(manager.get(&overflow).is_none());
    }

    #[test]
    fn retention_never_evicts_live_runs() {
        let manager = RunManager::default();
        let mut ids = Vec::new();
        for _ in 0..MAX_RUNS {
            ids.push(
                manager
                    .allocate("graph", "version", Value::Null)
                    .expect("allocate retained run"),
            );
        }
        assert!(manager.allocate("graph", "version", Value::Null).is_err());
        manager
            .update(&ids[0], |record| record.status = "completed".into())
            .expect("mark terminal");
        let replacement = manager
            .allocate("graph", "version", Value::Null)
            .expect("terminal record may be evicted");
        assert!(manager.get(&ids[0]).is_none());
        assert!(manager.get(&replacement).is_some());
    }
}

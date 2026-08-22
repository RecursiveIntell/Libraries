//! Python bindings for agent-graph with custom sync executor.

use agent_graph::event_sink::{ChannelEventSink, EventSink, GraphEvent};
use agent_graph::stream::StreamEvent;
use agent_graph::{AgentState as RustState, END, START};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[cfg(all(feature = "observability", unix))]
mod observability;
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("agent-graph-py")
            .build()
            .expect("tokio runtime")
    })
}

fn py_to_value(py: Python<'_>, obj: &Bound<'_, PyAny>) -> Result<Value, PyErr> {
    let json = py.import("json")?;
    let text: String = json.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&text).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

fn value_to_py(py: Python<'_>, value: &Value) -> Result<Py<PyAny>, PyErr> {
    let text = serde_json::to_string(value).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let json = py.import("json")?;
    Ok(json.call_method1("loads", (text,))?.unbind())
}

#[pyclass(name = "AgentState", skip_from_py_object)]
#[derive(Clone)]
pub struct PyAgentState {
    pub(crate) inner: RustState,
}

#[pymethods]
impl PyAgentState {
    #[new]
    fn new(py: Python<'_>, initial: Option<Bound<'_, PyAny>>) -> PyResult<Self> {
        let state = RustState::new();
        if let Some(obj) = initial {
            let val = py_to_value(py, &obj)?;
            let map: HashMap<String, Value> =
                serde_json::from_value(val).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            for (k, v) in map {
                runtime()
                    .block_on(state.set_raw(&k, v))
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            }
        }
        Ok(Self { inner: state })
    }

    fn get(&self, py: Python<'_>, key: &str) -> PyResult<Py<PyAny>> {
        let value: Value = runtime()
            .block_on(self.inner.get(key.as_ref()))
            .map_err(|e: agent_graph::AgentGraphError| PyRuntimeError::new_err(e.to_string()))?;
        value_to_py(py, &value)
    }

    fn set(&self, py: Python<'_>, key: &str, value: Bound<'_, PyAny>) -> PyResult<()> {
        let v = py_to_value(py, &value)?;
        runtime()
            .block_on(self.inner.set_raw(key, v))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn as_dict(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let data = runtime().block_on(self.inner.export());
        value_to_py(py, &Value::Object(data.into_iter().collect()))
    }

    fn get_all_keys(&self) -> PyResult<Vec<String>> {
        Ok(runtime().block_on(self.inner.keys()))
    }
}

// ── Synchronous graph execution ──────────────────────────────────────

#[pyclass]
pub struct StateGraph {
    nodes: HashMap<String, Py<PyAny>>,
    edges: HashMap<String, Vec<String>>,
    routers: HashMap<String, Py<PyAny>>,
}

#[pymethods]
impl StateGraph {
    #[new]
    fn new(_schema: Py<PyAny>) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            routers: HashMap::new(),
        }
    }

    fn add_node(&mut self, py: Python<'_>, name: String, callable: Py<PyAny>) -> PyResult<()> {
        self.nodes.insert(name, callable.clone_ref(py));
        Ok(())
    }

    fn add_edge(&mut self, from: String, to: String) -> PyResult<()> {
        self.edges.entry(from).or_default().push(to);
        Ok(())
    }

    fn add_conditional_edges(
        &mut self,
        py: Python<'_>,
        from: String,
        router: Py<PyAny>,
    ) -> PyResult<()> {
        self.routers.insert(from, router.clone_ref(py));
        Ok(())
    }

    fn compile(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn invoke(&self, py: Python<'_>, initial: Option<Py<PyAny>>) -> PyResult<PyAgentState> {
        let init_val = initial.map(|x| x.clone_ref(py));
        let state = PyAgentState::new(py, init_val.map(|v| v.bind(py).clone()))?;

        let mut current: Vec<String> = self.edges.get(START).cloned().unwrap_or_default();
        let mut iteration = 0;
        let max_iters = 100;

        while !current.is_empty() && iteration < max_iters {
            iteration += 1;
            let mut next_nodes = Vec::new();

            for node_name in &current {
                if node_name == END || node_name == START {
                    if let Some(targets) = self.edges.get(node_name) {
                        next_nodes.extend(targets.clone());
                    }
                    continue;
                }

                // Call Python node callback.
                if let Some(callable) = self.nodes.get(node_name) {
                    let wrapped = PyAgentState {
                        inner: state.inner.clone(),
                    };
                    let wrapped_py =
                        Py::new(py, wrapped).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                    let result = callable
                        .bind(py)
                        .call1((wrapped_py.clone_ref(py),))
                        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                    if let Ok(dict) = result.cast_exact::<pyo3::types::PyDict>() {
                        for (k, v) in dict {
                            let key: String = k.to_string();
                            let val = py_to_value(py, &v)?;
                            runtime()
                                .block_on(state.inner.set_raw(&key, val))
                                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                        }
                    }
                }

                // Check router.
                let mut routed = false;
                if let Some(router) = self.routers.get(node_name) {
                    let wrapped = PyAgentState {
                        inner: state.inner.clone(),
                    };
                    let wrapped_py =
                        Py::new(py, wrapped).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                    let result = router
                        .bind(py)
                        .call1((wrapped_py.clone_ref(py),))
                        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                    if let Ok(name) = result.extract::<String>() {
                        next_nodes.push(name);
                        routed = true;
                    } else if let Ok(names) = result.extract::<Vec<String>>() {
                        next_nodes.extend(names);
                        routed = true;
                    } else if result.is_none() {
                        routed = true;
                    }
                }

                if !routed {
                    if let Some(targets) = self.edges.get(node_name) {
                        for t in targets {
                            if t != END {
                                next_nodes.push(t.clone());
                            }
                        }
                    }
                }
            }

            let mut seen = HashSet::new();
            next_nodes.retain(|n| seen.insert(n.clone()));
            current = next_nodes;
        }

        Ok(state)
    }

    fn stream(&self, py: Python<'_>, initial: Option<Py<PyAny>>) -> PyResult<Vec<PyAgentState>> {
        let (result, _) = execute_stream(self, py, initial)?;
        Ok(vec![result])
    }

    fn stream_events(
        &self,
        py: Python<'_>,
        initial: Option<Py<PyAny>>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let (_, events) = execute_stream(self, py, initial)?;
        Ok(events)
    }
}

fn execute_stream(
    this: &StateGraph,
    py: Python<'_>,
    initial: Option<Py<PyAny>>,
) -> PyResult<(PyAgentState, Vec<Py<PyAny>>)> {
    // The event channel is bounded and drained only after execution completes.
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(256);
    let sink: Arc<dyn EventSink> = Arc::new(ChannelEventSink::new(tx));
    let graph = StateGraph {
        nodes: this
            .nodes
            .iter()
            .map(|(key, value)| (key.clone(), value.clone_ref(py)))
            .collect(),
        edges: this.edges.clone(),
        routers: this
            .routers
            .iter()
            .map(|(key, value)| (key.clone(), value.clone_ref(py)))
            .collect(),
    };
    let initial = initial.map(|value| value.clone_ref(py));
    let result = runtime().block_on(async move {
        tokio::task::spawn_blocking(move || {
            Python::attach(|py| {
                sink.emit(GraphEvent::RunStart {
                    run_id: "python-stream".into(),
                    trace_id: "python-stream".into(),
                    trace_ctx: None,
                    graph_name: None,
                });
                let result = graph.invoke(py, initial);
                sink.emit(GraphEvent::RunEnd {
                    run_id: "python-stream".into(),
                    trace_id: "python-stream".into(),
                    trace_ctx: None,
                });
                result
            })
        })
        .await
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
    })?;
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        let value = serde_json::to_value(event)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        events.push(value_to_py(py, &value)?);
    }
    Ok((result, events))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StateGraph>()?;
    m.add_class::<PyAgentState>()?;
    #[cfg(all(feature = "observability", unix))]
    m.add_class::<observability::ObservationClient>()?;
    m.add("START", agent_graph::START)?;
    m.add("END", agent_graph::END)?;
    Ok(())
}

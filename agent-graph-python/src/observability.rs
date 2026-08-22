//! Optional Python-to-stack-monitor Unix observation client.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use stack_monitor::{EmitStatus, UnixMonitorClient, UnixMonitorClientHandle};
use stack_observation::ObservationEnvelope;

/// Python client that submits canonical observation dictionaries to the local collector.
#[pyclass]
pub struct ObservationClient {
    client: UnixMonitorClient,
    handle: Option<UnixMonitorClientHandle>,
}

#[pymethods]
impl ObservationClient {
    #[new]
    fn new(path: String, capacity: Option<usize>) -> PyResult<Self> {
        let (client, handle) = stack_monitor::start_unix_client(path, capacity.unwrap_or(256))
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(Self {
            client,
            handle: Some(handle),
        })
    }

    /// Submit one canonical observation envelope without blocking on SQLite.
    fn emit(&self, py: Python<'_>, event: Bound<'_, PyAny>) -> PyResult<String> {
        let json = py.import("json")?;
        let text: String = json.call_method1("dumps", (event,))?.extract()?;
        let envelope: ObservationEnvelope = serde_json::from_str(&text)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let status = self
            .client
            .try_emit(envelope)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(match status {
            EmitStatus::Accepted => "accepted",
            EmitStatus::Dropped => "dropped",
            EmitStatus::CollectorUnavailable => "collector_unavailable",
        }
        .into())
    }

    /// Stop the background Unix sender and drain its accepted queue.
    fn close(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.shutdown();
        }
    }

    /// Return producer transport counters as a Python dictionary.
    fn stats(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let stats = self.client.stats();
        let json = serde_json::json!({
            "attempted": stats.attempted,
            "accepted": stats.accepted,
            "dropped": stats.dropped,
            "sent": stats.sent,
            "connection_failures": stats.connection_failures,
        });
        let module = py.import("json")?;
        let text = serde_json::to_string(&json)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(module.call_method1("loads", (text,))?.unbind())
    }
}

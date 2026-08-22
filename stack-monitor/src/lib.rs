//! # stack-monitor — LLM Activity Monitor for the RecursiveIntell Stack
//!
//! Captures LLM activity from stack crates into a queryable SQLite store.
//!
//! ## Three capture paths
//!
//! 1. **Tracing layer** — subscribes to `tracing` events from stack crates
//! 2. **Event bridge** — implements `llm-pipeline::EventHandler` to capture payload lifecycle
//! 3. **Direct instrumentation** — manual `record()` calls for custom Activity

pub mod event_bridge;
#[cfg(unix)]
pub mod ipc;
pub mod live;
#[cfg(unix)]
pub mod live_ipc;
pub mod models;
pub mod projections;
#[cfg(feature = "semantic-memory-bridge")]
pub mod semantic_memory_observation;
pub mod store;
#[cfg(feature = "tool-runtime-bridge")]
pub mod tool_observation;
pub mod tracing_layer;
pub mod tracing_observation;
pub mod transport;

#[cfg(feature = "agent-graph-bridge")]
pub mod agent_graph_bridge;
#[cfg(feature = "agent-graph-bridge")]
pub mod agent_graph_observation;

pub use event_bridge::{LlmPipelineEventHandler, LlmPipelineObservationHandler};
#[cfg(unix)]
pub use ipc::{
    start_unix_client, start_unix_collector, start_unix_collector_with_live, IpcStats,
    UnixCollectorHandle, UnixMonitorClient, UnixMonitorClientHandle,
};
pub use live::{LiveEvent, LiveHub, LiveReceive, LiveSubscription};
#[cfg(unix)]
pub use live_ipc::{
    start_unix_live_client, start_unix_live_server, LiveIpcError, UnixLiveClientHandle,
    UnixLiveServerHandle, UnixLiveSubscription,
};
pub use models::MonitoredEvent;
pub use projections::{HealthProjection, ProjectionService, TimelineProjection};
#[cfg(feature = "semantic-memory-bridge")]
pub use semantic_memory_observation::{
    EmbedderObservationWrapper, SemanticMemoryReceiptObservationSink,
};
pub use store::{ActivityStore, ObservationFilter};
#[cfg(feature = "tool-runtime-bridge")]
pub use tool_observation::ToolObservationSink;
pub use tracing_layer::TracingActivityLayer;
pub use tracing_observation::TracingObservationLayer;
pub use transport::{
    start_collector, start_collector_with_live, CollectorHandle, EmitStatus, MonitorClient,
    ObservationEmitter, TransportStats,
};

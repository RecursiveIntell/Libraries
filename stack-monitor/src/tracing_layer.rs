//! Tracing subscriber layer that captures stack crate tracing events into the ActivityStore.
//!
//! This is installed on top of the tracing registry:
//!
//! ```rust
//! use tracing_subscriber::registry;
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::util::SubscriberInitExt;
//! use stack_monitor::{ActivityStore, TracingActivityLayer};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let store = ActivityStore::open("activity.db")?;
//! let layer = TracingActivityLayer::new(&store);
//!
//! registry()
//!     .with(layer)
//!     .with(tracing_subscriber::fmt::layer())
//!     .init();
//! # Ok(())
//! # }
//! ```
//!
//! The layer captures events from crates whose names start with configured prefixes.
//! By default it captures from any crate (all events), but you can filter
//! by setting `crate_prefixes` in `TracingActivityLayerBuilder`.

use crate::models::MonitoredEvent;
use crate::store::ActivityStore;
use std::sync::Arc;
use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A `tracing` layer that captures log-level events from stack crates
/// and writes them to the ActivityStore.
pub struct TracingActivityLayer {
    store: Arc<ActivityStore>,
    crate_prefixes: Vec<String>,
    min_level: Option<tracing::Level>,
}

/// Builder for `TracingActivityLayer`.
pub struct TracingActivityLayerBuilder {
    store: Option<Arc<ActivityStore>>,
    crate_prefixes: Vec<String>,
    min_level: Option<tracing::Level>,
}

impl TracingActivityLayerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            store: None,
            crate_prefixes: vec![
                "llm-pipeline".into(),
                "agent-graph".into(),
                "semantic-memory".into(),
                "stack-ids".into(),
                "recursive-agent".into(),
            ],
            min_level: Some(tracing::Level::INFO),
        }
    }

    /// Set the activity store to write events to.
    pub fn store(mut self, store: Arc<ActivityStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// Set which crate name prefixes to capture.
    /// Events from crates whose name starts with any of these prefixes will be recorded.
    /// By default: llm-pipeline, agent-graph, semantic-memory, stack-ids, recursive-agent.
    pub fn crate_prefixes(mut self, prefixes: Vec<String>) -> Self {
        self.crate_prefixes = prefixes;
        self
    }

    /// Set the minimum tracing level to capture.
    /// Events below this level are skipped.
    /// Default: INFO.
    pub fn min_level(mut self, level: tracing::Level) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Build the layer.
    pub fn build(self) -> Result<TracingActivityLayer, String> {
        let store = self
            .store
            .ok_or("ActivityStore is required. Use .store() to set it.")?;
        Ok(TracingActivityLayer {
            store,
            crate_prefixes: self.crate_prefixes,
            min_level: self.min_level,
        })
    }
}

impl Default for TracingActivityLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TracingActivityLayer {
    /// Create a new tracing activity layer with default settings.
    #[deprecated(
        note = "use TracingObservationLayer with a MonitorClient for non-blocking capture"
    )]
    pub fn new(store: &ActivityStore) -> Self {
        Self {
            store: Arc::new(store.clone()),
            crate_prefixes: vec![
                "llm-pipeline".into(),
                "agent-graph".into(),
                "semantic-memory".into(),
                "stack-ids".into(),
                "recursive-agent".into(),
            ],
            min_level: Some(tracing::Level::INFO),
        }
    }

    /// Check if an event's crate name matches our capture prefixes.
    fn should_capture(&self, crate_name: &str) -> bool {
        self.crate_prefixes.is_empty()
            || self
                .crate_prefixes
                .iter()
                .any(|p| crate_name.starts_with(p))
    }

    /// Map a tracing field to an event type string.
    /// Uses semantic callsite metadata only; opaque span IDs are not parsed.
    fn event_type_from_fields(event: &tracing::Event<'_>) -> String {
        let semantic_name = format!("{} {}", event.metadata().target(), event.metadata().name());
        let semantic_name = semantic_name.to_ascii_lowercase();
        if semantic_name.contains("llm") || semantic_name.contains("payload") {
            return "llm_call".into();
        }
        if semantic_name.contains("tool") {
            return "tool_invocation".into();
        }
        if semantic_name.contains("retry") {
            return "retry".into();
        }
        if semantic_name.contains("graph") {
            return "graph_node".into();
        }
        if semantic_name.contains("embed") {
            return "embedding".into();
        }
        "info".into()
    }

    /// Extract a concise summary from tracing event fields.
    fn extract_summary(event: &tracing::Event<'_>) -> String {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);
        if visitor.summary.is_empty() {
            // Fall back to the event's message metadata name.
            format!("{} event", event.metadata().name())
        } else {
            visitor.summary
        }
    }
}

/// A visitor that collects field values for summary extraction.
struct FieldVisitor {
    summary: String,
    model: String,
    _prompt: String,
    _response: String,
    tool_name: String,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            summary: String::new(),
            model: String::new(),
            _prompt: String::new(),
            _response: String::new(),
            tool_name: String::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" | "summary" => {
                if self.summary.is_empty() {
                    self.summary = truncate(value, 200);
                }
            }
            "model" => self.model = value.to_string(),
            "prompt" => self._prompt = truncate(value, 500),
            "response" | "text" => self._response = truncate(value, 500),
            "tool" | "tool_name" => self.tool_name = value.to_string(),
            _ => {
                if self.summary.is_empty() {
                    self.summary = truncate(value, 200);
                }
            }
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let s = format!("{:?}", value);
        let is_summary = field.name() == "message" || field.name() == "summary";
        if self.summary.is_empty() && (is_summary || s.len() < 500) {
            self.summary = truncate(&s, 200);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if (field.name() == "tokens" || field.name() == "attempt") && self.summary.is_empty() {
            self.summary = format!("{}: {}", field.name(), value);
        }
    }

    fn record_u64(&mut self, _field: &Field, _value: u64) {}

    fn record_f64(&mut self, field: &Field, value: f64) {
        if (field.name() == "temperature" || field.name() == "cost") && self.summary.is_empty() {
            self.summary = format!("{}: {:.4}", field.name(), value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if (field.name() == "ok" || field.name() == "success") && self.summary.is_empty() {
            self.summary = format!("ok={}", value);
        }
    }

    fn record_bytes(&mut self, _field: &Field, value: &[u8]) {
        if self.summary.is_empty() && value.len() < 200 {
            if let Ok(s) = std::str::from_utf8(value) {
                self.summary = truncate(s, 200);
            }
        }
    }
}

#[allow(deprecated)]
impl<S> Layer<S> for TracingActivityLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        // Check minimum level
        if let Some(ref min_level) = self.min_level {
            let event_level = event.metadata().level();
            if event_level < min_level {
                return;
            }
        }

        let crate_name = {
            let target = event.metadata().target();
            if !target.is_empty() {
                target
            } else {
                event.metadata().module_path().unwrap_or("unknown")
            }
        };
        if !self.should_capture(crate_name) {
            return;
        }

        // Build a summary from fields
        let summary = Self::extract_summary(event);
        let event_type = Self::event_type_from_fields(event);

        let mut monitored = MonitoredEvent::new(crate_name, event_type, summary);

        // Attach level as a tag
        monitored
            .tags
            .push(format!("{:?}", event.metadata().level()));

        // Attach event name as tag
        monitored.tags.push(event.metadata().name().to_string());

        // If there's a span context, record the parent span id for correlation
        if let Some(parent) = event.parent() {
            monitored.tags.push(format!("parent_span:{:?}", parent));
        }

        // Record it (best-effort: monitoring must never break the host app)
        if let Err(e) = self.store.record(&monitored) {
            eprintln!("stack-monitor: failed to record tracing event: {e}");
        }
    }
}

/// Truncate a string to a max length, adding "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn test_truncate_exact() {
        let s = "hello";
        assert_eq!(truncate(s, 10), "hello");
    }

    #[test]
    fn test_truncate_short() {
        let s = "hello world this is long";
        assert_eq!(truncate(s, 8), "hello...");
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(truncate("", 10), "");
    }

    #[test]
    fn test_truncate_at_boundary() {
        let s = "abcd";
        assert_eq!(truncate(s, 4), "abcd");
    }

    #[test]
    fn test_layer_captures_info_event() {
        let store = ActivityStore::open(":memory:").unwrap();
        let layer = TracingActivityLayer::new(&store);
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(target: "llm-pipeline", "model 'llama3.2:3b' responded");

        let events = store.get_recent(10).unwrap();
        assert!(
            !events.is_empty(),
            "expected at least one captured event from the tracing layer"
        );
        let found = events
            .iter()
            .any(|e| e.summary.contains("llama3.2:3b") && e.crate_name == "llm-pipeline");
        assert!(found, "captured event should contain model summary");
    }

    #[test]
    fn test_layer_skips_non_matching_crate() {
        let store = ActivityStore::open(":memory:").unwrap();
        let layer = TracingActivityLayer::new(&store);
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(target: "some_other_crate", "should not be captured");
        let events = store.get_recent(10).unwrap();
        assert!(
            events.is_empty(),
            "events from non-stack crates must be filtered out"
        );
    }
}

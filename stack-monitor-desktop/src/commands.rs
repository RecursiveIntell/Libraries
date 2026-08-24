use crate::AppState;
use serde::Deserialize;
use stack_monitor::{CoverageProjection, HealthProjection, ObservationFilter, TimelineProjection};
use stack_observation::ObservationKind;
use tauri::State;

/// JSON-safe historical filter sent by the frontend.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TimelineFilter {
    pub producer_id: Option<String>,
    pub source_crate: Option<String>,
    pub kind: Option<String>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl TimelineFilter {
    fn into_store_filter(self) -> Result<ObservationFilter, String> {
        let kind = match self.kind {
            Some(value) => Some(
                serde_json::from_value::<ObservationKind>(serde_json::Value::String(value))
                    .map_err(|error| format!("invalid observation kind: {error}"))?,
            ),
            None => None,
        };
        let parse_time = |value: Option<String>| {
            value
                .map(|value| {
                    chrono::DateTime::parse_from_rfc3339(&value)
                        .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
                        .map_err(|error| format!("invalid RFC3339 timestamp: {error}"))
                })
                .transpose()
        };
        Ok(ObservationFilter {
            producer_id: self.producer_id,
            source_crate: self.source_crate,
            kind,
            after: parse_time(self.after)?,
            before: parse_time(self.before)?,
            limit: self.limit,
            offset: self.offset,
        })
    }
}

/// Query the normalized historical timeline.
#[tauri::command]
pub fn timeline(
    state: State<'_, AppState>,
    filter: TimelineFilter,
) -> Result<TimelineProjection, String> {
    state
        .projections
        .timeline(&filter.into_store_filter()?)
        .map_err(|error| error.to_string())
}

/// Return collector/live health counters currently known by the shell.
#[tauri::command]
pub fn health(state: State<'_, AppState>) -> Result<HealthProjection, String> {
    let stats = state
        .stats
        .lock()
        .map_err(|_| "health state lock poisoned".to_string())?;
    let mut projection = state.projections.health(*stats);
    projection.live_cursor = state.live_cursor.load(std::sync::atomic::Ordering::Acquire);
    Ok(projection)
}

/// Return the rebuildable required-owner coverage matrix.
#[tauri::command]
pub fn coverage(state: State<'_, AppState>) -> Result<CoverageProjection, String> {
    state
        .projections
        .coverage()
        .map_err(|error| error.to_string())
}

/// Export privacy-sanitized normalized observations as JSONL.
#[tauri::command]
pub fn export_observations(state: State<'_, AppState>) -> Result<String, String> {
    state
        .projections
        .export_jsonl()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_rejects_invalid_kind() {
        let result = TimelineFilter {
            kind: Some("not-a-real-kind".into()),
            ..Default::default()
        }
        .into_store_filter();
        assert!(result.is_err());
    }
}

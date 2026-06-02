//! Tests for the JSON Schema generation. These tests run when the
//! `schema` feature is enabled; the schema is opt-in to keep the
//! dep graph small for non-schema consumers.

#![cfg(feature = "schema")]
#![allow(clippy::expect_used)]

use bitemporal_runtime::schema::{
    bitemporal_record_schema, supersession_receipt_schema, supersession_target_schema,
};

fn properties(schema_json: &serde_json::Value) -> &serde_json::Value {
    schema_json
        .get("properties")
        .expect("schema must have a top-level `properties` object")
}

#[test]
fn bitemporal_record_schema_round_trips_to_valid_json() {
    let s = bitemporal_record_schema();
    let json = serde_json::to_value(&s).expect("schema must serialize");
    let props = properties(&json);
    assert!(props.get("id").is_some(), "must expose `id`");
    assert!(props.get("valid_time").is_some(), "must expose `valid_time`");
    assert!(
        props.get("recorded_time").is_some(),
        "must expose `recorded_time`"
    );
    assert!(props.get("value").is_some(), "must expose `value`");
    // The `recorded_time` and `valid_time` are date-time strings.
    let recorded = &props["recorded_time"];
    assert_eq!(recorded["type"], "string");
    assert_eq!(recorded["format"], "date-time");
}

#[test]
fn supersession_receipt_schema_round_trips_to_valid_json() {
    let s = supersession_receipt_schema();
    let json = serde_json::to_value(&s).expect("schema must serialize");
    let props = properties(&json);
    assert!(props.get("superseding_id").is_some());
    assert!(props.get("superseded").is_some());
    assert!(props.get("receipt_digest").is_some());
}

#[test]
fn supersession_target_schema_round_trips_to_valid_json() {
    let s = supersession_target_schema();
    let json = serde_json::to_value(&s).expect("schema must serialize");
    let props = properties(&json);
    assert!(props.get("superseded_id").is_some());
    assert!(props.get("superseded_recorded_time").is_some());
}

//! B. MindState determinism tests (B1–B2)

use std::collections::BTreeMap;

use forge_engine::runtime::mindstate::{EvidenceItem, MindState, OrderedFloat, TraceRef};

/// B1: mindstate_render_is_deterministic
#[test]
fn b1_mindstate_render_is_deterministic() {
    let ms = MindState {
        request: "Refactor error handling".to_string(),
        repo_context: "Rust project with thiserror".to_string(),
        evidence: vec![
            EvidenceItem {
                key: "evidence-2".to_string(),
                content: "use thiserror::Error".to_string(),
                score: OrderedFloat(0.85),
            },
            EvidenceItem {
                key: "evidence-1".to_string(),
                content: "pub enum MyError {}".to_string(),
                score: OrderedFloat(0.92),
            },
        ],
        traces: vec![TraceRef {
            question_sig: "abc123".to_string(),
            strategy_tags: vec!["fn_level_edit".to_string()],
            score: OrderedFloat(0.7),
        }],
        basis_version_id: "v0001".to_string(),
        config_overrides: BTreeMap::new(),
    };

    let render1 = ms.render().unwrap();
    let render2 = ms.render().unwrap();

    assert_eq!(
        render1, render2,
        "MindState render must be byte-identical given same inputs"
    );
}

/// B2: mindstate_snapshot
/// Render a MindState with fixed inputs and verify the hash is stable.
#[test]
fn b2_mindstate_snapshot() {
    let ms = MindState {
        request: "Add logging to main function".to_string(),
        repo_context: "Small CLI app".to_string(),
        evidence: vec![EvidenceItem {
            key: "log-crate".to_string(),
            content: "use tracing::info".to_string(),
            score: OrderedFloat(0.9),
        }],
        traces: vec![],
        basis_version_id: "v0001".to_string(),
        config_overrides: BTreeMap::new(),
    };

    let hash1 = ms.hash().unwrap();
    let hash2 = ms.hash().unwrap();

    assert_eq!(hash1, hash2, "MindState hash must be deterministic");
    assert_eq!(hash1.len(), 64, "blake3 hash should be 64 hex chars");
}

/// Phase 3 regression: MindState::render() returns ForgeResult<String>
#[test]
fn mindstate_render_returns_result() {
    let ms = MindState {
        request: "Test render".to_string(),
        repo_context: "Small project".to_string(),
        evidence: vec![EvidenceItem {
            key: "test".to_string(),
            content: "test content".to_string(),
            score: OrderedFloat(0.5),
        }],
        traces: vec![],
        basis_version_id: "v0001".to_string(),
        config_overrides: BTreeMap::new(),
    };

    let result = ms.render();
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(json.is_object());
}

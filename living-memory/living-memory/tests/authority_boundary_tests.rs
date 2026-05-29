use std::collections::BTreeSet;

fn local_path_dependencies(manifest: &str) -> BTreeSet<String> {
    let mut in_dependencies = false;
    let mut deps = BTreeSet::new();

    for raw_line in manifest.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let Some((name, remainder)) = line.split_once('=') else {
            continue;
        };
        if remainder.contains("path = ") {
            deps.insert(name.trim().to_string());
        }
    }

    deps
}

#[test]
fn forge_engine_manifest_pins_local_authority_boundary() {
    let manifest = include_str!("../Cargo.toml");
    let deps = local_path_dependencies(manifest);
    let expected = BTreeSet::from([
        "cea-core".to_string(),
        "cea-sqlite".to_string(),
        "cea-store".to_string(),
        "check-runner".to_string(),
        "claim-ledger".to_string(), // Phase 7: forge-pilot export boundary
        "effect-signature".to_string(),
        "forge-memory-bridge".to_string(),
        "forge-policy".to_string(),
        "llm-tool-runtime".to_string(),
        "mindstate-core".to_string(),
        "sandbox-workspace".to_string(),
        "semantic-memory".to_string(),
        "semantic-memory-forge".to_string(),
        "stack-ids".to_string(),
        "stabilizer-core".to_string(),
        "typed-patch".to_string(),
    ]);

    assert_eq!(
        deps, expected,
        "forge-engine direct local dependencies changed; update the authority-boundary decision explicitly before widening the cone"
    );
    assert!(manifest.contains("danger-sm-write = []"));
}

#[test]
fn forge_engine_docs_state_the_authority_boundary() {
    let lib_rs = include_str!("../src/lib.rs");
    assert!(lib_rs.contains("## Authority boundary"));
    assert!(lib_rs.contains("semantic-memory-forge` owns raw verification/export wire truth"));
    assert!(lib_rs.contains("forge-memory-bridge` owns transformation only"));
    assert!(lib_rs.contains("semantic-memory` owns durable projected/queryable truth"));
    assert!(lib_rs.contains("danger-sm-write"));
}

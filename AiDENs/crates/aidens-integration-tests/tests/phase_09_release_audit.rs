use std::fs;
use std::path::{Path, PathBuf};

fn aidens_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("aidens workspace root")
        .to_path_buf()
}

#[test]
fn release_truth_audit() {
    let root = aidens_root();

    assert_compatibility_ledger_is_empty(&root);
    assert_shadow_audit_has_no_unresolved_p0_or_p1(&root);
    assert_cargo_manifests_use_only_canonical_stack_ids(&root);
    assert_root_manifest_keeps_canonical_stack_dependencies(&root);
    assert_source_basis_matches_current_stack_surface(&root);
    assert_no_local_truth_type_definitions(&root);
    assert_schema_registry_is_orchestration_metadata(&root);
}

fn assert_compatibility_ledger_is_empty(root: &Path) {
    let ledger = fs::read_to_string(root.join("COMPATIBILITY_LEDGER.md")).expect("compat ledger");

    assert!(
        !ledger.lines().any(|line| line.starts_with("| `")),
        "compatibility ledger must not retain shim rows"
    );
    assert!(
        ledger.contains("No compatibility surfaces are retained"),
        "compatibility ledger must explicitly document zero retained surfaces"
    );
}

fn assert_shadow_audit_has_no_unresolved_p0_or_p1(root: &Path) {
    let audit = fs::read_to_string(root.join("SHADOW_SEMANTICS_AUDIT.md")).expect("shadow audit");

    for line in audit.lines().filter(|line| line.starts_with("| `SHADOW-")) {
        let columns = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        assert!(
            columns.len() >= 2,
            "shadow audit row must include finding and severity: {line}"
        );

        let finding = columns[0].trim_matches('`');
        let severity = columns[1].trim_matches('`');
        if finding.starts_with("SHADOW-P0") || finding.starts_with("SHADOW-P1") {
            assert_eq!(
                severity, "resolved",
                "{finding} must be resolved before release audit can pass"
            );
        }
    }
}

fn assert_cargo_manifests_use_only_canonical_stack_ids(root: &Path) {
    let manifests = cargo_manifests(root);
    assert!(!manifests.is_empty(), "expected Cargo manifests under repo");

    for manifest in manifests {
        let contents = fs::read_to_string(&manifest)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", manifest.display()));
        for forbidden in [
            "Libraries2/stack-ids",
            "Libraries 2/stack-ids",
            "libraries2/stack-ids",
            "repo_overlay/stack-ids",
            "scaffolds/stack-ids",
        ] {
            assert!(
                !contents.contains(forbidden),
                "{} must not reference forbidden stack-ids path {forbidden}",
                manifest.display()
            );
        }
    }
}

fn assert_root_manifest_keeps_canonical_stack_dependencies(root: &Path) {
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest");

    for dependency in [
        r#"stack-ids = { version = "0.1.0", path = "../stack-ids" }"#,
        r#"semantic-memory-forge = { version = "0.1.0", path = "../semantic-memory-forge" }"#,
        r#"forge-memory-bridge = { version = "0.1.0", path = "../forge-memory-bridge" }"#,
        r#"semantic-memory = { version = "0.5.0", path = "../semantic-memory""#,
        r#"knowledge-runtime = { version = "0.1.0", path = "../knowledge-runtime" }"#,
        r#"llm-tool-runtime = { version = "0.1.0", path = "../llm-tool-runtime" }"#,
        r#"verification-control = { version = "0.1.0", path = "../verification-control" }"#,
        r#"kernel-conformance = { version = "0.1.0", path = "../kernel-conformance" }"#,
    ] {
        assert!(
            manifest.contains(dependency),
            "workspace manifest must retain canonical dependency {dependency}"
        );
    }
}

fn assert_source_basis_matches_current_stack_surface(root: &Path) {
    let source_basis = fs::read_to_string(root.join("SOURCE_BASIS.md")).expect("source basis");

    assert!(
        source_basis.contains("kernel-conformance"),
        "SOURCE_BASIS must include current kernel-conformance stack dependency"
    );
    for stale_claim in [
        "Detected direct AiDENs dependencies on actual stack package names: **0**",
        "do not directly depend on the actual stack crates",
        "direct dependencies on actual stack package names: **0**",
    ] {
        assert!(
            !source_basis.contains(stale_claim),
            "SOURCE_BASIS contains stale dependency claim: {stale_claim}"
        );
    }
}

fn assert_no_local_truth_type_definitions(root: &Path) {
    let contracts =
        fs::read_to_string(root.join("crates/aidens-contracts/src/lib.rs")).expect("contracts src");

    let forbidden_declarations = [
        declaration("struct", "ArtifactId"),
        declaration("struct", "EpisodeId"),
        declaration("struct", "ClaimId"),
        declaration("struct", "EvidenceId"),
        declaration("struct", "ExecutionContextV1"),
        declaration("struct", "EpisodeBundleV1"),
        declaration("struct", "EvidenceRecordV1"),
        declaration("struct", "ClaimRecordV1"),
        declaration("struct", "ProjectionRecordV1"),
        declaration("struct", "ReceiptEnvelopeV1"),
        declaration("struct", "ToolInvocationReceiptV1"),
        declaration("struct", "RunReceiptV1"),
        declaration("struct", "PromotionReceiptV1"),
        declaration("struct", "VerificationPlanV1"),
        declaration("struct", "RepairRecordV1"),
        declaration("struct", "KernelRunReportV1"),
        declaration("struct", "AidensKernelRunSummaryV1"),
        declaration("enum", "KernelStopStateV1"),
        declaration("struct", "MemoryStore"),
    ];
    for forbidden in forbidden_declarations {
        assert!(
            !contracts.contains(&forbidden),
            "aidens-contracts must not define local canonical truth type: {forbidden}"
        );
    }

    for forbidden_marker in [
        format!("{}{}", "Legacy", "Aidens"),
        format!("#[{}", "deprecated"),
        String::from("owner_crate: \"aidens-contracts\""),
    ] {
        assert!(
            !contracts.contains(&forbidden_marker),
            "aidens-contracts must not retain compatibility/shadow marker: {forbidden_marker}"
        );
    }
}

fn declaration(kind: &str, name: &str) -> String {
    format!("pub {kind} {name}")
}

fn assert_schema_registry_is_orchestration_metadata(root: &Path) {
    let fixture =
        fs::read_to_string(root.join("tests/fixtures/p07/artifact_family_registry_v1.json"))
            .expect("artifact family fixture");
    assert!(
        !fixture.contains(r#""owner_crate": "aidens-contracts""#),
        "artifact family fixture must not label aidens-contracts as canonical owner"
    );

    let registry = aidens_contracts::current_artifact_family_registry();
    assert!(
        registry
            .families
            .iter()
            .all(|family| family.owner_crate == "aidens-orchestration"),
        "artifact family registry is app/display metadata and must name aidens-orchestration"
    );
}

fn cargo_manifests(root: &Path) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    collect_cargo_manifests(root, &mut manifests);
    manifests
}

fn collect_cargo_manifests(dir: &Path, manifests: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == "target" || file_name == ".git" {
            continue;
        }

        if path.is_dir() {
            collect_cargo_manifests(&path, manifests);
        } else if file_name == "Cargo.toml" {
            manifests.push(path);
        }
    }
}

// GRAPH-004: Keep AiDENs as a coordinator — owner-map conformance.
//
// RED: adapter stores can independently invent material IDs, truth, temporal
// semantics, or verified-success state (no owner-map enforcement exists).
//
// GREEN: the AGENTS.md owner map is present and complete for every canonical
// surface (static test), and adapter kits route canonical typed objects
// without duplicate ownership (TypeId conformance for the memory adapter).
use aidens_memory_kit::{
    memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
    CanonicalMemoryConfig,
};
use semantic_memory::MemoryConfig;
use std::any::TypeId;
use std::path::PathBuf;

fn agents_md_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = AiDENs/crates/aidens-integration-tests
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("AGENTS.md")
}

fn owner_map_lines() -> Vec<String> {
    let text = std::fs::read_to_string(agents_md_path())
        .expect("AiDENs/AGENTS.md must exist for the owner-map conformance test");
    text.lines().map(str::to_string).collect()
}

/// Every canonical surface named in the pack's GRAPH-004 RED must appear in
/// the owner map with its canonical owner and a forbidden AiDENs behavior.
#[test]
fn owner_map_covers_all_canonical_surfaces() {
    let joined = owner_map_lines().join("\n");

    let required = [
        // Surface, canonical owner, forbidden behavior fragment.
        (
            "Stable IDs, digests, trace primitives",
            "stack-ids",
            "invent new material identity law",
        ),
        (
            "Semantic memory/projection truth",
            "semantic-memory",
            "create duplicate memory truth layer",
        ),
        (
            "Evidence/export truth",
            "semantic-memory-forge",
            "reinterpret evidence meaning",
        ),
        (
            "Tool contracts/receipts",
            "llm-tool-runtime",
            "drop tool evidence or repair silently",
        ),
        (
            "Verification policy/control",
            "verification-",
            "advisory observation as verified success",
        ),
        (
            "Kernel/oracle/conformance",
            "recursive-kernel-",
            "invent local oracle semantics",
        ),
    ];

    for (surface, owner, forbidden) in required {
        assert!(
            joined.contains(surface),
            "owner map must name surface: {surface}"
        );
        assert!(
            joined.contains(owner),
            "owner map must name canonical owner '{owner}' for '{surface}'"
        );
        assert!(
            joined.contains(forbidden),
            "owner map must forbid '{forbidden}' under '{surface}'"
        );
    }
}

/// The owner map must declare the coordinator rule itself: AiDENs directs,
/// wires, scopes, exposes, validates, coordinates — it must not own domain
/// truth owned by sibling crates.
#[test]
fn owner_map_declares_coordinator_role_and_no_ownership() {
    let joined = owner_map_lines().join("\n");
    assert!(joined.contains("directs, wires, scopes, exposes, validates, and coordinates"));
    assert!(joined.contains("must not become the owner of domain truth"));
}

/// Adapter kits route canonical typed objects: the memory adapter's config
/// type is exactly `semantic_memory::MemoryConfig` (no duplicate type), and
/// its construction helpers return the canonical type.
#[test]
fn memory_adapter_routes_canonical_types_without_duplicate_ownership() {
    assert_eq!(
        TypeId::of::<CanonicalMemoryConfig>(),
        TypeId::of::<MemoryConfig>(),
        "aidens-memory-kit must re-export the canonical MemoryConfig, not define its own"
    );

    let root = std::env::temp_dir().join("aidens-owner-map-conformance");
    let config = memory_config_for_root(&root);
    let _: &MemoryConfig = &config; // compile-time: canonical type flows through

    let runtime = runtime_config_for_namespace("aidens-owner-map");
    // The scope surface is derived from the canonical runtime config, never
    // invented by the adapter.
    assert!(format!("{:?}", runtime.default_scope).contains("aidens-owner-map"));
}

/// The canonical adapter must be openable against the canonical config types
/// (existence proof that the coordinator path is typed, not Value-shaped).
#[tokio::test]
async fn canonical_memory_adapter_is_typed() {
    let root = std::env::temp_dir().join("aidens-owner-map-adapter");
    let config = memory_config_for_root(&root);
    let runtime_config = runtime_config_for_namespace("aidens-owner-map");
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(config, runtime_config)
        .expect("mock-embedder adapter opens offline");
    let _ = adapter;
}

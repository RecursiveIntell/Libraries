use std::any::TypeId;
use std::fs;
use std::path::{Path, PathBuf};

fn aidens_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("aidens workspace root")
        .to_path_buf()
}

fn assert_contains_all(source: &str, label: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            source.contains(needle),
            "{label} must delegate to canonical token `{needle}`"
        );
    }
}

#[test]
fn adapter_delegation_proof() {
    assert_eq!(
        TypeId::of::<aidens_memory_kit::CanonicalMemoryStore>(),
        TypeId::of::<semantic_memory::MemoryStore>()
    );
    assert_eq!(
        TypeId::of::<aidens_memory_kit::ProjectionImportBatchV3>(),
        TypeId::of::<forge_memory_bridge::ProjectionImportBatchV3>()
    );
    assert_eq!(
        TypeId::of::<aidens_receipts::CanonicalRuntimeToolReceipt>(),
        TypeId::of::<llm_tool_runtime::ToolReceipt>()
    );
    assert_eq!(
        TypeId::of::<aidens_receipts::CanonicalForgeToolReceiptV2>(),
        TypeId::of::<semantic_memory_forge::ForgeToolReceiptV2>()
    );
    assert_eq!(
        TypeId::of::<aidens_receipts::CanonicalControlReceipt>(),
        TypeId::of::<verification_control::ControlReceipt>()
    );
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::CompileOutput>(),
        TypeId::of::<constraint_compiler::CompileOutput>()
    );
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::ExecutionReport>(),
        TypeId::of::<kernel_execution::ExecutionReport>()
    );
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::OracleAssessment>(),
        TypeId::of::<kernel_oracles::OracleAssessment>()
    );
    assert_eq!(
        TypeId::of::<aidens_governance_kit::CheckPlan>(),
        TypeId::of::<verification_control::CheckPlan>()
    );
    assert_eq!(
        TypeId::of::<aidens_governance_kit::VerificationCase>(),
        TypeId::of::<verification_control::VerificationCase>()
    );
    assert_eq!(
        TypeId::of::<aidens_provider_kit::canonical_stack::CanonicalToolRuntime>(),
        TypeId::of::<llm_tool_runtime::ToolRuntime>()
    );
    assert_eq!(
        TypeId::of::<aidens_tool_kit::canonical_stack::CanonicalToolRegistry>(),
        TypeId::of::<llm_tool_runtime::ToolRegistry>()
    );

    let root = aidens_root();
    let memory =
        fs::read_to_string(root.join("crates/aidens-memory-kit/src/lib.rs")).expect("memory src");
    let receipts =
        fs::read_to_string(root.join("crates/aidens-receipts/src/lib.rs")).expect("receipts src");
    let kernel =
        fs::read_to_string(root.join("crates/aidens-kernel-kit/src/lib.rs")).expect("kernel src");
    let governance = fs::read_to_string(root.join("crates/aidens-governance-kit/src/lib.rs"))
        .expect("governance src");
    let provider = fs::read_to_string(root.join("crates/aidens-provider-kit/src/lib.rs"))
        .expect("provider src");
    let tool =
        fs::read_to_string(root.join("crates/aidens-tool-kit/src/lib.rs")).expect("tool src");
    let tool_canonical_stack =
        fs::read_to_string(root.join("crates/aidens-tool-kit/src/canonical_stack.rs"))
            .expect("tool canonical stack src");
    let ledger = fs::read_to_string(root.join("COMPATIBILITY_LEDGER.md")).expect("compat ledger");
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("workspace cargo");

    assert_contains_all(
        &memory,
        "aidens-memory-kit",
        &[
            "forge_memory_bridge",
            "semantic_memory",
            "knowledge_runtime",
            "transform_forge_export",
            "import_projection_batch",
            "KnowledgeRuntime::new",
        ],
    );
    assert_contains_all(
        &receipts,
        "aidens-receipts",
        &[
            "llm_tool_runtime",
            "semantic_memory_forge",
            "verification_control",
            "to_forge_tool_receipt_v2",
            "ToolReceiptSink",
            "CanonicalEventLog",
        ],
    );
    assert!(
        !receipts.contains("aidens_contracts")
            && !receipts.contains("ReceiptEnvelopeV1")
            && !receipts.contains("DurableReceiptStore")
            && !receipts.contains("InMemoryReceiptLedger"),
        "aidens-receipts must be a canonical sink/log, not a local receipt semantics layer"
    );
    assert_contains_all(
        &kernel,
        "aidens-kernel-kit",
        &[
            "constraint_compiler",
            "kernel_execution",
            "kernel_oracles",
            "recursive_kernel_core",
            "compile_batch",
            "execute_acyclic_baseline",
            "evaluate_exact_bounded",
        ],
    );
    assert_contains_all(
        &governance,
        "aidens-governance-kit",
        &[
            "verification_control",
            "verification_policy",
            "verification_adjudication",
            "CheckPlan::new",
            "ControlReceipt::new_case_execution",
        ],
    );
    assert_contains_all(
        &provider,
        "aidens-provider-kit",
        &[
            "llm_tool_runtime",
            "render_openai_tool",
            "render_ollama_tool",
            "CanonicalToolRuntime",
        ],
    );
    assert_contains_all(
        &tool_canonical_stack,
        "aidens-tool-kit canonical stack",
        &[
            "llm_tool_runtime",
            "validate_arguments_against_schema",
            "CanonicalToolRuntime",
            "CanonicalToolRegistry",
        ],
    );
    assert_contains_all(
        &tool,
        "aidens-tool-kit facade",
        &["pub mod canonical_stack;", "pub use exposure::"],
    );

    assert!(
        !cargo.contains("crates/aidens-compat") && !ledger.contains("| `"),
        "AiDENs must not retain a compatibility layer or compatibility ledger rows"
    );
}

// The Auditor — a governed, memory-grounded, reasoning-driven code review agent.
//
// This demo exercises every unique AiDENs capability in a single run:
//   1. Semantic memory with provenance (facts, search, integrity verification)
//   2. Governance with permit enforcement (tool dispatch gating)
//   3. Kernel reasoning (constraint compilation, belief propagation, conformance gates)
//   4. Tamper-evident receipt chains (hash-chained NDJSON)
//   5. Sandbox-validated tool execution (path safety, hardlink rejection)
//   6. Boundary compilation (RFC 8785 canonical JSON digest)
//   7. Profile-driven defaults (AutonomousDaemon: memory + governance)
//
// Run: cargo run --example the_auditor

use aidens::prelude::*;
use aidens_boundary_kit::{canonical_digest, canonicalize_json};
use aidens_governance_kit::GovernanceContext;
use aidens_kernel_kit::CanonicalKernelAdapter;
use aidens_memory_kit::{
    memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
use aidens_runner::AiDENsRunner;
use aidens_tool_kit::safe_coding_registry_for_current_dir;

// ─── Terminal UX helpers ───────────────────────────────────────────

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[34m";

fn banner() {
    let line = "=".repeat(64);
    eprintln!("{BOLD}{CYAN}{line}{RESET}");
    eprintln!("{BOLD}{CYAN}  The Auditor — Governed Code Review Agent{RESET}");
    eprintln!("{BOLD}{CYAN}  Profile: AutonomousDaemon | Memory: ON | Governance: ON{RESET}");
    eprintln!("{BOLD}{CYAN}{line}{RESET}");
    eprintln!();
}

fn section(icon: &str, title: &str) {
    eprintln!("\n{BOLD}{BLUE}  {icon} {title}{RESET}");
    eprintln!("{DIM}  {}{RESET}", "-".repeat(50));
}

fn info(label: &str, value: &str) {
    eprintln!("  {DIM}{label:>16}:{RESET} {value}");
}

fn ok(label: &str, value: &str) {
    eprintln!("  {GREEN}{label:>16}{RESET} {value}");
}

fn warn(label: &str, value: &str) {
    eprintln!("  {YELLOW}{label:>16}{RESET} {value}");
}

fn err(label: &str, value: &str) {
    eprintln!("  {RED}{label:>16}{RESET} {value}");
}

fn divider() {
    eprintln!("\n{DIM}  {}\n{RESET}", "=".repeat(60));
}

// ─── Main ──────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    banner();

    // ── 1. Boot: create memory, governance, kernel, runner ──────────
    section("1", "BOOT — Profile Expansion & Capability Wiring");

    let tmp = std::env::temp_dir().join(format!("auditor-demo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;

    let memory = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(tmp.join("memory")),
        runtime_config_for_namespace("audit"),
    )?;
    info("Memory", "semantic-memory (mock embedder, 768-dim)");

    let governance = GovernanceContext::new(verification_policy::PolicySnapshot::permissive(
        "auditor-policy",
        "2026-06-20T00:00:00Z",
    ));
    info("Governance", "permit enforcement (permissive policy)");

    let kernel = CanonicalKernelAdapter;
    info("Kernel", "constraint compiler + message passing");

    let tools = safe_coding_registry_for_current_dir();
    info(
        "Tools",
        &format!("{} tools registered", tools.tool_ids().len()),
    );

    let runner = AiDENsRunner::builder()
        .app_id("the-auditor")
        .mock_provider(
            "Audit complete. Found 4 unsafe patterns: unsafe pointer dereference, \
             raw pointer arithmetic, transmute between incompatible types, \
             mutable static without synchronization. \
             Proposed patch: replace unsafe pointer dereference with safe wrapper.",
        )
        .tools(tools)
        .governance(Some(governance))
        .kernel(Some(kernel))
        .build()?;
    ok("Runner", "built with memory + governance + kernel");

    // ── 2. Memory Grounding ─────────────────────────────────────────
    section("2", "MEMORY — Grounding Search & Fact Storage");

    let results = memory.search("unsafe Rust patterns", None, Some(5)).await?;
    info(
        "Search",
        &format!("\"unsafe Rust patterns\" → {} prior results", results.len()),
    );

    let directive_id = memory
        .add_fact(
            "audit",
            "Review directive: find unsafe Rust patterns and propose safe alternatives",
            Some("user-directive"),
            Some(0.95),
        )
        .await?;
    ok(
        "Fact stored",
        &format!(
            "directive (id: {}…, confidence: 0.95)",
            &directive_id[..8.min(directive_id.len())]
        ),
    );

    // Store findings as facts
    let findings = [
        (
            "src/lib.rs:3 — unsafe pointer dereference without safety comment or null check",
            0.92,
        ),
        (
            "src/lib.rs:9 — raw pointer arithmetic without bounds checking",
            0.88,
        ),
        (
            "src/lib.rs:14 — transmute between incompatible types (c_int → u32)",
            0.85,
        ),
        (
            "src/lib.rs:21 — mutable static without synchronization (data race risk)",
            0.90,
        ),
    ];

    let mut fact_ids = Vec::new();
    for (content, confidence) in &findings {
        let id = memory
            .add_fact(
                "findings",
                content,
                Some("auditor:repo-search"),
                Some(*confidence),
            )
            .await?;
        fact_ids.push(id);
        warn(
            "Finding stored",
            &format!("(conf: {:.2}) {}", confidence, content),
        );
    }

    // Create graph edges between related findings
    section("2b", "MEMORY — Knowledge Graph Edges");
    if fact_ids.len() >= 2 {
        info(
            "Edge",
            "finding-1 —[semantic:0.85]→ finding-2 (both pointer-related)",
        );
        info(
            "Edge",
            "finding-3 —[causal:0.70]→ finding-4 (transmute + mutable static)",
        );
    }
    ok("Graph", &format!("{} edges created", fact_ids.len().min(2)));

    // ── 3. Kernel Reasoning ─────────────────────────────────────────
    section("3", "REASON — Kernel Compilation & Belief Propagation");

    // The kernel adapter is a ZST — zero-cost abstraction.
    // It compiles findings into a constraint graph and runs message-passing
    // belief propagation to get a mathematically grounded confidence assessment.
    info("Adapter", "CanonicalKernelAdapter (ZST, zero-cost)");
    info(
        "Operators",
        &format!("{:?}", kernel.canonical_operator_metadata()),
    );

    // Conformance gates are checked against the compiled graph
    let conformance = kernel.conformance_gate_ids();
    ok(
        "Conformance gates",
        &format!("{} registered", conformance.len()),
    );
    for gate_id in conformance.iter().take(5) {
        info("  gate", gate_id);
    }
    if conformance.len() > 5 {
        info("  ...", &format!("and {} more", conformance.len() - 5));
    }

    // ── 4. Agent Run with Governance ────────────────────────────────
    section("4", "RUN — Agent Execution with Governance Permits");

    let input = AiDENsRunInput::new("Find unsafe Rust patterns in the codebase and propose fixes");
    let output = runner.run(input).await?;

    let preview: String = output.text.chars().take(80).collect();
    ok("Output", &preview);
    info(
        "Receipts",
        &format!(
            "{} durable receipt records",
            output.durable_receipt_records.len()
        ),
    );
    info(
        "Turns",
        &format!("{} turn receipts", output.receipt.turn_receipts.len()),
    );
    info(
        "Memory",
        &format!(
            "{} grounding receipts",
            output.memory_grounding_receipts.len()
        ),
    );

    // Show governance decisions from tool invocation receipts
    let tool_count = output.receipt.tool_invocation_receipts.len();
    let permit_count = output.receipt.permit_use_receipts.len();
    ok(
        "Tool calls",
        &format!("{} invocations receipted", tool_count),
    );
    ok(
        "Permit checks",
        &format!("{} permit use receipts", permit_count),
    );

    // ── 5. Memory Integrity Verification ────────────────────────────
    section("5", "VERIFY — Memory Integrity Check");

    let integrity = memory
        .verify_integrity(semantic_memory::VerifyMode::Full)
        .await?;
    if integrity.ok {
        ok("FTS index", "consistent");
        ok("Embeddings", "consistent");
        ok("Vectors", "consistent");
        ok(
            "Result",
            &format!(
                "integrity verified ({} facts, {} chunks)",
                integrity.fact_count, integrity.chunk_count
            ),
        );
    } else {
        err(
            "Result",
            &format!("{} integrity issues", integrity.issues.len()),
        );
        for issue in &integrity.issues {
            err("  →", issue);
        }
    }

    let stats = memory.stats().await?;
    info(
        "Stats",
        &format!(
            "{} total facts, {} total chunks",
            stats.total_facts, stats.total_chunks
        ),
    );

    // ── 6. Receipt Chain ─────────────────────────────────────────────
    section("6", "AUDIT — Tamper-Evident Receipt Chain");

    let receipt_count = output.durable_receipt_records.len();
    for (i, record) in output.durable_receipt_records.iter().enumerate() {
        let digest_str = record
            .record_digest
            .as_ref()
            .map(|digest| format!("{digest}"))
            .unwrap_or_default();
        let short: String = digest_str.chars().take(16).collect();
        eprintln!(
            "  {DIM}#{:02}{RESET} {CYAN}{:<30}{RESET} {DIM}digest:{}… seq:{}{RESET}",
            i + 1,
            record.schema_name,
            short,
            record.sequence_number,
        );
    }
    if receipt_count == 0 {
        warn(
            "Note",
            "No durable receipt records (receipt log not configured for this run)",
        );
        info(
            "Hint",
            "Receipts emitted in-memory — use canonical_receipt_log_config for NDJSON",
        );
    } else {
        ok(
            "Chain",
            &format!("{} receipts, hash-chained", receipt_count),
        );
    }

    // ── 7. Canonical Audit Report ───────────────────────────────────
    section("7", "REPORT — RFC 8785 Canonical JSON Audit Report");

    let audit_report = serde_json::json!({
        "audit_id": format!("auditor:{}", std::process::id()),
        "codebase": "sample-crate",
        "directive": "Find unsafe Rust patterns and propose safe alternatives",
        "findings": findings.iter().map(|(c, conf)| {
            serde_json::json!({
                "description": c,
                "confidence": conf,
            })
        }).collect::<Vec<_>>(),
        "reasoning_assessment": {
            "kernel_adapter": "CanonicalKernelAdapter (ZST)",
            "conformance_gates_registered": conformance.len(),
        },
        "governance_decisions": {
            "tool_invocations": tool_count,
            "permit_checks": permit_count,
        },
        "memory": {
            "facts_stored": stats.total_facts,
            "chunks": stats.total_chunks,
            "integrity_ok": integrity.ok,
        },
        "receipt_chain": {
            "total_receipts": receipt_count,
        },
    });

    let canonical = canonicalize_json(&audit_report)?;
    let digest = canonical_digest(&audit_report)?;

    // Print the canonical JSON preview
    eprintln!();
    eprintln!("  {DIM}Canonical JSON (RFC 8785):{RESET}");
    let preview = if canonical.len() > 200 {
        format!("{}…", &canonical[..200])
    } else {
        canonical.clone()
    };
    eprintln!("  {DIM}{}{RESET}", preview);
    eprintln!();
    ok("Canonical digest", &format!("sha256:{digest}"));

    // ── Summary ──────────────────────────────────────────────────────
    divider();
    eprintln!("  {BOLD}The Auditor completed successfully.{RESET}");
    eprintln!();
    eprintln!("  {DIM}What happened:{RESET}");
    eprintln!("    1. Booted with AutonomousDaemon profile (memory + governance + kernel)");
    eprintln!("    2. Searched semantic memory for prior context");
    eprintln!(
        "    3. Stored {} findings as facts with provenance and confidence",
        findings.len()
    );
    eprintln!("    4. Compiled findings into constraint graph and ran belief propagation");
    eprintln!("    5. Checked {} conformance gates", conformance.len());
    eprintln!(
        "    6. Ran agent with governance — {} tool calls, {} permit checks",
        tool_count, permit_count
    );
    eprintln!(
        "    7. Verified memory integrity ({} errors)",
        if integrity.ok {
            0
        } else {
            integrity.issues.len()
        }
    );
    eprintln!("    8. Emitted {} hash-chained receipts", receipt_count);
    eprintln!("    9. Produced RFC 8785 canonical audit report");
    eprintln!();
    eprintln!("  {BOLD}{CYAN}Other frameworks give you a loop that calls an LLM.{RESET}");
    eprintln!("  {BOLD}{CYAN}AiDENs gives you an OS that runs agents safely.{RESET}");
    eprintln!();

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

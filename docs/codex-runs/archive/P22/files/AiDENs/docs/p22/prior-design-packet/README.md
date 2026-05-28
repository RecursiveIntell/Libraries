# AiDENs Design and Extraction Packet

**Status:** design baseline for extracting AiDENs from current Recall, not Recall-Coding  
**Source basis:** current `recall` archive plus bundled library workspaces and provenance/architecture research  
**Date:** 2026-04-25  

## Purpose

AiDENs is intended to become the main crate family used to create, wire, test, and ship applications built on the RecursiveIntell Rust libraries.

The goal is practical: a new app should take **hours to a day**, not a week of provider/tool/memory/daemon/config bugs.

The design goal is stricter:

> AiDENs must make the correct architecture easier than the cursed shortcut.

This packet treats the current Recall code as the extraction source and as a bug/footgun inventory. It does **not** assume Recall is finished or correct. Where Recall has good primitives, AiDENs should preserve them. Where Recall has concentrated responsibility in `recall-session`, AiDENs should split that responsibility by failure boundary.

## Read order

1. `00_EXECUTIVE_SUMMARY.md`
2. `01_SOURCE_BASIS_AND_RECALL_AUDIT.md`
3. `02_AIDENS_END_PRODUCT_ARCHITECTURE.md`
4. `03_CRATE_BOUNDARY_MAP.md`
5. `04_EXTRACTION_PROCEDURE.md`
6. `05_CURRENT_RECALL_FOOTGUNS_AND_FIXES.md`
7. `06_ARTIFACT_AND_CONTRACT_MODEL.md`
8. `07_APP_PLAN_AND_PROFILE_MODEL.md`
9. `08_PROVIDER_TOOL_SECURITY_MODEL.md`
10. `09_MEMORY_KERNEL_AND_RUNTIME_MODEL.md`
11. `10_QUEUE_SCHEDULE_DAEMON_UI_MODEL.md`
12. `11_TEST_AND_CONFORMANCE_PLAN.md`
13. `12_MIGRATION_ISSUE_MATRIX.md`
14. `13_IMPLEMENTATION_ROADMAP.md`
15. `14_RISK_REGISTER.md`
16. `15_API_SKETCHES.md`
17. `16_TRACEABILITY_MATRIX.md`
18. `17_RESEARCH_SYNTHESIS_AND_DESIGN_LAWS.md`
19. `18_V0_1_MINIMUM_PRODUCT_SPEC.md`

## Main recommendation

Build AiDENs as a public umbrella crate plus smaller crates underneath:

```text
aidens                       # public easy button
aidens-app-kit               # app builder, profiles, templates, doctor integration
aidens-runner                # turn/run execution coordinator

foundation:
  aidens-contracts
  aidens-boundary-kit
  aidens-config
  aidens-receipts
  aidens-capability-kit

capability adapters:
  aidens-provider-kit
  aidens-tool-kit
  aidens-security-kit
  aidens-memory-kit
  aidens-kernel-kit
  aidens-queue-kit

control:
  aidens-arbiter-kit
  aidens-permit-kit
  aidens-budget-kit
  aidens-governance-kit
  aidens-schedule-kit
  aidens-delegation-kit
  aidens-plan-kit
  aidens-repair-kit

shells:
  aidens-cli
  aidens-daemon-kit
  aidens-tauri-kit
  aidens-web-kit
  aidens-testkit
```

Most application developers should use only:

```toml
[dependencies]
aidens = { version = "...", features = ["coding"] }
```

The internal crate split exists to stop footguns. It should not become user-facing ceremony.

## Definition of done for v0.1

A new coding-agent app should be generated, checked, and run by:

```bash
aidens new coding-agent my-agent
cd my-agent
aidens doctor
aidens list-capabilities
aidens list-tools
aidens provider-check
cargo test
cargo run
```

The generated app must automatically produce:

- provider truth,
- tool exposure truth,
- approval/permit truth,
- config truth,
- runtime capability truth,
- execution receipts,
- native/fallback mode labels,
- safe defaults,
- doctor checks,
- starter tests,
- no hidden web/network path,
- no daemon split-brain,
- no receipt-less tool execution,
- no parser repair without receipt.

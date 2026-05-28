# Codex Prompt — P10 Coding-agent tool suite, sandbox discipline, and Codex packet generator

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P10_CODING_AGENT_TOOLING_SANDBOX_AND_CODEX_PACKETS.md`.

Implement P10 only. Do not start later passes.

## Goal

Make AiDENs actually useful as a governed coding agent: read, inspect, propose patch, apply approved patch, run checks, and export Codex-ready packets.

## Primary crates

- `aidens-tool-kit`
- `aidens-security-kit`
- `aidens-permit-kit`
- `aidens-runner`
- `aidens-cli`
- `aidens-profile-coding`

## Required artifacts

- `RepoReadReceiptV1`
- `RepoListReceiptV1`
- `PatchProposalV1`
- `PatchApplyReceiptV1`
- `CommandRunReportV1`
- `CodexPacketV1`
- `SandboxCapabilityTruthV1`

## Acceptance gates

- Coding profile can read repo, propose a patch, request approval, apply patch after permit, and run cargo checks with receipts.
- Attempted path traversal/sensitive prefix access fails with receipt.
- Codex packet contains enough state for another agent to resume without archaeology.

## Forbidden shortcuts

- Do not allow shell/file-write/network by default.
- Do not let patch-propose mutate files.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.

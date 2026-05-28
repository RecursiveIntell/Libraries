# P25 Main Codex Run Prompt

You are executing **P25 for AiDENs**.

## Goal

Perform a surgical release-hygiene and phase-gate hardening pass after P24. Preserve AiDENs as a supported-local, evidence-bearing agent/profile/operator layer over the canonical libraries.

Do not implement V10+ runtime architecture. Do not expand `z.py` except for the root Markdown archive feature described in this packet.

## Required source files to read first

1. `P25_MASTER_PACKET.md`
2. `P25_HARD_AUDIT.md`
3. `P25_PHASE_GATE_PROTOCOL.md`
4. `P25_PHASE_PLAN.md`
5. `P25_ZPY_ROOT_MARKDOWN_ARCHIVER_SPEC.md`
6. `P25_VERIFIER_SPEC.md`
7. `P25_FLAGSHIP_AGENT_DEMO_SPEC.md`
8. `P25_ACCEPTANCE_GATES.md`
9. `P25_SUPPORT_PROFILE_TARGET.md`
10. `P25_CLAUDE_AUDIT_ABSORPTION.md`

## Hard constraints

1. AiDENs is consumer-only. Do not invent canonical memory, evidence, execution, repair, schema, verification, or runtime semantics.
2. `z.py` is operator-local packaging/certifier tooling. Touch it only to add root workspace Markdown noise archiving and necessary report/manifest checks.
3. Phase gates are blocking human approval gates. At every specified gate, stop and wait for the operator's pasted injection prompt.
4. No silent widening.
5. No compatibility shims.
6. No fake completion.
7. All changes must emit evidence: changed files, commands, validation results, unresolved risks.

## Required z.py feature

Add root workspace Markdown noise archiving.

It must:
- only target Markdown files directly in the archive root/workspace root;
- not target nested docs;
- preserve protected root docs;
- archive run/audit/spec/prompt/matrix residue to `docs/root-markdown-archive/<timestamp>/files/`;
- emit `ROOT_MARKDOWN_ARCHIVE_MANIFEST.json`;
- support dry-run and verify-only without moving files;
- fail closed on collisions;
- leave ambiguous files untouched and report them.

## Required P25 outcomes

- root Markdown archive feature with dry-run and strict behavior;
- phase-injection files updated to P25 with explicit STOP/WAIT language;
- verifier check that fails on stale phase IDs or missing blocking language;
- codex artifact classification/current-run docs cleaned;
- package validation remains 0 findings/errors/warnings;
- supported-local flagship coding-agent demo exists and replays deterministically;
- support profile updated;
- final hostile audit and evidence manifest emitted.

## Stop rule

At every configured phase boundary, emit the phase report and stop. Do not begin the next phase until the operator pastes the matching manual phase-injection prompt.

If you cross a phase boundary without stopping, mark it as a run violation, quarantine work performed after the missed gate, and wait for operator approval.

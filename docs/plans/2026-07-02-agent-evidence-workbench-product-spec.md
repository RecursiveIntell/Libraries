# Agent Evidence Workbench Product Specification

Date: 2026-07-02
Author: Hermes Agent
Status: strategic/product spec, not implementation-complete
Canonical working name: Agent Evidence Workbench
Recommended public name: Agent Evidence Workbench or Agent Black Box only if trademark/name collision is cleared

## Executive verdict

Build a thin product layer on top of `semantic-memory-mcp`, `context-governor`, and `claim-ledger`:

> A local-first evidence and replay layer for AI coding agents. It records agent runs, ties claims to receipts, preserves compressed context with exact fallback, and produces audit reports showing what changed, what was verified, what is unsupported, and what to run next.

Do not build a new memory server. Do not replace semantic-memory-mcp. Do not lead with a general agent framework.

The highest-ROI wedge is accountability, not memory:

- semantic-memory helps agents remember.
- Agent Evidence Workbench makes agents accountable.

## Why this is worth building

### Market pull

The coding-agent market is already large and accelerating. Spot-checked adjacent projects:

| Project | Stars | Relevance |
|---|---:|---|
| openai/codex | 94,957 | Major terminal coding agent |
| cline/cline | 64,198 | Autonomous coding agent SDK/IDE/CLI |
| aider-ai/aider | 46,933 | Terminal pair-programming agent |
| continuedev/continue | 34,634 | Open-source coding agent |
| langfuse/langfuse | 30,254 | LLM observability/evals platform |
| Arize-ai/phoenix | 10,371 | AI observability/evaluation |
| AgentOps-AI/agentops | 5,673 | Agent monitoring/cost/tracking |
| Helicone/helicone | 5,891 | LLM observability |
| mem0ai/mem0 | 59,888 | Agent memory layer |
| getzep/graphiti | 28,245 | Real-time KG memory for agents |
| letta-ai/letta | 23,622 | Stateful agent memory |

Existing products validate demand for memory, tracing, observability, evaluation, and agent harnesses. The gap is a local-first, proof-led, coding-agent-specific accountability layer that treats final-agent claims as hypotheses requiring receipts.

### Direct competitor reality

This space is not empty. Direct or near-direct projects exist:

| Project | Stars | Position | Gap/opportunity against it |
|---|---:|---|---|
| TaewoooPark/Agent-Blackbox | 42 | Local-first flight recorder/context-efficiency profiler for coding agents | Good name/positioning, early traction. Need differentiate with claim-ledger/proof gates and semantic-memory integration. |
| B33BMO/vaportrail | 3 | Local-first multi-agent transcript reader/search/stats | Mostly transcript history/search; less proof/claim gating. |
| Feloguarin/claude-insight | 75 | Claude transcript analyzer/fluency report | Personal behavior analytics, not evidence ledger. |
| AndrewK404/tracebook | 13 | Claude Code transcript dashboard + optional hooks | Read-only dashboard, not claim/proof governance. |
| ReinaMacCredy/maestro | 204 | Local-first task/verdict ledger for codebases | Strong adjacent; card/task governance more than transcript/context proof. |
| sipyourdrink-ltd/bernstein | 615 | Audit-grade multi-agent orchestration with HMAC audit logs | More orchestration/compliance-heavy; potentially strong competitor for enterprise angle. |
| cordum-io/cordum | 485 | Agent control plane, policy gates, audit trails | Cloud/control-plane/security positioning; source-available/BUSL. |
| Justin0504/Aegis | 360 | AI-agent firewall, policy enforcement, cryptographic audit trail | Security/firewall-first. |
| zistica/korveo | 5 | Local-first firewall + flight recorder | Similar broad positioning, early. |
| MLaminekane/hawkeye | 6 | Flight recorder/observability/security for Claude/Codex/Cline | Similar broad positioning, early. |
| sheeki03/logbook | 1 | Local-first black-box recorder with redaction/revert/UI | Similar capture layer, early. |

Conclusion: do not claim nobody has thought of “agent flight recorder.” They have. The winning wedge must be narrower and stronger:

> Claim-level proof and context-loss accountability for coding agents, backed by semantic-memory, bitemporal supersession, and receipt-bearing context governance.

## Product boundary

### What this is

A local-first CLI + report generator + optional local dashboard that answers, for any AI coding-agent run:

1. What did the agent do?
2. What files changed?
3. What commands ran and what did they return?
4. What did the agent claim in its final answer?
5. Which claims are supported by receipts?
6. Which claims are unsupported, stale, contradicted, or need replay?
7. What context was compressed away?
8. Can exact fallback recover the relevant omitted context?
9. What is the next command/action required to close proof debt?
10. What public-safe summary can be emitted without overclaiming?

### What this is not

- Not a replacement for semantic-memory-mcp.
- Not a new vector DB.
- Not a general-purpose hosted observability SaaS.
- Not an enterprise compliance product in v1.
- Not an agent orchestrator in v1.
- Not a security firewall in v1.
- Not a RAG app.
- Not an “autonomous agent OS.”

## Core product claim

Safe public claim after MVP:

> Agent Evidence Workbench is a local-first audit and receipt layer for AI coding-agent runs. It records session events, links final claims to command/file/test receipts, preserves compressed context with exact fallback, and generates replayable reports so operators can separate verified work from unsupported agent claims.

Unsafe until proven:

- “Best flight recorder.”
- “Enterprise compliance.”
- “Prevents all harmful agent actions.”
- “Outperforms Agent-Blackbox/Bernstein/Aegis/etc.”
- “Production security platform.”
- “Complete provenance for every possible agent host.”

## Existing RecursiveIntell assets to reuse

### Substrate: semantic-memory-mcp

Current role:
- Durable fact/document/chunk storage.
- Hybrid retrieval.
- Graph/second-order search.
- Bitemporal/supersession/provenance mechanisms.
- Hermes/Codex/Claude plugin ecosystem.

Do not overload semantic-memory with raw transcript dumps. It should index summaries, claims, durable decisions, and evidence anchors.

### Context survival: context-governor

Current verified capabilities from local code/memory:
- `ContextCompactionReceiptV1` includes transcript hashes, token counts, exact fallback refs, semantic-memory IDs, and summary loss report.
- `CompactResponse` carries receipt, allocation plan, compacted messages, exact store.
- Boundary audit and replay/eval tooling exist.
- Hermes context engine integration exists.
- Existing evidence reports show strong local replay/recoverability metrics, but not external superiority.

Workbench role:
- Store context-governor receipts as first-class session evidence.
- Report what context was omitted/recoverable.
- Warn when compression summaries introduce unsafe relinking or unsupported memory writes.

### Truth layer: claim-ledger

Current role:
- Atomic claims.
- Evidence bundles.
- Support judgments.
- Support admissions.
- Contradiction records.
- Hash-chained ledger.
- Export receipts.

Workbench role:
- Extract final-answer claims.
- Convert evidence anchors into evidence bundles.
- Assign support states.
- Supersede stale claims.
- Export claim boundary report.

### Tool receipts: llm-tool-runtime / existing hooks

Current role:
- `ToolReceipt` with tool name/version, input digest, output refs, approval state, host identity, times, trace, retry/error fields.
- Hermes already has a post-tool receipt hook that records lightweight non-sm tool receipts into the `tool-receipts` namespace, but recall filters that namespace by default to prevent pollution.

Workbench role:
- Use tool receipts as cold evidence, not default semantic recall.
- Promote only selected evidence anchors into semantic-memory/claim-ledger.

### Governance and supporting crates

Potential supporting crates/surfaces:
- `bitemporal-runtime`: valid/recorded time, append/supersede.
- `boundary-compiler`: duplicate-key rejection/canonicalization.
- `verification-control`: verification plans and receipts.
- `job-queue`: background indexing/report generation.
- `agent-graph`: execution graph if/when workflow visualization is needed.
- `AiDENs/aidens-receipts`: NDJSON append + chain verification + digests.
- `Forge-Audit`, `benchmark`, `rust-ai-quality-benchmark`: verification/evaluation harness ideas.

## Differentiation strategy

The direct competitors mostly emphasize:
- transcript parsing;
- dashboards;
- token/cost analytics;
- file/action timelines;
- policy/firewall enforcement;
- orchestration;
- observability traces.

RecursiveIntell should emphasize:

1. Final-answer claim checking.
   - Treat every “done,” “tests pass,” “published,” “fixed,” “no regressions” as a claim needing evidence.

2. Proof-debt reporting.
   - Produce a remaining-proof checklist, not just a pretty timeline.

3. Context-loss receipts.
   - Show whether important task anchors were visible, recoverable, or lost after compaction.

4. Bitemporal truth.
   - “Was true during run” vs “current now” vs “superseded later.”

5. Semantic recall without recall pollution.
   - Raw traces remain cold; promoted claims/evidence anchors become searchable.

6. Public-safe summary generation.
   - Built-in claim boundary discipline for posts, READMEs, PR summaries, release notes.

This is not just “what happened?” It is “what can we safely believe?”

## User personas

### Primary v1 persona: local power user / solo operator

Needs:
- Audit personal AI coding-agent runs.
- Recover previous work.
- Avoid fake completion.
- Generate honest summaries.
- Keep all data local.

This is Josh and people like Josh.

### Secondary persona: open-source maintainer using coding agents

Needs:
- Know which agent changes were verified.
- Avoid merging unsupported claims.
- Produce PR evidence bundles.

### Later persona: team lead / compliance/security

Needs:
- Shared policy.
- Multi-user audit trails.
- Approval workflows.
- Tamper-evident logs.

Do not optimize v1 for this persona. Bernstein/Cordum/Aegis already lean there, and overclaiming compliance is dangerous.

## Product architecture

### High-level shape

```
Agent host(s)
  Claude Code / Codex / Hermes / OpenCode / Aider / generic CLI
        |
        | hooks / wrapper / transcript import
        v
Capture adapters
        |
        v
Event ledger SQLite + compressed blob store
        |
        +--> Semantic promotion pipeline --> semantic-memory-mcp
        +--> Claim extraction pipeline ----> claim-ledger
        +--> Context pipeline -------------> context-governor receipts
        +--> Report generator ------------> markdown/json/html
        +--> Optional local UI ------------> timeline + claim status
```

### Storage tiers

Rule:

> Raw data is cold evidence. Receipts are hot truth. Embeddings are selective recall.

#### Tier 0: Event ledger

SQLite tables for structured events:
- sessions
- turns
- tool calls
- commands
- file changes
- git snapshots
- model calls if available
- context events
- claims
- evidence links
- reports

This tier stores metadata and short previews, not giant raw payloads.

#### Tier 1: Compressed blob store

Content-addressed blobs for:
- full tool output;
- terminal transcript segments;
- raw diffs;
- test logs;
- provider request/response if enabled;
- imported transcript fragments.

Blobs should be compressed and addressed by BLAKE3/SHA-256 digest.

#### Tier 2: Semantic index

semantic-memory stores only:
- session summaries;
- claim text;
- evidence snippets;
- decision points;
- failure/error anchors;
- file/path anchors;
- durable lessons promoted by policy.

It must not store every raw event as a fact.

#### Tier 3: Claim ledger

Claim-ledger stores:
- atomic final-answer claims;
- support status;
- evidence bundle references;
- contradictions;
- supersessions;
- proof-debt gate result.

#### Tier 4: Context receipts

context-governor stores:
- compaction receipts;
- exact fallback refs;
- loss reports;
- replay/answerability scores;
- boundary-audit warnings.

## Storage growth model

Storage growth is manageable if the tiering rules are enforced.

Rough yearly estimates from local model:

| Usage | Raw retained per turn | No compression | Conservative compression | Aggressive/log folding |
|---|---:|---:|---:|---:|
| 5 sessions/day, 50 turns/session | 25 KB | 3.53 GB/yr | 1.48 GB/yr | 1.01 GB/yr |
| 10 sessions/day, 50 turns/session | 25 KB | 7.06 GB/yr | 2.96 GB/yr | 2.02 GB/yr |
| 20 sessions/day, 100 turns/session | 50 KB | 53.49 GB/yr | 20.69 GB/yr | 13.12 GB/yr |

Assumptions:
- 768-dim embedding = about 3 KB per embedded chunk before SQLite overhead.
- SQLite overhead modeled at 1.35x.
- Only selected chunks get embeddings.
- Compression ratio 0.35 conservative; 0.20 for aggressive log folding/dedup.

Design implication:
- Solo operator use can stay single-digit GB/year.
- Heavy use still stays manageable if raw blobs are compressed and embeddings are selective.
- The failure mode is embedding and indexing every raw tool result.

### Retention policy

Default v1 retention:

| Data kind | Default retention | Reason |
|---|---:|---|
| Session metadata | Forever | Small, useful |
| Claim ledger | Forever | Truth history |
| Evidence metadata | Forever | Small |
| Short previews | Forever | Search/display |
| Raw blobs | 180 days | Large but useful for replay |
| Failed-run raw blobs | 365 days | Higher diagnostic value |
| Published/release receipts | Forever | Public artifact support |
| Secret-tainted blobs | Redact immediately; optional quarantine | Safety |
| Embeddings | Until promoted item deleted/superseded | Search value |

Add commands:
- `aew retention status`
- `aew retention prune --dry-run`
- `aew retention prune --older-than 180d`
- `aew retention protect <session_id>`

## Data model v1

### SessionRunV1

```json
{
  "schema": "SessionRunV1",
  "session_id": "run_...",
  "agent_host": "codex|claude_code|hermes|opencode|aider|generic",
  "project_root": "/abs/path",
  "git_remote": "optional",
  "git_branch_start": "main",
  "git_head_start": "sha",
  "git_head_end": "sha_or_null",
  "started_at": "RFC3339",
  "ended_at": "RFC3339",
  "operator_goal": "text",
  "capture_mode": "wrapper|hook|import|hybrid",
  "privacy_mode": "local_only|redacted_export",
  "status": "completed|failed|interrupted|unknown",
  "event_count": 123,
  "claim_count": 8,
  "unsupported_claim_count": 2,
  "evidence_bundle_ids": ["evb_..."]
}
```

### AgentEventV1

```json
{
  "schema": "AgentEventV1",
  "event_id": "evt_...",
  "session_id": "run_...",
  "parent_event_id": null,
  "event_type": "prompt|assistant|tool_call|command|file_change|git|test|model_call|context|claim|system",
  "started_at": "RFC3339",
  "finished_at": "RFC3339|null",
  "source": "hook|wrapper|transcript_import|derived",
  "summary": "short display text",
  "preview": "short sanitized preview",
  "blob_digest": "blake3:...|null",
  "input_digest": "blake3:...|null",
  "output_digest": "blake3:...|null",
  "status": "ok|error|blocked|skipped|unknown",
  "exit_code": 0,
  "paths": ["relative/path.rs"],
  "tags": ["cargo-test", "verification"],
  "sensitivity": "public|internal|confidential|restricted"
}
```

### AgentClaimV1

```json
{
  "schema": "AgentClaimV1",
  "claim_id": "claim_...",
  "session_id": "run_...",
  "source_event_id": "evt_final_answer",
  "claim_text": "Tests pass.",
  "claim_type": "tests_pass|implemented|fixed|published|no_regressions|file_changed|performance|security|other",
  "risk_class": "low|medium|high|critical",
  "support_state": "supported|unsupported|contested|stale|heuristic_only",
  "required_evidence": ["test_command_receipt"],
  "evidence_ids": ["ev_..."],
  "proof_debt": 1.0,
  "superseded_by": null,
  "created_at": "RFC3339"
}
```

### EvidenceBundleRefV1

```json
{
  "schema": "EvidenceBundleRefV1",
  "evidence_id": "ev_...",
  "session_id": "run_...",
  "evidence_type": "command_output|git_diff|file_hash|test_result|registry_api|context_receipt|operator_admission|artifact",
  "event_ids": ["evt_..."],
  "blob_digest": "blake3:...|null",
  "summary": "cargo test --workspace exited 0",
  "verification_strength": "direct|indirect|operator_admitted|heuristic",
  "valid_time": "RFC3339|null",
  "recorded_time": "RFC3339",
  "source_path": "relative/or/absolute/path|null"
}
```

### RunReportV1

```json
{
  "schema": "RunReportV1",
  "report_id": "rpt_...",
  "session_id": "run_...",
  "generated_at": "RFC3339",
  "verdict": "verified|partial|unsupported|failed|needs_review",
  "changed_files": ["..."],
  "commands_run": ["cargo test --workspace"],
  "supported_claims": ["claim_..."],
  "unsupported_claims": ["claim_..."],
  "stale_claims": [],
  "context_receipt_ids": ["ctxr_..."],
  "next_required_actions": ["Run cargo test --workspace --all-targets"],
  "public_safe_summary": "..."
}
```

## Evidence rules

### Claim type support matrix

| Claim type | Required evidence | Unsupported if |
|---|---|---|
| tests_pass | command receipt with matching test command and exit 0 | no test command, failed exit, stale before final changes |
| cargo_check_pass | `cargo check`/`cargo check --workspace` receipt exit 0 | command missing or failed |
| implemented | git diff/file hash evidence + optionally tests | only assistant statement |
| fixed_bug | reproduction/failing test before + passing after, or operator admission | no reproduction or before/after evidence |
| published | registry/API receipt or release artifact | only git tag or assistant statement |
| no_regressions | relevant test suite receipt after final diff | no broad test receipt |
| performance_improved | benchmark before/after same fixture | one-sided benchmark or different fixture |
| safe/security_fixed | targeted security test/audit result | only code diff or intuition |
| no_secret_leak | redaction/secret scan receipt | no scan |

### Final answer claim extraction

The first version can use deterministic patterns before LLM extraction:

- “done”, “implemented”, “fixed”, “patched”, “added”, “removed” -> implementation claims.
- “tests pass”, “cargo test passed”, “check passed” -> verification claims.
- “published”, “released”, “pushed” -> publication claims.
- “no regressions”, “safe”, “secure” -> high-risk claims.

Later version may use LLM extraction, but extracted claims must be marked `heuristic_only` until tied to evidence.

## Capture modes

### Mode A: Transcript import

Read existing local transcripts:
- Claude Code JSONL.
- Codex logs/state where available.
- Hermes state.db/session DB.
- OpenCode/Aider logs later.

Pros: easiest MVP, no runtime risk.
Cons: may miss live command stdout or exact model requests.

### Mode B: Hook capture

Use host hooks:
- Hermes pre/post tool hooks.
- Claude Code hooks.
- Codex hooks.
- Context-governor PreCompact/Stop style hooks.

Pros: structured events.
Cons: host-specific integration complexity.

### Mode C: Wrapper capture

Run:

```bash
aew run -- codex exec "fix failing tests"
aew run -- claude -p "implement feature"
aew run -- aider ...
```

Pros: agent-agnostic process boundary, captures stdout/stderr/exit code and git diffs.
Cons: less semantic structure than hooks.

### Recommended MVP order

1. Import + report for Hermes current session DB and Claude transcript JSONL.
2. Wrapper mode for arbitrary command + git diff + command receipts.
3. Codex/Claude/Hermes hook adapters for richer events.
4. Optional local UI.

## CLI spec

Binary name options:
- `aew`
- `ri-agent-audit`
- `agent-evidence`

Recommended internal binary: `ri-agent-audit`.
Recommended public command later: `aew` if name is available.

### Commands

```bash
ri-agent-audit init
ri-agent-audit run -- codex exec "fix failing test"
ri-agent-audit import --host claude-code --path ~/.claude/projects
ri-agent-audit import --host hermes --state ~/.hermes/state.db
ri-agent-audit list
ri-agent-audit show <session_id>
ri-agent-audit report <session_id> --format md,json,html
ri-agent-audit verify <session_id>
ri-agent-audit claims <session_id>
ri-agent-audit evidence <claim_id>
ri-agent-audit search "published semantic-memory"
ri-agent-audit storage status
ri-agent-audit retention prune --dry-run
ri-agent-audit export <session_id> --redacted
```

### MVP report files

For a run, generate:

```
.agent-evidence/runs/<session_id>/
  SESSION_RECEIPT.md
  SESSION_RECEIPT.json
  CLAIM_LEDGER.md
  CLAIM_LEDGER.json
  UNSUPPORTED_CLAIMS.md
  FILE_CHANGE_LEDGER.md
  COMMAND_RECEIPTS.jsonl
  CONTEXT_RECEIPTS.jsonl
  PUBLIC_SAFE_SUMMARY.md
  blobs/
```

## Local dashboard spec, later not first

Dashboard tabs:

1. Runs
   - list sessions, agent host, repo, verdict, unsupported count, changed files.

2. Timeline
   - prompt, tool calls, commands, diffs, tests, final answer.

3. Claims
   - claim text, support state, evidence links, proof debt.

4. Evidence
   - command receipts, git diffs, test logs, registry checks.

5. Context
   - compaction receipts, lost/recoverable anchors, exact fallback search.

6. Public summary
   - copy-safe summary and claim boundary.

Dashboard is P2. CLI + markdown reports are P0.

## Implementation plan

### Phase 0: Product skeleton and storage foundations

Goal: create the crate/binary and local storage layout without host-specific complexity.

Recommended location:

```
/home/sikmindz/Coding/Libraries/agent-evidence-workbench/
```

or, if keeping it inside the existing workspace:

```
/home/sikmindz/Coding/Libraries/agent-evidence/
```

P0 tasks:

1. Create Rust crate `agent-evidence` with CLI binary `ri-agent-audit`.
2. Add SQLite schema migrations for sessions/events/claims/evidence/reports/blob_index.
3. Add content-addressed blob store with zstd feature optional.
4. Add BLAKE3 hashing for blobs and event payloads.
5. Add `init`, `storage status`, and `list` commands.
6. Add unit tests for schema migration and blob dedupe.

Acceptance gate:

```bash
cargo test -p agent-evidence --all-targets
ri-agent-audit init --store /tmp/aew-test
ri-agent-audit storage status --store /tmp/aew-test
```

Expected:
- SQLite DB created.
- Blob dir created.
- No raw blob duplicated when same content is inserted twice.

### Phase 1: Import one real host transcript

Goal: prove value from existing data without runtime capture.

Start with Hermes state/session DB because it is local and already available. Claude Code JSONL can be second.

Tasks:

1. Implement `import hermes --state ~/.hermes/state.db`.
2. Parse user/assistant/tool-ish rows into `AgentEventV1`.
3. Store transcript text as compressed blob if large.
4. Extract final assistant response for claim extraction.
5. Generate timeline report.

Acceptance gate:

```bash
ri-agent-audit import hermes --state ~/.hermes/state.db --limit 1
ri-agent-audit report --last --format md
```

Expected report includes:
- session id;
- prompt count;
- final answer excerpt;
- tool/command events where recoverable;
- raw blob digests, not giant inline dumps.

### Phase 2: Git/file/command evidence wrapper

Goal: make a new recorded run produce hard evidence.

Tasks:

1. Implement `run -- <command...>` wrapper.
2. Capture start git status/head/branch.
3. Run child process with stdout/stderr capture.
4. Capture exit code and duration.
5. Capture end git status/head/branch.
6. Generate file change ledger using `git diff --name-status` and `git diff --stat`.
7. Store full stdout/stderr as compressed blobs.
8. Create command receipt events.

Acceptance gate:

```bash
ri-agent-audit run -- bash -lc 'echo hello && true'
ri-agent-audit report --last
```

Then run on a real low-risk command:

```bash
ri-agent-audit run -- cargo test -p context-governor --all-targets
```

Expected:
- command receipt with exit 0;
- stdout blob stored;
- report says verification command supported;
- no semantic-memory fact pollution.

### Phase 3: Deterministic final-answer claim checker

Goal: produce the killer feature.

Tasks:

1. Add deterministic claim extraction from final assistant text.
2. Add claim types: tests_pass, cargo_check_pass, implemented, fixed_bug, published, no_regressions, performance, security, other.
3. Add support rules mapping events to evidence.
4. Generate `CLAIM_LEDGER.md` and `UNSUPPORTED_CLAIMS.md`.
5. Add proof-debt score.

Acceptance gate fixture:

Input final answer:

```text
Implemented the parser. Tests pass. Published v0.2.0. No regressions.
```

Evidence:
- git diff exists;
- `cargo test` absent;
- registry receipt absent.

Expected:
- “Implemented the parser” = heuristic/partial support from diff.
- “Tests pass” = unsupported.
- “Published v0.2.0” = unsupported.
- “No regressions” = unsupported/high risk.

This gate matters more than UI.

### Phase 4: claim-ledger integration

Goal: stop using ad-hoc JSON once the flow is stable.

Tasks:

1. Map `AgentClaimV1` into `claim-ledger::Claim`.
2. Map `EvidenceBundleRefV1` into `claim-ledger::EvidenceBundle`/links.
3. Record support judgments.
4. Export claim bundle with receipt.
5. Add `ri-agent-audit claims export --last`.

Acceptance gate:

```bash
ri-agent-audit verify --last
ri-agent-audit claims export --last
```

Expected:
- JSON claim bundle validates.
- Unsupported claims stay unsupported.
- Operator admissions are explicit, never silent.

### Phase 5: context-governor integration

Goal: make context loss visible in the run report.

Tasks:

1. Accept context-governor receipt JSON import.
2. Link context receipt to session/run.
3. Display token savings, warnings, exact fallback count, summary loss report.
4. Add `ri-agent-audit context search <session> <query>` backed by context-governor store if available.
5. Add boundary-audit warning section.

Acceptance gate:

```bash
context-governor compact < fixture.json > response.json
ri-agent-audit context import --session <id> response.json
ri-agent-audit report <id>
```

Expected report includes:
- original/compacted token estimate;
- fallback refs;
- warnings;
- “not proof of downstream LLM quality” claim boundary.

### Phase 6: semantic-memory promotion

Goal: make runs searchable without polluting default recall.

Tasks:

1. Add promotion policy.
2. Promote only session summaries, claims, evidence anchors, decisions, errors, durable lessons.
3. Namespace: `agent-evidence` or `agent-runs`.
4. Sensitivity default: internal.
5. Do not promote raw transcripts/tool logs.
6. Add `--promote` flag and dry-run.

Acceptance gate:

```bash
ri-agent-audit promote --last --dry-run
ri-agent-audit promote --last
sm_search "<distinctive claim text>"
```

Expected:
- semantic-memory finds the claim/summary.
- raw stdout is not in semantic-memory facts.
- tool-receipts namespace remains filtered from default recall.

### Phase 7: host hooks

Goal: richer capture after the report value is proven.

Order:
1. Hermes adapter/hook.
2. Codex plugin hook/wrapper.
3. Claude Code plugin hook/wrapper.
4. OpenCode/Aider importers.

Acceptance gate:
- Run one real Hermes/Codex/Claude session.
- Report shows prompt, tool calls, commands, final claims, and evidence.

### Phase 8: local UI

Goal: make it demoable.

Use Tauri/React only after CLI reports are valuable.

Minimum UI:
- runs list;
- timeline;
- claims table;
- evidence preview;
- context receipt panel;
- public-safe summary copy button.

Acceptance gate:
- UI opens a real recorded run and matches CLI report counts.

## Testing strategy

### Unit tests

- Blob store dedup.
- SQLite schema migration.
- Event insertion/retrieval.
- Claim extraction patterns.
- Claim support rules.
- Redaction filters.
- Retention pruning dry-run.

### Integration tests

- Import fixture transcript.
- Run wrapper around known shell command.
- Git diff capture in temp repo.
- Claim checker fixture.
- Context receipt import.
- Semantic-memory promotion dry-run.

### Golden reports

Keep fixtures for:
- successful run;
- failed run;
- unsupported final claims;
- publish claim without registry proof;
- test claim with failed test receipt;
- context-governor warning;
- secret-redacted output.

### Dogfood test

Use it on its own implementation:

```bash
ri-agent-audit run -- codex exec "implement next task from docs/plans/..."
ri-agent-audit verify --last
ri-agent-audit report --last
```

The dogfood report is the demo.

## Security and privacy requirements

1. Local-only by default.
2. No telemetry.
3. Redact secrets before writing blobs where possible.
4. Store sensitivity class on every event/blob.
5. Redacted export by default.
6. Raw provider traffic opt-in only.
7. Never promote confidential/restricted blobs to semantic-memory facts.
8. Never treat LLM-generated summaries as supported facts without evidence/admission.

Secret scan minimum:
- common API key patterns;
- `.env` lines;
- GitHub tokens;
- OpenAI/Anthropic keys;
- AWS keys;
- private keys;
- bearer tokens.

## Public positioning

### Best positioning

“Local-first flight recorder and proof ledger for AI coding-agent runs.”

### More concrete

“Agent Evidence Workbench records coding-agent sessions, links final claims to command/file/test receipts, preserves compressed context with exact fallback, and generates replayable audit reports.”

### README opener

```text
AI coding agents move fast. Their summaries are not evidence.

Agent Evidence Workbench is a local-first receipt layer for Codex, Claude Code, Hermes, and other coding agents. It records what changed, what commands ran, what the agent claimed, and which claims are actually supported by receipts.
```

### Demo script

1. Run an agent on a small repo.
2. Agent final answer says “tests pass” without running tests.
3. Workbench report marks the claim unsupported.
4. Run tests under `ri-agent-audit run -- cargo test`.
5. Regenerate report.
6. Claim becomes supported.
7. Show context-governor receipt: token reduction + exact fallback.
8. Export public-safe summary.

This demo is stronger than a dashboard screenshot.

## Competitive response matrix

If asked how this differs from Agent-Blackbox/vaportrail/tracebook:

> They reconstruct and visualize agent activity. Agent Evidence Workbench treats the agent’s final answer as a claim set and verifies it against receipts, bitemporal state, and context-loss records. The goal is not just replay; it is safe belief.

If asked how this differs from Langfuse/Phoenix/AgentOps:

> Those are LLM/agent observability and eval platforms. Agent Evidence Workbench is local-first and coding-agent specific: git diffs, shell commands, tests, final-answer claims, context receipts, and proof-debt reports.

If asked how this differs from semantic-memory-mcp:

> semantic-memory-mcp is the memory substrate. Agent Evidence Workbench is the accountability product layer: run capture, claim checking, evidence bundles, and operator reports.

If asked how this differs from Aegis/Cordum/Korveo:

> Those emphasize prevention/policy/firewall/control-plane. Agent Evidence Workbench starts with after-action proof, replay, and claim verification. Policy enforcement can come later.

If asked how this differs from Bernstein/maestro:

> Those orchestrate/manage agent work. Agent Evidence Workbench audits any run regardless of orchestrator and focuses on final claims versus evidence.

## Product risks

### Risk 1: crowded naming/positioning

“Flight recorder for agents” is already used by several projects. Use that as explanatory phrase, not sole differentiation.

Mitigation: lead with claim verification / proof ledger.

### Risk 2: DB becomes a junk drawer

Mitigation: strict tiering. Raw blobs cold, selected anchors hot, semantic promotion opt-in/policy-driven.

### Risk 3: host integrations become a tarpit

Mitigation: start with import + wrapper mode. Hooks later.

### Risk 4: overclaiming security/compliance

Mitigation: avoid enterprise/compliance language until tamper-evidence, policy enforcement, and adversarial tests exist.

### Risk 5: UI-first trap

Mitigation: markdown/json reports first. Dashboard only after reports are obviously useful.

### Risk 6: claim extraction false positives

Mitigation: unsupported by default. Claim extraction creates hypotheses, not truth.

## Metrics that matter

MVP metrics:
- Percent of final-answer claims classified.
- Percent of claims with direct evidence.
- Unsupported claim count per run.
- Time to generate report.
- Raw blob storage per run.
- Semantic facts promoted per run.
- Replay answerability on known questions.
- Secret redaction hits.

Do not optimize first for:
- token-cost dashboard;
- pretty graph rendering;
- multi-agent orchestration;
- cloud sync.

## “Perfect v1” definition

A perfect v1 is not broad. It is sharp:

Given one AI coding-agent run, it produces a report that a skeptical operator trusts more than the agent’s own final answer.

Minimum report must say:

1. Goal.
2. Agent/host.
3. Repo/path/git state.
4. Files changed.
5. Commands run.
6. Tests/checks run and exit status.
7. Final-answer claims.
8. Supported claims.
9. Unsupported claims.
10. Proof-debt next actions.
11. Context-governor receipt summary if available.
12. Public-safe summary.

If v1 does that, it is useful.

## Recommended build decision

Build it, but only as a thin layer:

- Reuse semantic-memory-mcp.
- Reuse context-governor.
- Reuse claim-ledger.
- Reuse llm-tool-runtime receipt ideas.
- Do not fork into a new giant platform.

First deliverable:

```bash
ri-agent-audit run -- <agent-or-command>
ri-agent-audit report --last
ri-agent-audit verify --last
```

If that produces one brutally honest report from a real coding-agent run, the product thesis is validated.

## Immediate next actions

1. Create `agent-evidence` crate.
2. Implement local SQLite + blob store.
3. Implement wrapper mode.
4. Implement deterministic claim checker.
5. Generate markdown reports.
6. Dogfood on its own implementation.
7. Only then add semantic-memory promotion and host hooks.

## Evidence basis for this spec

Internal evidence checked:
- semantic-memory routed recall on product scope, plugins, and portfolio surfaces.
- Local repo inventory in `/home/sikmindz/Coding/Libraries`.
- `context-governor/src/lib.rs` for `ContextCompactionReceiptV1`, `CompactResponse`, memory sink behavior.
- `llm-tool-runtime/src/contracts.rs` for `ToolReceipt` and Forge receipt normalization.
- `claim-ledger/src/lib.rs` for claims/evidence/support/contradiction/export receipt primitives.
- Existing context-governor ROI plan at `/home/sikmindz/Coding/Libraries/context-governor/docs/plans/2026-07-01-next-level-roi-audit.md`.
- Hermes hooks showing semantic-memory recall and tool-receipt filtering behavior.
- crates.io and GitHub API snapshots for internal and external projects.

Current local state caveat:
- `/home/sikmindz/Coding/Libraries/semantic-memory` has a dirty working tree unrelated to this spec.
- `/home/sikmindz/Coding/Libraries/context-governor` appeared clean during this audit.
- This spec was written as a planning artifact only; no implementation code was changed.

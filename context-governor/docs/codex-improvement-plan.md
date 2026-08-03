# Codex stack improvement plan

Scope: make the local context-governor / semantic-memory / Codex plugin stack improve Codex reliability on long, high-risk software tasks.

## Phase 0: lock core invariants

- Keep latest user task after the context-governor summary in compacted prompts.
- Compute compacted transcript hashes and token counts after all message mutations.
- Keep exact fallback records for summarized, omitted, quarantined, receipt-only, and archived items.
- Preserve a minimal recovery pointer in hard-cascade mode whenever it fits.
- Verify with regression tests before changing plugin hooks.

## Phase 1: measure Codex benefit

- Run `python scripts/codex_roi_eval.py` before and after compaction changes.
- Set `RUN_CODEX_EVAL=1` only when a live `codex exec` smoke check is worth the model spend.
- Track pass/fail, elapsed time, final findings, and whether receipts remain recoverable.

## Phase 2: improve retrieval surfaces

- Prefer existing plugin scripts for receipt search, expand, diff, and benchmark workflows.
- Add MCP read tools only after the core receipt format is stable.
- Keep store/search/expand read operations low-friction; keep writes and destructive operations explicit.

## Phase 3: tune memory and guidance

- Keep global `AGENTS.md` short and repo-agnostic.
- Put crate-specific verification commands in this repo's `AGENTS.md`.
- Store only verified durable outcomes in semantic-memory, with source paths and commit or receipt identifiers.
- Supersede stale memory instead of adding competing current facts.

## Phase 4: subagent workflows

- Use subagents for read-heavy audits, test triage, and memory curation.
- Keep the main thread focused on requirements, decisions, and final edits.
- Avoid parallel write-heavy subagent workflows unless each agent owns disjoint files.

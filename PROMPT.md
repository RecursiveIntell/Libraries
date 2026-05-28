# PROMPT.md — Libraries audit packet usage

## For use with Claude Code, Cursor, or any AI coding agent

### Step 1: Load context
Upload or paste the following files into the agent's context:
- `HOSTILE_AUDIT_SYNTHESIS_V5.md` (start here — it's the executive summary)
- `MASTER_ISSUE_TENSOR.json` (full issue list with priorities and statuses)
- `CLAUDE.md` (working rules)

### Step 2: Choose your fix target
Tell the agent which issue ID(s) you want to fix. Example:

> Fix CLIB-007 — filter write tools from the tool prompt when approval_handler is None.
> The evidence location is session.rs:815-876 and session.rs:1091-1099.
> Include a test that proves write tools are excluded from the prompt in the no-handler case.

### Step 3: For a full fix sweep
> Read the HOSTILE_AUDIT_SYNTHESIS_V5.md and MASTER_ISSUE_TENSOR.json.
> Fix issues in the recommended order from the synthesis doc.
> For each fix: reference the issue ID in a code comment, make the change at the evidence location, and add a test.
> After each fix, tell me what you changed and which issue ID it closes.

### Step 4: Verify closure
After fixes, re-run the audit by uploading the modified source and asking:

> Here is the updated source. Re-audit from the same 10 perspectives as claude_hard_audit_2026-03-30.md.
> For each issue in MASTER_ISSUE_TENSOR.json, tell me: still open, partially closed, or closed.

### Tensor axes reference
The severity heatmap uses these axes (0 = clean, 5 = severe):
- **Truth** — repo truth, support-claim clarity
- **Contracts** — schema/type boundary rigor
- **Runtime** — execution semantics, failure behavior
- **Governance** — governance authority, constitutional enforcement
- **Ops** — release gates, reproducibility
- **Scale** — hotspots, growth pressure
- **Security** — capability control, misuse resistance
- **Human** — maintainer load, cognitive surface

# z.py Universal Packager Hardening Bundle

Purpose: turn `z.py` from a repo-specific source zip certifier into a reusable, cross-repo package/handoff certifier that preserves RecursiveIntell evidence discipline while learning from mature package ecosystems.

Use this bundle as a Codex implementation pass. It does not ask Codex to rewrite `z.py` blindly. It requires staged implementation, regression fixtures, ecosystem parity checks, and hostile-auditor output.

Start with:

```text
codex/prompts/MASTER_PROMPT.md
```

Then run the phase prompts in order. At every boundary paste:

```text
codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md
```

Final completion requires a hostile-auditor handoff, changed-file list, commands run, tests passed/failed/skipped, and rollback instructions.

# Executive summary

The 2026-03-22 hardening lane is already **closed by evidence**: the active proof ledger records passing results for repo surface, doc truth, manifest truth, schema ownership, no production panics, mirror discipline, hotspot budgets, public type drift, root archive manifest, public API docs, schema compatibility, selected cargo tests, and closeout receipt generation.

What is *not* finished is the **front-door and external demonstration layer**:

1. the supplied source tree is missing the root control-plane pack that the scripts and receipt assume exists,
2. the stale scan summary still needs explicit reconciliation so reviewers do not mistake superseded failures for current truth,
3. the public-facing endgame still lacks one narrated end-to-end demonstration and one benchmark/forge-bench proof package.

This pack fixes the first two problems by restoring the control-plane documents and prompts, and it makes the third problem explicit and bounded instead of letting horizon work sprawl.

## Current finish bar

- Keep the 17-crate hardening support lane green.
- Restore the missing root docs and prompts so the repo root tells one truth again.
- Ship one v21 → v22 → v23 demonstrator.
- Ship one benchmark package proving replayable evidence-bound superiority.
- Finish the last physical root archive reduction.

## Not part of the finish bar

- reopening V10 kernel research,
- reopening V14/V15 causal or remote-exchange expansion,
- reopening V16-V20 wave work,
- inventing new owner crates or schema families just to make the demo look larger.

# P24 run order

## One-shot super-pass order

1. Read `P24_CODEX_RUN_PROMPT.md`.
2. Run Phase 00 exactly.
3. Run Phase 01 before changing product code. A broken verifier invalidates completion.
4. Run Phase 02 before adding new features. Ownership drift invalidates completion.
5. Run Phase 03. This is the execution-evidence spine.
6. Run Phase 04. This is the supported-local product slice.
7. Run Phase 05. This is the canonical memory/runtime proof.
8. Run Phase 06 only if queue/daemon work can be completed without weakening gates.
9. Run Phase 07 to harden boundary/repair/failure honesty.
10. Run Phase 08 to collapse docs/support profile.
11. Run Phase 09 to package and hand off.

## Stop rules

Stop and produce a partial but honest handoff if:

- canonical seam ownership cannot be proven;
- run-bundle V2 cannot be made schema-validated and replay-normalized;
- verifier hangs or cannot be bounded;
- cargo gates fail and cannot be fixed without large architecture churn;
- coding-agent lane cannot be made permit-gated;
- memory/runtime seam cannot preserve digest/backpointer truth.

Do not use a stop as a reason to produce no output. Produce a blocked handoff with exact failing command and next action.

## Work allocation rule

Priority order:

1. P0 issue matrix items.
2. Coding-agent supported-local lane.
3. Memory/runtime canonical seam proof.
4. Boundary/repair verification honesty.
5. Daemon-safe local queue if time remains.
6. UX/docs cleanup.

Never spend the pass on V10+ horizon features before P0 closure.

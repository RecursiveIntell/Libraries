# 16_HOSTILE_REVIEW_PROMPT

Review the patched repo like a skeptical technical auditor.

## Questions you must answer

1. Does `make gate` actually work from the front door now?
2. Does the root archive manifest pass, and do status surfaces stop lying about it?
3. Are the root canonical specs still stubs, or was that surface fixed honestly?
4. Do support profile, Makefile, receipt, and dashboard now describe the same release lane?
5. Does the runtime now surface precise degraded reasons instead of hiding them?
6. Has execution evidence actually converged across tool runtime / pilot / verification?
7. Are thin governance/runtime crates still misnamed?
8. Did the repo reduce panic surface honestly or just rename the script?
9. Did package hygiene improve without deleting necessary history?

## Review posture

Do not be impressed by policy prose.
Prefer passing proof commands and exact file surfaces.
If the repo still cannot testify, say so clearly.

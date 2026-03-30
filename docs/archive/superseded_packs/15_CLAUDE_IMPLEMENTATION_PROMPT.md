# 15_CLAUDE_IMPLEMENTATION_PROMPT

Finish the repo using the hostile finish pack.

## What matters most

This is not a feature pass.
It is a truth pass.

The repo already has strong architecture. The immediate goal is to make the package surface, canonical docs, support lane, and runtime provenance stop contradicting that architecture.

## Work sequence

- start with PACK-001 through GATE-001
- then land RUNTIME-001
- then converge execution-evidence and primitive drift
- then clean naming, docs, panic surface, and hotspot modules
- then remove package trash and polish the demo lane

## Behavioral constraints

- prefer small, exact patches over wide speculative rewrites
- keep changes local to the issue row being closed
- strengthen guards that are currently too weak
- preserve or improve round-trip / replay semantics
- do not make the taught story more complex than the canonical story

## Delivery shape

For each issue row you close:
1. say what you changed
2. name the files changed
3. show the proof command(s)
4. state any remaining risk honestly

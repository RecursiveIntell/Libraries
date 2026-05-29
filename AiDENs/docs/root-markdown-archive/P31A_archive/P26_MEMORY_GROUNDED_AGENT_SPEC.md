# P26 Memory-Grounded Agent Spec

## Goal

Make canonical memory seam evidence usable by local agents as grounding context.

## Required behavior

- AgentSpec can request memory grounding.
- The run invokes the existing memory seam lane or equivalent canonical path.
- RunBundleV3 records:
  - export envelope path/digest,
  - bridge import/report path/digest,
  - semantic-memory store evidence path/digest,
  - knowledge-runtime query result path/digest,
  - view/widening/degradation disclosure if available.

## Forbidden behavior

- No AiDENs-local memory database.
- No local redefinition of episode identity.
- No local promotion of memory claims.
- No silent widening.

## Tests

- agent with memory enabled gets grounding evidence;
- agent with memory disabled does not query memory;
- failed memory seam emits abstention/repair evidence;
- query result absence is not fake success.

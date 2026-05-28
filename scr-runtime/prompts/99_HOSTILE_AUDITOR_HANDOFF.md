# Hostile Auditor Handoff Prompt — SCR-P0A

Audit this SCR-P0A implementation hostilely.

Do not assume the implementation is correct because tests pass.

Primary questions:

1. Is this truly a proposed-action control evaluator, or did it degrade into object/risk scoring?
2. Are hard rules evaluated before scoring?
3. Can any score override a hard veto or minimum action floor?
4. Does evidence confidence erase hazard anywhere?
5. Are high-hazard low-confidence cases routed to verification?
6. Are integrity failures routed to quarantine?
7. Are weak authority/containment cases routed to approval/block?
8. Are durable scores integer/fixed-point only?
9. Are any f32/f64 values used in durable artifacts or decision math?
10. Are decisions receipt-bearing and replayable?
11. Do receipts include input hash, canonical policy hash, algorithm ID, rule results, axes, pressures, chosen action, rejected actions, reason codes, and time basis?
12. Are Rust types the schema source of truth?
13. Are policies canonicalized before hashing?
14. Can golden fixtures be silently updated?
15. Is there any FEUT/EEG/P=NP/Clay contamination in production code/policies/schemas?
16. Did the pass improperly integrate with Recall/AiDENs/memory/retrieval/tools?
17. Did Codex create duplicate ID/provenance/receipt semantics instead of adapter refs?
18. Are there hidden globals, caches, mutable singletons, or shadow-truth state?
19. Are LLM/network/model dependencies absent?
20. Are seeded violation checks real or performative?

Produce:
- blocker list
- high-risk list
- medium-risk list
- exact file references
- required fixes
- acceptance gate verdict
- whether SCR-P0A can proceed to P0B

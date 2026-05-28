# Architecture Finish Execution Map — V3

This map turns the issue matrix into an execution order for Claude. It is optimized for **architecture closure**, not for maximizing commit count.

## Prime directive
Finish the architecture by making the wrong behavior harder or impossible — do not reopen settled doctrine, do not invent a new layer, and do not hide remaining lag behind nicer comments.

## Non-goals
- Do **not** introduce a runtime shadow database.
- Do **not** make the bridge responsible for query/runtime policy.
- Do **not** join fresh raw Forge truth directly into runtime answers.
- Do **not** fix importer weakness by creating the wrong dependency direction.
- Do **not** add new compatibility surfaces.

## Phase 1 — importer and memory hardening (P0)
1. Tighten `semantic-memory` canonical import validation.
   - Close I006, I007, I008.
   - Fail fast on malformed canonical rows instead of default-filling semantics.
2. Fix relation truth-order invariants in storage.
   - Close I011; review I012 while touching indexes.
3. Keep the old import path frozen.
   - Do not expand `import_envelope()`; document and isolate it further if touched.

## Phase 2 — runtime closure (P0/P1)
1. Land the first real runtime execution upgrade.
   - Close I015 and I016 first.
2. Decide the fate of projection persistence/rebuild surfaces.
   - Close or explicitly narrow I017 and I018.
3. Wire the Forge-visible projection path into runtime planning once the basic execution model is less fake.
   - Advance I019.

## Phase 3 — control-flow compat burn-down (P1)
1. Burn down `agent-graph` legacy emission debt.
   - Close I022, I023, I024.
2. Burn down `job-queue` legacy event/context debt.
   - Close I026; review I027 and I028 while there.
3. Burn down `LLM-Pipeline` legacy trace surface.
   - Close I029 and I030.

## Phase 4 — bridge / forge seam cleanup (P1/P2)
1. Finish real supersession lineage in the bridge/export contract.
   - Close I003.
2. Clean up version vocabulary.
   - Close I002; keep I004 consistent if touched.
3. Add forge integration proof.
   - Close I033; clarify I034 if tests/docs require it.

## Phase 5 — low-risk compat debt and polish (P3)
Work through I005, I010, I014, I021, I025, I031, I032, I035 only after the hard closure work is done.

## Required output from Claude
- code changes
- tests proving the changed behavior
- short change log mapped to issue IDs
- explicit list of issues intentionally not closed in the pass

## Done when
- malformed canonical imports fail loudly instead of defaulting into plausible rows
- relation preferred-open invariants match full scope semantics
- runtime no longer relies on downgrade-first behavior for its most important target-state semantics
- control-flow crates are materially more canonical-first and materially less legacy-first
- forge/bridge/memory proof coverage exists for the architecture seams that matter
# Forge Standalone Extraction Plan

Date: 2026-07-04
Scope: Prepare the Forge crate family (12 crates) for standalone repo extraction and crates.io publication.

## Target repo

`recursiveintell/forge` — positioned as "Forge: causal edit attribution and receipt-backed patch verification."

Lead line: "Know which agent edit caused which check result."

## Crate family (12 crates)

### Phase 1: Foundation primitives (publish first, no internal forge deps)

1. **typed-patch** — structured patch schema + validation/apply helpers
   - Path: `Primitives/typed-patch`
   - Deps: forge-policy, sandbox-workspace
   - LOC: 987, 6 tests
   - README needed: example patch object, rendered diff, non-goals

2. **effect-signature** — stable effect payloads + hashing helpers
   - Path: `Primitives/effect-signature`
   - Deps: none (external only)
   - LOC: 132, 5 tests
   - README needed: one hash example, stable comparison explanation

3. **forge-policy** — filesystem/env/patch-cap guardrails
   - Path: `Primitives/forge-policy`
   - Deps: none
   - LOC: 445, 7 tests
   - README needed: policy examples (forbidden paths, allowed env, DB identity)

4. **stabilizer-core** — attempt-phase/delta-policy primitives
   - Path: `Primitives/stabilizer-core`
   - Deps: typed-patch
   - LOC: 487, 6 tests
   - README needed: innovate->stabilize->clamp example

5. **mindstate-core** — serializable mindstate payload types
   - Path: `Primitives/mindstate-core`
   - Deps: none
   - LOC: 285, 7 tests
   - README needed: rendered mindstate, hash/signature example

6. **cea-core** — causal edit attribution core
   - Path: `Primitives/cea-core`
   - Deps: check-runner, typed-patch
   - LOC: 2444, 11 tests
   - README needed: patch->check->effect->graph->prediction diagram, expand acronym

7. **cea-store** — storage contract + row types for CEA graphs
   - Path: `Primitives/cea-store`
   - Deps: cea-core, check-runner
   - LOC: 657, 5 tests
   - README needed: adapter-boundary diagram

8. **cea-sqlite** — SQLite persistence adapter for CEA graphs
   - Path: `Primitives/cea-sqlite`
   - Deps: cea-core, cea-store, forge-policy
   - LOC: 1219, 10 tests
   - README needed: schema sketch, persistence/replay example

9. **check-runner** — normalized check/command execution
   - Path: `Primitives/check-runner`
   - Deps: effect-signature, check-runner-sys
   - LOC: 857, 12 tests
   - README needed: minimal example, safety model, check-runner-sys relationship

10. **sandbox-workspace** — safe workspace staging + patch filesystem helpers
    - Path: `Primitives/sandbox-workspace`
    - Deps: forge-policy
    - LOC: 386, 11 tests
    - README needed: temp workspace/staged patch example, file-access boundary diagram

### Phase 2: Main engine

11. **forge-engine** — operational verification/evaluation engine
    - Path: `living-memory/living-memory`
    - Deps: cea-core, cea-sqlite, cea-store, check-runner, effect-signature, forge-policy, mindstate-core, sandbox-workspace, claim-ledger, llm-tool-runtime, semantic-memory, stack-ids
    - LOC: 16099, 181 tests
    - README needed: architecture diagram, minimal patch verification example, receipts emitted, relation to forge-pilot, non-goals

### Phase 3: Orchestrator

12. **forge-pilot** — OODA governance orchestrator
    - Path: `forge-pilot`
    - Deps: forge-engine + many governance crates
    - LOC: 14447, 78 tests
    - README needed: OODA diagram, complete toy run

## Publication order

```
Phase 1 (no forge internal deps, publish first):
  effect-signature → forge-policy → sandbox-workspace → typed-patch
  → stabilizer-core → mindstate-core → check-runner
  → cea-core → cea-store → cea-sqlite

Phase 2 (depends on Phase 1):
  forge-engine

Phase 3 (depends on Phase 2):
  forge-pilot
```

## check-runner-sys

Path: `Primitives/check-runner-sys`
LOC: 42, 0 tests
Decision: HOLD — publish only as dependency of check-runner if needed. Unsafe syscall wrappers, not user-facing.

## README requirements per crate

Every public README must have:
1. One-sentence use case
2. "What this crate owns."
3. "What this crate explicitly does not own."
4. Minimal runnable example
5. Architecture or flow diagram (mermaid or ASCII)
6. Integration map to adjacent RecursiveIntell crates
7. Claim boundary: what is tested, what is experimental, what is not claimed

Current gap: All 12 crates lack obvious README visual/diagram signals.

## Dry-run packaging checklist

Before publishing each crate:
- [ ] `cargo publish --dry-run -p <crate>` passes
- [ ] All transitive deps are already on crates.io or published in same batch
- [ ] README.md exists and has all 7 sections above
- [ ] LICENSE file present
- [ ] Crate description in Cargo.toml is clear (not abstract doctrine names)
- [ ] Keywords and categories set in Cargo.toml
- [ ] No absolute user paths in code or tests

## Public positioning

- Repo name: `recursiveintell/forge`
- Tagline: "Know which agent edit caused which check result."
- Description: "Causal edit attribution and receipt-backed patch verification for local-first AI agents."
- NOT: "agent framework", "doctrine runtime", "governance engine"
- Categories: development-tools, testing
- Keywords: forge, cea, patch-verification, causal-edit-attribution, agent-safety

## Standalone extraction steps

1. Create `recursiveintell/forge` repo on GitHub
2. Copy crate directories preserving structure
3. Set up workspace Cargo.toml with all 12 crates as members
4. Fix path dependencies → crates.io version dependencies where possible
5. Run `cargo check --workspace` and `cargo test --workspace`
6. Upgrade READMEs (Phase 1 crates first)
7. Dry-run publish in dependency order
8. Publish to crates.io
9. Wire forge-workbench to use crates.io deps instead of path deps
10. Update public site with Forge project page

## Claim boundary

- Forge is "causal edit attribution and receipt-backed patch verification"
- It proves command execution and check results, NOT total correctness
- It is local-first and deterministic
- It does not require network or cloud services
- CEA predictions are statistical estimates, not guarantees
# TurboQuant × Semantic-Memory Super-Pass Bundle

**Purpose:** execution-ready Codex super-pass bundle to integrate `turbo-quant` with `semantic-memory` safely.

**Core judgment:** do this. It is high-ROI if and only if the pass is staged as:

1. harden `turbo-quant` codec contracts and compact encoding;
2. add `semantic-memory` codec abstraction and optional `turbo-quant` integration;
3. run turbo-quant in shadow mode first;
4. persist codec profiles, encode/search/evaluation receipts;
5. expose approximation/degradation in query results;
6. only promote turbo-quant-backed search after measured gates pass.

**Do not** make turbo-quant the default semantic-memory vector path in this pass.

## Required repo/source layout

Preferred layout before running Codex:

```text
/home/sikmindz/Coding/Libraries/
  Cargo.toml
  semantic-memory/
  stack-ids/
  forge-memory-bridge/
  semantic-memory-forge/
  turbo-quant/                 # preferred sibling location
```

Current source basis observed:

```text
turbo-quant:     /home/sikmindz/Documents/turbo-quant
semantic-memory: /home/sikmindz/Coding/Libraries/semantic-memory
workspace root:  /home/sikmindz/Coding/Libraries
```

If `turbo-quant` is not available as a sibling of `semantic-memory`, Codex must either:
- stop with a clear precondition failure, or
- if explicitly operating on both repositories, modify both repos but must not add brittle absolute Cargo paths.

## Bundle contents

- `AGENTS.md` — hard doctrine for this run.
- `MAIN_CODEX_PROMPT.md` — paste this as the main Codex run prompt.
- `PHASE_PROMPTS/` — phase-by-phase execution prompts.
- `MANUAL_INJECTIONS/` — human phase-injection prompts to paste between phases.
- `SPEC/` — target integration spec, artifact contracts, DB/API plans.
- `MATRICES/` — issue matrix, ownership map, acceptance gates, source-of-truth matrix.
- `SCRIPTS/` — assertion scripts to copy into `scripts/` or run from the repo root.
- `AUDIT/` — hostile auditor checklist and final report template.
- `PROMPTS/` — standalone auditor prompt for another model.

## Recommended execution

1. Put this bundle beside the repositories or paste `MAIN_CODEX_PROMPT.md` into Codex.
2. Paste `MANUAL_INJECTIONS/INJECTION_00_PRESTART.md` before implementation begins.
3. Paste phase injections after Phase 0, Phase 2, Phase 4, Phase 6, and before final report.
4. Require Codex to emit a changed-file summary, test output, and unresolved-risk list at every phase boundary.

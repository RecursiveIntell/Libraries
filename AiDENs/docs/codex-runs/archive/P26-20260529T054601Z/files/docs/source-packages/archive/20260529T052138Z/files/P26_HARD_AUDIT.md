# P26 Hard Audit — Starting Assessment

## Verdict

AiDENs is now clean enough to advance capability. P25 did meaningful release/control work, but P26 must further actual AiDENs behavior rather than continue packaging cleanup.

## What is strong now

- Current run is P25 and stale active artifacts are zero.
- Package validation is strict and clean.
- P25 phase gates exist and are auditable.
- Supported-local flagship coding-agent demo exists.
- Support profile is honest about local/fixture/deferred surfaces.
- z.py has bounded root Markdown archive reporting and stayed out of runtime semantics.

## What is still blocking final-vision progress

### H-AUDIT-1: reusable advanced agent abstraction is missing

Current commands show useful pieces (`run-test-agent`, `run-coding-agent`, `inspect-run`, `memory seam-fixture`, coding tools, permits, receipts), but not a single reusable advanced agent contract tying these together.

P26 must define and implement `AgentSpecV1` as the user-facing unit of advanced local agent construction.

### H-AUDIT-2: the plan/act/verify loop is not yet the framework center

P25 proved fixture paths. P26 must promote a bounded local Plan → Act → Verify loop that can run from an `AgentSpecV1`, with explicit evidence and stop rules.

### H-AUDIT-3: memory grounding exists as fixture, not agent primitive

`memory seam-fixture` exists. P26 must make memory grounding part of the agent loop without creating AiDENs-local memory truth.

### H-AUDIT-4: coding agent remains too fixture-shaped

The flagship demo exists, but P26 must make supported-local coding-agent behavior reusable across sandbox roots.

### H-AUDIT-5: package self-replay failure is unresolved

P25 final audit reported extracted package self-replay cargo-gate failure. P26 must either fix it or emit a precise environment/manifest cause with a reproducible failure artifact. Silent acceptance is forbidden.

### H-AUDIT-6: large files are a maintainability cliff

Large files remain, but P26 should not derail into broad refactor. Split only where required to make capability clean. Otherwise leave a stronger containment backlog.

## No-go zones

- Do not implement V10 runtime geometry.
- Do not build cloud provider loops.
- Do not build broad autonomous daemon behavior.
- Do not add canonical memory/evidence/verification semantics inside AiDENs.
- Do not turn `z.py` into a product feature.

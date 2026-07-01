# Graph Edge Admission Policy

Status: active policy for semantic-memory graph edges.
Date: 2026-06-29

## Purpose

Graph edges are discovery scaffolding. They are not proof.

A graph edge may help recall adjacent context, but it must not make a fact authoritative. Final answers still require direct fact provenance, source receipts, or live verification for current-state claims.

## Authority rule

Allowed:
- Curated `Entity`, `Causal`, `Temporal`, and `Semantic` edges when backed by explicit source facts or code/test receipts.
- Auto-generated `shared_terms:*` edges only when they pass the conservative admission policy below.

Blocked:
- Model-generated/autonomous observations writing graph edges directly.
- Social/personal/general/test/tool-receipt facts participating in auto-edge generation.
- Session bundles, phase packs, prompt templates, code blocks, and long omnibus facts participating in auto-edge generation.
- Graph-expanded recall being rendered as evidence. It must be labelled “related via graph, not proof.”

## Auto-edge admission policy

Default denylisted namespaces:

```text
mixed
chatgpt
twitter
tool-receipts
agentguard
autonomous
personal
behavioral
preferences
general
test
```

Default skipped artifact markers:

```text
.zip]
_bundle
_pack
PHASE_
CODEX_DELEGATE
CONTEXT_PACK
ACCEPTANCE_GATES
PROOF_PACKET
```

Default numeric gates:

```text
min_shared_terms = 4
min_jaccard = 0.08
max_fact_chars = 1200
max_edges_per_fact = 12
```

## Required metadata for generated edges

Each generated edge should preserve or encode:

- generator name/version
- relation family, e.g. `shared_terms`
- shared term count
- Jaccard score
- source namespace
- target namespace
- policy thresholds used
- content digests where available

Current compatibility note: the active DB stores typed graph edge payloads in `graph_edges.edge_type` as JSON. Audit tooling must parse that field, not assume a separate `relation` column exists.

## Rebuild policy

Rebuilds must:

1. Run dry-run first.
2. Report proposed creates, skips, namespace distribution, and hub risks.
3. Abort if proposed edge creation exceeds budget.
4. Invalidate only auto-generated `shared_terms:*` edges from the same generator family.
5. Preserve curated edges such as `supersedes`, `belongs_to`, `depends_on`, verified causal edges, and temporal edges unless a dedicated audit invalidates them.

## Edge-count budget

Maintenance must warn or abort when:

- active edge count jumps by more than 25%, or
- active edge count jumps by more than 500 edges, or
- auto-edge dry-run proposes more than 500 new edges, or
- any skipped namespace appears in generated candidates.

## Regression lessons

Prior failure:

- graph ballooned to 42,128 active edges
- low-precision `shared_terms` edges formed hubs and hurt recall
- autonomous/model-written `fills_gap` edges asserted unverified relations
- cleanup invalidated 41,852 noisy `shared_terms` edges and 216 autonomous `fills_gap` edges
- graph was intentionally rebuilt sparse: roughly 590 active edges, 530 shared_terms

Policy consequence:

Precision beats coverage. A small graph with trusted edges is better than a dense graph that teaches the agent false associations.

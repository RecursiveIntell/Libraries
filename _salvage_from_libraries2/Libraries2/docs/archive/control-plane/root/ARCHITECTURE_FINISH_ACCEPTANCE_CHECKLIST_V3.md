# Architecture Finish Acceptance Checklist — V3

Use this as the pass-exit checklist. A pass that only improves prose or compatibility labeling without moving these checks is not good enough.

## Import / memory
- [ ] Canonical import rejects malformed required fields instead of default-filling them.
- [ ] Alias / evidence / episode timestamp semantics are explicit and tested.
- [ ] Relation `preferred_open` uniqueness matches the **full** logical scope key.
- [ ] Legacy `import_envelope()` path was not expanded and is still clearly compat-only.

## Bridge / forge
- [ ] Bridge/export contract can preserve real superseded claim-version lineage, or the limitation is made explicit with no fake values.
- [ ] Import/export version vocabulary is unambiguous in code and docs.
- [ ] Forge crate has at least one cross-crate proof test through bridge + memory.

## Runtime
- [ ] Temporal behavior is upgraded from downgrade-first best-effort or explicitly rejected under strict semantics.
- [ ] Scope enforcement is stronger than namespace-only pushdown plus warnings.
- [ ] Projection persistence/rebuild surfaces are either real or explicitly narrowed.
- [ ] Runtime still does not become authoritative for source truth or identity truth.

## Control-flow / lineage
- [ ] `agent-graph` events are materially more canonical-first; legacy fields are no longer the practical normal path.
- [ ] Legacy attempt counters do not lie about retries where they still exist.
- [ ] `job-queue` public surfaces are less legacy-first and more canonical-first.
- [ ] `LLM-Pipeline` normal-path tracing is canonical-first.

## Proof
- [ ] Added/updated tests map to closed issue IDs.
- [ ] Claude reports exactly which issue IDs were closed, advanced, or left open.
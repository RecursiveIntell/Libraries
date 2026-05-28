# Architecture Closure Acceptance Checklist — V4

Use this after the Claude pass.

## Import / bridge
- [ ] Bridge version fields are documented unambiguously.
- [ ] Bridge still refuses incompatible export versions.
- [ ] No synthetic superseded claim-version IDs are minted.
- [ ] Non-core import defaults are explicitly reviewed and either retained as V1 canonical defaults or converted to hard-fail requirements.
- [ ] Entity alias review_state / merge_decision writes are constrained or validated strongly enough to prevent junk values.
- [ ] Canonical import proof tests cover strict validation, idempotency, derivation edges, and queryable post-import state.

## Runtime
- [ ] Temporal mode is either genuinely implemented over imported temporal metadata or explicitly non-supporting without pretending to work.
- [ ] Scope beyond namespace never widens silently.
- [ ] Rebuild execution semantics are no longer tracker-only ambiguity.
- [ ] Cross-crate proof tests cover query_with_trace continuity, strict scope behavior, and temporal downgrade/non-downgrade rules.

## Retry / trace / compat debt
- [ ] Agent-graph canonical event/retry fields are the dominant path.
- [ ] Job-queue examples/docs are TraceCtx-first and canonical lineage is authoritative when present.
- [ ] LLM-Pipeline and Tauri-Queue remain compat-safe but do not advertise legacy trace as the normal path.
- [ ] No new code introduces additional legacy trace/attempt wrappers.

## End-to-end proof
- [ ] Forge evidence/export metadata survives bridge->memory import in a tested path.
- [ ] Evidence remains opaque by default in runtime/query surfaces.
- [ ] One dedicated architecture-closure proof suite exists and passes.

## Docs
- [ ] Root docs and touched crate docs match the actual code.
- [ ] Deferred features are named honestly.
- [ ] Compat surfaces are labeled as compat-only, not normative.

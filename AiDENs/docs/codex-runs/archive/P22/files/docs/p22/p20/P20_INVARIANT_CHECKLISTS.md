# P20 Invariant Checklists

Use this checklist before and after every phase.

## Provenance-first design

- [ ] All meaningful state has provenance or is clearly ephemeral.
- [ ] Transformations preserve backpointers.
- [ ] Generated reports cite code/test evidence where possible.

## No shadow truth

- [ ] AiDENs does not own memory truth.
- [ ] AiDENs does not own evidence truth.
- [ ] AiDENs does not own kernel/witness/syndrome truth.
- [ ] AiDENs does not own verification/control truth.
- [ ] AiDENs does not own repair/contradiction law.

## Contract-first boundaries

- [ ] Canonical Rust types/re-exports used where available.
- [ ] Local DTOs are labeled as orchestration/report/display only.
- [ ] JSON repair is receipt-bearing and bounded.
- [ ] No lenient parser silently widens semantics.

## Bitemporal integrity

- [ ] Valid time and recorded/transaction time are not collapsed.
- [ ] Any missing bitemporal behavior is documented as partial/deferred.
- [ ] As-of semantics are tested or demoted.

## Execution as evidence

- [ ] Provider route recorded.
- [ ] Tool calls recorded.
- [ ] Retries/fallbacks/degradations recorded.
- [ ] Budget/deadline state recorded or explicitly unavailable.

## Graph separation

- [ ] Storage, retrieval, inference, repair, and control/receipt graphs are not conflated.
- [ ] Any local graph/report in AiDENs is non-authoritative.

## Agency/influence

- [ ] High-impact recommendations pass through agency gate.
- [ ] Memory-personalized advice records memory influence trace.
- [ ] Repeated nudges are counted and gated.
- [ ] Tool-origin urgency/persuasion is classified.
- [ ] Receipts are emitted.

## Documentation honesty

- [ ] No unsupported feature is described as supported.
- [ ] Scaffold and deferred status are explicit.
- [ ] Known limitations are visible in final handoff.

## Lawful subtraction/quarantine

- [ ] Removed compatibility/shadow surfaces have removal rationale.
- [ ] Quarantined code has reason, owner, and next action.
- [ ] Audit history is preserved.

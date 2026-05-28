# SCR-P0A Specification

## One-line objective

Build a deterministic, receipt-bearing control-policy evaluator that evaluates **proposed actions** using explicit authority, evidence, policy, and time basis.

## Core input

```text
ControlEvaluationInputV1 {
  input_id
  actor_ref
  permit_ref
  subject_ref
  domain
  proposed_action
  requested_effect
  evidence_refs
  environment_ref
  valid_time_basis
  recorded_time
  schema_version
}
```

## Core axes

```text
Hazard              how bad if action/claim is wrong or unsafe
EvidenceConfidence  how strongly evidence supports the hazard/claim
Uncertainty         how much is missing/ambiguous/conflicting/stale
Authority           whether actor is allowed to do the thing
Containment         rollback/sandbox/scope/kill-switch quality
IntegrityRisk       source-of-truth/schema/provenance boundary risk
```

## Derived pressures

```text
AutonomyPressure     restricts autonomous action when hazard is high and authority/containment are weak
VerificationPressure routes high-hazard uncertain cases to verification
RepairPriority       routes high-hazard high-confidence fixable cases to repair
QuarantinePressure   routes integrity/schema/contamination failures to quarantine
```

## Decision resolver precedence

```text
1. schema invalidity
2. authority/permit failure
3. hard veto
4. quarantine rule
5. minimum action floor
6. score-derived action
7. operator override if permitted
8. post-decision invariant validation
```

## Critical semantic rules

- Evidence confidence does not erase hazard.
- Low confidence redirects high hazard to verification.
- Integrity failure routes to quarantine.
- Weak authority/containment restricts autonomy.
- Hard rules outrank scores.
- SCR emits decisions and receipts, not factual truth.

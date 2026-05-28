# Phase 3 — Pressure Derivation

Implement deterministic integer pressure derivation.

Axes:

```text
hazard
evidence_confidence
uncertainty
authority
containment
integrity_risk
```

Derived pressures:

```text
autonomy_pressure
verification_pressure
repair_priority
quarantine_pressure
```

## Required semantics

Evidence confidence must not erase hazard.

Low confidence redirects high hazard toward verification.

Integrity risk routes toward quarantine.

Weak authority/containment routes toward approval/block.

Score math is deterministic and integer-only.

## Required behavior

```text
High hazard + low confidence    -> RequireVerification
High hazard + high confidence + fixable -> GenerateRepairPacket
Invalid integrity boundary      -> QuarantineArtifact
Weak authority                  -> RequireApproval or BlockMutation
Missing rollback destructive    -> BlockRelease
```

## Property tests

Add property tests or table tests proving:

- increasing hazard cannot reduce verification pressure
- increasing integrity risk cannot reduce quarantine pressure
- increasing containment cannot increase autonomy pressure, all else equal
- hard veto always outranks pressure
- same input + same policy + same algorithm = same pressures

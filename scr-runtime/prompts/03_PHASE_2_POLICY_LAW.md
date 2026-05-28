# Phase 2 — Policy and Hard-Rule Evaluator

Implement:

```text
policy parser
canonical policy normalization
canonical policy JSON generation
policy hash
hard-rule evaluator
minimum-action floors
action precedence resolver
```

Create:

```text
policies/audit_policy_v1.toml
policies/audit_policy_v1.canonical.json
docs/POLICY_MODEL.md
docs/ACTION_RESOLUTION.md
```

## Required rules

Hard rules execute before scoring.

Minimum-action floors cannot be weakened by score thresholds.

Policy conflict resolution is deterministic.

Receipts record:

- policy hash
- hard rules checked
- hard rules triggered
- minimum action floors applied
- rejected actions and reasons

## Required hard rules

```text
HR-SCHEMA-INVALID              -> QuarantineArtifact
HR-AUTHORITY-MISSING           -> RequireApproval or BlockMutation
HR-FEUT-CONTAMINATION          -> QuarantineArtifact
HR-UNKNOWN-OWNER-MUTATION      -> RequireOwnerResolution or BlockMutation
HR-SOURCE-TRUTH-DRIFT          -> minimum RequireVerification
HR-FALSE-COMPLETION-MISSING-TESTS -> GenerateRepairPacket
HR-DESTRUCTIVE-MISSING-ROLLBACK -> BlockRelease
HR-DURABLE-FLOAT-SCORE         -> fail hostile script
HR-NAKED-DECISION-BOOL         -> fail hostile script
```

## Tests

- hard veto precedes score-derived action
- minimum floor cannot be downgraded
- changing policy changes canonical policy hash
- same policy source canonicalizes deterministically

# Policy Model

Human policy source is TOML:

```text
policies/audit_policy_v1.toml
```

The canonical policy artifact is generated as:

```text
TOML -> parsed PolicyModelV1 -> normalized canonical JSON -> BLAKE3 hash
```

The checked-in canonical artifact is:

```text
policies/audit_policy_v1.canonical.json
```

Receipts store the canonical policy hash, not a raw TOML hash.

Policy sections:

- `policy`: identity, version, algorithm version, canonicalization mode.
- `action_precedence`: deterministic action strength ordering.
- `thresholds`: integer pressure thresholds for score-derived actions.
- `minimum_actions`: named floors that cannot be weakened by scores.
- `hard_rules`: enabled rules with hard actions or minimum actions.

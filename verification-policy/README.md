# verification-policy

Canonical verification policy, approval, and execution-permit artifacts.

## Usage

```sh
cargo add verification-policy
```

```rust
use verification_policy::{
    PolicyDecision, ExecutionPermit, CommitToken, V25CitationContext,
};
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (policy, approval, and profile IDs)
- `llm-tool-runtime` -- tool receipt types
- `verification-control` -- check plans, promotion class, and case types

**Depended on by:**
- `verification-adjudication`
- `profile-runtime`
- `contract-schema-gen`
- `forge-pilot`
- `kernel-conformance`

## stack-ids integration

Policy and profile IDs (`PolicyDecisionId`, `ApprovalRecordId`,
`EffectPolicyProfileId`, `DelegationPolicyProfileId`, etc.) are all sourced
from `stack-ids`. The `V25CitationContext` type is re-exported from
`verification-control`.

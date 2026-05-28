# Per-crate closeout plan

## New owners

### `effect-runtime`
Land the v21 effect families and one bounded preflight -> commit -> receipt -> observation -> compensation slice.

### `authority-delegation`
Land the v22 capability, lease, delegation, SoD, break-glass, and acting-on-behalf artifacts.

### `assurance-runtime`
Land the v23 deployment profile, assurance case, hazard/control mapping, release decision, monitoring, and certification artifacts.

### `continuity-runtime`
Land the v24 SLO, error budget, incident, containment, recovery, replay, exception, and postmortem artifacts.

## Existing canonical lane touches

### `stack-ids`
Add the missing strong ID wrappers once the new owner crates are present.

### `contract-schema-gen`
Register and publish every new v21–v24 schema family in one pass.

### `semantic-memory`
Add additive storage tables or preservation lanes for the new artifact families.

### `knowledge-runtime`
Expose bounded read/query surfaces only; remain consumer-only with respect to truth.

### `verification-control`
Add case/decision objects for effect gates, delegation review, release readiness, and continuity review.

### `verification-policy`
Add policy profiles for effect law, delegation law, deployability law, and continuity law.

### `verification-adjudication`
Add effect and release adjudication receipts; do not let runtime behavior outrun typed receipts.

### `llm-tool-runtime`
Surface typed tool-dispatch receipt data that can flow into v21 effect receipts.

## Do not touch unless needed
- `federated-settlement`
- `mechanism-runtime`
- `discovery-portfolio`
- `constitutional-memory`
- `spec-execution`

Those crates remain the already-frozen preconditions, not the main target of this final pass.

# Crate Boundary and Owner Map — V27

## Dependency Graph (simplified)

```
stack-ids ─────────────────────────────────────────────────────────────┐
  │                                                                     │
recursive-kernel-core ──────────────────────┐                          │
  │                                          │                          │
semantic-memory-forge ─────────┐             │                          │
  │                             │             │                          │
forge-memory-bridge ──┐        │             │                          │
  │                    │        │             │                          │
semantic-memory ──────┤        │             │                          │
  │                    │        │             │                          │
llm-tool-runtime ─────┤        │             │                          │
  │                    │        │             │                          │
constraint-compiler ──┤────────┤─────────────┤                          │
  │                    │        │             │                          │
kernel-execution ─────┤        │             │                          │
  │                    │        │             │                          │
kernel-oracles ───────┤        │             │                          │
  │                    │        │             │                          │
knowledge-runtime ────┤        │             │                          │
  │                    │        │             │                          │
verification-control ─┤────────┤─────────────┤                          │
  │                    │        │             │                          │
verification-policy ──┤        │             │                          │
  │                    │        │             │                          │
verification-calibration ──────┤             │                          │
  │                             │             │                          │
verification-adjudication ─────┤             │                          │
  │                             │             │                          │
living-memory (forge-engine) ──┤             │                          │
  │                             │             │                          │
  ├─── GOVERNANCE CRATES ───────┤─────────────┤──────────────────────────┤
  │    effect-runtime           │             │                          │
  │    assurance-runtime        │ (no deps    │ (stack-ids only)         │
  │    authority-delegation     │  currently) │                          │
  │    continuity-runtime       │             │                          │
  │    attestation-exchange ────┤             │                          │
  │    constitutional-memory ───┤             │                          │
  │    mechanism-runtime ───────┤             │                          │
  │    profile-runtime ─────────┤             │                          │
  │                             │             │                          │
forge-pilot ═══════════════════╧═════════════╧══════════════════════════╧
  │
kernel-conformance (integration tests)
  │
contract-schema-gen (schema output)
```

## Authority Boundaries

| Crate | Owns | Does NOT Own |
|-------|------|-------------|
| stack-ids | Opaque ID newtypes, scope, trace, content digest | What IDs point to |
| semantic-memory | Durable storage, search, graph views | Export format, verification |
| semantic-memory-forge | Export/verification wire format | Storage, query |
| forge-memory-bridge | Translation between forge ↔ memory | Truth on either side |
| knowledge-runtime | Query pipeline, entity resolution, projection lifecycle | Durable records |
| llm-tool-runtime | Tool contracts, registry, dispatch | Tool implementations |
| recursive-kernel-core | Operator contracts, kernel artifact types | Execution logic |
| constraint-compiler | Hypergraph compilation, invalidation cones | Execution, storage |
| kernel-execution | Execution modes (acyclic, message-passing, delta, residual) | Compilation, policy |
| kernel-oracles | Oracle slice evaluation, refutation | Execution modes |
| verification-control | Case/plan/attempt/receipt lifecycle, replay, scheduling | Policy, calibration |
| verification-policy | Policy evaluation, execution permits | Case lifecycle |
| verification-calibration | Evidence trustworthiness, abstention | Policy, adjudication |
| verification-adjudication | Terminal disposition, rollback planning | Case lifecycle, policy |
| living-memory (forge-engine) | CEA, mindstate, patches, checks, scoring, experiments | Query, export format |
| forge-pilot | OODA loop orchestration, bootstrap, CLI | Verification logic, storage |
| **effect-runtime** | **Effect lifecycle: window → intent → preflight → commit** | Execution, verification |
| **assurance-runtime** | **Deployability profiles, certification artifacts** | Policy, delegation |
| **authority-delegation** | **Capability grants, emergency override, separation of duties** | Authentication |
| **continuity-runtime** | **Incident cases, SLO tracking, recovery replay** | Observation sources |
| **attestation-exchange** | **Vendor trust roots, exchange contracts** | Trust establishment |
| **constitutional-memory** | **Charter amendments, archive compaction** | Policy composition |
| **mechanism-runtime** | **Theory fitness, refuter suites, stability reports** | Execution |
| **profile-runtime** | **Fold-class composition, effective constitutions, policy impact diff** | Profile content |

## Integration Points After GOV-001

| Governance Crate | forge-pilot Phase | Check Type |
|-----------------|-------------------|------------|
| effect-runtime | act (pre-execution) | EffectPreflightReport disposition gates execution |
| assurance-runtime | observe (readiness) | AssuranceCaseV1 completeness informs promotion eligibility |
| authority-delegation | act (pre-execution) | CapabilityGrantV1 validates agent has required authority |
| continuity-runtime | observe (disposition) | IncidentCaseV1 active state triggers incident halt |
| attestation-exchange | observe (trust) | Vendor attestation validity informs evidence admissibility |
| constitutional-memory | observe (amendment) | Pending amendments flag advisory-only mode |
| mechanism-runtime | orient (fitness) | FitRunV1 disposition informs target scoring |
| profile-runtime | act (pre-execution) | EffectiveConstitutionV1 mode gates execution |

# Profile suite overview

| Profile | Title | Primary owner | New families | Fixture bundles |
|---|---|---|---:|---:|
| P1 | Privacy, Retention, Disclosure, and Redaction | `verification-policy` | 4 | 2 |
| P2 | Locality, Tenancy, Residency, and Boundary Overlay | `verification-policy` | 4 | 2 |
| P3 | Role Catalog, Duty Segregation, and Approval Matrix | `authority-delegation` | 4 | 2 |
| P4 | Regulated Deployment, Control Mapping, and Recertification | `assurance-runtime` | 4 | 2 |
| P5 | Sector Hazard Library, Monitor Catalog, and Mitigation Playbook | `assurance-runtime` | 4 | 2 |
| P6 | Vendor Certification Adapter and External Evidence Translation | `attestation-exchange` | 4 | 2 |
| P7 | Incident Taxonomy, Escalation, and Pager Routing Profile | `continuity-runtime` | 4 | 2 |

## Sequence

1. P1 and P2 first because privacy, retention, locality, tenancy, and boundary semantics constrain
   what the later profiles are even allowed to publish.
2. P3 next because approval and segregation rules need to be fixed before regime-specific release bars
   or incident routing can be trusted.
3. P4 and P5 next because regulated assurance and hazard doctrine consume the earlier policy profiles.
4. P6 after that because vendor adapters must translate into already-defined assurance, trust, and disclosure surfaces.
5. P7 last because incident taxonomy and pager routing should be bound to the final hazard and approval story.

## No-new-crate rule

This pack assumes **no new workspace members**.
All profile families land through already-present owners.

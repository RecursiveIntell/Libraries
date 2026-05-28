# Schema registry and compatibility plan — post-v24 profiles

Every profile-layer family below needs one owner, one schema, one example, and one manifest entry.

## P1 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| PrivacyRetentionProfileV1 | verification-policy | privacy-retention-profile-v1.schema.json | privacy-retention-profile-v1.example.json | required |
| RedactionRuleSetV1 | verification-policy | redaction-rule-set-v1.schema.json | redaction-rule-set-v1.example.json | required |
| AccessPurposeMatrixV1 | verification-policy | access-purpose-matrix-v1.schema.json | access-purpose-matrix-v1.example.json | required |
| AuditExtractionPolicyV1 | verification-policy | audit-extraction-policy-v1.schema.json | audit-extraction-policy-v1.example.json | required |

## P2 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| ResidencyPolicyProfileV1 | verification-policy | residency-policy-profile-v1.schema.json | residency-policy-profile-v1.example.json | required |
| TenantBoundaryProfileV1 | verification-policy | tenant-boundary-profile-v1.schema.json | tenant-boundary-profile-v1.example.json | required |
| CrossBoundaryTransferClassV1 | verification-policy | cross-boundary-transfer-class-v1.schema.json | cross-boundary-transfer-class-v1.example.json | required |
| LocalityExceptionV1 | verification-policy | locality-exception-v1.schema.json | locality-exception-v1.example.json | required |

## P3 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| RoleCatalogV1 | authority-delegation | role-catalog-v1.schema.json | role-catalog-v1.example.json | required |
| DelegationMatrixV1 | authority-delegation | delegation-matrix-v1.schema.json | delegation-matrix-v1.example.json | required |
| ApprovalMatrixV1 | authority-delegation | approval-matrix-v1.schema.json | approval-matrix-v1.example.json | required |
| ConflictClassCatalogV1 | authority-delegation | conflict-class-catalog-v1.schema.json | conflict-class-catalog-v1.example.json | required |

## P4 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| RegulatoryRegimeProfileV1 | assurance-runtime | regulatory-regime-profile-v1.schema.json | regulatory-regime-profile-v1.example.json | required |
| RequirementControlMapV1 | assurance-runtime | requirement-control-map-v1.schema.json | requirement-control-map-v1.example.json | required |
| EvidenceCollectionPlanV1 | assurance-runtime | evidence-collection-plan-v1.schema.json | evidence-collection-plan-v1.example.json | required |
| RecertificationScheduleV1 | assurance-runtime | recertification-schedule-v1.schema.json | recertification-schedule-v1.example.json | required |

## P5 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| HazardLibraryV1 | assurance-runtime | hazard-library-v1.schema.json | hazard-library-v1.example.json | required |
| HazardScenarioV1 | assurance-runtime | hazard-scenario-v1.schema.json | hazard-scenario-v1.example.json | required |
| MonitorCatalogV1 | assurance-runtime | monitor-catalog-v1.schema.json | monitor-catalog-v1.example.json | required |
| MitigationPlaybookV1 | assurance-runtime | mitigation-playbook-v1.schema.json | mitigation-playbook-v1.example.json | required |

## P6 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| VendorCertificationAdapterV1 | attestation-exchange | vendor-certification-adapter-v1.schema.json | vendor-certification-adapter-v1.example.json | required |
| VendorEvidenceTranslationV1 | attestation-exchange | vendor-evidence-translation-v1.schema.json | vendor-evidence-translation-v1.example.json | required |
| VendorTrustRootBindingV1 | attestation-exchange | vendor-trust-root-binding-v1.schema.json | vendor-trust-root-binding-v1.example.json | required |
| VendorRevocationHandlingV1 | attestation-exchange | vendor-revocation-handling-v1.schema.json | vendor-revocation-handling-v1.example.json | required |

## P7 families
| Artifact family | Owner | Schema | Example | Status |
|---|---|---|---|---|
| IncidentTaxonomyV1 | continuity-runtime | incident-taxonomy-v1.schema.json | incident-taxonomy-v1.example.json | required |
| SeverityMatrixV1 | continuity-runtime | severity-matrix-v1.schema.json | severity-matrix-v1.example.json | required |
| PagerRouteProfileV1 | continuity-runtime | pager-route-profile-v1.schema.json | pager-route-profile-v1.example.json | required |
| EscalationClockPolicyV1 | continuity-runtime | escalation-clock-policy-v1.schema.json | escalation-clock-policy-v1.example.json | required |

## Compatibility rule

If a profile family exists in local code but not in this registry, it is not yet a canonical published family
for the post-v24 completion pass.

Every non-additive schema change MUST declare:
- compatibility class,
- migration owner,
- coexistence window,
- fixture updates required,
- and removal gate.

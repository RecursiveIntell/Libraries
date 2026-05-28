// Suggested `contract-schema-gen` registry additions for the post-v24 profile suite.

// P1 — Privacy, Retention, Disclosure, and Redaction
register_schema("PrivacyRetentionProfileV1", "schemas/privacy-retention-profile-v1.schema.json");
register_schema("RedactionRuleSetV1", "schemas/redaction-rule-set-v1.schema.json");
register_schema("AccessPurposeMatrixV1", "schemas/access-purpose-matrix-v1.schema.json");
register_schema("AuditExtractionPolicyV1", "schemas/audit-extraction-policy-v1.schema.json");

// P2 — Locality, Tenancy, Residency, and Boundary Overlay
register_schema("ResidencyPolicyProfileV1", "schemas/residency-policy-profile-v1.schema.json");
register_schema("TenantBoundaryProfileV1", "schemas/tenant-boundary-profile-v1.schema.json");
register_schema("CrossBoundaryTransferClassV1", "schemas/cross-boundary-transfer-class-v1.schema.json");
register_schema("LocalityExceptionV1", "schemas/locality-exception-v1.schema.json");

// P3 — Role Catalog, Duty Segregation, and Approval Matrix
register_schema("RoleCatalogV1", "schemas/role-catalog-v1.schema.json");
register_schema("DelegationMatrixV1", "schemas/delegation-matrix-v1.schema.json");
register_schema("ApprovalMatrixV1", "schemas/approval-matrix-v1.schema.json");
register_schema("ConflictClassCatalogV1", "schemas/conflict-class-catalog-v1.schema.json");

// P4 — Regulated Deployment, Control Mapping, and Recertification
register_schema("RegulatoryRegimeProfileV1", "schemas/regulatory-regime-profile-v1.schema.json");
register_schema("RequirementControlMapV1", "schemas/requirement-control-map-v1.schema.json");
register_schema("EvidenceCollectionPlanV1", "schemas/evidence-collection-plan-v1.schema.json");
register_schema("RecertificationScheduleV1", "schemas/recertification-schedule-v1.schema.json");

// P5 — Sector Hazard Library, Monitor Catalog, and Mitigation Playbook
register_schema("HazardLibraryV1", "schemas/hazard-library-v1.schema.json");
register_schema("HazardScenarioV1", "schemas/hazard-scenario-v1.schema.json");
register_schema("MonitorCatalogV1", "schemas/monitor-catalog-v1.schema.json");
register_schema("MitigationPlaybookV1", "schemas/mitigation-playbook-v1.schema.json");

// P6 — Vendor Certification Adapter and External Evidence Translation
register_schema("VendorCertificationAdapterV1", "schemas/vendor-certification-adapter-v1.schema.json");
register_schema("VendorEvidenceTranslationV1", "schemas/vendor-evidence-translation-v1.schema.json");
register_schema("VendorTrustRootBindingV1", "schemas/vendor-trust-root-binding-v1.schema.json");
register_schema("VendorRevocationHandlingV1", "schemas/vendor-revocation-handling-v1.schema.json");

// P7 — Incident Taxonomy, Escalation, and Pager Routing Profile
register_schema("IncidentTaxonomyV1", "schemas/incident-taxonomy-v1.schema.json");
register_schema("SeverityMatrixV1", "schemas/severity-matrix-v1.schema.json");
register_schema("PagerRouteProfileV1", "schemas/pager-route-profile-v1.schema.json");
register_schema("EscalationClockPolicyV1", "schemas/escalation-clock-policy-v1.schema.json");

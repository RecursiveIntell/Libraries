
// Suggested `stack-ids` additions for the post-v24 profile completion suite.
//
// Primary rule:
// - add opaque newtypes only,
// - keep parsing / serde / formatting in the existing style,
// - no business logic.

pub struct PrivacyRetentionProfileId(String);
pub struct RedactionRuleSetId(String);
pub struct AccessPurposeMatrixId(String);
pub struct AuditExtractionPolicyId(String);

pub struct ResidencyPolicyProfileId(String);
pub struct TenantBoundaryProfileId(String);
pub struct CrossBoundaryTransferClassId(String);
pub struct LocalityExceptionId(String);

pub struct RoleCatalogId(String);
pub struct DelegationMatrixId(String);
pub struct ApprovalMatrixId(String);
pub struct ConflictClassCatalogId(String);

pub struct RegulatoryRegimeProfileId(String);
pub struct RequirementControlMapId(String);
pub struct EvidenceCollectionPlanId(String);
pub struct RecertificationScheduleId(String);

pub struct HazardLibraryId(String);
pub struct HazardScenarioId(String);
pub struct MonitorCatalogId(String);
pub struct MitigationPlaybookId(String);

pub struct VendorCertificationAdapterId(String);
pub struct VendorEvidenceTranslationId(String);
pub struct VendorTrustRootBindingId(String);
pub struct VendorRevocationHandlingId(String);

pub struct IncidentTaxonomyId(String);
pub struct SeverityMatrixId(String);
pub struct PagerRouteProfileId(String);
pub struct EscalationClockPolicyId(String);

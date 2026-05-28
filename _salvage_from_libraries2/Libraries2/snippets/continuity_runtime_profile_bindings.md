
# Suggested `continuity-runtime` profile bindings

Add `src/profile_p7_incident_routing.rs` and export:
- `IncidentTaxonomyV1`
- `SeverityMatrixV1`
- `PagerRouteProfileV1`
- `EscalationClockPolicyV1`

Bind them to:
- `IncidentCaseV1`
- `ContainmentDecisionV1`
- `ContinuityExceptionV1`
- `PostmortemBundleV1`
- `ResilienceExerciseV1`

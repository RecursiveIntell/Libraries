Manual invariant injection after Phase 05:

Prove replay/audit completeness:
1. receipt includes all candidates and selected candidate id,
2. rejected candidates include source and rejection reason,
3. raw input digest and typed digest are distinct where applicable,
4. policy digest is canonicalized under documented profile,
5. evaluator/build digest covers all claimed sources.

If any digest overstates coverage, rename it or fix coverage.

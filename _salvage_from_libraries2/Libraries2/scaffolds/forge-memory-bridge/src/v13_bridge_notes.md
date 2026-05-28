# v13 bridge notes

## Additive transform responsibilities
- map `SupportSetV1` through the bridge as a first-class artifact or strong reference
- map `ContradictionWitnessV1` through the bridge as a first-class artifact or strong reference
- map `RetractionRecordV1` through the bridge as a first-class artifact or strong reference
- preserve `support_set_digest` on any transformed claim-state object

## Forbidden shortcuts
- do not synthesize a support expression from claim adjacency
- do not synthesize a contradiction witness because `contradiction_status != none`
- do not synthesize a retraction record from absence alone

## Test anchor
Add one golden transform fixture where:
- one support token supports,
- one support token refutes,
- the claim is retracted and superseded later,
- and the bridge output preserves all IDs/digests/references exactly.

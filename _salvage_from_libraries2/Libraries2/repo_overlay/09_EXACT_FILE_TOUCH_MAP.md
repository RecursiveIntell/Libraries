# Exact file touch map — post-v24 profiles

This is an additive / targeted modification plan against the current checkout.

## Root docs and plans
- `00_START_HERE.md (update active closeout pointer or add cross-link)`
- `20_TERMINAL_DESIGN_POSITION.md (link profile suite as the next-phase lane)`
- `21_PROFILE_BACKLOG_AFTER_V24.md (supersede backlog with this pack)`
- `plans/post-v24-profile-completion.execplan.md (new)`
- `docs/profile_completion_post_v24/README.md (new)`
- `docs/profile_completion_post_v24/MASTER_ISSUE_MATRIX.md (new)`
- `docs/profile_completion_post_v24/EXACT_FILE_TOUCH_MAP.md (new)`
- `docs/profile_completion_post_v24/PER_CRATE_APPLY_PLAN.md (new)`
- `docs/profile_completion_post_v24/RELEASE_BAR_AND_ACCEPTANCE.md (new)`

## Existing owner crates
### `stack-ids`
- `src/ids.rs — add new profile-layer ID newtypes`

### `contract-schema-gen`
- `src/lib.rs — register all profile schemas`
- `src/main.rs — include manifests if generated here`

### `verification-policy`
- `src/lib.rs — export P1/P2 profile modules`
- `src/profile_p1_privacy.rs — new`
- `src/profile_p2_locality.rs — new`

### `authority-delegation`
- `src/lib.rs — export P3 module`
- `src/profile_p3_roles.rs — new`

### `assurance-runtime`
- `src/lib.rs — export P4 and P5 modules`
- `src/profile_p4_regulated.rs — new`
- `src/profile_p5_hazard.rs — new`

### `attestation-exchange`
- `src/lib.rs — export P6 module`
- `src/profile_p6_vendor.rs — new`

### `continuity-runtime`
- `src/lib.rs — export P7 module`
- `src/profile_p7_incident_routing.rs — new`

## Published artifacts
- `schemas/*.schema.json` — 28 new profile-layer schemas
- `examples/*.example.json` — 28 new profile-layer examples
- `contracts/schemas/profile-p*/manifest.json` — 7 new manifests
- `contracts/fixtures/profile-p*/*.bundle.json` — 14 new fixture bundles
- `conformance/profile-p*/README.md` — 7 new conformance notes

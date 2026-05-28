# Repo gap report — March 15, 2026 checkout

## Baseline judgment

This checkout is already **effectively closed on v21–v24** and is **not** missing another base-spec wave.

Comparison against the previously generated closeout packs shows:

- `v21_v24_final_closeout_pack.zip`: 213 repo-relative files, **212 already present**, **1 missing**
- `post_v24_profile_completion_pack.zip`: 129 repo-relative files, **11 already present**, **118 missing**

The single missing item from the v21–v24 pack was `06_FINAL_MASTER_ISSUE_MATRIX.csv`, which is not required to land the profile suite.

## What was actually missing

The March 15 checkout was missing almost the entire **profile / overlay** completion layer:

- 7 profile specs
- 28 schemas
- 28 examples
- 14 fixture bundles
- 7 conformance directories
- owner-crate profile modules for:
  - `verification-policy`
  - `authority-delegation`
  - `assurance-runtime`
  - `attestation-exchange`
  - `continuity-runtime`
- schema-registry additions in `contract-schema-gen`
- shared ID additions in `stack-ids`

## Important repo-awareness note

The raw profile pack should **not** be unpacked blindly at repo root.

Why:
- several root control docs in the profile pack collide with already-active v21–v24 closeout docs,
- the raw `scripts/check_post_v24_profile_pack_truth.sh` checks the pack itself, not the full repository,
- the repository needs a **selective overlay + code edits**, not a wholesale replacement of active root materials.

## Resulting policy

For this checkout, the correct terminal Codex pass is:

1. keep the existing v21–v24 closeout lane intact;
2. add the missing post-v24 profile artifacts;
3. add the owner-crate profile modules and tests;
4. update `stack-ids` and `contract-schema-gen`;
5. verify with `scripts/check_post_v24_profile_repo_truth.sh`;
6. run targeted cargo tests for the affected crates.

# Codex master prompt

You are working in the March 15, 2026 libraries workspace.

Your job is to land the **post-v24 profile completion suite** as the final terminal pass for this design horizon.

## Read first

- existing repo docs:
  - `00_START_HERE.md`
  - `20_TERMINAL_DESIGN_POSITION.md`
  - `21_PROFILE_BACKLOG_AFTER_V24.md`
- kit docs:
  - `02_REPO_GAP_REPORT.md`
  - `03_SCOPE_AND_NON_GOALS.md`
  - `04_CHANGESET_SUMMARY.md`
  - `10_ACCEPTANCE_AND_COMMANDS.md`

## Non-negotiable rules

- do **not** invent or imply a `v25`
- do **not** add a new workspace member
- do **not** overwrite the existing root v21–v24 control docs with the raw profile-pack root docs
- keep the P1–P7 suite as **profiles / overlays** over existing law
- keep vendor translations explicit about caveats and lossiness
- keep approval, residency exception, and pager semantics as typed local artifacts

## Required implementation work

1. apply the repo overlay from `repo_overlay/`
2. preserve the existing v21–v24 lane
3. wire the new profile modules into:
   - `verification-policy`
   - `authority-delegation`
   - `assurance-runtime`
   - `attestation-exchange`
   - `continuity-runtime`
4. add the shared profile IDs in `stack-ids/src/ids.rs`
5. register the 28 profile schema types in `contract-schema-gen/src/lib.rs`
6. keep the profile schemas/examples/fixtures/manifests/conformance notes exactly named
7. ensure the repo-aware truth script passes

## Finish bar

You are done only when:
- the repo-aware truth script passes,
- the affected crates compile and test cleanly,
- and nothing in the resulting repo implies a new base-spec wave.

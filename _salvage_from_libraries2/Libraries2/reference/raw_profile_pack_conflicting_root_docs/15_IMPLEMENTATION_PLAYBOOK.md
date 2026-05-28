
# Implementation playbook — post-v24 profiles

## Commit order
1. root docs + exec plan + release bar
2. stack-ids additions
3. schema registry and file publication
4. owner-crate profile modules
5. fixtures and conformance notes
6. docs / curriculum truth update
7. final pack-truth run

## Working rules
- keep filenames stable,
- keep owners explicit,
- prefer additive modules over wide refactors,
- do not move base-spec responsibility into profile code,
- do not let external tooling become the sole home for profile semantics.

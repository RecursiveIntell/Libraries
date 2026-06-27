# Current code snapshot notes — v25 (2026-03-17)

## Snapshot basis

This repo pack targets the March 16, 2026 code snapshot carried in `libraries-source-clean-20260316.zip` and extends the prior v25 next-pass overlay.

## Important truth about the repo state

The repo was not blank.
It already carried:

- the older v21–v24 terminal closeout pack,
- the post-v24 profile-completion pack,
- and a first repo-targeted v25 overlay.

That means correctness here is not just “add more files.”
It is “make the repo self-consistent again.”

## Highest-leverage fixes in this pass

1. Explicit v25 supersession note.
2. Root-local v25 and v26 canonical spec files.
3. Repo-facing docs/v25 execution pack.
4. Fixture manifest and broader fixture coverage.
5. Whole-tree mirror sync.
6. Non-Rust validation scripts.

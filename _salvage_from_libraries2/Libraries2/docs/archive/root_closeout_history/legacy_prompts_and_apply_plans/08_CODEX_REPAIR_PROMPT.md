# Codex repair prompt

The first pass landed the profile suite imperfectly.

Repair only the failing parts.

Checklist:
- compare the repo against `repo_overlay/`
- inspect `patches/post_v24_profile_completion_repo.patch`
- rerun `bash scripts/check_post_v24_profile_repo_truth.sh`
- fix any missing module exports, missing schema registry entries, missing shared IDs, or misnamed profile files
- do not widen scope beyond the P1–P7 profile suite
- do not introduce `v25`

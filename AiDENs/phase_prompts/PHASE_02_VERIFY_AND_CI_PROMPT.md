# Phase 2 Prompt — Build Scope and Final Command Bar

Create `docs/codex-runs/BUILD_SCOPE.md` and replace `scripts/verify_current.sh` with the complete final command bar.

Update `.github/workflows/ci.yml` to run `bash scripts/verify_current.sh` with no stale P27/P28/P30 environment assumptions.

Run:

```bash
bash scripts/verify_current.sh
```

If cargo/build is unavailable or fails, record blocker evidence. Do not set build_certified true.

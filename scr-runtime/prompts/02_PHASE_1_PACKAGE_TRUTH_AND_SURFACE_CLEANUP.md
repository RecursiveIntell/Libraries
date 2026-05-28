# Phase 1 — Package Truth and Source Surface Cleanup

## Objective

Eliminate false handoff surfaces before hardening runtime behavior.

## Required actions

1. Add or update `scripts/verify_archive_manifest_parity.py`.
2. Add or update `scripts/assert_required_archive_paths.py`.
3. Add or update `scripts/assert_no_stale_surfaces.py`.
4. Move root `z.py` to `scripts/zip_source_certifier.py` if it is the certifier, and update references. If it is not needed, archive/delete it.
5. Delete `testtmp/` and add it to exclusion/assertion checks.
6. Resolve `target_files/`:
   - If active implementation exists, move to `docs/codex-runs/archive/P31-target-files-legacy/` or delete.
   - Do not allow active `target_files/` to coexist with active implementation unless every file is marked non-authoritative.
7. Resolve `manual_injections/`:
   - Move to `docs/codex-runs/archive/P31-manual-injections-legacy/` or rewrite README to mark as legacy only.
   - Active workflow must use automated phase gates.
8. Scrub non-SCR labels:

```bash
rg -n "SCR|PASS_SCR|scr-runtime|manual injection|paste the matching phase gate" . || true
```

Every hit must be deleted, archived, or explicitly marked historical/non-authoritative.

9. Update README to describe active repo, current CLI, automated gates, and P31 completion flow.

## Acceptance gate

```bash
python3 scripts/assert_no_stale_surfaces.py
```

Must pass.

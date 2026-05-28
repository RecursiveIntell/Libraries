# Phase 02 — Package Policy and Verifier Integration

## Tasks

1. Change default `codex-context` policy so run archives are excluded from normal context packages.
2. Add `audit-full` mode and/or explicit `--include-codex-archive` behavior.
3. Update sidecar manifest/report to include `codex_archive` summary:
   - archive mode;
   - moved count;
   - skipped-existing count;
   - unclassified count;
   - active stale count after normalization;
   - archive manifest paths.
4. Implement or adapt:
   - `scripts/assert_p22_release_package_clean.py`
   - `scripts/p22_verify_release_archive.sh`
5. Keep P21 verifier if still needed for historical release replay, but P22 normal flow must use P22 verifier.

## Acceptance Gate

```bash
python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run
python3 scripts/assert_p22_release_package_clean.py --manifest <dry-run-or-generated-manifest>
```

If the manifest file is not generated in dry-run, modify `z.py` so sidecars still emit under dry-run.

# Phase 02 — Implement root package artifact archival

Add a new root-package hygiene archival pass to `z.py`.

Required CLI:

```text
--archive-root-package-artifacts
--no-archive-root-package-artifacts
--verify-root-package-hygiene
--root-package-archive-root docs/source-packages/archive
--root-package-archive-dry-run
--include-root-package-archive
```

Default: enabled for next-codex-context, codex-run-full, audit-full.

Move root package residue to:

```text
docs/source-packages/archive/<UTC_STAMP>/files/
```

Write `PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json`.

Use hash/collision-safe movement. Do not delete without archived copy or same-hash existing archive.

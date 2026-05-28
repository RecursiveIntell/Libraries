# Phase 7 — Certifier and Fresh-Unzip Hardening

## Objective

Produce a package whose report/manifest/ZIP agree and whose fresh unzip passes checks.

## Required actions

1. Fix packager/certifier so manifest/report are verified against actual ZIP bytes after writing.
2. Add package parity script to normal checks.
3. Ensure active required files are present if report claims them.
4. Ensure `.codex`/`.agents` handling is explicit:
   - included and verified if active;
   - excluded and not claimed if inactive.
5. Remove junk and generated sidecars from package unless intentionally included.
6. Fresh unzip test must run from outside the repo:

```bash
rm -rf /tmp/scr-runtime-fresh
mkdir -p /tmp/scr-runtime-fresh
unzip -q <produced_zip> -d /tmp/scr-runtime-fresh
cd /tmp/scr-runtime-fresh
bash scripts/run_p31_completion_checks.sh
```

7. Write `docs/P31_FRESH_UNZIP_CERTIFICATION.md` containing:
   - produced ZIP path;
   - SHA-256;
   - manifest hash;
   - file count;
   - commands run;
   - results.

## Acceptance gate

```bash
python3 scripts/verify_archive_manifest_parity.py <produced_zip> <manifest.json>
python3 scripts/assert_required_archive_paths.py <produced_zip>
```

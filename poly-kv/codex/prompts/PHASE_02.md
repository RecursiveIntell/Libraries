# PHASE 02 — Portable package verification

Implement package verification that does not depend on build-machine absolute paths.

Tasks:

1. Add one accepted CLI shape:
   - `python3 z.py verify --package PKG --manifest MANIFEST --strict`, or
   - `python3 z.py --verify-package PKG --manifest MANIFEST --strict`.
2. Verify archive entries against manifest file hashes.
3. Verify safe archive paths.
4. Verify required files are present.
5. Verify manifest/package digest fields where possible.
6. Update `scripts/assert_source_package_hygiene.py` to accept explicit `--package`.

Acceptance:

Copy a package and manifest to `/tmp` or another temp directory. Verification must pass from that new location.

# Historical generated-tree quarantine

This is a reversible source-hygiene change, not a security or whole-audit completion claim. It removes only the tracked generated outputs beneath `_salvage_from_libraries2/Libraries2/target_schema_gen/` from the current Git index. Other salvaged source and historical receipts remain unchanged; Git history is not rewritten. Existing local ignored files may remain.

## Exact recovery identity

- Source commit: `d7cf668b35f18fbc3dd8ce9e2627521e63a82583`.
- Source tree: `f17814d70e6dda7c5f82d23d76f205b8f4a1af29`.
- Inventory: [target-schema-gen.manifest.json](target-schema-gen.manifest.json), **272 regular blobs, 232,661,833 bytes**; all original Git modes `100755` are retained, not normalized.
- Manifest SHA-256: `69ea1b9884692678efa12d79f6c9c98b1f336f22e64158e7f9b985e0f80ac194`.
- Archive: `target_schema_gen-d7cf668b35f18fbc3dd8ce9e2627521e63a82583.tar.xz`.
- Archive SHA-256: `fada7517725585453edcf87035cfe753dcd5440bcb72bf05bc48f20f17a6dce3`.

The controller created the archive outside Git, retained a second local copy outside the source workspace, and restored all entries into a fresh external destination before staging removal. Each source and restored file's bytes/mode were checked, and archive content was checked against the source Git tree, blob IDs and per-file SHA-256 values. Local receipts bind the manifest and helper digests. These are same-host/same-filesystem copies, **not an off-host backup, immutable retention or remote-source authentication**. The archive is not shipped in Git or asserted to be publicly downloadable. Obtain it from the retaining operator, or recreate it from the source commit below. Retain trusted digests separately when transferring it.

## Verification and rollback (Python 3.14+)

Use a checkout with the full named source commit and blobs available locally. The helper deliberately disables lazy Git fetching. Set `ARCHIVE_DIR` to the directory holding the retained archive, and `RESTORE_DIR` to a **nonexistent external** directory. From the repository root:

```sh
python3.14 scripts/verify_quarantine_manifest.py \
  --manifest docs/quarantine/target-schema-gen.manifest.json \
  --source d7cf668b35f18fbc3dd8ce9e2627521e63a82583 \
  --archive "$ARCHIVE_DIR/target_schema_gen-d7cf668b35f18fbc3dd8ce9e2627521e63a82583.tar.xz" \
  --extract-to "$RESTORE_DIR"
```

The verifier checks an exact closed manifest, duplicate fields/paths, Git identity, hashes, sizes, regular-member types and modes. Only canonical PAX long-path metadata is accepted. Existing destinations, path escapes, links, sparse members, extra or missing entries fail closed. A failed extraction can leave an incomplete destination; it never yields a verified result. Reuse requires a new destination after inspection. This is a local integrity tool, not an adversarial same-UID sandbox.

If the archive is unavailable, reconstruct it from the retained Git source without modifying the current source/index (set `NEW_EXTERNAL_MANIFEST` to a nonexistent manifest path outside the checkout):

```sh
python3.14 scripts/verify_quarantine_manifest.py --create \
  --source d7cf668b35f18fbc3dd8ce9e2627521e63a82583 \
  --manifest "$NEW_EXTERNAL_MANIFEST"
```

Creation requires a clean target subtree. It reads original Git blobs, verifies a fresh restoration, then publishes final archive/manifest names without overwriting existing artifacts. A process crash between the two final links may leave an orphan archive, never a successful manifest. A newly generated archive can have a different compression digest across toolchain versions; its newly generated manifest must still match the exact Git inventory. Do not edit the recorded manifest merely to make verification pass.

For a reviewed source rollback, restore **only this exact subtree** from the named commit in a clean isolated checkout, inspect the staged paths, and force-add it if intentionally reversing the ignore rule. Do not reset or clean an operator's dirty checkout. The helper never removes files or modifies the Git index.

## Source packs and retained documents

`zip.py` excludes only this repo-relative generated subtree, even when ignored local files remain. Its regression test inspects actual ZIP members and retains unrelated same-basename source as a positive control. This patch does not change its pre-existing absolute-ancestor temporary-directory exclusion.

The canonical root-document classification remains [the existing manifest](../archive/root_closeout_history/manifest.json), including its active-root list and supersession map. No second classification map is introduced, and no governing document is relocated. Its checker remains `python3.14 scripts/check_root_archive_manifest.py`.

## Checks and limits

The focused suite has 25 tests, including corruption, archive/manifest co-tampering against Git IDs, path traversal, mode changes, duplicate/missing/extra members, unsupported extensions, failed-creation cleanup, no-clobber publication, exact restoration, versioned-manifest default scratch and actual ZIP membership. Run:

```sh
python3.14 -m unittest discover -s scripts -p test_verify_quarantine_manifest.py -v
python3.14 scripts/check_root_archive_manifest.py
git diff --check
```

A dedicated Python 3.14 hosted job executes the fixture suite and retained-document checker. Fixture CI does not independently restore the operator-held full archive. Data-lineage specialist design advice was reviewed; no independent code-verifier verdict was obtained. The independent verifier was skipped under the operator's instruction, not passed. H1/H2 isolation/custody and the installed-release audit matrix remain separate open work.

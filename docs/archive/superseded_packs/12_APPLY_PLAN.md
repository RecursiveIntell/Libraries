
# 12_APPLY_PLAN

## Minimal application sequence

1. Copy the `overlay/` files into the repo root.
2. Re-run:
   - `bash scripts/check_pack_truth.sh`
   - `python3 scripts/check_root_archive_manifest.py`
3. Rewrite:
   - `STATUS_DASHBOARD.md`
   - `STATUS_EVIDENCE_MANIFEST.json`
   - `release/closeout_receipt_v1.json`
4. Align `Makefile`, the evidence manifest generation path, and the receipt generator.
5. Fix `check_no_prod_panics.sh` or relocate inline test modules.
6. Add `.github/workflows/ci.yml`.
7. Restore/retire v25 scripts and missing pack files.
8. Finish the v25 production-closure code/schema gaps.
9. Centralize `V25ConstitutionCitation`.
10. Do the credibility cleanup (thin runtime names/docs, giant modules, llm-refinement, extractor).

## Stop rule

Do not move to Phase 1 until Phase 0 is actually green in the repo, not just in docs.

# Phase 03 — Packaging Policy Repair

Goal: make the next archive preserve active control files required by tests.

Known failure pattern:
- package policy pruned `.codex/`;
- tests expected `.codex/`;
- archive passed package validation but failed project validation.

Required fixes:
1. Include active `.codex/` control files in next-codex-context/release archives.
2. Exclude only volatile `.codex/runs/`, temporary logs, caches, and archived stale copies where appropriate.
3. Preserve `.agents/skills/`.
4. Remove generated `claim_ledger.egg-info/` from source packaging.
5. Move root `z.py` to `scripts/zip_source_certifier.py` or explicitly exclude it if not product source.
6. Add `scripts/assert_archive_includes_codex.py`.

Validation command:

```bash
python z.py --mode next-codex-context --output /tmp/claimledger-phase-03-packaging.zip && \
python scripts/assert_archive_includes_codex.py /tmp/claimledger-phase-03-packaging.zip && \
rm -f /tmp/claimledger-phase-03-packaging.zip
```

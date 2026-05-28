# Phase 03 — Packaging Policy Repair

Validation command:

```bash
python z.py --mode next-codex-context --no-archive-codex-runs --no-strict --output /tmp/claimledger-phase-03-packaging.zip && \
python scripts/assert_archive_includes_codex.py /tmp/claimledger-phase-03-packaging.zip && \
rm -f /tmp/claimledger-phase-03-packaging.zip
```

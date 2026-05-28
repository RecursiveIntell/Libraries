# Phase 04 — Release Truth Docs and Operator State

## Tasks

Update active docs to P22 truth:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md` if present
- `MANIFEST.md/json/txt` if used as active release manifest
- `docs/codex-runs/CURRENT_RUN.md`

Docs must state:

- supported surfaces;
- partial surfaces;
- scaffold/deferred surfaces;
- packaging flow;
- how to include audit archives intentionally;
- how to reproduce verification.

## Acceptance Gate

```bash
bash scripts/assert_docs_source_basis_current.sh || true
python3 scripts/assert_p22_codex_archival_hygiene.py .
grep -R "P20\|P21" README.md STATUS.md SOURCE_BASIS.md SUPPORT_PROFILE.md 2>/dev/null || true
```

Any P20/P21 references that remain in active docs must be historical and explicitly non-normative.

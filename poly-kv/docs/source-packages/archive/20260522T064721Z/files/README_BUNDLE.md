# poly-kv z.py + source-package hygiene Codex bundle — 2026-05-22

Purpose: repair the current `poly-kv` handoff/package pipeline so future context zips are self-contained, reproducible, and free of root-level package/Codex detritus.

This bundle specifically targets:

1. `z.py` allowlist defects that excluded `python/poly_kv/_native.pyi` and `python/poly_kv/py.typed`.
2. Missing command receipt logs in hostile-audit packages.
3. Root package hygiene residue: previous `poly-kv-generic-*.zip`, sidecars, `README_BUNDLE.md`, `BUNDLE_MANIFEST.json`, old codex-archive reports, and ambiguous root Markdown.
4. Weak validation: the certifier can pass while omitting files required by repo-side validators.
5. Final package handoff quality: package itself must prove that all required files and receipts are present.

Expected use:

```bash
# from repo root
mkdir -p codex/next-pass/zpy-hygiene-20260522
# copy this bundle there or paste prompts phase-by-phase
```

Start with `codex/prompts/MASTER_PROMPT.md`, then execute phases in order. Use `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md` between phases.

Do not publish the crate or PyPI package from this run. This run is about package/handoff correctness.

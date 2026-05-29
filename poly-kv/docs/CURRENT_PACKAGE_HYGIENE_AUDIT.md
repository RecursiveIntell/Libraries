# Current package hygiene audit

Observed from the 2026-05-22 package manifest/report:

- Package passed certifier validation with 0 findings.
- Included files: 240.
- Top-level root still included `README_BUNDLE.md`, `BUNDLE_MANIFEST.json`, and prior package/codex artifact `poly-kv-generic-rust-next-codex-context-20260520.codex-archive.json`.
- Excluded files included:
  - `.codex-runs/20260520T174516Z-alpha1/commands_run.log`
  - `.codex-runs/20260522T045320Z-poly-kv-next/commands_run.log`
  - `python/poly_kv/_native.pyi`
  - `python/poly_kv/py.typed`
- `scripts/assert_python_sidecar_layout.py` requires `_native.pyi` and `py.typed`, so the package can pass while being non-self-contained for that validator.
- Root Markdown hygiene marked `README_BUNDLE.md` and a previous `.report.md` as ambiguous root Markdown.

Hostile conclusion:

The current package is useful, but not yet a solid pass-off package. It can omit required Python typing files, omit raw command receipts, and carry stale/generated root artifacts. The next pass must make package self-containment a hard gate, not a post-hoc observation.

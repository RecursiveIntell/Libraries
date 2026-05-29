# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-22T08:02:01Z`
- Root: `/home/sikmindz/Coding/Libraries/poly-kv`
- Archive root: `/home/sikmindz/Coding/Libraries/poly-kv`
- Output: `/home/sikmindz/Coding/Libraries/poly-kv/poly-kv-generic-rust-next-codex-context-20260522T080201Z.zip`
- Include roots: `1`
- External Cargo path dependency roots: `0`
- Profile: `generic-rust` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `277`
- Included bytes: `663365`
- Excluded files: `1`
- Pruned dirs: `9`
- Findings: `1` (`0` errors, `0` warnings)
- Archive zip-byte SHA-256: `0d0e495485165d1f9a88b4a7dedc4886c72aad30e29f0fe7e5989807a1b0f8aa`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `0f17afa9152dcbec37236e57d4e83e88b853fddd6e82c807525b64fcf932ca9c`
- Ecosystems detected: `rust, python, git`
- Codex archive enabled: `True`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `4`
- Root Markdown protected: `4`
- Root Markdown candidates: `0`
- Root Markdown ambiguous: `0`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `14`
- Root package protected: `7`
- Root package candidates: `7`
- Root package moved: `7`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 4 | 0 | `available-not-run` |
| `python` | `True` | 1 | 0 | `available-not-run` |
| `node` | `False` | 0 | 0 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `True` | 0 | 0 | `available-not-run` |

## Decision provenance

- Decisions recorded: `287`
- Includes: `277`
- Excludes: `1`
- Pruned dirs: `9`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| info | `git-metadata-excluded` | `.git/` | Git metadata detected and intentionally excluded from transferable package contents. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.md` | 103 |
| `.json` | 87 |
| `.rs` | 34 |
| `.py` | 24 |
| `.txt` | 10 |
| `.sh` | 6 |
| `.toml` | 6 |
| `.log` | 3 |
| `.lock` | 1 |
| `.patch` | 1 |
| `.pyi` | 1 |
| `.typed` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `.codex-runs` | 155 |
| `docs` | 39 |
| `crates` | 38 |
| `scripts` | 21 |
| `python` | 10 |
| `.agents` | 5 |
| `AGENTS.md` | 1 |
| `Cargo.lock` | 1 |
| `Cargo.toml` | 1 |
| `README.md` | 1 |
| `fixtures` | 1 |
| `patches` | 1 |
| `pyproject.toml` | 1 |
| `schemas` | 1 |
| `z.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `generated-output` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/poly-kv/poly-kv-generic-rust-next-codex-context-20260522T080201Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/poly-kv/poly-kv-generic-rust-next-codex-context-20260522T080201Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/poly-kv/poly-kv-generic-rust-next-codex-context-20260522T080201Z.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/poly-kv/poly-kv-generic-rust-next-codex-context-20260522T080201Z.findings.json`

## Interpretation

This package passed the configured validation gates.

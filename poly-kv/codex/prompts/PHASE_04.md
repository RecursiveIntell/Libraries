# PHASE 04 — Ecosystem detection and parity adapters

Implement static detection and minimal parity reports for Rust, Python, Node/npm, Git. Add Docker and Go detection if time permits.

Tasks:

1. Detect manifests and ecosystem signals.
2. Run optional dry-run commands only if tools exist.
3. Report missing tools rather than silently passing.
4. Add critical required-file checks per ecosystem.
5. Add `ecosystem_parity` section to manifest/report.

Minimum checks:

- Rust: Cargo manifests, Cargo.lock, include_str/include_bytes references, target exclusion.
- Python: pyproject, py.typed, *.pyi, MANIFEST.in, dist exclusion.
- Node: package.json, lockfiles, files field, .npmignore, node_modules exclusion.
- Git: git status/ls-files where available, .gitattributes export-ignore signal.

Acceptance:

Mixed fixture produces parity report with at least Rust/Python/Node/Git sections.

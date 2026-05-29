# Implementation Plan

## Phase 0 — Preflight and source inventory

- Record current `z.py` script version and git state.
- Inventory current CLI args, policy defaults, and tests.
- Run existing validation scripts.
- Preserve current behavior with fixture outputs before edits.

## Phase 1 — Policy schema and compatibility layer

- Add `PackagePolicyV1` dataclass/schema.
- Add optional `--policy` argument.
- Keep old CLI arguments; old args override defaults or merge into policy.
- Add `z.py policy validate` and `z.py policy init` if subcommands are introduced.

## Phase 2 — Portable verify mode

- Add `verify_package(package, manifest)` independent of source tree.
- Add `--verify-package`, `--package`, or `verify` subcommand.
- Fix absolute-path assumptions in `assert_source_package_hygiene.py` or fold that verifier into `z.py`.

## Phase 3 — Controlled audit evidence extensions

- Include `.patch` and `.diff` in Codex/audit/context modes.
- Keep release/source-clean policy stricter unless configured.
- Add regression fixture proving `touched_diff.patch` is included in `next-codex-context`.

## Phase 4 — Ecosystem adapters

Implement static detection first; external dry-run commands second.

Minimum adapter outputs:

- detected ecosystems;
- manifests found;
- optional dry-run command status;
- expected critical files;
- missing/extra/drift findings.

Start with Rust, Python, Node/npm, Git. Add Docker and Go after the core path is stable.

## Phase 5 — Safety and portability gates

- Unicode normalization collisions.
- Casefold collisions.
- Windows reserved basename checks.
- Archive path safety verifier.
- Symlink/hardlink/special-file policy.
- Nested archive handling by mode.
- Compression-ratio anomaly report.

## Phase 6 — Deterministic/provenance outputs

- Bump `SCRIPT_VERSION`.
- Add `SOURCE_DATE_EPOCH` support.
- Add optional tar.gz writer with normalized metadata.
- Add checksums file.
- Add provenance-lite in-toto/SLSA-style predicate.
- Add SBOM-lite SPDX/CycloneDX-style component inventory.

## Phase 7 — Package diff and explainability

- Add decision log.
- Add `--explain <path>` using manifest decision log.
- Add `--compare-manifest old new`.
- Surface risky additions: binary, archive, generated, secret-like, path dependency, root residue.

## Phase 8 — CI templates and docs

- Add docs explaining modes and policy config.
- Add repo-local example `zpy.package.example.toml`.
- Add GitHub Actions/local shell template.
- Document exact release/non-release boundaries.

## Phase 9 — Final audit

- Run all existing tests.
- Run new fixture suite.
- Create dirty fixture repo with Rust/Python/Node/Docker/Git signs.
- Package twice and compare content manifest.
- Verify package after copying to temp directory.
- Produce hostile-auditor handoff.

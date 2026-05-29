# High-ROI z.py Change Matrix

| ID | Priority | Change | Why it is high ROI | Applies to repos | Acceptance gate |
|---|---:|---|---|---|---|
| ZPY-001 | P0 | Introduce `PackagePolicyV1` config file and JSON schema | Hardcoded profile rules do not scale across repos; config lets each repo declare canonical roots, protected files, evidence modes, and cleanup policy. | All | `python3 z.py --policy zpy.package.toml --dry-run --strict` works; invalid policy fails schema validation. |
| ZPY-002 | P0 | Add ecosystem detection and parity adapters | Mature packagers expose expected file sets; z.py should compare against them rather than rely only on heuristics. | Rust, Python, Node, Go, Docker, Git | Report includes `ecosystem_parity` section with pass/warn/error status per adapter. |
| ZPY-003 | P0 | Add transfer-portable package verification | Current validators can depend on build-machine absolute paths. Handoff packages must verify after transfer. | All | `python3 z.py --verify-package package.zip --manifest package.manifest.json` passes from any directory. |
| ZPY-004 | P0 | Include `.patch`/`.diff` as controlled audit evidence | Patch files are high-value Codex/audit evidence. Excluding them weakens handoff receipts. | All Codex/audit repos | `touched_diff.patch` included in context/audit modes or excluded with explicit policy reason. |
| ZPY-005 | P0 | Generalized root/package artifact hygiene | Any repo can accumulate old zips, manifests, reports, run bundles, and generated sidecars. Automatically archive before packaging. | All | Root package residue moved to `docs/source-packages/archive/<stamp>/files/` with manifest. |
| ZPY-006 | P0 | Required file gates by project type | Prevent omissions like `py.typed`, `.pyi`, lockfiles, `LICENSE`, package manifests, and command receipts. | All | Missing required files become strict findings by mode/profile. |
| ZPY-007 | P0 | Deterministic archive improvements | Reproducibility requires stable path order, timestamps, modes, source-date-epoch support, and content-hash semantics. | All | Two dry-run packages from same tree produce identical content manifest; optional byte-identical archives. |
| ZPY-008 | P0 | Security/portability gates | Prevent path traversal, symlink escape, hardlink surprise, Unicode/case collisions, Windows reserved names, secret leaks, zip bombs. | All | Fixture suite covers each risk; strict mode fails on violations. |
| ZPY-009 | P1 | Explainable include/exclude decisions | Auditors need to know whether a path was included because of policy, ecosystem adapter, profile, or heuristic. | All | `--explain path` and manifest `decision_source` exist. |
| ZPY-010 | P1 | Package diff mode | Compare current package to previous manifest to identify drift, missing files, new binaries, new secrets risk, new root residue. | All | `--compare-manifest old.manifest.json` produces added/removed/changed/risk report. |
| ZPY-011 | P1 | Provenance-lite and SBOM-lite sidecars | Align with SLSA/in-toto/SPDX/CycloneDX without requiring full signing or registry workflows. | All | Emits `*.provenance.intoto.json`, `*.sbom.spdx.json` or `*.sbom.cdx.json` in optional mode. |
| ZPY-012 | P1 | CI/hook integration templates | Consistent package gates across repos prevent regressions. | All | Generated GitHub Action or local shell gate can run package verification. |
| ZPY-013 | P1 | Monorepo component packaging | Utility/RecursiveIntell repos may contain multiple crates/packages. z.py should package components and full context separately. | Monorepos | Per-component manifests and global root manifest emitted. |
| ZPY-014 | P2 | Optional signing/attestation hooks | Useful for public releases; less urgent for internal Codex handoff. | Public releases | Optional `cosign sign-blob` / GitHub attestation instructions and receipt fields. |
| ZPY-015 | P2 | Registry publish preflight mode | Simulate or dry-run cargo/npm/python package outputs before publish. | Release repos | `--release-preflight` checks `cargo package --list`, `npm pack --dry-run`, Python sdist/wheel contents where applicable. |

## P0 target

Implement ZPY-001 through ZPY-008 first. They make the tool useful in every repo and prevent the exact class of failures already observed in `poly-kv`.

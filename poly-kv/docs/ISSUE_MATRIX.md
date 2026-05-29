# Issue Matrix

| ID | Severity | Surface | Evidence / trigger | Failure mode | Fix direction | Acceptance gate |
|---|---:|---|---|---|---|---|
| ZPY-I01 | S1 | Transfer verification | Validator uses manifest package absolute path | Package cannot be verified off build machine | Add `--package`/verify mode and relative fallback | Verify copied package from `/tmp` with manifest. |
| ZPY-I02 | S1 | Audit evidence | `.patch` / `.diff` excluded | Missing Codex change evidence | Add controlled audit extension policy | Fixture patch included in context/audit modes. |
| ZPY-I03 | S1 | Policy scalability | Rules hardcoded by profile/mode | New repos require z.py edits | Add PackagePolicyV1 config/schema | Invalid config fails; valid config changes behavior. |
| ZPY-I04 | S1 | Ecosystem drift | z.py package can omit files standard packagers need | Handoff/release mismatch | Add adapters and dry-run parity checks | Rust/Python/Node fixtures produce parity reports. |
| ZPY-I05 | S1 | Required metadata | Typing, stubs, lockfiles, LICENSE, README can be accidentally omitted | Broken consumers / audit gaps | Required-file gates by detected ecosystem | Missing critical file produces strict error. |
| ZPY-I06 | S1 | Portability/security | No complete collision/special-path suite | Package extracts or behaves differently across OSes | Add path, symlink, Unicode, case, Windows reserved gates | Fixture suite fails on unsafe paths. |
| ZPY-I07 | S2 | Reproducibility | Script has deterministic ZIP timestamp but no SOURCE_DATE_EPOCH control | Harder to align with reproducible-build tools | Add source-date-epoch and tar.gz option | Same tree yields stable content and optional byte hash. |
| ZPY-I08 | S2 | Explainability | Include/exclude reasons not always source-attributed | Auditor cannot distinguish policy vs heuristic | Add decision log/source | `z.py explain path` reports reason/source. |
| ZPY-I09 | S2 | Provenance | Manifests exist but no standard predicate shape | Harder to connect to SLSA/in-toto/GitHub attestations | Add provenance-lite optional output | Predicate subject digest matches package. |
| ZPY-I10 | S2 | SBOM | No component inventory | Less useful in broad repos | Add SBOM-lite optional output | Emits SPDX/CycloneDX-like JSON for manifests/deps where parseable. |
| ZPY-I11 | S2 | Monorepos | One package may be too broad or too narrow | Utility-style repos lose context or include too much | Add component packages and root aggregate | Per-component manifests emitted. |
| ZPY-I12 | S3 | Script version | Version may not bump on behavioral changes | Audit ambiguity | Enforce version bump on policy/output changes | Test checks report version changed. |

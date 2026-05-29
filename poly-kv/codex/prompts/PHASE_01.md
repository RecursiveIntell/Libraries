# PHASE 01 — Policy schema and compatibility layer

Implement `PackagePolicyV1` support without breaking existing CLI behavior.

Tasks:

1. Add policy loading from optional `--policy` path.
2. Add default policy construction from existing args/profile/mode.
3. Validate policy against schema or an internal equivalent.
4. Add policy details to manifest/report.
5. Add fixture/example policy under docs or fixtures.

Acceptance:

```bash
python3 z.py --root . --mode next-codex-context --strict --dry-run
python3 z.py --root . --mode source-clean --strict --dry-run
python3 z.py --policy fixtures/zpy.package.example.toml --root . --dry-run --strict
```

If TOML policy validation cannot be fully implemented in this pass, implement JSON policy first and record TOML as deferred.

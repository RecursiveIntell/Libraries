# PHASE 07 — Explain and compare modes

Tasks:

1. Emit decision log with one record per considered path.
2. Add `explain` behavior for a path against a manifest/decision log.
3. Add previous-manifest comparison.
4. Flag risky additions: binary, archive, generated, secret-like, external path dep, root residue.

Acceptance:

```bash
python3 z.py explain --manifest <manifest> --path python/poly_kv/py.typed
python3 z.py compare --old old.manifest.json --new new.manifest.json
```

Equivalent legacy options are acceptable if documented.

# z.py Next Architecture Spec

## Role boundary

`z.py` is a **source/context/handoff package certifier**.

It is not:

- a replacement for Cargo/npm/PyPI/Go/Docker packagers;
- a release registry publisher;
- a signing authority;
- a general DLP scanner;
- a build system.

It should produce clean, auditable packages and tell the operator when the package disagrees with ecosystem packagers or local policy.

## Core pipeline

```text
Preflight
  → Detect repository ecosystems
  → Load PackagePolicyV1
  → Normalize/clean root and package residue
  → Collect candidate files
  → Apply inclusion/exclusion rules
  → Run ecosystem parity adapters
  → Run required-file and safety gates
  → Write archive(s)
  → Emit manifest/report/excluded/findings/provenance/SBOM/checksums
  → Verify package transferability
```

## Main modules/functions to add or refactor

```text
zpy/policy.py             # PackagePolicyV1 loading/defaulting/schema validation
zpy/ecosystems.py         # Rust/Python/Node/Go/Docker/Git detection and adapters
zpy/hygiene.py            # root/source-package/artifact cleanup engines
zpy/decisions.py          # include/exclude decisions with decision_source and evidence
zpy/security.py           # path, symlink, secret, collision, zipbomb guards
zpy/archive.py            # deterministic zip/tar writer + verification
zpy/provenance.py         # provenance-lite and SBOM-lite emitters
zpy/compare.py            # previous manifest comparison
zpy/cli.py                # argparse and subcommands
```

Current single-file `z.py` may remain for portability, but Codex should internally structure it with clearly bounded function groups and docstrings. Do not add hidden behavior without report fields.

## New CLI surface

```bash
python3 z.py package --root . --mode next-codex-context --strict
python3 z.py verify --package out.zip --manifest out.manifest.json --strict
python3 z.py explain --manifest out.manifest.json --path python/poly_kv/py.typed
python3 z.py compare --old old.manifest.json --new new.manifest.json
python3 z.py policy init --root .
python3 z.py policy validate --policy zpy.package.toml
```

Backward compatibility: existing invocation must still work:

```bash
python3 z.py --root . --profile auto --mode next-codex-context --strict
```

Internally this may dispatch to `package`.

## PackagePolicyV1

Policy fields should be explicit and mode-aware. See `schemas/PackagePolicyV1.schema.json`.

Key concepts:

- `package_name`
- `package_role`
- `modes`
- `protected_root_files`
- `required_files`
- `audit_evidence`
- `ecosystem_parity`
- `root_hygiene`
- `security`
- `archive`
- `provenance`
- `sbom`
- `allowed_extensions`
- `allowed_basenames`
- `path_rules`

## Ecosystem adapters

Every adapter returns:

```json
{
  "ecosystem": "rust|python|node|go|docker|git",
  "detected": true,
  "manifests": [],
  "dry_run_available": true,
  "expected_files": [],
  "missing_from_zpy_package": [],
  "extra_in_zpy_package": [],
  "findings": []
}
```

Adapters must never mutate the repo. They may run dry-run commands only when available.

## Decision provenance

Every included or excluded file should carry a reason and source:

```json
{
  "path": "python/poly_kv/py.typed",
  "decision": "include",
  "reason": "required-python-typed-marker",
  "source": "python-adapter|required-file-policy",
  "mode": "next-codex-context"
}
```

This prevents silent heuristic drift.

## Archive outputs

Default outputs remain:

```text
<stem>.zip
<stem>.manifest.json
<stem>.report.md
<stem>.excluded.json
<stem>.findings.json
```

Optional outputs:

```text
<stem>.tar.gz
<stem>.checksums.txt
<stem>.provenance.intoto.json
<stem>.sbom.spdx.json
<stem>.sbom.cdx.json
<stem>.decision-log.jsonl
<stem>.diff-from-previous.json
```

## Strict-mode rule

A strict package fails on:

- missing required files;
- secret findings;
- path traversal or symlink escape;
- ambiguous root/package residue after hygiene pass;
- ecosystem parity hard failures;
- unsupported binary or archive in included set unless explicitly allowed;
- non-portable validator references;
- manifest/package mismatch.

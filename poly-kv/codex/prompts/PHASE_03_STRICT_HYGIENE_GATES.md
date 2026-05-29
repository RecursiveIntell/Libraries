# Phase 03 — Strict hygiene gates

Strict packaging must fail if:

- root package artifacts remain;
- ambiguous root Markdown remains;
- `_native.pyi` or `py.typed` is absent from manifest;
- command evidence is absent;
- package archive reports errors/collisions.

Integrate `root_package_archive` into report/manifest/console output.

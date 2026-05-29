# Invariant Report

- Deterministic profile digests: not changed; Rust tests passed.
- Manifest/readers/receipts: not changed; receipt roundtrip/integrity validators passed.
- Exact fallback: not changed; synthetic exact fallback tests passed.
- Shape/layout mismatch behavior: not changed; shape rejection tests passed.
- Byte accounting: not changed; memory and realized accounting tests passed.
- Package self-containment: fixed and validated for `_native.pyi`, `py.typed`, command evidence, and root package archive summary.
- Secret scanning: remains enabled in `z.py`; no global weakening added.
- Unsupported-extension checks: remain enabled; narrow allowlist additions only.
- Root package deletion safety: root artifacts were moved or same-hash removable only through archive records with SHA-256 evidence.

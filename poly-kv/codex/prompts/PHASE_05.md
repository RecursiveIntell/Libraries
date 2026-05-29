# PHASE 05 — Security and portability gates

Implement strict gates from `docs/SECURITY_AND_PORTABILITY_GATES.md`.

Tasks:

1. Path traversal and absolute archive entry checks.
2. Symlink/hardlink/special-file policy.
3. Unicode normalization collisions.
4. Casefold collisions.
5. Windows reserved basename detection.
6. Nested archive policy by mode.
7. Binary allowlist and oversize handling.
8. Compression-ratio anomaly reporting in verifier.

Acceptance:

Create unsafe fixture paths and prove strict mode fails with specific finding codes.

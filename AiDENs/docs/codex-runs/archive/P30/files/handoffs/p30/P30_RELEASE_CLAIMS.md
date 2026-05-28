# P30 Release Claims

Allowed claims:

- `build-certified` for the AiDENs local workspace command bar captured under `target/p30/audit/`: fmt, check, test, clippy, and doc passed.
- `static-audit-hardened` for P30 hard patterns: `python3 scripts/p30_guard.py` exits 0 with `hard=0`.
- `v11B-draft-runtime-spine` only in the narrow sense that existing v11B executable seed checks pass without a completion/conformance claim.

Forbidden claims:

- Do not claim `release-certified`; parent `make -C .. gate` fails pack-truth validation.
- Do not claim full v11A compliance.
- Do not claim v11B-conformant runtime.
- Do not claim package cleanliness proves build, semantic, or release correctness.

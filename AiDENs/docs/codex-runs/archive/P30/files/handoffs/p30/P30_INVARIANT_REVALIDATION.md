# P30 Invariant Revalidation

- Provenance-first: improved durable default canonical receipt logging and failure receipts; verified by `cargo test --workspace --all-targets`.
- Strict executable boundaries: parser fallback uses strict JSON boundary policy and blocks repaired/malformed executable tool calls; covered by runner tests.
- No silent patch/read rollback truth loss: missing patch targets fail closed and rollback write errors are returned; covered by `p30_tool_hardening`.
- Deterministic identity pressure: constant tool exposure IDs, old generated-artifact symbol use, and agency random UUIDs were removed or replaced.
- Proof/degradation honesty: advisory-only control records no longer report `Succeeded`; `scripts/verify.sh` and `p30_guard.py` pass.
- Claim discipline: parent release gate failure is recorded, and no full v11A/v11B conformance claim is made.

Proceed condition: AiDENs-local runtime hardening can proceed to targeted P31 closure. Release-certified/v11B-conformant claims cannot proceed until quarantines and parent pack-truth gate failures are closed.

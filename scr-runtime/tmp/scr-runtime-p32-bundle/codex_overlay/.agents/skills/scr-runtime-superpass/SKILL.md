---
name: scr-runtime-superpass
description: "Use for scr-runtime P32 completion super-pass: source-truth preflight, phase gates, SCR evaluator semantics, schema parity, receipts, fixtures, and hostile final audit."
---

Follow the P32 super-pass bundle.

Required behavior:
1. Run preflight before edits.
2. Execute phases in order.
3. At every phase boundary, run invariant checks and write a phase report.
4. Never scan opaque refs as control truth.
5. Never claim external owner-crate integration without compiled/tested evidence.
6. End with P32 command receipts, changed files, unresolved risks, rollback plan, and hostile-auditor handoff.

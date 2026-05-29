# Scaffold Notes

The `boundary-compiler-core` crate is an optional starter implementation for P31.

It is intentionally standalone and narrow:

- strict JSON boundary parsing;
- duplicate-key rejection before conversion to ordinary `serde_json::Value`;
- stable sorted JSON canonicalization;
- parse receipts for accepted and rejected results;
- treatment-integrity receipts for missing critical paths;
- NoRepair default with no fake `RepairedAccept`.

This environment did not have `cargo` installed, so the scaffold could not be compiled here. Treat it as a drop-in starting point for Codex to inspect, adapt, and test inside your repository.

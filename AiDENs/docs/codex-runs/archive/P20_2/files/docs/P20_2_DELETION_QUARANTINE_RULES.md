# P20.2 Deletion and Quarantine Rules

## Delete only when

- file is generated stale output and has no current references;
- test was duplicate and equivalent coverage remains;
- local type is an invalid duplicate and canonical owner exists;
- docs are obsolete and archived with source-basis note.

## Quarantine when

- ownership is ambiguous;
- a feature is partially implemented but unsafe to advertise;
- a provider route exists but lacks executable test proof;
- a reference interpreter is incomplete.

## Never delete to fake a pass

Do not delete failing tests, eval cases, or assertions merely to make `cargo test` pass. Repair or demote with explicit evidence.
